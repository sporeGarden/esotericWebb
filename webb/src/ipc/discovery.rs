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
//! **UDS** (5-tier, per wateringHole UNIVERSAL_IPC_STANDARD_V3):
//! 1. `$BIOMEOS_SOCKET_DIR/<domain>.sock`
//! 2. `$XDG_RUNTIME_DIR/biomeos/<domain>.sock`
//! 3. `/run/user/<uid>/biomeos/<domain>.sock`
//! 4. `/tmp/biomeos-<uid>/<domain>.sock`
//! 5. Songbird `discovery.query` (if available)
//!
//! **TCP** (checked first, highest priority):
//! 1. `<PRIMAL>_ADDRESS` env var (e.g. `RHIZOCRYPT_ADDRESS=127.0.0.1:9401`)
//! 2. `<PRIMAL>_JSONRPC_PORT` env var → `127.0.0.1:<port>`
//! 3. `plasmidBin/<primal>/metadata.toml` `[transport]` section

use std::collections::HashMap;
use std::path::PathBuf;

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

/// Registry of discovered primals, keyed by domain.
#[derive(Debug, Default)]
pub struct PrimalRegistry {
    /// Endpoints indexed by domain.
    pub by_domain: HashMap<String, PrimalEndpoint>,
    /// Capability to domain mapping.
    pub capability_index: HashMap<String, String>,
}

/// Known domain-to-primal name mappings for TCP env-var discovery.
const KNOWN_PRIMALS: &[(&str, &str)] = &[
    ("dag", "rhizocrypt"),
    ("lineage", "loamspine"),
    ("provenance", "sweetgrass"),
    ("ai", "squirrel"),
    ("visualization", "petaltongue"),
    ("compute", "toadstool"),
    ("storage", "nestgate"),
    ("game", "ludospring"),
];

impl PrimalRegistry {
    /// Discover primals from TCP env vars, `plasmidBin/` metadata, and UDS socket directories.
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
        for &(domain, name) in KNOWN_PRIMALS {
            let upper = name.to_uppercase();

            let addr = std::env::var(format!("{upper}_ADDRESS"))
                .ok()
                .or_else(|| {
                    std::env::var(format!("{upper}_JSONRPC_PORT"))
                        .ok()
                        .map(|port| format!("127.0.0.1:{port}"))
                })
                .or_else(|| std::env::var(format!("{upper}_HTTP_ADDRESS")).ok());

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
                    .map(|port| format!("127.0.0.1:{port}"))
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
    fn probe_directory(&mut self, dir: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sock") {
                let domain = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned);
                if let Some(domain) = domain {
                    let name = KNOWN_PRIMALS
                        .iter()
                        .find(|&&(d, _)| d == domain)
                        .map_or_else(|| domain.clone(), |&(_, n)| n.to_owned());

                    let ep =
                        self.by_domain
                            .entry(domain.clone())
                            .or_insert_with(|| PrimalEndpoint {
                                domain: domain.clone(),
                                name: name.clone(),
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
}

/// Standard socket directory search order.
fn socket_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        dirs.push(PathBuf::from(dir));
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
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

    if let Ok(dir) = std::env::var("ECOPRIMALS_PLASMID_BIN") {
        dirs.push(PathBuf::from(dir));
    }

    dirs.push(PathBuf::from("./plasmidBin"));
    dirs.push(PathBuf::from("../plasmidBin"));
    dirs.push(PathBuf::from("../../plasmidBin"));
    dirs.push(PathBuf::from("../../../plasmidBin"));

    dirs
}

fn process_uid() -> u32 {
    #[cfg(unix)]
    {
        std::process::id()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn socket_directories_include_run_user_biomeos() {
        let uid = std::process::id();
        let dirs = socket_directories();
        let expected = PathBuf::from(format!("/run/user/{uid}/biomeos"));
        assert!(dirs.contains(&expected));
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
        assert!(registry.find_by_capability("game.evaluate_flow").is_none());
    }

    #[test]
    fn registry_find_registered_capability() {
        let mut registry = PrimalRegistry::default();
        registry.by_domain.insert(
            "game".to_owned(),
            PrimalEndpoint {
                domain: "game".to_owned(),
                name: "ludospring".to_owned(),
                socket_path: Some(PathBuf::from("/tmp/game.sock")),
                tcp_addr: None,
                capabilities: vec!["game.evaluate_flow".to_owned()],
                healthy: true,
            },
        );
        registry
            .capability_index
            .insert("game.evaluate_flow".to_owned(), "game".to_owned());
        assert!(registry.find_by_capability("game.evaluate_flow").is_some());
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
}
