// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability-based primal discovery.
//!
//! Discovers live primals by probing Unix sockets in XDG-compliant
//! directories **and** TCP addresses from environment variables and
//! `plasmidBin/` metadata.
//!
//! Primals are found by **capability**, never by hardcoded name or path.
//! Primal binaries are expected in `ecoPrimals/plasmidBin/` — this module
//! discovers their running sockets or TCP addresses, not their source.
//!
//! Discovery strategy:
//!
//! **UDS** (4-tier implemented, per wateringHole `UNIVERSAL_IPC_STANDARD_V3`):
//! 1. `$BIOMEOS_SOCKET_DIR/<domain>.sock`
//! 2. `$XDG_RUNTIME_DIR/biomeos/<domain>.sock`
//! 3. `/run/user/<uid>/biomeos/<domain>.sock`
//! 4. `/tmp/biomeos-<uid>/<domain>.sock`
//!
//! Tier 5 (Songbird `discovery.query`) is planned but not yet implemented
//! — see `EVOLUTION_GAPS.md` GAP-006.
//!
//! **TCP** (checked first, highest priority):
//! 1. `<PRIMAL>_ADDRESS` env var (e.g. `RHIZOCRYPT_ADDRESS=127.0.0.1:9401`)
//! 2. `<PRIMAL>_JSONRPC_PORT` env var → `127.0.0.1:<port>`
//! 3. `plasmidBin/<primal>/metadata.toml` `[transport]` section

use std::collections::HashMap;
use std::path::PathBuf;

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

/// A discovered primal endpoint (UDS and/or TCP).
#[derive(Debug, Clone)]
pub struct PrimalEndpoint {
    /// The domain this primal serves (e.g. "game", "ai", "dag").
    pub domain: String,
    /// Primal name (e.g. "rhizocrypt", "squirrel").
    pub name: String,
    /// Filesystem path to the Unix socket (if found).
    pub socket_path: Option<PathBuf>,
    /// TCP address (host:port) if known.
    pub tcp_addr: Option<String>,
    /// Capabilities advertised by this primal.
    pub capabilities: Vec<String>,
    /// Whether the primal responded to a health check.
    pub healthy: bool,
}

impl PrimalEndpoint {
    /// Resolve the best `TransportEndpoint` for this primal.
    ///
    /// Priority: UDS (lowest latency) > TCP (wider platform support).
    /// Returns `None` if no transport is available.
    #[must_use]
    pub fn resolve_transport(&self) -> Option<TransportEndpoint> {
        if let Some(ref path) = self.socket_path {
            return Some(TransportEndpoint::uds(path.to_string_lossy()));
        }
        if let Some(ref addr) = self.tcp_addr {
            return TransportEndpoint::from_tcp_addr(addr);
        }
        None
    }

    /// Return all available transports (may be empty, 1, or 2).
    #[must_use]
    pub fn available_transports(&self) -> Vec<TransportEndpoint> {
        let mut transports = Vec::with_capacity(2);
        if let Some(ref path) = self.socket_path {
            transports.push(TransportEndpoint::uds(path.to_string_lossy()));
        }
        if let Some(ref addr) = self.tcp_addr {
            if let Some(ep) = TransportEndpoint::from_tcp_addr(addr) {
                transports.push(ep);
            }
        }
        transports
    }
}

/// Registry of discovered primals, keyed by domain.
#[derive(Debug, Default)]
pub struct PrimalRegistry {
    /// Endpoints indexed by domain.
    pub by_domain: HashMap<String, PrimalEndpoint>,
    /// Capability to domain mapping.
    pub capability_index: HashMap<String, String>,
}

use super::primal_names::DOMAIN_PRIMAL_MAP;

impl PrimalRegistry {
    /// Discover primals from TCP env vars, `plasmidBin/` metadata, and UDS socket directories.
    #[must_use]
    pub fn discover() -> Self {
        let mut registry = Self::default();

        // Phase 1: TCP discovery from env vars
        registry.discover_tcp_from_env();

        // Phase 2: TCP/metadata discovery from plasmidBin
        registry.discover_from_plasmid_bin();

        // Phase 3: UDS socket directory scan (fills in socket_path for existing
        // endpoints or creates new ones for primals only reachable via UDS)
        for dir in socket_directories() {
            if dir.is_dir() {
                registry.probe_directory(&dir);
            }
        }

        registry
    }

    /// Find the endpoint that provides a given capability.
    #[must_use]
    pub fn find_by_capability(&self, capability: &str) -> Option<&PrimalEndpoint> {
        self.capability_index
            .get(capability)
            .and_then(|domain| self.by_domain.get(domain))
    }

    /// Check environment variables for TCP addresses.
    ///
    /// Patterns checked per primal:
    /// - `<PRIMAL>_ADDRESS` (e.g. `RHIZOCRYPT_ADDRESS=127.0.0.1:9401`)
    /// - `<PRIMAL>_JSONRPC_PORT` (e.g. `RHIZOCRYPT_JSONRPC_PORT=9401`)
    /// - `<PRIMAL>_HTTP_ADDRESS` (e.g. `SWEETGRASS_HTTP_ADDRESS=127.0.0.1:9403`)
    fn discover_tcp_from_env(&mut self) {
        for &(domain, name) in DOMAIN_PRIMAL_MAP {
            let upper = name.to_uppercase();

            let addr = std::env::var(format!("{upper}{}", crate::env_keys::ADDR_SUFFIX))
                .ok()
                .or_else(|| {
                    std::env::var(format!("{upper}{}", crate::env_keys::PORT_SUFFIX))
                        .ok()
                        .map(super::host_port)
                })
                .or_else(|| {
                    std::env::var(format!("{upper}{}", crate::env_keys::HTTP_ADDR_SUFFIX)).ok()
                });

            if let Some(tcp_addr) = addr {
                let ep =
                    self.by_domain
                        .entry(domain.to_owned())
                        .or_insert_with(|| PrimalEndpoint {
                            domain: domain.to_owned(),
                            name: name.to_owned(),
                            socket_path: None,
                            tcp_addr: None,
                            capabilities: Vec::new(),
                            healthy: false,
                        });
                ep.tcp_addr = Some(tcp_addr);
            }
        }
    }

    /// Scan `plasmidBin/` metadata for transport hints.
    fn discover_from_plasmid_bin(&mut self) {
        let plasmidbins = plasmid_bin_directories();
        for base in &plasmidbins {
            if !base.is_dir() {
                continue;
            }
            let Ok(entries) = std::fs::read_dir(base) else {
                continue;
            };
            for entry in entries.flatten() {
                let meta_path = entry.path().join("metadata.toml");
                if meta_path.is_file() {
                    self.ingest_metadata(&meta_path);
                }
            }
        }
    }

    /// Parse a single `metadata.toml` and merge transport/capability info.
    fn ingest_metadata(&mut self, path: &std::path::Path) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(table) = contents.parse::<toml::Table>() else {
            return;
        };

        let primal_section = table.get("primal").and_then(toml::Value::as_table);
        let caps_section = table.get("capabilities").and_then(toml::Value::as_table);

        let name = primal_section
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        let domain = caps_section
            .and_then(|c| c.get("domain"))
            .and_then(toml::Value::as_str)
            .unwrap_or("");

        if name.is_empty() || domain.is_empty() {
            return;
        }

        let methods: Vec<String> = caps_section
            .and_then(|c| c.get("methods"))
            .and_then(toml::Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();

        // Check for [transport] section (future-proofing for explicit TCP hints)
        let transport_section = table.get("transport").and_then(toml::Value::as_table);
        let tcp_addr = transport_section
            .and_then(|t| t.get("tcp_address"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                transport_section
                    .and_then(|t| t.get("port"))
                    .and_then(toml::Value::as_integer)
                    .map(super::host_port)
            });

        let ep = self
            .by_domain
            .entry(domain.to_owned())
            .or_insert_with(|| PrimalEndpoint {
                domain: domain.to_owned(),
                name: name.to_owned(),
                socket_path: None,
                tcp_addr: None,
                capabilities: Vec::new(),
                healthy: false,
            });

        if ep.tcp_addr.is_none() {
            ep.tcp_addr = tcp_addr;
        }

        if ep.capabilities.is_empty() {
            ep.capabilities.clone_from(&methods);
        }

        for method in &methods {
            self.capability_index
                .entry(method.clone())
                .or_insert_with(|| domain.to_owned());
        }
    }

    /// Probe a directory for `.sock` files and merge into existing endpoints.
    ///
    /// Resolves the socket file stem in two passes:
    /// 1. Match as a **domain** name (`dag.sock` -> domain "dag")
    /// 2. Match as a **primal slug** reverse-mapped through `DOMAIN_PRIMAL_MAP`
    ///    (`rhizocrypt.sock` -> primal "rhizocrypt" -> domain "dag")
    ///
    /// This handles the ecosystem reality where some primals register
    /// domain-named sockets and others register primal-named sockets.
    fn probe_directory(&mut self, dir: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sock") {
                let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };

                let (resolved_domain, resolved_name) = DOMAIN_PRIMAL_MAP
                    .iter()
                    .find(|&&(d, _)| d == file_stem)
                    .or_else(|| {
                        DOMAIN_PRIMAL_MAP
                            .iter()
                            .find(|&&(_, n)| n == file_stem)
                    })
                    .map_or_else(
                        || (file_stem.to_owned(), file_stem.to_owned()),
                        |&(d, n)| (d.to_owned(), n.to_owned()),
                    );

                let ep =
                    self.by_domain
                        .entry(resolved_domain.clone())
                        .or_insert_with(|| PrimalEndpoint {
                            domain: resolved_domain,
                            name: resolved_name,
                            socket_path: None,
                            tcp_addr: None,
                            capabilities: Vec::new(),
                            healthy: false,
                        });
                ep.socket_path = Some(path);
            }
        }
    }
}

/// Standard socket directory search order.
fn socket_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(dir) = std::env::var(crate::env_keys::BIOMEOS_SOCKET_DIR) {
        dirs.push(PathBuf::from(dir));
    }

    if let Ok(xdg) = std::env::var(crate::env_keys::XDG_RUNTIME_DIR) {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }

    let uid = process_uid();
    dirs.push(PathBuf::from(format!("/run/user/{uid}/biomeos")));
    dirs.push(PathBuf::from(format!("/tmp/biomeos-{uid}")));

    dirs
}

/// Candidate `plasmidBin/` directories.
fn plasmid_bin_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(dir) = std::env::var(crate::env_keys::ECOPRIMALS_PLASMID_BIN) {
        dirs.push(PathBuf::from(dir));
    }

    dirs.push(PathBuf::from("./plasmidBin"));
    dirs.push(PathBuf::from("../plasmidBin"));
    dirs.push(PathBuf::from("../../plasmidBin"));
    dirs.push(PathBuf::from("../../../plasmidBin"));

    dirs
}

/// Resolve the current user's UID for socket path construction.
///
/// Reads `/proc/self/status` (pure Rust, no libc) to get the real UID.
/// Falls back to `$UID` env var, then 0 on non-Unix.
fn process_uid() -> u32 {
    #[cfg(unix)]
    {
        uid_from_proc_status().or_else(uid_from_env).unwrap_or(0)
    }
    #[cfg(not(unix))]
    {
        uid_from_env().unwrap_or(0)
    }
}

/// Parse real UID from `/proc/self/status` — toadStool sysmon pattern.
#[cfg(unix)]
fn uid_from_proc_status() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// Fall back to `$UID` environment variable.
fn uid_from_env() -> Option<u32> {
    std::env::var(crate::env_keys::UID).ok()?.parse().ok()
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn socket_directories_include_run_user_biomeos() {
        let uid = super::process_uid();
        let dirs = socket_directories();
        let expected = PathBuf::from(format!("/run/user/{uid}/biomeos"));
        assert!(dirs.contains(&expected));
    }

    #[test]
    fn process_uid_returns_real_uid() {
        let uid = super::process_uid();
        // On a running Linux system the real UID is never u32::MAX
        assert_ne!(uid, u32::MAX);
    }

    #[test]
    fn plasmidbins_include_relative_paths() {
        let dirs = plasmid_bin_directories();
        assert!(dirs.contains(&PathBuf::from("./plasmidBin")));
        assert!(dirs.contains(&PathBuf::from("../plasmidBin")));
    }

    #[test]
    fn registry_find_missing_capability() {
        let registry = PrimalRegistry::default();
        assert!(registry.find_by_capability("dag.session.create").is_none());
    }

    #[test]
    fn registry_find_registered_capability() {
        let mut registry = PrimalRegistry::default();
        registry.by_domain.insert(
            "dag".to_owned(),
            PrimalEndpoint {
                domain: "dag".to_owned(),
                name: "rhizocrypt".to_owned(),
                socket_path: Some(PathBuf::from("/tmp/rhizocrypt.sock")),
                tcp_addr: None,
                capabilities: vec!["dag.session.create".to_owned()],
                healthy: true,
            },
        );
        registry
            .capability_index
            .insert("dag.session.create".to_owned(), "dag".to_owned());
        assert!(registry.find_by_capability("dag.session.create").is_some());
    }

    #[test]
    fn endpoint_can_have_both_tcp_and_uds() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: Some(PathBuf::from("/tmp/dag.sock")),
            tcp_addr: Some("127.0.0.1:9401".to_owned()),
            capabilities: vec!["dag.session.create".to_owned()],
            healthy: false,
        };
        assert!(ep.socket_path.is_some());
        assert!(ep.tcp_addr.is_some());
    }

    #[test]
    fn metadata_ingestion_sets_tcp_and_capabilities() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(
            &meta_path,
            r#"
[primal]
name = "testprimal"

[capabilities]
domain = "testdomain"
methods = ["test.method.one", "test.method.two"]

[transport]
tcp_address = "127.0.0.1:9999"
"#,
        )
        .unwrap();

        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(&meta_path);

        let ep = registry.by_domain.get("testdomain");
        assert!(ep.is_some());
        let ep = ep.unwrap();
        assert_eq!(ep.name, "testprimal");
        assert_eq!(ep.tcp_addr.as_deref(), Some("127.0.0.1:9999"));
        assert_eq!(ep.capabilities.len(), 2);

        assert!(registry.capability_index.contains_key("test.method.one"));

        let _ = std::fs::remove_file(&meta_path);
    }

    #[test]
    fn metadata_ingestion_port_forms_localhost() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta2");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(
            &meta_path,
            r#"
[primal]
name = "portprimal"

[capabilities]
domain = "portdomain"
methods = ["port.test"]

[transport]
port = 9402
"#,
        )
        .unwrap();

        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(&meta_path);

        let ep = registry.by_domain.get("portdomain").unwrap();
        assert_eq!(ep.tcp_addr.as_deref(), Some("127.0.0.1:9402"));

        let _ = std::fs::remove_file(&meta_path);
    }

    #[test]
    fn metadata_ingestion_skips_missing_primal_name() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta-noname");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(
            &meta_path,
            r#"
[capabilities]
domain = "testdomain"
methods = ["test.method"]
"#,
        )
        .unwrap();

        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(&meta_path);
        assert!(registry.by_domain.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_ingestion_skips_missing_domain() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta-nodomain");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(
            &meta_path,
            r#"
[primal]
name = "orphan"
"#,
        )
        .unwrap();

        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(&meta_path);
        assert!(registry.by_domain.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_ingestion_skips_bad_toml() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta-bad");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(&meta_path, "{{{ not toml").unwrap();

        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(&meta_path);
        assert!(registry.by_domain.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadata_ingestion_skips_missing_file() {
        let mut registry = PrimalRegistry::default();
        registry.ingest_metadata(std::path::Path::new("/nonexistent/metadata.toml"));
        assert!(registry.by_domain.is_empty());
    }

    #[test]
    fn probe_directory_finds_sock_files() {
        let dir = std::env::temp_dir().join("esotericwebb-test-probe");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("ai.sock"), "").unwrap();
        std::fs::write(dir.join("dag.sock"), "").unwrap();
        std::fs::write(dir.join("not_a_socket.txt"), "").unwrap();

        let mut registry = PrimalRegistry::default();
        registry.probe_directory(&dir);

        assert!(registry.by_domain.contains_key("ai"));
        assert!(registry.by_domain.contains_key("dag"));
        assert!(!registry.by_domain.contains_key("not_a_socket"));

        let ai = registry.by_domain.get("ai").unwrap();
        assert!(ai.socket_path.is_some());
        assert_eq!(ai.name, "squirrel");

        let dag = registry.by_domain.get("dag").unwrap();
        assert_eq!(dag.name, "rhizocrypt");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_directory_nonexistent_is_noop() {
        let mut registry = PrimalRegistry::default();
        registry.probe_directory(std::path::Path::new("/nonexistent/probe/dir"));
        assert!(registry.by_domain.is_empty());
    }

    #[test]
    fn probe_directory_unknown_domain_uses_filename() {
        let dir = std::env::temp_dir().join("esotericwebb-test-probe-unknown");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("custom.sock"), "").unwrap();

        let mut registry = PrimalRegistry::default();
        registry.probe_directory(&dir);

        let ep = registry.by_domain.get("custom").unwrap();
        assert_eq!(ep.name, "custom");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_directory_reverse_maps_primal_slug_to_domain() {
        let dir = std::env::temp_dir().join("esotericwebb-test-probe-reverse");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("rhizocrypt.sock"), "").unwrap();
        std::fs::write(dir.join("loamspine.sock"), "").unwrap();
        std::fs::write(dir.join("toadstool.sock"), "").unwrap();

        let mut registry = PrimalRegistry::default();
        registry.probe_directory(&dir);

        let dag = registry.by_domain.get("dag").unwrap();
        assert_eq!(dag.name, "rhizocrypt");
        assert!(dag.socket_path.is_some());

        let lineage = registry.by_domain.get("lineage").unwrap();
        assert_eq!(lineage.name, "loamspine");

        let compute = registry.by_domain.get("compute").unwrap();
        assert_eq!(compute.name, "toadstool");

        assert!(!registry.by_domain.contains_key("rhizocrypt"));
        assert!(!registry.by_domain.contains_key("loamspine"));
        assert!(!registry.by_domain.contains_key("toadstool"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_directory_domain_named_still_works() {
        let dir = std::env::temp_dir().join("esotericwebb-test-probe-domain");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("dag.sock"), "").unwrap();
        std::fs::write(dir.join("visualization.sock"), "").unwrap();

        let mut registry = PrimalRegistry::default();
        registry.probe_directory(&dir);

        let dag = registry.by_domain.get("dag").unwrap();
        assert_eq!(dag.name, "rhizocrypt");

        let viz = registry.by_domain.get("visualization").unwrap();
        assert_eq!(viz.name, "petaltongue");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── TransportEndpoint tests ──

    #[test]
    fn transport_endpoint_uds_serialization() {
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/dag.sock");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "uds");
        assert_eq!(json["path"], "/run/user/1000/biomeos/dag.sock");
    }

    #[test]
    fn transport_endpoint_tcp_serialization() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 9401);
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "tcp");
        assert_eq!(json["host"], "127.0.0.1");
        assert_eq!(json["port"], 9401);
    }

    #[test]
    fn transport_endpoint_mesh_relay_serialization() {
        let ep = TransportEndpoint::mesh_relay("eastGate", "157.230.3.183:7700");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "mesh_relay");
        assert_eq!(json["peer_id"], "eastGate");
        assert_eq!(json["relay"], "157.230.3.183:7700");
    }

    #[test]
    fn transport_endpoint_deserialization_uds() {
        let json = serde_json::json!({"transport": "uds", "path": "/tmp/test.sock"});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::uds("/tmp/test.sock"));
    }

    #[test]
    fn transport_endpoint_deserialization_tcp() {
        let json = serde_json::json!({"transport": "tcp", "host": "10.0.0.1", "port": 8080});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("10.0.0.1", 8080));
    }

    #[test]
    fn transport_endpoint_deserialization_mesh_relay() {
        let json =
            serde_json::json!({"transport": "mesh_relay", "peer_id": "south", "relay": "vps:7700"});
        let ep: TransportEndpoint = serde_json::from_value(json).unwrap();
        assert_eq!(ep, TransportEndpoint::mesh_relay("south", "vps:7700"));
    }

    #[test]
    fn from_tcp_addr_valid() {
        let ep = TransportEndpoint::from_tcp_addr("127.0.0.1:9401").unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("127.0.0.1", 9401));
    }

    #[test]
    fn from_tcp_addr_invalid_port() {
        assert!(TransportEndpoint::from_tcp_addr("127.0.0.1:notaport").is_none());
    }

    #[test]
    fn from_tcp_addr_no_colon() {
        assert!(TransportEndpoint::from_tcp_addr("127.0.0.1").is_none());
    }

    #[test]
    fn resolve_transport_prefers_uds() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: Some(PathBuf::from("/run/user/1000/biomeos/dag.sock")),
            tcp_addr: Some("127.0.0.1:9401".to_owned()),
            capabilities: vec![],
            healthy: true,
        };
        let resolved = ep.resolve_transport().unwrap();
        assert_eq!(
            resolved,
            TransportEndpoint::uds("/run/user/1000/biomeos/dag.sock")
        );
    }

    #[test]
    fn resolve_transport_falls_back_to_tcp() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: None,
            tcp_addr: Some("127.0.0.1:9401".to_owned()),
            capabilities: vec![],
            healthy: true,
        };
        let resolved = ep.resolve_transport().unwrap();
        assert_eq!(resolved, TransportEndpoint::tcp("127.0.0.1", 9401));
    }

    #[test]
    fn resolve_transport_none_when_empty() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: None,
            tcp_addr: None,
            capabilities: vec![],
            healthy: false,
        };
        assert!(ep.resolve_transport().is_none());
    }

    #[test]
    fn available_transports_returns_both() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: Some(PathBuf::from("/tmp/dag.sock")),
            tcp_addr: Some("127.0.0.1:9401".to_owned()),
            capabilities: vec![],
            healthy: true,
        };
        let transports = ep.available_transports();
        assert_eq!(transports.len(), 2);
    }

    #[test]
    fn available_transports_empty_endpoint() {
        let ep = PrimalEndpoint {
            domain: "dag".to_owned(),
            name: "rhizocrypt".to_owned(),
            socket_path: None,
            tcp_addr: None,
            capabilities: vec![],
            healthy: false,
        };
        assert!(ep.available_transports().is_empty());
    }

    // NOTE: discover_tcp_from_env tests omitted — set_var/remove_var
    // are unsafe in edition 2024, forbidden by workspace lints.
    // TCP env var discovery is exercised indirectly via integration tests.

    #[test]
    fn socket_directories_includes_run_user() {
        let dirs = socket_directories();
        let uid = super::process_uid();
        assert!(
            dirs.iter()
                .any(|d| d.to_string_lossy().contains(&format!("/run/user/{uid}")))
        );
    }

    #[cfg(unix)]
    #[test]
    fn uid_from_proc_status_returns_some() {
        let uid = uid_from_proc_status();
        assert!(uid.is_some());
    }

    #[test]
    fn metadata_does_not_overwrite_existing_tcp_addr() {
        let dir = std::env::temp_dir().join("esotericwebb-test-meta-nooverwrite");
        let _ = std::fs::create_dir_all(&dir);
        let meta_path = dir.join("metadata.toml");
        std::fs::write(
            &meta_path,
            r#"
[primal]
name = "keeper"

[capabilities]
domain = "keepdomain"
methods = ["keep.test"]

[transport]
tcp_address = "127.0.0.1:8888"
"#,
        )
        .unwrap();

        let mut registry = PrimalRegistry::default();
        registry.by_domain.insert(
            "keepdomain".to_owned(),
            PrimalEndpoint {
                domain: "keepdomain".to_owned(),
                name: "keeper".to_owned(),
                socket_path: None,
                tcp_addr: Some("10.0.0.1:1111".to_owned()),
                capabilities: vec![],
                healthy: false,
            },
        );
        registry.ingest_metadata(&meta_path);

        let ep = registry.by_domain.get("keepdomain").unwrap();
        assert_eq!(ep.tcp_addr.as_deref(), Some("10.0.0.1:1111"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
