// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the provenance trio (session DAG, certificates, attribution).
//!
//! Consumes `provenance.*`, `certificate.*`, and `attribution.*` capabilities.
//! Primal binaries resolved from `plasmidBin/` — no spring source dependency.
//! All three are optional — Webb degrades gracefully without provenance
//! by maintaining a local session log.
//!
//! When a live [`PrimalClient`] connection exists, vertices are also sent
//! to the remote DAG. The local log is always maintained as a fallback.

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::envelope::IpcError;

/// Session DAG primal method names.
pub const METHOD_SESSION_CREATE: &str = "provenance.session_create";
/// Append vertex to session DAG.
pub const METHOD_VERTEX_APPEND: &str = "provenance.vertex_append";
/// Query vertices in session DAG.
pub const METHOD_VERTEX_QUERY: &str = "provenance.vertex_query";
/// Certificate primal: mint certificate.
pub const METHOD_CERT_MINT: &str = "certificate.mint";
/// Certificate primal: query certificate.
pub const METHOD_CERT_QUERY: &str = "certificate.query";
/// Attribution primal: record attribution.
pub const METHOD_ATTRIBUTION_RECORD: &str = "attribution.record";

/// A provenance vertex representing a game event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceVertex {
    /// Vertex identifier.
    pub id: String,
    /// Parent vertex IDs (DAG edges).
    pub parents: Vec<String>,
    /// Event type (e.g. `player_action`, `npc_response`, `state_change`).
    pub event_type: String,
    /// Event payload.
    pub data: serde_json::Value,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

/// Client for the provenance trio.
#[derive(Debug)]
pub struct ProvenanceClient {
    available: bool,
    local_log: Vec<ProvenanceVertex>,
}

impl ProvenanceClient {
    /// Create a new client.
    #[must_use]
    pub const fn new(available: bool) -> Self {
        Self {
            available,
            local_log: Vec::new(),
        }
    }

    /// Whether the provenance trio was discovered.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Append a vertex to the session DAG.
    ///
    /// If a live [`PrimalClient`] connection exists, the vertex is also
    /// sent to the remote DAG primal. The local log is always maintained
    /// as a fallback for later replay or export.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the remote call fails.
    pub fn append_vertex(
        &mut self,
        vertex: ProvenanceVertex,
        client: Option<&mut PrimalClient>,
    ) -> Result<(), IpcError> {
        if let Some(c) = client {
            let params = serde_json::to_value(&vertex).map_err(|e| IpcError::Serialization {
                detail: e.to_string(),
            })?;
            let _ = c.call("dag.event.append", params)?;
        } else if !self.available {
            tracing::debug!("provenance unavailable — logging locally: {}", vertex.id);
        }
        self.local_log.push(vertex);
        Ok(())
    }

    /// Get the local vertex log (for export when provenance is unavailable).
    #[must_use]
    pub fn local_log(&self) -> &[ProvenanceVertex] {
        &self.local_log
    }

    /// Number of vertices recorded.
    #[must_use]
    pub const fn vertex_count(&self) -> usize {
        self.local_log.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fallback_records_vertices() {
        let mut client = ProvenanceClient::new(false);
        let vertex = ProvenanceVertex {
            id: "v1".to_owned(),
            parents: vec![],
            event_type: "player_action".to_owned(),
            data: serde_json::json!({"action": "examine"}),
            timestamp: "2026-03-23T12:00:00Z".to_owned(),
        };
        let result = client.append_vertex(vertex, None);
        assert!(result.is_ok());
        assert_eq!(client.vertex_count(), 1);
    }

    #[test]
    fn available_client_also_logs_locally() {
        let mut client = ProvenanceClient::new(true);
        let vertex = ProvenanceVertex {
            id: "v2".to_owned(),
            parents: vec!["v1".to_owned()],
            event_type: "npc_response".to_owned(),
            data: serde_json::json!({"npc": "maren"}),
            timestamp: "2026-03-23T12:01:00Z".to_owned(),
        };
        let result = client.append_vertex(vertex, None);
        assert!(result.is_ok());
        assert_eq!(client.vertex_count(), 1);
    }
}
