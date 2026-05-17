// SPDX-License-Identifier: AGPL-3.0-or-later
//! Synchronous JSON-RPC 2.0 client over Unix domain sockets or TCP.
//!
//! Pure Rust, zero async runtime required. Supports both UDS
//! (`std::os::unix::net`) and TCP (`std::net`) transports with
//! line-delimited JSON-RPC 2.0.
//!
//! Transport priority configurable via `ESOTERICWEBB_TRANSPORT_PRIORITY`
//! environment variable (`tcp` default for platform portability, `uds`
//! for biomeOS-first deployments).
//!
//! Same wire protocol as all ecoPrimals primals — this is Webb's own
//! implementation with no Cargo dependency on any spring or primal crate.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::envelope::{IpcError, JsonRpcRequest, JsonRpcResponse, classify_io_error};

/// Default timeout for socket operations.
///
/// Overridable via `ESOTERICWEBB_IPC_TIMEOUT_SECS` environment variable.
fn default_timeout() -> Duration {
    let secs = std::env::var("ESOTERICWEBB_IPC_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5);
    Duration::from_secs(secs)
}

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

    /// Whether this transport is a TCP socket.
    #[cfg(test)]
    const fn is_tcp(&self) -> bool {
        matches!(self, Self::Tcp(_))
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
    /// Returns [`IpcError::ConnectionRefused`] if the socket is unreachable.
    pub fn connect(socket: &Path, primal: &str) -> Result<Self, IpcError> {
        let stream = UnixStream::connect(socket).map_err(|e| classify_io_error(&e))?;
        let timeout = default_timeout();
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| classify_io_error(&e))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| classify_io_error(&e))?;
        Ok(Self {
            transport: Transport::Uds(BufReader::new(stream)),
            primal: primal.to_owned(),
        })
    }

    /// Connect to a primal via TCP (host:port).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::ConnectionRefused`] if the address is unreachable.
    pub fn connect_tcp(addr: &str, primal: &str) -> Result<Self, IpcError> {
        let stream = TcpStream::connect(addr).map_err(|e| classify_io_error(&e))?;
        let timeout = default_timeout();
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| classify_io_error(&e))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| classify_io_error(&e))?;
        Ok(Self {
            transport: Transport::Tcp(BufReader::new(stream)),
            primal: primal.to_owned(),
        })
    }

    /// Connect using a transport address string.
    ///
    /// Supports:
    /// - `unix:/path/to/socket.sock` — Unix domain socket
    /// - `tcp:127.0.0.1:9100` — TCP socket
    /// - `/path/to/socket.sock` — implicit Unix socket (path starts with `/`)
    /// - `127.0.0.1:9100` — implicit TCP (anything else)
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the address cannot be parsed or connection fails.
    pub fn connect_transport(address: &str, primal: &str) -> Result<Self, IpcError> {
        address.strip_prefix("unix:").map_or_else(
            || {
                address.strip_prefix("tcp:").map_or_else(
                    || {
                        if address.starts_with('/') {
                            Self::connect(Path::new(address), primal)
                        } else {
                            Self::connect_tcp(address, primal)
                        }
                    },
                    |addr| Self::connect_tcp(addr, primal),
                )
            },
            |path| Self::connect(Path::new(path), primal),
        )
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

        let mut line = serde_json::to_string(&request).map_err(|e| IpcError::Serialization {
            detail: e.to_string(),
        })?;
        line.push('\n');

        self.transport
            .write_all(line.as_bytes())
            .map_err(|e| classify_io_error(&e))?;

        let mut response_line = String::new();
        self.transport
            .read_line(&mut response_line)
            .map_err(|e| classify_io_error(&e))?;

        if response_line.is_empty() {
            return Err(IpcError::ProtocolError {
                detail: "empty response from primal".to_owned(),
            });
        }

        serde_json::from_str::<JsonRpcResponse>(&response_line).map_err(|e| {
            IpcError::ProtocolError {
                detail: e.to_string(),
            }
        })
    }

    /// Send a `health.liveness` probe with fallback chain.
    ///
    /// Tries `health.liveness`, then `health.check`, then `health`,
    /// then `{primal}.health`. Aligns with `SEMANTIC_METHOD_NAMING_STANDARD`
    /// v2.2 fallback convention.
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
                    if resp
                        .error
                        .as_ref()
                        .is_some_and(|e| e.code == super::envelope::ERROR_METHOD_NOT_FOUND)
                    {
                        continue;
                    }
                    return Ok(resp.error.is_none());
                }
                Err(IpcError::MethodNotFound { .. }) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }

    /// Request the primal's capability list.
    ///
    /// Tries `capabilities.list`, then `capability.list`, then `primal.capabilities`.
    /// Unwraps the Wave 20 canonical envelope `{ capabilities, count, primal }`
    /// and falls back to raw array responses from pre-Wave-20 primals.
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
                    if resp
                        .error
                        .as_ref()
                        .is_some_and(|e| e.code == super::envelope::ERROR_METHOD_NOT_FOUND)
                    {
                        last_err = Some(IpcError::MethodNotFound {
                            method: method.to_owned(),
                        });
                        continue;
                    }
                    if let Some(err) = resp.error {
                        return Err(IpcError::from(err));
                    }
                    let raw = resp.result.unwrap_or(serde_json::Value::Null);
                    return Ok(unwrap_capabilities_envelope(raw));
                }
                Err(IpcError::MethodNotFound { .. }) => {
                    last_err = Some(IpcError::MethodNotFound {
                        method: method.to_owned(),
                    });
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| IpcError::PrimalNotFound {
            domain: "capabilities.list".to_owned(),
        }))
    }
}

/// Unwrap the Wave 20 canonical envelope `{ capabilities, count, primal }`.
///
/// If the response is the canonical shape, returns it as-is. If the response
/// is a raw array (pre-Wave-20), wraps it in the canonical envelope. This
/// ensures consumers always see a consistent shape.
fn unwrap_capabilities_envelope(value: serde_json::Value) -> serde_json::Value {
    if value
        .get("capabilities")
        .is_some_and(serde_json::Value::is_array)
    {
        if value.get("count").is_some() {
            return value;
        }
        let count = value["capabilities"].as_array().map_or(0, Vec::len);
        let primal = value
            .get("primal")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        return serde_json::json!({
            "capabilities": value["capabilities"],
            "count": count,
            "primal": primal,
        });
    }
    if value.is_array() {
        let count = value.as_array().map_or(0, Vec::len);
        return serde_json::json!({
            "capabilities": value,
            "count": count,
            "primal": serde_json::Value::Null,
        });
    }
    value
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::os::unix::net::UnixListener;

    #[test]
    fn connect_fails_for_nonexistent_socket() {
        let result = PrimalClient::connect(Path::new("/nonexistent/socket.sock"), "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_connection_error());
    }

    #[test]
    fn connect_tcp_fails_for_unreachable_addr() {
        let result = PrimalClient::connect_tcp("127.0.0.1:1", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_connection_error());
    }

    #[test]
    fn connect_transport_tcp_implicit() {
        let result = PrimalClient::connect_transport("127.0.0.1:1", "test");
        assert!(result.is_err());
    }

    #[test]
    fn connect_transport_tcp_explicit() {
        let result = PrimalClient::connect_transport("tcp:127.0.0.1:1", "test");
        assert!(result.is_err());
    }

    #[test]
    fn connect_transport_unix_implicit() {
        let result = PrimalClient::connect_transport("/nonexistent/socket.sock", "test");
        assert!(result.is_err());
    }

    #[test]
    fn connect_transport_unix_explicit() {
        let result = PrimalClient::connect_transport("unix:/nonexistent/socket.sock", "test");
        assert!(result.is_err());
    }

    #[test]
    fn connect_transport_tcp_creates_tcp_transport() {
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
                format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"ok\":true}},\"id\":{id}}}\n");
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_transport(&addr, "test").unwrap();
        assert!(client.transport.is_tcp());
        let resp = client
            .call("health.check", serde_json::Value::Null)
            .unwrap();
        assert!(resp.is_success());

        server.join().unwrap();
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

    #[test]
    fn primal_accessor_returns_name() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let _server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];
            let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":null,\"id\":{id}}}\n");
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let client = PrimalClient::connect_tcp(&addr, "myprimal").unwrap();
        assert_eq!(client.primal(), "myprimal");
    }

    #[test]
    fn capabilities_returns_list() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];
            let response = format!(
                "{{\"jsonrpc\":\"2.0\",\"result\":{{\"methods\":[\"dag.query\",\"dag.append\"]}},\"id\":{id}}}\n"
            );
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let caps = client.capabilities().unwrap();
        assert!(caps["methods"].is_array());

        server.join().unwrap();
    }

    #[test]
    fn capabilities_fallback_to_second_method() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);

            // First call: method not found
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];
            let response = format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32601,\"message\":\"not found\"}},\"id\":{id}}}\n"
            );
            (&stream).write_all(response.as_bytes()).unwrap();

            // Second call: success
            let mut line2 = String::new();
            reader.read_line(&mut line2).unwrap();
            let req2: serde_json::Value = serde_json::from_str(&line2).unwrap();
            let id2 = &req2["id"];
            let response2 =
                format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"caps\":[\"a\"]}},\"id\":{id2}}}\n");
            (&stream).write_all(response2.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let caps = client.capabilities().unwrap();
        assert!(caps["caps"].is_array());

        server.join().unwrap();
    }

    #[test]
    fn default_timeout_returns_positive_duration() {
        let t = default_timeout();
        assert!(t.as_secs() > 0);
    }

    #[test]
    fn transport_debug_uds() {
        let dir = std::env::temp_dir().join("esotericwebb-debug-test");
        std::fs::create_dir_all(&dir).unwrap();
        let sock_path = dir.join("debug.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).unwrap();
        let stream = UnixStream::connect(&sock_path).unwrap();
        let t = Transport::Uds(BufReader::new(stream));
        let debug = format!("{t:?}");
        assert_eq!(debug, "Transport::Uds");

        drop(listener);
        let _ = std::fs::remove_file(&sock_path);
    }

    #[test]
    fn transport_debug_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        let t = Transport::Tcp(BufReader::new(stream));
        let debug = format!("{t:?}");
        assert_eq!(debug, "Transport::Tcp");
    }

    #[test]
    fn health_liveness_all_methods_fail_returns_false() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            for _ in 0..4 {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                let id = &req["id"];
                let response = format!(
                    "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32601,\"message\":\"not found\"}},\"id\":{id}}}\n"
                );
                (&stream).write_all(response.as_bytes()).unwrap();
            }
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let result = client.health_liveness().unwrap();
        assert!(!result);

        server.join().unwrap();
    }

    #[test]
    fn health_liveness_error_response_returns_false() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req: serde_json::Value = serde_json::from_str(&line).unwrap();
            let id = &req["id"];
            let response = format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32000,\"message\":\"internal error\"}},\"id\":{id}}}\n"
            );
            (&stream).write_all(response.as_bytes()).unwrap();
        });

        let mut client = PrimalClient::connect_tcp(&addr, "test").unwrap();
        let result = client.health_liveness().unwrap();
        assert!(!result);

        server.join().unwrap();
    }
}
