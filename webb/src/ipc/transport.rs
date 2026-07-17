// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured transport endpoint — ecosystem wire format (Wave 107).
//!
//! Matches the format returned by songBird `capability.resolve` and `ipc.resolve`.
//! Decoupled from discovery logic so other modules can use the type without
//! pulling in filesystem probing.

use serde::{Deserialize, Serialize};

/// Structured transport endpoint — ecosystem wire format (Wave 107).
///
/// Matches the format returned by songBird `capability.resolve` and `ipc.resolve`:
/// - UDS: `{"transport":"uds","path":"/run/user/1000/biomeos/dag.sock"}`
/// - TCP: `{"transport":"tcp","host":"127.0.0.1","port":9401}`
/// - Mesh relay: `{"transport":"mesh_relay","peer_id":"eastGate","relay":"157.230.3.183:7700"}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum TransportEndpoint {
    /// Unix domain socket.
    Uds {
        /// Absolute filesystem path to the socket.
        path: String,
    },
    /// TCP connection.
    Tcp {
        /// Hostname or IP address.
        host: String,
        /// Port number.
        port: u16,
    },
    /// Mesh relay via songBird federation.
    MeshRelay {
        /// Identity of the remote gate hosting the primal.
        peer_id: String,
        /// Relay address (songBird federation endpoint).
        relay: String,
    },
}

impl TransportEndpoint {
    /// Create a UDS endpoint from a socket path.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds { path: path.into() }
    }

    /// Create a TCP endpoint from host and port.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Create a mesh relay endpoint.
    #[must_use]
    pub fn mesh_relay(peer_id: impl Into<String>, relay: impl Into<String>) -> Self {
        Self::MeshRelay {
            peer_id: peer_id.into(),
            relay: relay.into(),
        }
    }

    /// Parse a TCP address string (host:port) into a TCP endpoint.
    #[must_use]
    pub fn from_tcp_addr(addr: &str) -> Option<Self> {
        let (host, port_str) = addr.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;
        Some(Self::tcp(host, port))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn uds_serialization() {
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/dag.sock");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "uds");
        assert_eq!(json["path"], "/run/user/1000/biomeos/dag.sock");
    }

    #[test]
    fn tcp_serialization() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 9401);
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "tcp");
        assert_eq!(json["host"], "127.0.0.1");
        assert_eq!(json["port"], 9401);
    }

    #[test]
    fn mesh_relay_serialization() {
        let ep = TransportEndpoint::mesh_relay("eastGate", "157.230.3.183:7700");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "mesh_relay");
        assert_eq!(json["peer_id"], "eastGate");
        assert_eq!(json["relay"], "157.230.3.183:7700");
    }

    #[test]
    fn deserialization_uds() {
        let json = serde_json::json!({"transport":"uds","path":"/tmp/test.sock"});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::uds("/tmp/test.sock"));
    }

    #[test]
    fn deserialization_tcp() {
        let json = serde_json::json!({"transport":"tcp","host":"10.0.0.1","port":8080});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("10.0.0.1", 8080));
    }

    #[test]
    fn deserialization_mesh_relay() {
        let json =
            serde_json::json!({"transport":"mesh_relay","peer_id":"south","relay":"vps:7700"});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::mesh_relay("south", "vps:7700"));
    }

    #[test]
    fn from_tcp_addr_parses() {
        let ep = TransportEndpoint::from_tcp_addr("127.0.0.1:9401").unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("127.0.0.1", 9401));
    }

    #[test]
    fn from_tcp_addr_rejects_bad_port() {
        assert!(TransportEndpoint::from_tcp_addr("127.0.0.1:notaport").is_none());
    }

    #[test]
    fn from_tcp_addr_rejects_no_port() {
        assert!(TransportEndpoint::from_tcp_addr("127.0.0.1").is_none());
    }
}
