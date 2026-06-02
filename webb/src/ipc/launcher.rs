// SPDX-License-Identifier: AGPL-3.0-or-later
//! `PrimalLauncher` — spawn primal binaries from `plasmidBin/`.
//!
//! Mirrors `primalSpring`'s launcher patterns:
//! - Binary discovery across `$ECOPRIMALS_PLASMID_BIN`, relative paths,
//!   and arch/OS variants
//! - Process spawning with `<binary> server --port <port>`
//! - TCP readiness polling (connect loop)
//! - Graph-driven topological ordering from deploy TOML files
//!
//! Webb owns the child processes and kills them on `Drop`.

use std::collections::VecDeque;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A spawned primal process.
#[derive(Debug)]
pub struct SpawnedPrimal {
    /// Primal name (e.g. "rhizocrypt").
    pub name: String,
    /// Process ID.
    pub pid: u32,
    /// TCP address the primal is listening on.
    pub address: String,
    /// Capability domain (e.g. "dag").
    pub domain: String,
}

/// Manages spawned primal child processes.
#[derive(Debug)]
pub struct PrimalLauncher {
    children: Vec<(String, Child)>,
    spawned: Vec<SpawnedPrimal>,
}

/// A node in a deploy graph TOML.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GraphNode {
    /// Primal name (e.g. "rhizocrypt").
    pub name: String,
    /// Binary name override (defaults to primal name).
    #[serde(default)]
    pub binary: Option<String>,
    /// Spawn order hint (lower = earlier within a wave).
    #[serde(default)]
    pub order: u32,
    /// Whether to actually spawn this node (false = validation-only).
    #[serde(default = "default_true")]
    pub spawn: bool,
    /// Names of nodes that must be healthy before this one starts.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Capability domain this node provides.
    #[serde(default)]
    pub by_capability: Option<String>,
    /// TCP port override (auto-assigned if absent).
    #[serde(default)]
    pub port: Option<u16>,
}

const fn default_true() -> bool {
    true
}

/// Top-level deploy graph TOML structure.
#[derive(Debug, serde::Deserialize)]
pub struct DeployGraph {
    /// The `[graph]` section containing metadata and nodes.
    pub graph: GraphSection,
}

/// The `[graph]` section of a deploy TOML.
#[derive(Debug, serde::Deserialize)]
pub struct GraphSection {
    /// Human-readable graph name.
    #[serde(default)]
    pub name: String,
    /// Ordered list of graph nodes.
    #[serde(default)]
    pub node: Vec<GraphNode>,
}

/// Readiness timeout — overridable via `ESOTERICWEBB_READINESS_TIMEOUT_SECS`.
fn readiness_timeout() -> Duration {
    let secs = std::env::var(crate::env_keys::ESOTERICWEBB_READINESS_TIMEOUT_SECS)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);
    Duration::from_secs(secs)
}

const READINESS_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Port base for auto-assigned ports — overridable via `ESOTERICWEBB_PORT_BASE`.
fn default_port_base() -> u16 {
    std::env::var(crate::env_keys::ESOTERICWEBB_PORT_BASE)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(9401)
}

impl Default for PrimalLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimalLauncher {
    /// Create an empty launcher.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
            spawned: Vec::new(),
        }
    }

    /// Spawn a single primal binary.
    ///
    /// Discovers the binary, starts it with `server --port <port>`,
    /// and polls for TCP readiness.
    ///
    /// # Errors
    ///
    /// Returns a human-readable error on binary-not-found or spawn failure.
    pub fn spawn(
        &mut self,
        primal: &str,
        port: u16,
        domain: &str,
    ) -> Result<&SpawnedPrimal, String> {
        let binary = discover_binary(primal)?;
        let addr = format!("127.0.0.1:{port}");

        tracing::info!(primal, binary = %binary.display(), "spawning primal");

        let child = Command::new(&binary)
            .arg("server")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {primal}: {e}"))?;

        let pid = child.id();
        self.children.push((primal.to_owned(), child));

        await_tcp_ready(&addr)?;

        tracing::info!(primal, addr = %addr, pid, "primal ready");

        let idx = self.spawned.len();
        self.spawned.push(SpawnedPrimal {
            name: primal.to_owned(),
            pid,
            address: addr,
            domain: domain.to_owned(),
        });

        Ok(&self.spawned[idx])
    }

    /// Spawn primals from a deploy graph TOML, in topological order.
    ///
    /// Only nodes with `spawn = true` are started. Port assignment uses
    /// the node's `port` field if set, otherwise auto-increments from
    /// the `DEFAULT_PORT_BASE`.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph has cycles, a binary is missing, or
    /// a process fails to start.
    pub fn spawn_from_graph(&mut self, graph_path: &str) -> Result<&[SpawnedPrimal], String> {
        let contents =
            std::fs::read_to_string(graph_path).map_err(|e| format!("read {graph_path}: {e}"))?;
        let graph: DeployGraph =
            toml::from_str(&contents).map_err(|e| format!("parse {graph_path}: {e}"))?;

        let waves = topological_waves(&graph)?;
        let mut next_port = default_port_base();
        let start_idx = self.spawned.len();

        for wave in &waves {
            for node in wave {
                if !node.spawn {
                    continue;
                }
                let port = node.port.unwrap_or_else(|| {
                    let p = next_port;
                    next_port += 1;
                    p
                });
                let domain = node.by_capability.as_deref().unwrap_or(&node.name);
                self.spawn(&node.name, port, domain)?;
            }
        }

        Ok(&self.spawned[start_idx..])
    }

    /// All spawned primals so far.
    #[must_use]
    pub fn spawned(&self) -> &[SpawnedPrimal] {
        &self.spawned
    }

    /// Shut down all child processes.
    pub fn shutdown(&mut self) {
        for (name, child) in self.children.iter_mut().rev() {
            tracing::info!(name = %name, pid = child.id(), "stopping primal");
            let _ = child.kill();
            let _ = child.wait();
        }
        self.children.clear();
    }
}

impl Drop for PrimalLauncher {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Discover a primal binary in `plasmidBin/` directories.
///
/// Search order mirrors `primalSpring`'s `discover_binary`:
/// 1. `$ECOPRIMALS_PLASMID_BIN/<pattern>`
/// 2. `$BIOMEOS_PLASMID_BIN_DIR/<pattern>`
/// 3. `./plasmidBin/<pattern>`
/// 4. `../plasmidBin/<pattern>`
/// 5. `../../plasmidBin/<pattern>`
///
/// Within each base, tries arch/OS variants then flat paths.
///
/// # Errors
///
/// Returns a descriptive error listing all searched paths if the binary
/// is not found in any candidate location.
pub fn discover_binary(primal: &str) -> Result<PathBuf, String> {
    let base_dirs: Vec<Option<PathBuf>> = vec![
        std::env::var(crate::env_keys::ECOPRIMALS_PLASMID_BIN)
            .ok()
            .map(PathBuf::from),
        std::env::var(crate::env_keys::BIOMEOS_PLASMID_BIN_DIR)
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")),
        Some(PathBuf::from("../../../plasmidBin")),
    ];

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let patterns = [
        format!("{primal}_{arch}_{os}_musl/{primal}"),
        format!("{primal}_{arch}_{os}/{primal}"),
        format!("primals/{primal}/{primal}"),
        format!("primals/{primal}"),
        format!("{primal}/{primal}"),
        primal.to_string(),
    ];

    let mut searched = Vec::new();

    for base in base_dirs.iter().filter_map(Option::as_ref) {
        if !base.exists() {
            continue;
        }
        for pattern in &patterns {
            let candidate = base.join(pattern);
            if candidate.is_file() {
                return Ok(candidate);
            }
            searched.push(candidate);
        }
    }

    Err(format!(
        "binary not found for '{primal}'. Searched:\n{}",
        searched
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

/// Kahn's algorithm — returns nodes grouped by wave (parallelizable tiers).
fn topological_waves(graph: &DeployGraph) -> Result<Vec<Vec<GraphNode>>, String> {
    let nodes = &graph.graph.node;
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    let name_to_idx: std::collections::HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.name.as_str(), i))
        .collect();

    let mut in_degree: Vec<usize> = vec![0; nodes.len()];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];

    for (i, node) in nodes.iter().enumerate() {
        for dep in &node.depends_on {
            let &dep_idx = name_to_idx.get(dep.as_str()).ok_or_else(|| {
                format!(
                    "node '{}' depends on '{}' which is not in the graph",
                    node.name, dep
                )
            })?;
            in_degree[i] += 1;
            dependents[dep_idx].push(i);
        }
    }

    let mut waves: Vec<Vec<GraphNode>> = Vec::new();
    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|&(_, d)| *d == 0)
        .map(|(i, _)| i)
        .collect();
    let mut processed = 0;

    while !queue.is_empty() {
        let wave_size = queue.len();
        let mut wave = Vec::with_capacity(wave_size);
        for _ in 0..wave_size {
            let Some(idx) = queue.pop_front() else { break };
            wave.push(nodes[idx].clone());
            processed += 1;
            for &dep_idx in &dependents[idx] {
                in_degree[dep_idx] -= 1;
                if in_degree[dep_idx] == 0 {
                    queue.push_back(dep_idx);
                }
            }
        }
        wave.sort_by_key(|n| n.order);
        waves.push(wave);
    }

    if processed != nodes.len() {
        return Err("deploy graph contains a dependency cycle".to_owned());
    }

    Ok(waves)
}

/// Poll a TCP address until it accepts connections, or time out.
fn await_tcp_ready(addr: &str) -> Result<(), String> {
    let timeout = readiness_timeout();
    let start = Instant::now();
    loop {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "primal at {addr} did not become ready within {}s",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(READINESS_POLL_INTERVAL);
    }
}

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod launcher_tests;
