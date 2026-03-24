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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::discovery::PrimalRegistry;
use super::envelope::IpcError;
use super::ludospring::{DdaResult, FlowResult};
use super::squirrel::ChatResponse;

/// Which primals the bridge can connect to.
const PRIMAL_DOMAINS: &[(&str, &str)] = &[
    ("ai", "squirrel"),
    ("visualization", "petaltongue"),
    ("compute", "toadstool"),
    ("storage", "nestgate"),
    ("game", "ludospring"),
    ("dag", "rhizocrypt"),
    ("lineage", "loamspine"),
    ("provenance", "sweetgrass"),
];

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
}

impl PrimalBridge {
    /// Discover live primals and connect to healthy ones.
    ///
    /// Tries TCP first (if a `tcp_addr` was discovered), falls back to UDS.
    pub fn discover() -> Self {
        let registry = PrimalRegistry::discover();
        let mut clients = HashMap::new();
        let mut statuses = Vec::new();

        for &(domain, name) in PRIMAL_DOMAINS {
            let endpoint = registry.by_domain.get(domain);
            let discovered = endpoint.is_some();
            let mut healthy = false;
            let mut transport_used: Option<String> = None;

            if let Some(ep) = endpoint {
                // Try TCP first — platform-agnostic, works in containers
                if let Some(ref addr) = ep.tcp_addr {
                    if let Ok(mut client) = PrimalClient::connect_tcp(addr, name) {
                        if client.health_liveness().unwrap_or(false) {
                            healthy = true;
                            transport_used = Some(format!("tcp:{addr}"));
                            clients.insert(domain.to_owned(), client);
                        }
                    }
                }

                // Fall back to UDS if TCP didn't succeed
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

        Self { clients, statuses }
    }

    /// Create an empty bridge with no connections (for standalone mode).
    pub fn standalone() -> Self {
        let statuses = PRIMAL_DOMAINS
            .iter()
            .map(|&(domain, name)| PrimalStatus {
                name: name.to_owned(),
                domain: domain.to_owned(),
                discovered: false,
                healthy: false,
                transport: None,
            })
            .collect();

        Self {
            clients: HashMap::new(),
            statuses,
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
    pub fn statuses(&self) -> &[PrimalStatus] {
        &self.statuses
    }

    /// Whether a specific domain has a healthy connection.
    pub fn has(&self, domain: &str) -> bool {
        self.clients.contains_key(domain)
    }

    /// Number of connected primals.
    pub fn connected_count(&self) -> usize {
        self.clients.len()
    }

    // ── AI domain (Squirrel) ────────────────────────────────

    /// Generate narration via the AI primal.
    ///
    /// Degrades to a placeholder if Squirrel is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_narrate(&mut self, prompt: &str) -> Result<ChatResponse, IpcError> {
        if let Some(client) = self.clients.get_mut("ai") {
            let params = serde_json::json!({
                "messages": [{"role": "user", "content": prompt}],
            });
            let resp = client.call("ai.chat", params)?;
            if let Some(result) = resp.result {
                if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                    return Ok(chat);
                }
            }
        }
        Ok(ChatResponse {
            text: format!("[AI primal unavailable — narration placeholder for: {prompt}]"),
            model: "none".to_owned(),
            tokens: 0,
        })
    }

    /// Summarize context via the AI primal.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_summarize(&mut self, context: &str) -> Result<String, IpcError> {
        if let Some(client) = self.clients.get_mut("ai") {
            let params = serde_json::json!({ "text": context });
            let resp = client.call("ai.summarize", params)?;
            if let Some(serde_json::Value::String(text)) = resp.result {
                return Ok(text);
            }
        }
        let truncated: String = context.chars().take(200).collect();
        Ok(format!("[summary unavailable] {truncated}..."))
    }

    // ── Visualization domain (PetalTongue) ──────────────────

    /// Push a scene payload for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn render_scene(&mut self, scene: &serde_json::Value) -> Result<(), IpcError> {
        if let Some(client) = self.clients.get_mut("visualization") {
            let _ = client.call("visualization.render.scene", scene.clone())?;
        }
        Ok(())
    }

    // ── Compute domain (ToadStool) ──────────────────────────

    /// Submit a compute task.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn compute_submit(&mut self, task: &serde_json::Value) -> Result<Option<String>, IpcError> {
        if let Some(client) = self.clients.get_mut("compute") {
            let resp = client.call("compute.dispatch.submit", task.clone())?;
            if let Some(serde_json::Value::String(job_id)) =
                resp.result.as_ref().and_then(|r| r.get("job_id").cloned())
            {
                return Ok(Some(job_id));
            }
        }
        Ok(None)
    }

    // ── Storage domain (NestGate) ───────────────────────────

    /// Store a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn store(&mut self, key: &str, value: &serde_json::Value) -> Result<bool, IpcError> {
        if let Some(client) = self.clients.get_mut("storage") {
            let params = serde_json::json!({ "key": key, "value": value });
            let resp = client.call("storage.store", params)?;
            return Ok(resp.error.is_none());
        }
        Ok(false)
    }

    /// Retrieve a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn retrieve(&mut self, key: &str) -> Result<Option<serde_json::Value>, IpcError> {
        if let Some(client) = self.clients.get_mut("storage") {
            let params = serde_json::json!({ "key": key });
            let resp = client.call("storage.retrieve", params)?;
            return Ok(resp.result);
        }
        Ok(None)
    }

    // ── Game science domain (LudoSpring) ────────────────────

    /// Evaluate flow state.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn evaluate_flow(&mut self, params: &serde_json::Value) -> Result<FlowResult, IpcError> {
        if let Some(client) = self.clients.get_mut("game") {
            let resp = client.call("game.evaluate_flow", params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(flow) = serde_json::from_value::<FlowResult>(result) {
                    return Ok(flow);
                }
            }
        }
        Ok(FlowResult {
            flow_score: 0.5,
            in_flow: false,
        })
    }

    /// Get DDA recommendation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn difficulty_adjustment(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<DdaResult, IpcError> {
        if let Some(client) = self.clients.get_mut("game") {
            let resp = client.call("game.difficulty_adjustment", params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(dda) = serde_json::from_value::<DdaResult>(result) {
                    return Ok(dda);
                }
            }
        }
        Ok(DdaResult {
            adjustment: 0.0,
            reason: "game science primal unavailable — no adjustment".to_owned(),
        })
    }

    // ── Provenance domain (RootPulse trio) ──────────────────

    /// Append a provenance vertex to the session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn provenance_append(&mut self, vertex: &serde_json::Value) -> Result<bool, IpcError> {
        if let Some(client) = self.clients.get_mut("dag") {
            let resp = client.call("dag.event.append", vertex.clone())?;
            return Ok(resp.error.is_none());
        }
        Ok(false)
    }

    // ── DAG domain (rhizoCrypt) — typed API ─────────────────

    /// Create a new session on the DAG primal.
    ///
    /// Returns the session ID if rhizoCrypt is connected, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_session_create(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if let Some(client) = self.clients.get_mut("dag") {
            let resp = client.call("dag.session.create", params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("session_id")
                    .or_else(|| result.get("id"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Append an event to a session DAG.
    ///
    /// Returns the vertex ID if successful.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_event_append(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if let Some(client) = self.clients.get_mut("dag") {
            let resp = client.call("dag.event.append", params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("vertex_id")
                    .or_else(|| result.get("id"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
                return Ok(Some("ok".to_owned()));
            }
        }
        Ok(None)
    }

    /// Get the current frontier of a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_frontier_get(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        if let Some(client) = self.clients.get_mut("dag") {
            let resp = client.call("dag.frontier.get", params.clone())?;
            return Ok(resp.result);
        }
        Ok(None)
    }

    /// Get the Merkle root of a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_merkle_root(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if let Some(client) = self.clients.get_mut("dag") {
            let resp = client.call("dag.merkle.root", params.clone())?;
            if let Some(result) = resp.result {
                if let Some(root) = result
                    .get("root")
                    .or_else(|| result.get("merkle_root"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(root.to_owned()));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn standalone_bridge_has_no_connections() {
        let bridge = PrimalBridge::standalone();
        assert_eq!(bridge.connected_count(), 0);
        assert!(!bridge.has("ai"));
        assert!(!bridge.has("visualization"));
        assert!(!bridge.has("compute"));
        assert!(!bridge.has("storage"));
        assert_eq!(bridge.statuses().len(), PRIMAL_DOMAINS.len());
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
    fn discover_with_no_sockets_is_standalone() {
        let bridge = PrimalBridge::discover();
        assert_eq!(bridge.statuses().len(), PRIMAL_DOMAINS.len());
        for s in bridge.statuses() {
            assert!(!s.healthy);
        }
    }
}
