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
pub mod launcher;
pub mod listener;
pub mod ludospring;
pub mod petaltongue;
pub mod provenance;
pub mod server;
pub mod squirrel;

pub use envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

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
