// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix domain socket and TCP listeners for Webb's JSON-RPC server.
//!
//! Accepts connections, reads newline-delimited JSON-RPC requests,
//! dispatches them through [`super::handlers::dispatch_with_session`],
//! and writes back newline-delimited JSON-RPC responses.
//!
//! Both UDS ([`serve`]) and TCP ([`serve_tcp`]) are supported.
//! TCP aligns with `UniBin` v1.2 `--listen addr:port`.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::envelope::JsonRpcRequest;
use super::handlers::{SharedSession, dispatch_with_session};

/// Resolve the socket path for Webb's IPC server.
///
/// Delegates to [`crate::niche::resolve_server_socket`] — the single
/// source of truth for Webb's identity and socket naming.
#[must_use]
pub fn socket_path() -> PathBuf {
    crate::niche::resolve_server_socket()
}

/// Start the Webb IPC server on a Unix domain socket.
///
/// Blocks the current thread, accepting connections and handling
/// JSON-RPC requests until the process is interrupted.
///
/// # Errors
///
/// Returns an error if the socket cannot be bound.
pub fn serve(path: &Path, session: &SharedSession) -> crate::error::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;

    listener.set_nonblocking(true)?;

    let shutdown = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))
        .map_err(|e| crate::error::WebbError::Signal(format!("register SIGINT: {e}")))?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .map_err(|e| crate::error::WebbError::Signal(format!("register SIGTERM: {e}")))?;

    let owned_path = path.to_owned();
    tracing::info!(path = %path.display(), "Webb IPC listening");

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let _ = stream.set_nonblocking(false);
                let sess = Arc::clone(session);
                std::thread::spawn(move || handle_connection(&stream, &sess));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                tracing::warn!(error = %e, "accept error");
            }
        }
    }

    tracing::info!("shutting down Webb IPC server");
    let _ = std::fs::remove_file(&owned_path);
    Ok(())
}

/// Start a TCP listener for Webb's JSON-RPC server.
///
/// `UniBin` v1.2 `--listen addr:port` support. Runs until the process is
/// interrupted.
///
/// # Errors
///
/// Returns an error if the address cannot be bound.
pub fn serve_tcp(addr: &str, session: &SharedSession) -> crate::error::Result<()> {
    let listener = std::net::TcpListener::bind(addr)?;

    listener.set_nonblocking(true)?;

    let shutdown = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))
        .map_err(|e| crate::error::WebbError::Signal(format!("register SIGINT: {e}")))?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .map_err(|e| crate::error::WebbError::Signal(format!("register SIGTERM: {e}")))?;

    tracing::info!(addr, "Webb TCP IPC listening");

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let _ = stream.set_nonblocking(false);
                let sess = Arc::clone(session);
                std::thread::spawn(move || handle_tcp_connection(&stream, &sess));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                tracing::warn!(error = %e, "TCP accept error");
            }
        }
    }

    tracing::info!("shutting down Webb TCP IPC server");
    Ok(())
}

fn handle_tcp_connection(stream: &std::net::TcpStream, session: &SharedSession) {
    let mut reader = BufReader::new(stream);
    let writer = stream;

    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() {
        return;
    }
    let first_line = first_line.trim().to_owned();

    if is_http_request(&first_line) {
        handle_http_request(&first_line, &mut reader, writer, session);
    } else {
        handle_jsonrpc_line(&first_line, writer, session);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let line = line.trim().to_owned();
            if line.is_empty() {
                continue;
            }
            handle_jsonrpc_line(&line, writer, session);
        }
    }
}

fn is_http_request(line: &str) -> bool {
    line.starts_with("POST ")
        || line.starts_with("GET ")
        || line.starts_with("PUT ")
        || line.starts_with("OPTIONS ")
}

fn handle_http_request(
    _request_line: &str,
    reader: &mut BufReader<&std::net::TcpStream>,
    mut writer: &std::net::TcpStream,
    session: &SharedSession,
) {
    let mut content_length: usize = 0;

    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return;
        }
        let header = header.trim().to_owned();
        if header.is_empty() {
            break;
        }
        if let Some(val) = header.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = header.strip_prefix("content-length:") {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }

    let body = if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        if std::io::Read::read_exact(reader, &mut buf).is_err() {
            let _ = write!(writer, "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            let _ = writer.flush();
            return;
        }
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    };

    let response = if body.is_empty() {
        super::envelope::JsonRpcResponse::error(
            super::envelope::JsonRpcError {
                code: super::envelope::ERROR_PARSE,
                message: "empty request body".to_string(),
                data: None,
            },
            serde_json::Value::Null,
        )
    } else {
        match serde_json::from_str::<JsonRpcRequest>(&body) {
            Ok(req) => dispatch_with_session(&req, session),
            Err(e) => super::envelope::JsonRpcResponse::error(
                super::envelope::JsonRpcError {
                    code: super::envelope::ERROR_PARSE,
                    message: format!("parse error: {e}"),
                    data: None,
                },
                serde_json::Value::Null,
            ),
        }
    };

    let resp_json = serde_json::to_string(&response).unwrap_or_default();
    let _ = write!(
        writer,
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         \r\n\
         {}",
        resp_json.len(),
        resp_json
    );
    let _ = writer.flush();
}

fn handle_jsonrpc_line(line: &str, mut writer: &std::net::TcpStream, session: &SharedSession) {
    if line.is_empty() {
        return;
    }
    let response = match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(req) => dispatch_with_session(&req, session),
        Err(e) => super::envelope::JsonRpcResponse::error(
            super::envelope::JsonRpcError {
                code: super::envelope::ERROR_PARSE,
                message: format!("parse error: {e}"),
                data: None,
            },
            serde_json::Value::Null,
        ),
    };

    let Ok(resp_json) = serde_json::to_string(&response) else {
        return;
    };
    let _ = writeln!(writer, "{resp_json}");
    let _ = writer.flush();
}

fn handle_connection(stream: &std::os::unix::net::UnixStream, session: &SharedSession) {
    let reader = BufReader::new(stream);
    let mut writer = stream;

    for line in reader.lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => dispatch_with_session(&req, session),
            Err(e) => super::envelope::JsonRpcResponse::error(
                super::envelope::JsonRpcError {
                    code: super::envelope::ERROR_PARSE,
                    message: format!("parse error: {e}"),
                    data: None,
                },
                serde_json::Value::Null,
            ),
        };

        let Ok(resp_json) = serde_json::to_string(&response) else {
            continue;
        };
        let _ = writeln!(writer, "{resp_json}");
        let _ = writer.flush();
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code uses unwrap for brevity")]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};

    #[test]
    fn socket_path_is_xdg_compliant() {
        let path = socket_path();
        let s = path.to_string_lossy();
        assert!(s.contains("biomeos"));
        assert!(s.ends_with("esotericwebb.sock"));
    }

    #[test]
    fn socket_path_falls_back_to_tmp_or_xdg() {
        let path = socket_path();
        let s = path.to_string_lossy();
        // XDG_RUNTIME_DIR is set: path starts there, else /tmp fallback.
        let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
        assert!(s.starts_with(&xdg));
    }

    #[test]
    fn handle_connection_returns_parse_error_for_garbage() {
        let session = super::super::handlers::new_shared_session();
        let sock_dir = std::env::temp_dir().join("esotericwebb_listener_test");
        let _ = std::fs::create_dir_all(&sock_dir);
        let sock_path = sock_dir.join("test.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = std::os::unix::net::UnixListener::bind(&sock_path).unwrap();
        let client = std::os::unix::net::UnixStream::connect(&sock_path).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        std::thread::spawn(move || {
            handle_connection(&server_stream, &session);
        });

        let mut client_writer = client.try_clone().unwrap();
        writeln!(client_writer, "not valid json").unwrap();
        client_writer.flush().unwrap();
        client_writer.shutdown(std::net::Shutdown::Write).unwrap();

        let reader = BufReader::new(&client);
        let mut response = String::new();
        reader.lines().for_each(|l| {
            if let Ok(line) = l {
                response = line;
            }
        });

        assert!(response.contains("parse error"));
        assert!(response.contains("-32700"));

        let _ = std::fs::remove_file(&sock_path);
        let _ = std::fs::remove_dir(&sock_dir);
    }

    #[test]
    fn handle_tcp_connection_valid_request() {
        let session = super::super::handlers::new_shared_session();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let client = std::net::TcpStream::connect(addr).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        std::thread::spawn(move || {
            handle_tcp_connection(&server_stream, &session);
        });

        let mut client_writer = client.try_clone().unwrap();
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "webb.health",
            "id": 1
        });
        writeln!(client_writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        client_writer.flush().unwrap();
        client_writer.shutdown(std::net::Shutdown::Write).unwrap();

        let reader = BufReader::new(&client);
        let mut response = String::new();
        for line in reader.lines().map_while(Result::ok) {
            response = line;
        }

        assert!(response.contains("healthy"));
        assert!(response.contains("2.0"));
    }

    #[test]
    fn handle_tcp_connection_parse_error() {
        let session = super::super::handlers::new_shared_session();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let client = std::net::TcpStream::connect(addr).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        std::thread::spawn(move || {
            handle_tcp_connection(&server_stream, &session);
        });

        let mut client_writer = client.try_clone().unwrap();
        writeln!(client_writer, "garbage").unwrap();
        client_writer.flush().unwrap();
        client_writer.shutdown(std::net::Shutdown::Write).unwrap();

        let reader = BufReader::new(&client);
        let mut response = String::new();
        for line in reader.lines().map_while(Result::ok) {
            response = line;
        }

        assert!(response.contains("parse error"));
    }

    #[test]
    fn handle_tcp_connection_empty_lines_ignored() {
        let session = super::super::handlers::new_shared_session();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let client = std::net::TcpStream::connect(addr).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        std::thread::spawn(move || {
            handle_tcp_connection(&server_stream, &session);
        });

        let mut client_writer = client.try_clone().unwrap();
        writeln!(client_writer).unwrap();
        writeln!(client_writer).unwrap();
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "identity.get",
            "id": 2
        });
        writeln!(client_writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        client_writer.flush().unwrap();
        client_writer.shutdown(std::net::Shutdown::Write).unwrap();

        let reader = BufReader::new(&client);
        let mut response = String::new();
        for line in reader.lines().map_while(Result::ok) {
            if !line.is_empty() {
                response = line;
            }
        }

        assert!(response.contains("esotericwebb"));
    }

    #[test]
    fn handle_connection_valid_request() {
        let session = super::super::handlers::new_shared_session();
        let sock_dir = std::env::temp_dir().join("esotericwebb_listener_test2");
        let _ = std::fs::create_dir_all(&sock_dir);
        let sock_path = sock_dir.join("valid.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = std::os::unix::net::UnixListener::bind(&sock_path).unwrap();
        let client = std::os::unix::net::UnixStream::connect(&sock_path).unwrap();
        let (server_stream, _) = listener.accept().unwrap();

        std::thread::spawn(move || {
            handle_connection(&server_stream, &session);
        });

        let mut client_writer = client.try_clone().unwrap();
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "webb.health",
            "id": 1
        });
        writeln!(client_writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        client_writer.flush().unwrap();
        client_writer.shutdown(std::net::Shutdown::Write).unwrap();

        let reader = BufReader::new(&client);
        let mut response = String::new();
        for line in reader.lines().map_while(Result::ok) {
            response = line;
        }

        assert!(response.contains("healthy"));

        let _ = std::fs::remove_file(&sock_path);
        let _ = std::fs::remove_dir(&sock_dir);
    }
}
