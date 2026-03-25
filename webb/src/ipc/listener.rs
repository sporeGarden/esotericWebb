// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix domain socket listener for Webb's JSON-RPC server.
//!
//! Accepts connections, reads newline-delimited JSON-RPC requests,
//! dispatches them through [`super::handlers::dispatch_with_session`],
//! and writes back newline-delimited JSON-RPC responses.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::envelope::JsonRpcRequest;
use super::handlers::{SharedSession, dispatch_with_session};

/// Resolve the XDG-compliant socket path for Webb.
#[must_use]
pub fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    Path::new(&runtime_dir)
        .join("biomeos")
        .join("esotericwebb.sock")
}

/// Start the Webb IPC server on a Unix domain socket.
///
/// Blocks the current thread, accepting connections and handling
/// JSON-RPC requests until the process is interrupted.
///
/// # Errors
///
/// Returns an error if the socket cannot be bound.
pub fn serve(path: &Path, session: &SharedSession) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create socket dir: {e}"))?;
    }

    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("remove stale socket: {e}"))?;
    }

    let listener =
        UnixListener::bind(path).map_err(|e| format!("bind socket {}: {e}", path.display()))?;

    listener
        .set_nonblocking(true)
        .map_err(|e| format!("set non-blocking: {e}"))?;

    let shutdown = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))
        .map_err(|e| format!("register SIGINT handler: {e}"))?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .map_err(|e| format!("register SIGTERM handler: {e}"))?;

    let owned_path = path.to_owned();
    eprintln!("Webb IPC listening on {}", path.display());

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let _ = stream.set_nonblocking(false);
                let sess = Arc::clone(session);
                std::thread::spawn(move || handle_connection(stream, &sess));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("accept error: {e}");
            }
        }
    }

    eprintln!("Shutting down Webb IPC server...");
    let _ = std::fs::remove_file(&owned_path);
    Ok(())
}

#[allow(clippy::needless_pass_by_value)] // thread::spawn requires owned values
fn handle_connection(stream: std::os::unix::net::UnixStream, session: &SharedSession) {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => dispatch_with_session(&req, session),
            Err(e) => super::envelope::JsonRpcResponse {
                jsonrpc: "2.0".to_owned(),
                result: None,
                error: Some(super::envelope::JsonRpcError {
                    code: -32700,
                    message: format!("parse error: {e}"),
                    data: None,
                }),
                id: serde_json::Value::Null,
            },
        };

        let Ok(resp_json) = serde_json::to_string(&response) else {
            continue;
        };
        let _ = writeln!(writer, "{resp_json}");
        let _ = writer.flush();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            handle_connection(server_stream, &session);
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
}
