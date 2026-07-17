// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 IPC — client and server for Esoteric Webb.
//!
//! ## Client
//!
//! Discovers and consumes primals via capability-based discovery.
//! All primal binaries resolved from `plasmidBin/` — zero Rust crate
//! dependencies on any spring. Pure IPC.
//!
//! ## Server
//!
//! Exposes Webb's own capabilities: health, narrative status,
//! content listing, MCP tools.
//!
//! ## Transport
//!
//! TCP (preferred for platform portability — containers, Graphene) and
//! Unix domain sockets (XDG-compliant path resolution).
//! Transport priority configurable via `ESOTERICWEBB_TRANSPORT_PRIORITY`.
//! Protocol: newline-delimited JSON-RPC 2.0.
//!
//! ## Why JSON-RPC only (no tarpc)
//!
//! The wateringHole `PRIMAL_IPC_PROTOCOL.md` defines a dual-protocol
//! standard: JSON-RPC 2.0 (mandatory) + tarpc (optional, for high-
//! throughput intra-host calls). Webb is a **composition substrate** —
//! it consumes primals at human-interaction speed (one action per
//! second). JSON-RPC over TCP/UDS provides platform-agnostic, debuggable
//! IPC with negligible overhead at this cadence. tarpc would add a Rust
//! crate dependency and compile-time coupling for zero measurable gain.
//! If a future primal requires sub-millisecond RPC (e.g. real-time
//! rendering pipeline), tarpc can be added per-domain without changing
//! the bridge architecture.

pub mod bridge;
pub mod client;
pub mod discovery;
pub mod envelope;
pub mod handlers;
pub mod launcher;
pub mod listener;
pub mod petaltongue;
pub mod primal_names;
pub mod resilience;
pub mod squirrel;

pub use envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use primal_names::DOMAIN_PRIMAL_MAP;
pub use primal_names::domain;

/// Resolve the default host address for TCP connections.
///
/// Overridable via `ESOTERICWEBB_DEFAULT_HOST` for containerized or
/// Graphene deployments where `127.0.0.1` may not be correct.
/// Returns `127.0.0.1` as the ecosystem default.
#[must_use]
pub fn default_host() -> String {
    std::env::var(crate::env_keys::ESOTERICWEBB_DEFAULT_HOST)
        .unwrap_or_else(|_| "127.0.0.1".to_owned())
}

/// Build a TCP address from a port using the default host.
#[must_use]
pub fn host_port(port: impl std::fmt::Display) -> String {
    format!("{}:{port}", default_host())
}

// ── Compute domain methods ─────────────────────────────────

/// Submit a compute task.
pub const METHOD_COMPUTE_SUBMIT: &str = "compute.dispatch.submit";

// ── Storage domain methods ─────────────────────────────────

/// Store a key-value pair.
pub const METHOD_STORAGE_STORE: &str = "storage.store";
/// Retrieve a value by key.
pub const METHOD_STORAGE_RETRIEVE: &str = "storage.retrieve";

// ── DAG domain methods (rhizoCrypt) ────────────────────────

/// Create a new session DAG.
pub const METHOD_DAG_SESSION_CREATE: &str = "dag.session.create";
/// Append an event vertex to a session DAG.
pub const METHOD_DAG_EVENT_APPEND: &str = "dag.event.append";
/// Get the frontier of a session DAG.
pub const METHOD_DAG_FRONTIER_GET: &str = "dag.frontier.get";
/// Get the Merkle root of a session DAG.
pub const METHOD_DAG_MERKLE_ROOT: &str = "dag.merkle.root";
/// Complete a session DAG.
pub const METHOD_DAG_SESSION_COMPLETE: &str = "dag.session.complete";
/// Query vertices in a session DAG.
pub const METHOD_DAG_QUERY_VERTICES: &str = "dag.query.vertices";

// ── Lineage domain methods (loamSpine) ─────────────────────

/// Mint a certificate.
pub const METHOD_CERT_MINT: &str = "certificate.mint";

// ── Crypto domain methods (bearDog) ────────────────────────

/// Sign a payload with the session key.
pub const METHOD_CRYPTO_SIGN: &str = "crypto.sign";
/// Verify a signed payload.
pub const METHOD_CRYPTO_VERIFY: &str = "crypto.verify";
/// Hash arbitrary data (content-addressable identity).
pub const METHOD_CRYPTO_HASH: &str = "crypto.hash";

// ── Mesh domain methods (songBird) ─────────────────────────

/// Query the live ecosystem topology.
pub const METHOD_MESH_TOPOLOGY: &str = "discovery.topology";
/// Query health of all known primals in the mesh.
pub const METHOD_MESH_HEALTH: &str = "discovery.health";
/// Query a specific primal's status by name.
pub const METHOD_MESH_QUERY: &str = "discovery.query";
/// List active bonds between primals.
pub const METHOD_MESH_BONDS: &str = "discovery.bonds";

// ── Composition dispatch (Wave 17 Neural API — atomic orchestration) ──
// Ecosystem vocabulary: "compositions" (Wire names preserved as biomeOS contract)

/// Atomic provenance step: content.put + dag.event.append + spine.seal + braid.create.
pub const SIGNAL_NEST_STORE: &str = "nest.store";
/// Atomic session finalization: dehydrate + sign + store + seal.
pub const SIGNAL_NEST_COMMIT: &str = "nest.commit";
/// Meta-tier observation (analytics without live renderer).
pub const SIGNAL_META_OBSERVE: &str = "meta.observe";
/// Meta-tier rendering intent declaration.
pub const SIGNAL_META_INTENT: &str = "meta.intent";

// ── Lifecycle / announce (Wave 17) ────────────────────────────

/// Single-call primal registration (replaces 3-call lifecycle pattern).
pub const METHOD_PRIMAL_ANNOUNCE: &str = "primal.announce";
/// Read-only niche metadata query.
pub const METHOD_PRIMAL_INFO: &str = "primal.info";
/// Detailed version/build info.
pub const METHOD_HEALTH_VERSION: &str = "health.version";
/// Graceful shutdown acknowledgment.
pub const METHOD_HEALTH_DRAIN: &str = "health.drain";

// ── Mesh routing (Wave 73 — cross-gate access) ──────────────

/// Register capabilities with the mesh router for cross-gate discovery.
pub const METHOD_ROUTE_REGISTER: &str = "route.register";

// ── Introspection (Wave 107 — barraCuda pattern) ──────────────

/// Runtime method introspection — describe any exposed method.
pub const METHOD_METHOD_DESCRIBE: &str = "method.describe";

// ── Webb's own methods ─────────────────────────────────────

/// Webb health method.
pub const METHOD_HEALTH: &str = "webb.health";
/// Webb liveness check.
pub const METHOD_LIVENESS: &str = "webb.liveness";
/// Webb readiness check.
pub const METHOD_READINESS: &str = "webb.readiness";
/// Current game scene.
pub const METHOD_SCENE_CURRENT: &str = "webb.scene.current";
/// Narrative DAG status.
pub const METHOD_NARRATIVE_STATUS: &str = "webb.narrative.status";
/// List loaded content.
pub const METHOD_CONTENT_LIST: &str = "webb.content.list";
/// MCP tools enumeration.
pub const METHOD_TOOLS_LIST: &str = "tools.list";
/// MCP tool invocation.
pub const METHOD_TOOLS_CALL: &str = "tools.call";
/// sourDough health.liveness (Kubernetes-style).
pub const METHOD_HEALTH_LIVENESS: &str = "health.liveness";
/// sourDough health.readiness (Kubernetes-style).
pub const METHOD_HEALTH_READINESS: &str = "health.readiness";
/// sourDough health.check.
pub const METHOD_HEALTH_CHECK: &str = "health.check";
/// sourDough capabilities.list.
pub const METHOD_CAPABILITIES_LIST: &str = "capabilities.list";
/// sourDough identity.get.
pub const METHOD_IDENTITY_GET: &str = "identity.get";

// ── Session methods ─────────────────────────────────────

/// Start a new game session.
pub const METHOD_SESSION_START: &str = "session.start";
/// Get full game state.
pub const METHOD_SESSION_STATE: &str = "session.state";
/// List available actions.
pub const METHOD_SESSION_ACTIONS: &str = "session.actions";
/// Perform an action.
pub const METHOD_SESSION_ACT: &str = "session.act";
/// Get session history.
pub const METHOD_SESSION_HISTORY: &str = "session.history";
/// Get narration context for AI-as-generator.
pub const METHOD_SESSION_NARRATE: &str = "session.narrate";
/// Get DOT graph with live session overlay.
pub const METHOD_SESSION_GRAPH: &str = "session.graph";
/// Get session engagement metrics (V13 — game science / DDA).
pub const METHOD_SESSION_METRICS: &str = "session.metrics";
