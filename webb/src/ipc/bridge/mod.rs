// SPDX-License-Identifier: AGPL-3.0-or-later
//! `PrimalBridge` — runtime coordinator for primal composition.
//!
//! Discovers live primals at startup, holds connections, and delegates
//! domain-specific calls with graceful degradation. If a primal is
//! absent, every method returns a sensible default rather than failing.
//!
//! ## Transport priority
//!
//! TCP is preferred when a `tcp_addr` is available (platform-agnostic,
//! works inside containers and on Graphene). Falls back to UDS when only
//! a socket path was discovered.
//!
//! ## Module layout
//!
//! Core struct, generic call helpers, and resilience live here.
//! Domain-specific delegations (AI, game, DAG, …) live in `domains`.

mod domains;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::discovery::PrimalRegistry;
use super::envelope::{IpcError, JsonRpcResponse};
use super::primal_names::DOMAIN_PRIMAL_MAP;
use super::resilience::{CircuitBreaker, RetryPolicy};

/// Maximum characters for degraded summary truncation.
///
/// Overridable via `ESOTERICWEBB_SUMMARY_LIMIT` environment variable.
fn degraded_summary_limit() -> usize {
    std::env::var("ESOTERICWEBB_SUMMARY_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

/// Status of a single primal connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    /// Primal name.
    pub name: String,
    /// Capability domain.
    pub domain: String,
    /// Whether an endpoint was discovered (UDS or TCP).
    pub discovered: bool,
    /// Whether the primal responded to a health check.
    pub healthy: bool,
    /// Transport used for the connection (if connected).
    pub transport: Option<String>,
}

/// Runtime coordinator that discovers and holds live primal connections.
#[derive(Debug)]
pub struct PrimalBridge {
    clients: HashMap<String, PrimalClient>,
    statuses: Vec<PrimalStatus>,
    circuits: HashMap<String, CircuitBreaker>,
    retry_policy: RetryPolicy,
}

impl PrimalBridge {
    /// Discover live primals and connect to healthy ones.
    ///
    /// Tries TCP first (if a `tcp_addr` was discovered), falls back to UDS.
    #[must_use]
    pub fn discover() -> Self {
        let registry = PrimalRegistry::discover();
        let mut clients = HashMap::new();
        let mut statuses = Vec::new();

        for &(domain, name) in DOMAIN_PRIMAL_MAP {
            let endpoint = registry.by_domain.get(domain);
            let discovered = endpoint.is_some();
            let mut healthy = false;
            let mut transport_used: Option<String> = None;

            if let Some(ep) = endpoint {
                if let Some(ref addr) = ep.tcp_addr {
                    if let Ok(mut client) = PrimalClient::connect_tcp(addr, name) {
                        if client.health_liveness().unwrap_or(false) {
                            healthy = true;
                            transport_used = Some(format!("tcp:{addr}"));
                            clients.insert(domain.to_owned(), client);
                        }
                    }
                }

                if !healthy {
                    if let Some(ref sock) = ep.socket_path {
                        if let Ok(mut client) = PrimalClient::connect(sock, name) {
                            if client.health_liveness().unwrap_or(false) {
                                healthy = true;
                                transport_used = Some(format!("uds:{}", sock.display()));
                                clients.insert(domain.to_owned(), client);
                            }
                        }
                    }
                }
            }

            statuses.push(PrimalStatus {
                name: name.to_owned(),
                domain: domain.to_owned(),
                discovered,
                healthy,
                transport: transport_used,
            });
        }

        let circuits = DOMAIN_PRIMAL_MAP
            .iter()
            .map(|&(domain, _)| (domain.to_owned(), CircuitBreaker::from_env()))
            .collect();
        Self {
            clients,
            statuses,
            circuits,
            retry_policy: RetryPolicy::from_env(),
        }
    }

    /// Create an empty bridge with no connections (for standalone mode).
    #[must_use]
    pub fn standalone() -> Self {
        let statuses = DOMAIN_PRIMAL_MAP
            .iter()
            .map(|&(domain, name)| PrimalStatus {
                name: name.to_owned(),
                domain: domain.to_owned(),
                discovered: false,
                healthy: false,
                transport: None,
            })
            .collect();
        let circuits = DOMAIN_PRIMAL_MAP
            .iter()
            .map(|&(domain, _)| (domain.to_owned(), CircuitBreaker::from_env()))
            .collect();

        Self {
            clients: HashMap::new(),
            statuses,
            circuits,
            retry_policy: RetryPolicy::from_env(),
        }
    }

    /// Inject a pre-connected client for a domain (used by launcher).
    pub fn inject(&mut self, domain: &str, client: PrimalClient, transport_label: &str) {
        let name = client.primal().to_owned();
        self.clients.insert(domain.to_owned(), client);
        for s in &mut self.statuses {
            if s.domain == domain {
                s.discovered = true;
                s.healthy = true;
                s.transport = Some(transport_label.to_owned());
                s.name.clone_from(&name);
            }
        }
    }

    /// Status of all primal connections.
    #[must_use]
    pub fn statuses(&self) -> &[PrimalStatus] {
        &self.statuses
    }

    /// Whether a specific domain has a healthy connection.
    #[must_use]
    pub fn has(&self, domain: &str) -> bool {
        self.clients.contains_key(domain)
    }

    /// Number of connected primals.
    #[must_use]
    pub fn connected_count(&self) -> usize {
        self.clients.len()
    }

    // ── Generic call helpers ──────────────────────────────────
    //
    // Four patterns that all domain methods share. Using helpers
    // keeps the per-domain methods to 1-3 lines each.

    /// Resilient call: circuit breaker + retry with exponential backoff.
    ///
    /// Returns `Ok(response)` on success, or the last error after exhausting
    /// retries. Non-recoverable errors are returned immediately.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "params is cloned per retry attempt; owned avoids extra clone at call sites"
    )]
    fn resilient_call(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcResponse, IpcError> {
        let circuit = self.circuits.get(domain);
        if circuit.is_some_and(|cb| !cb.is_allowed()) {
            return Err(IpcError::ConnectionRefused(format!(
                "circuit open for domain {domain} — skipping {method}"
            )));
        }

        let client = self
            .clients
            .get_mut(domain)
            .ok_or_else(|| IpcError::PrimalNotFound {
                domain: domain.to_owned(),
            })?;

        let mut last_err = None;
        let max = self.retry_policy.max_retries;

        for attempt in 0..=max {
            match client.call(method, params.clone()) {
                Ok(resp) => {
                    if let Some(cb) = self.circuits.get(domain) {
                        cb.record_success();
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    if !e.is_recoverable() {
                        if let Some(cb) = self.circuits.get(domain) {
                            cb.record_failure();
                        }
                        return Err(e);
                    }
                    tracing::debug!(
                        domain,
                        method,
                        attempt,
                        error = %e,
                        "IPC call failed, will retry"
                    );
                    last_err = Some(e);
                    if attempt < max {
                        std::thread::sleep(self.retry_policy.delay_for_attempt(attempt));
                    }
                }
            }
        }

        if let Some(cb) = self.circuits.get(domain) {
            cb.record_failure();
        }
        Err(last_err
            .unwrap_or_else(|| IpcError::ConnectionReset("all retries exhausted".to_owned())))
    }

    /// Call a domain method and deserialize the result, returning
    /// `default` when the primal is absent or deserialization fails.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "uniform Result return for bridge call chain"
    )]
    fn call_or_default<T: serde::de::DeserializeOwned>(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
        default: T,
    ) -> Result<T, IpcError> {
        if !self.has(domain) {
            return Ok(default);
        }
        match self.resilient_call(domain, method, params) {
            Ok(resp) => {
                if let Some(result) = resp.result {
                    if let Ok(val) = serde_json::from_value::<T>(result) {
                        return Ok(val);
                    }
                }
                Ok(default)
            }
            Err(_) => Ok(default),
        }
    }

    /// Call a domain method, discarding the response (fire-and-forget).
    fn call_fire(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), IpcError> {
        if self.has(domain) {
            let _ = self.resilient_call(domain, method, params)?;
        }
        Ok(())
    }

    /// Call a domain method and extract a string ID from the result
    /// using a list of candidate field names.
    fn call_extract_id(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
        fields: &[&str],
    ) -> Result<Option<String>, IpcError> {
        if !self.has(domain) {
            return Ok(None);
        }
        let resp = self.resilient_call(domain, method, params)?;
        if let Some(result) = resp.result {
            for &field in fields {
                if let Some(id) = result.get(field).and_then(serde_json::Value::as_str) {
                    return Ok(Some(id.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Call a domain method and return the raw result value.
    fn call_passthrough(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        if !self.has(domain) {
            return Ok(None);
        }
        let resp = self.resilient_call(domain, method, params)?;
        Ok(resp.result)
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::ipc::primal_names::{DOMAIN_PRIMAL_MAP, domain};

    #[test]
    fn standalone_bridge_has_no_connections() {
        let bridge = PrimalBridge::standalone();
        assert_eq!(bridge.connected_count(), 0);
        assert!(!bridge.has(domain::AI));
        assert!(!bridge.has(domain::VISUALIZATION));
        assert!(!bridge.has(domain::COMPUTE));
        assert!(!bridge.has(domain::STORAGE));
        assert_eq!(bridge.statuses().len(), DOMAIN_PRIMAL_MAP.len());
    }

    #[test]
    fn standalone_ai_narrate_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let resp = bridge.ai_narrate("test prompt").unwrap();
        assert!(resp.text.contains("unavailable"));
        assert_eq!(resp.model, "none");
    }

    #[test]
    fn standalone_ai_summarize_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let resp = bridge.ai_summarize("some context").unwrap();
        assert!(resp.contains("summary unavailable"));
    }

    #[test]
    fn standalone_render_scene_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.render_scene(&serde_json::json!({"type": "test"}));
        assert!(result.is_ok());
    }

    #[test]
    fn standalone_compute_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.compute_submit(&serde_json::json!({})).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_store_returns_false() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.store("key", &serde_json::json!("value")).unwrap();
        assert!(!result);
    }

    #[test]
    fn standalone_retrieve_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.retrieve("key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_evaluate_flow_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.evaluate_flow(&serde_json::Value::Null).unwrap();
        assert!((result.flow_score - 0.5).abs() < f64::EPSILON);
        assert!(!result.in_flow);
    }

    #[test]
    fn standalone_difficulty_adjustment_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .difficulty_adjustment(&serde_json::Value::Null)
            .unwrap();
        assert!((result.adjustment).abs() < f64::EPSILON);
    }

    #[test]
    fn standalone_provenance_returns_false() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .provenance_append(&serde_json::json!({"id": "v1"}))
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn standalone_dag_session_create_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.dag_session_create(&serde_json::json!({})).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_event_append_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_event_append(&serde_json::json!({"session_id": "s1", "data": {}}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_frontier_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_frontier_get(&serde_json::json!({"session_id": "s1"}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_merkle_root_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_merkle_root(&serde_json::json!({"session_id": "s1"}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_engagement_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.engagement(&serde_json::Value::Null).unwrap();
        assert!((result.engagement_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn standalone_npc_dialogue_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .npc_dialogue(&serde_json::json!({"npc_id": "test"}))
            .unwrap();
        assert!(result.text.contains("degraded"));
        assert!(result.voice_notes.is_empty());
    }

    #[test]
    fn standalone_narrate_action_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .narrate_action(&serde_json::json!({"action": "test"}))
            .unwrap();
        assert_eq!(result.model, "none");
    }

    #[test]
    fn standalone_voice_check_returns_empty() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .voice_check(&serde_json::json!({"voice_id": "logic"}))
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn standalone_game_push_scene_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .game_push_scene(&serde_json::json!({"type": "test"}))
                .is_ok()
        );
    }

    #[test]
    fn standalone_game_begin_session_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .game_begin_session(&serde_json::json!({}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_game_complete_session_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(bridge.game_complete_session(&serde_json::json!({})).is_ok());
    }

    #[test]
    fn standalone_dag_session_complete_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .dag_session_complete(&serde_json::json!({"session_id": "s1"}))
                .is_ok()
        );
    }

    #[test]
    fn standalone_dag_query_vertices_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .dag_query_vertices(&serde_json::json!({"session_id": "s1"}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_mint_certificate_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .mint_certificate(&serde_json::json!({"npc": "test"}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_poll_input_returns_empty() {
        let mut bridge = PrimalBridge::standalone();
        assert!(bridge.poll_input().unwrap().is_empty());
    }

    #[test]
    fn discover_with_no_sockets_is_standalone() {
        let bridge = PrimalBridge::discover();
        assert_eq!(bridge.statuses().len(), DOMAIN_PRIMAL_MAP.len());
        for s in bridge.statuses() {
            assert!(!s.healthy);
        }
    }
}
