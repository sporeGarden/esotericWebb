// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end IPC tests — full JSON-RPC roundtrip over Unix domain sockets.
//!
//! Tests exercise the real socket listener, handler dispatch, and session
//! lifecycle without requiring live primals. Also includes chaos and fault
//! injection scenarios.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "E2E test code")]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn test_socket_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("esotericwebb_e2e_tests");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{name}.sock"))
}

fn start_server(name: &str) -> (PathBuf, esoteric_webb::ipc::handlers::SharedSession) {
    let sock = test_socket_path(name);
    let _ = std::fs::remove_file(&sock);

    let session = esoteric_webb::ipc::handlers::new_shared_session();
    let session_clone = std::sync::Arc::clone(&session);
    let sock_clone = sock.clone();

    if let Some(parent) = sock.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let listener = std::os::unix::net::UnixListener::bind(&sock).unwrap();

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let sess = std::sync::Arc::clone(&session_clone);
                    std::thread::spawn(move || {
                        let reader = BufReader::new(&stream);
                        let mut writer = &stream;
                        for line in reader.lines() {
                            let Ok(line) = line else { break };
                            let line = line.trim().to_owned();
                            if line.is_empty() {
                                continue;
                            }
                            let response = match serde_json::from_str::<
                                esoteric_webb::ipc::envelope::JsonRpcRequest,
                            >(&line)
                            {
                                Ok(req) => {
                                    esoteric_webb::ipc::handlers::dispatch_with_session(&req, &sess)
                                }
                                Err(e) => esoteric_webb::ipc::envelope::JsonRpcResponse {
                                    jsonrpc: "2.0".to_owned(),
                                    result: None,
                                    error: Some(esoteric_webb::ipc::envelope::JsonRpcError {
                                        code: -32700,
                                        message: format!("parse error: {e}"),
                                        data: None,
                                    }),
                                    id: serde_json::Value::Null,
                                },
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = writeln!(writer, "{json}");
                                let _ = writer.flush();
                            }
                        }
                    });
                }
                Err(_) => break,
            }
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(50));
    (sock_clone, session)
}

fn rpc_call(
    stream: &UnixStream,
    method: &str,
    params: Option<&serde_json::Value>,
) -> serde_json::Value {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut writer = stream;
    writeln!(writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
    writer.flush().unwrap();

    let reader = BufReader::new(stream);
    let line = reader.lines().next().unwrap().unwrap();
    serde_json::from_str(&line).unwrap()
}

// ── E2E: Full roundtrip ─────────────────────────────────────

#[test]
fn e2e_health_over_socket() {
    let (sock, _session) = start_server("e2e_health");
    let stream = UnixStream::connect(&sock).unwrap();
    let resp = rpc_call(&stream, "webb.health", None);

    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["error"].is_null());
    assert_eq!(resp["result"]["status"], "healthy");

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn e2e_identity_over_socket() {
    let (sock, _session) = start_server("e2e_identity");
    let stream = UnixStream::connect(&sock).unwrap();
    let resp = rpc_call(&stream, "identity.get", None);

    assert_eq!(resp["result"]["primal"], "esotericwebb");

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn e2e_capabilities_over_socket() {
    let (sock, _session) = start_server("e2e_caps");
    let stream = UnixStream::connect(&sock).unwrap();
    let resp = rpc_call(&stream, "capabilities.list", None);

    assert!(resp["result"]["capabilities"].is_array());
    assert!(
        !resp["result"]["capabilities"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn e2e_session_lifecycle_over_socket() {
    let (sock, _session) = start_server("e2e_session");
    let stream = UnixStream::connect(&sock).unwrap();

    let state_before = rpc_call(&stream, "session.state", None);
    assert!(
        state_before["error"].is_object(),
        "should error without session"
    );

    let params = serde_json::json!({"content_path": "content"});
    let start = rpc_call(&stream, "session.start", Some(&params));
    if start["error"].is_null() {
        let state = rpc_call(&stream, "session.state", None);
        assert!(state["error"].is_null());
        assert!(state["result"]["session_active"].as_bool().unwrap_or(false));

        let actions = rpc_call(&stream, "session.actions", None);
        assert!(actions["error"].is_null());
        assert!(actions["result"]["actions"].is_array());
    }

    let _ = std::fs::remove_file(&sock);
}

// ── Chaos / fault injection ─────────────────────────────────

#[test]
fn chaos_garbage_input() {
    let (sock, _session) = start_server("chaos_garbage");
    let stream = UnixStream::connect(&sock).unwrap();
    let mut writer = &stream;

    writeln!(writer, "not json at all").unwrap();
    writer.flush().unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(resp["error"]["code"], -32700);

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_empty_lines_ignored() {
    let (sock, _session) = start_server("chaos_empty");
    let stream = UnixStream::connect(&sock).unwrap();
    let mut writer = stream.try_clone().unwrap();

    writeln!(writer).unwrap();
    writeln!(writer, "   ").unwrap();
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "webb.health",
        "id": 1
    });
    writeln!(writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
    writer.flush().unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert!(resp["error"].is_null());

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_unknown_method() {
    let (sock, _session) = start_server("chaos_unknown");
    let stream = UnixStream::connect(&sock).unwrap();
    let resp = rpc_call(&stream, "nonexistent.rpc.method", None);

    assert_eq!(resp["error"]["code"], -32601);
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not found")
    );

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_missing_jsonrpc_field() {
    let (sock, _session) = start_server("chaos_nover");
    let stream = UnixStream::connect(&sock).unwrap();
    let mut writer = &stream;

    let malformed = serde_json::json!({"method": "webb.health", "id": 1});
    writeln!(writer, "{}", serde_json::to_string(&malformed).unwrap()).unwrap();
    writer.flush().unwrap();

    let reader = BufReader::new(&stream);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert!(
        resp["error"]["code"] == -32700 || resp["result"].is_object(),
        "should either parse-error or handle gracefully"
    );

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_rapid_reconnection() {
    let (sock, _session) = start_server("chaos_rapid");

    for i in 0..5 {
        let stream = UnixStream::connect(&sock).unwrap();
        let resp = rpc_call(&stream, "webb.health", None);
        assert!(resp["error"].is_null(), "connection {i} should succeed");
        drop(stream);
    }

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_multiple_requests_per_connection() {
    let (sock, _session) = start_server("chaos_multi");
    let stream = UnixStream::connect(&sock).unwrap();
    let mut writer = stream.try_clone().unwrap();
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    for id in 1..=3 {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "webb.health",
            "id": id
        });
        writeln!(writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        writer.flush().unwrap();

        let line = lines.next().unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert!(resp["error"].is_null());
        assert_eq!(resp["id"], id);
    }

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_session_act_without_start() {
    let (sock, _session) = start_server("chaos_nostart");
    let stream = UnixStream::connect(&sock).unwrap();

    let params = serde_json::json!({"kind": "exit", "id": "room"});
    let resp = rpc_call(&stream, "session.act", Some(&params));
    assert!(resp["error"].is_object());

    let _ = std::fs::remove_file(&sock);
}

#[test]
fn chaos_tools_call_unknown_tool() {
    let (sock, _session) = start_server("chaos_badtool");
    let stream = UnixStream::connect(&sock).unwrap();

    let params = serde_json::json!({"name": "nonexistent.tool"});
    let resp = rpc_call(&stream, "tools.call", Some(&params));
    assert!(resp["error"].is_object());

    let _ = std::fs::remove_file(&sock);
}

// ── TCP E2E tests ───────────────────────────────────────────

fn start_tcp_server() -> (
    std::net::SocketAddr,
    esoteric_webb::ipc::handlers::SharedSession,
) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let session = esoteric_webb::ipc::handlers::new_shared_session();
    let session_clone = std::sync::Arc::clone(&session);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let sess = std::sync::Arc::clone(&session_clone);
                    std::thread::spawn(move || {
                        let reader = BufReader::new(&stream);
                        let mut writer = &stream;
                        for line in reader.lines() {
                            let Ok(line) = line else { break };
                            let line = line.trim().to_owned();
                            if line.is_empty() {
                                continue;
                            }
                            let response = match serde_json::from_str::<
                                esoteric_webb::ipc::envelope::JsonRpcRequest,
                            >(&line)
                            {
                                Ok(req) => {
                                    esoteric_webb::ipc::handlers::dispatch_with_session(&req, &sess)
                                }
                                Err(e) => esoteric_webb::ipc::envelope::JsonRpcResponse {
                                    jsonrpc: "2.0".to_owned(),
                                    result: None,
                                    error: Some(esoteric_webb::ipc::envelope::JsonRpcError {
                                        code: -32700,
                                        message: format!("parse error: {e}"),
                                        data: None,
                                    }),
                                    id: serde_json::Value::Null,
                                },
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = writeln!(writer, "{json}");
                                let _ = writer.flush();
                            }
                        }
                    });
                }
                Err(_) => break,
            }
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(50));
    (addr, session)
}

fn tcp_rpc_call(
    stream: &std::net::TcpStream,
    method: &str,
    params: Option<&serde_json::Value>,
) -> serde_json::Value {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut writer = stream;
    writeln!(writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
    writer.flush().unwrap();

    let reader = BufReader::new(stream);
    let line = reader.lines().next().unwrap().unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn e2e_tcp_health() {
    let (addr, _session) = start_tcp_server();
    let stream = std::net::TcpStream::connect(addr).unwrap();
    let resp = tcp_rpc_call(&stream, "webb.health", None);

    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp["error"].is_null());
    assert_eq!(resp["result"]["status"], "healthy");
}

#[test]
fn e2e_tcp_identity() {
    let (addr, _session) = start_tcp_server();
    let stream = std::net::TcpStream::connect(addr).unwrap();
    let resp = tcp_rpc_call(&stream, "identity.get", None);

    assert_eq!(resp["result"]["primal"], "esotericwebb");
}

#[test]
fn e2e_tcp_capabilities() {
    let (addr, _session) = start_tcp_server();
    let stream = std::net::TcpStream::connect(addr).unwrap();
    let resp = tcp_rpc_call(&stream, "capabilities.list", None);

    assert!(resp["result"]["capabilities"].is_array());
    assert!(
        !resp["result"]["capabilities"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn e2e_tcp_multiple_requests() {
    let (addr, _session) = start_tcp_server();
    let stream = std::net::TcpStream::connect(addr).unwrap();
    let mut writer = stream.try_clone().unwrap();
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    for id in 1..=3 {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "webb.health",
            "id": id
        });
        writeln!(writer, "{}", serde_json::to_string(&req).unwrap()).unwrap();
        writer.flush().unwrap();

        let line = lines.next().unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert!(resp["error"].is_null());
        assert_eq!(resp["id"], id);
    }
}

#[test]
fn e2e_tcp_session_lifecycle() {
    let (addr, _session) = start_tcp_server();
    let stream = std::net::TcpStream::connect(addr).unwrap();

    let state_before = tcp_rpc_call(&stream, "session.state", None);
    assert!(
        state_before["error"].is_object(),
        "should error without session"
    );
}

// ── Capability registry cross-validation ────────────────────

#[test]
fn capability_registry_methods_all_dispatch() {
    let (sock, _session) = start_server("cap_registry");
    let stream = UnixStream::connect(&sock).unwrap();

    let registry_toml = include_str!("../capability_registry.toml");
    let registry: toml::Value = toml::from_str(registry_toml).unwrap();

    let capabilities = registry["capabilities"]
        .as_array()
        .expect("capabilities should be an array");

    for cap in capabilities {
        let method = cap["method"].as_str().unwrap();
        let resp = rpc_call(&stream, method, None);

        let is_method_not_found = resp["error"]["code"] == -32601;
        assert!(
            !is_method_not_found,
            "registered method '{method}' returned method-not-found — \
             capability_registry.toml and handler dispatch are out of sync"
        );
    }

    let _ = std::fs::remove_file(&sock);
}
