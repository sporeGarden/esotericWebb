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
//! Protocol: newline-delimited JSON-RPC 2.0.

pub mod bridge;
pub mod client;
pub mod discovery;
pub mod envelope;
pub mod handlers;
pub mod launcher;
pub mod listener;
pub mod ludospring;
pub mod petaltongue;
pub mod provenance;
pub mod resilience;
pub mod server;
pub mod squirrel;

pub use envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// ── Capability domain identifiers ──────────────────────────
// Primals are discovered by domain, never by name.

/// AI domain (Squirrel).
pub const DOMAIN_AI: &str = "ai";
/// Visualization domain (petalTongue).
pub const DOMAIN_VISUALIZATION: &str = "visualization";
/// Compute domain (toadStool).
pub const DOMAIN_COMPUTE: &str = "compute";
/// Storage domain (nestGate).
pub const DOMAIN_STORAGE: &str = "storage";
/// Game science domain (ludoSpring).
pub const DOMAIN_GAME: &str = "game";
/// DAG domain (rhizoCrypt).
pub const DOMAIN_DAG: &str = "dag";
/// Lineage domain (loamSpine).
pub const DOMAIN_LINEAGE: &str = "lineage";
/// Provenance domain (sweetGrass).
pub const DOMAIN_PROVENANCE: &str = "provenance";

/// Domain→default primal name mapping for discovery.
///
/// The bridge discovers by domain and uses names only for logging.
/// Primal code only has self-knowledge — these names come from the
/// ecosystem registry, not from importing primal code.
pub const PRIMAL_DOMAINS: &[(&str, &str)] = &[
    (DOMAIN_AI, "squirrel"),
    (DOMAIN_VISUALIZATION, "petaltongue"),
    (DOMAIN_COMPUTE, "toadstool"),
    (DOMAIN_STORAGE, "nestgate"),
    (DOMAIN_GAME, "ludospring"),
    (DOMAIN_DAG, "rhizocrypt"),
    (DOMAIN_LINEAGE, "loamspine"),
    (DOMAIN_PROVENANCE, "sweetgrass"),
];

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
