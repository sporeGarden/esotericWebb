// SPDX-License-Identifier: AGPL-3.0-or-later
//! Synchronous JSON-RPC 2.0 client over Unix domain sockets or TCP.
//!
//! Pure Rust, zero async runtime required. Supports both UDS
//! (`std::os::unix::net`) and TCP (`std::net`) transports with
//! line-delimited JSON-RPC 2.0. TCP enables platform-agnostic
//! connectivity (containers, Graphene, remote hosts); UDS is the
//! traditional local-machine path.
//!
//! Same wire protocol as all ecoPrimals primals — this is Webb's own
//! implementation with no Cargo dependency on any spring or primal crate.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::envelope::{IpcError, JsonRpcRequest, JsonRpcResponse};

/// Default timeout for socket operations (5 seconds).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Monotonically increasing request ID.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Transport layer — UDS or TCP, both carrying line-delimited JSON-RPC.
enum Transport {
    Uds(BufReader<UnixStream>),
    Tcp(BufReader<TcpStream>),
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds(_) => f.write_str("Transport::Uds"),
            Self::Tcp(_) => f.write_str("Transport::Tcp"),
        }
    }
}

impl Transport {
    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Uds(r) => r.read_line(buf),
            Self::Tcp(r) => r.read_line(buf),
        }
    }

    fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Uds(r) => r.get_mut().write_all(data),
            Self::Tcp(r) => r.get_mut().write_all(data),
        }
    }
}

/// A synchronous JSON-RPC 2.0 client connected to a primal via UDS or TCP.
#[derive(Debug)]
pub struct PrimalClient {
    transport: Transport,
    primal: String,
}

impl PrimalClient {
    /// Connect to a primal via a Unix domain socket.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::ConnectionFailed`] if the socket is unreachable.
    pub fn connect(socket: &Path, primal: &str) -> Result<Self, IpcError> {
        let stream =
            UnixStream::connect(socket).map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
        stream
            .set_read_timeout(Some(DEFAULT_TIMEOUT))
            .map_err(|e| IpcError::Io(e.to_string()))?;
        stream
            .set_write_timeout(Some(DEFAULT_TIMEOUT))
            .map_err(|e| IpcError::Io(e.to_string()))?;
        Ok(Self {
            transport: Transport::Uds(BufReader::new(stream)),
            primal: primal.to_owned(),
        })
    }

    /// Connect to a primal via TCP (host:port).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::ConnectionFailed`] if the address is unreachable.
    pub fn connect_tcp(addr: &str, primal: &str) -> Result<Self, IpcError> {
        let stream =
            TcpStream::connect(addr).map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
        stream
            .set_read_timeout(Some(DEFAULT_TIMEOUT))
            .map_err(|e| IpcError::Io(e.to_string()))?;
        stream
            .set_write_timeout(Some(DEFAULT_TIMEOUT))
            .map_err(|e| IpcError::Io(e.to_string()))?;
        Ok(Self {
            transport: Transport::Tcp(BufReader::new(stream)),
            primal: primal.to_owned(),
        })
    }

    /// The primal this client is connected to.
    #[must_use]
    pub fn primal(&self) -> &str {
        &self.primal
    }

    /// Send a JSON-RPC request and read the response.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on serialization, I/O, or parse failure.
    pub fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcResponse, IpcError> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            method: method.to_owned(),
            params: Some(params),
            id: serde_json::Value::Number(serde_json::Number::from(id)),
        };

        let mut line =
            serde_json::to_string(&request).map_err(|e| IpcError::Serialization(e.to_string()))?;
        line.push('\n');

        self.transport
            .write_all(line.as_bytes())
            .map_err(|e| IpcError::Io(e.to_string()))?;

        let mut response_line = String::new();
        self.transport
            .read_line(&mut response_line)
            .map_err(|e| IpcError::Io(e.to_string()))?;

        if response_line.is_empty() {
            return Err(IpcError::Io("empty response from primal".to_owned()));
        }

        serde_json::from_str::<JsonRpcResponse>(&response_line)
            .map_err(|e| IpcError::Serialization(e.to_string()))
    }

    /// Send a `health.liveness` probe with fallback chain.
    ///
    /// Tries `health.liveness`, then `health.check`, then `health`,
    /// then `{primal}.health`.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] only on transport-level failures.
    pub fn health_liveness(&mut self) -> Result<bool, IpcError> {
        let methods = [
            "health.liveness".to_owned(),
            "health.check".to_owned(),
            "health".to_owned(),
            format!("{}.health", self.primal),
        ];
        for method in &methods {
            match self.call(method, serde_json::Value::Null) {
                Ok(resp) => {
                    if resp.error.as_ref().is_some_and(|e| e.code == -32601) {
                        continue;
                    }
                    return Ok(resp.error.is_none());
                }
                Err(IpcError::Remote { code: -32601, .. }) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }

    /// Request the primal's capability list.
    ///
    /// Tries `capabilities.list`, then `capability.list`, then `primal.capabilities`.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if all method names fail.
    pub fn capabilities(&mut self) -> Result<serde_json::Value, IpcError> {
        let methods = [
            "capabilities.list",
            "capability.list",
            "primal.capabilities",
        ];
        let mut last_err = None;
        for method in methods {
            match self.call(method, serde_json::Value::Null) {
                Ok(resp) => {
                    if resp.error.as_ref().is_some_and(|e| e.code == -32601) {
                        last_err = Some(IpcError::Remote {
                            code: -32601,
                            message: "method not found".to_owned(),
                        });
                        continue;
                    }
                    if let Some(err) = resp.error {
                        return Err(IpcError::Remote {
                            code: err.code,
                            message: err.message,
                        });
                    }
                    return Ok(resp.result.unwrap_or(serde_json::Value::Null));
                }
                Err(IpcError::Remote { code: -32601, .. }) => {
                    last_err = Some(IpcError::Remote {
                        code: -32601,
                        message: "method not found".to_owned(),
                    });
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| IpcError::PrimalNotFound("capabilities.list".to_owned())))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::os::unix::net::UnixListener;

    #[test]
    fn connect_fails_for_nonexistent_socket() {
        let result = PrimalClient::connect(Path::new("/nonexistent/socket.sock"), "test");
        assert!(result.is_err());
    }

    #[test]
    fn connect_tcp_fails_for_unreachable_addr() {
        let result = PrimalClient::connect_tcp("127.0.0.1:1", "test");
        assert!(result.is_err());
    }

    #[test]
    fn round_trip_with_mock_server() {
        let dir = std::env::temp_dir().join("esotericwebb-test");
        std::fs::create_dir_all(&dir).unwrap();
        let sock_path = dir.join("test-roundtrip.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).unwrap();

        let sock_clone = sock_path.clone();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];

            let response =
                format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"status\":\"ok\"}},\"id\":{id}}}\n");
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect(&sock_clone, "test").unwrap();
        let resp = client
            .call("health.check", serde_json::Value::Null)
            .unwrap();
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());

        server.join().unwrap();
        let _ = std::fs::remove_file(&sock_path);
    }

    #[test]
    fn tcp_round_trip_with_mock_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];

            let response =
                format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"alive\":true}},\"id\":{id}}}\n");
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let resp = client
            .call("health.check", serde_json::Value::Null)
            .unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["alive"], true);

        server.join().unwrap();
    }

    #[test]
    fn health_liveness_fallback_chain() {
        let dir = std::env::temp_dir().join("esotericwebb-test");
        std::fs::create_dir_all(&dir).unwrap();
        let sock_path = dir.join("test-health-fallback.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).unwrap();

        let sock_clone = sock_path.clone();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);

            for i in 0..4 {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                let id = &req["id"];
                let method = req["method"].as_str().unwrap_or("");

                let response = if i < 3 && method != "test.health" {
                    format!(
                        "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32601,\"message\":\"not found\"}},\"id\":{id}}}\n"
                    )
                } else {
                    format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"alive\":true}},\"id\":{id}}}\n")
                };
                (&stream).write_all(response.as_bytes()).unwrap();
            }
        });

        let mut client = PrimalClient::connect(&sock_clone, "test").unwrap();
        let result = client.health_liveness().unwrap();
        assert!(result);

        server.join().unwrap();
        let _ = std::fs::remove_file(&sock_path);
    }

    #[test]
    fn tcp_health_liveness_works() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];
            let response =
                format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"alive\":true}},\"id\":{id}}}\n");
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let result = client.health_liveness().unwrap();
        assert!(result);

        server.join().unwrap();
    }
}
