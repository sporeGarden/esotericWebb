// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix domain socket listener for Webb's JSON-RPC server.
//!
//! Accepts connections, reads newline-delimited JSON-RPC requests,
//! dispatches them through [`super::server::dispatch_with_session`],
//! and writes back newline-delimited JSON-RPC responses.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::envelope::JsonRpcRequest;
use super::server::{SharedSession, dispatch_with_session};

/// Resolve the XDG-compliant socket path for Webb.
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

    eprintln!("Webb IPC listening on {}", path.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let sess = Arc::clone(session);
                std::thread::spawn(move || handle_connection(stream, &sess));
            }
            Err(e) => {
                eprintln!("accept error: {e}");
            }
        }
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)] // owned: thread boundary
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
