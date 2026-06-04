// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC handler dispatch — domain-split (ludoSpring pattern).
//!
//! Each handler module owns a concern:
//! - [`lifecycle`]: health, readiness, identity, capabilities
//! - [`narrative`]: scene, narrative status, content listing
//! - [`session`]: game session lifecycle (start, act, state, ...)
//! - [`mcp`]: MCP `tools.list` / `tools.call` with JSON Schema

use std::sync::{Arc, Mutex};

use super::envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::{
    METHOD_CAPABILITIES_LIST, METHOD_CONTENT_LIST, METHOD_HEALTH, METHOD_HEALTH_CHECK,
    METHOD_HEALTH_DRAIN, METHOD_HEALTH_LIVENESS, METHOD_HEALTH_READINESS, METHOD_HEALTH_VERSION,
    METHOD_IDENTITY_GET, METHOD_LIVENESS, METHOD_NARRATIVE_STATUS, METHOD_PRIMAL_ANNOUNCE,
    METHOD_PRIMAL_INFO, METHOD_READINESS, METHOD_SCENE_CURRENT, METHOD_SESSION_ACT,
    METHOD_SESSION_ACTIONS, METHOD_SESSION_GRAPH, METHOD_SESSION_HISTORY, METHOD_SESSION_METRICS,
    METHOD_SESSION_NARRATE, METHOD_SESSION_START, METHOD_SESSION_STATE, METHOD_TOOLS_CALL,
    METHOD_TOOLS_LIST,
};
use crate::session::GameSession;

pub mod lifecycle;
pub mod mcp;
pub mod narrative;
pub mod session;

/// Shared session handle for the IPC server.
pub type SharedSession = Arc<Mutex<Option<GameSession>>>;

/// Create a new shared session handle (initially empty).
#[must_use]
pub fn new_shared_session() -> SharedSession {
    Arc::new(Mutex::new(None))
}

/// Dispatch a JSON-RPC request to the appropriate handler.
#[must_use]
pub fn dispatch(request: &JsonRpcRequest) -> JsonRpcResponse {
    dispatch_with_session(request, &new_shared_session())
}

/// Dispatch with access to a shared session.
pub fn dispatch_with_session(request: &JsonRpcRequest, session: &SharedSession) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        METHOD_HEALTH
        | METHOD_LIVENESS
        | METHOD_READINESS
        | METHOD_HEALTH_LIVENESS
        | METHOD_HEALTH_CHECK => Ok(lifecycle::handle_health()),
        METHOD_HEALTH_READINESS => Ok(lifecycle::handle_readiness(session)),
        METHOD_HEALTH_VERSION => Ok(lifecycle::handle_health_version()),
        METHOD_HEALTH_DRAIN => Ok(lifecycle::handle_health_drain()),
        METHOD_IDENTITY_GET => Ok(lifecycle::handle_identity()),
        METHOD_CAPABILITIES_LIST => Ok(lifecycle::handle_capabilities_list()),
        METHOD_PRIMAL_ANNOUNCE => Ok(lifecycle::handle_primal_announce(request.params.as_ref())),
        METHOD_PRIMAL_INFO => Ok(lifecycle::handle_primal_info()),

        METHOD_SCENE_CURRENT => Ok(narrative::handle_scene_current(session)),
        METHOD_NARRATIVE_STATUS => Ok(narrative::handle_narrative_status(session)),
        METHOD_CONTENT_LIST => Ok(narrative::handle_content_list(session)),

        METHOD_TOOLS_LIST => Ok(mcp::handle_tools_list()),
        METHOD_TOOLS_CALL => mcp::handle_tools_call(request.params.as_ref(), session),

        METHOD_SESSION_START => session::handle_session_start(request.params.as_ref(), session),
        METHOD_SESSION_STATE => session::handle_session_state(session),
        METHOD_SESSION_ACTIONS => session::handle_session_actions(session),
        METHOD_SESSION_ACT => session::handle_session_act(request.params.as_ref(), session),
        METHOD_SESSION_HISTORY => session::handle_session_history(session),
        METHOD_SESSION_NARRATE => session::handle_session_narrate(session),
        METHOD_SESSION_GRAPH => session::handle_session_graph(session),
        METHOD_SESSION_METRICS => session::handle_session_metrics(session),

        _ => Err(JsonRpcError::method_not_found(format!(
            "method not found: {}",
            request.method
        ))),
    };

    match result {
        Ok(value) => JsonRpcResponse::success(value, request.id.clone()),
        Err(err) => JsonRpcResponse::error(err, request.id.clone()),
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn health_returns_ok() {
        let req = JsonRpcRequest::new("webb.health", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let status = resp
            .result
            .as_ref()
            .and_then(|r| r.get("status"))
            .and_then(Value::as_str);
        assert_eq!(status, Some("healthy"));
    }

    #[test]
    fn identity_get_returns_primal() {
        let req = JsonRpcRequest::new("identity.get", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let primal = resp
            .result
            .as_ref()
            .and_then(|r| r.get("primal"))
            .and_then(Value::as_str);
        assert_eq!(primal, Some("esotericwebb"));
    }

    #[test]
    fn unknown_method_returns_error() {
        let req = JsonRpcRequest::new("nonexistent.method", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_some());
        if let Some(err) = &resp.error {
            assert_eq!(err.code, -32601);
        }
    }

    #[test]
    fn tools_list_returns_tools_with_schema() {
        let req = JsonRpcRequest::new("tools.list", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let tools = resp
            .result
            .as_ref()
            .and_then(|r| r.get("tools"))
            .and_then(Value::as_array);
        assert!(tools.is_some());
        let tools = tools.unwrap();
        assert!(!tools.is_empty());
        for tool in tools {
            assert!(tool.get("input_schema").is_some());
        }
    }

    #[test]
    fn tools_call_known_tool() {
        let req = JsonRpcRequest::new(
            "tools.call",
            Some(serde_json::json!({"name": "webb.scene.current"})),
        );
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn tools_call_session_methods() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new(
            "tools.call",
            Some(serde_json::json!({"name": "session.state"})),
        );
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some(), "should fail without active session");
    }

    #[test]
    fn tools_call_unknown_tool() {
        let req = JsonRpcRequest::new(
            "tools.call",
            Some(serde_json::json!({"name": "nonexistent"})),
        );
        let resp = dispatch(&req);
        assert!(resp.error.is_some());
    }

    #[test]
    fn session_state_without_start_errors() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("session.state", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some());
    }

    #[test]
    fn take_bridge_preserves_bridge_across_sessions() {
        use crate::ipc::bridge::PrimalBridge;
        use crate::session::GameSession;

        let session = new_shared_session();

        let bridge = PrimalBridge::standalone();
        assert_eq!(bridge.connected_count(), 0);

        {
            let mut guard = session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let game = GameSession::new("content");
            if let Ok(mut game) = game {
                assert!(game.bridge().is_none(), "new() has no bridge");
                let _ = game.take_bridge();
                *guard = Some(game);
            }
        }
    }

    #[test]
    fn liveness_returns_ok() {
        let req = JsonRpcRequest::new("webb.liveness", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn readiness_returns_ok() {
        let req = JsonRpcRequest::new("webb.readiness", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn health_liveness_returns_ok() {
        let req = JsonRpcRequest::new("health.liveness", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn health_readiness_returns_ok() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("health.readiness", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_none());
        let ready = resp
            .result
            .as_ref()
            .and_then(|r| r.get("ready"))
            .and_then(Value::as_bool);
        assert!(ready.is_some());
    }

    #[test]
    fn health_check_returns_ok() {
        let req = JsonRpcRequest::new("health.check", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn capabilities_list_returns_array() {
        let req = JsonRpcRequest::new("capabilities.list", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let caps = resp
            .result
            .as_ref()
            .and_then(|r| r.get("capabilities"))
            .and_then(Value::as_array);
        assert!(caps.is_some());
        assert!(!caps.unwrap().is_empty());
    }

    #[test]
    fn scene_current_without_session() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("webb.scene.current", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_none());
        let scene = resp.result.as_ref().and_then(|r| r.get("scene"));
        assert!(scene.is_some());
    }

    #[test]
    fn narrative_status_without_session() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("webb.narrative.status", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_none());
    }

    #[test]
    fn content_list_without_session() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("webb.content.list", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_none());
    }

    #[test]
    fn session_actions_without_start_errors() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("session.actions", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some());
    }

    #[test]
    fn session_history_without_start_errors() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("session.history", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some());
    }

    #[test]
    fn session_narrate_without_start_errors() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("session.narrate", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some());
    }

    #[test]
    fn session_graph_without_start_errors() {
        let session = new_shared_session();
        let req = JsonRpcRequest::new("session.graph", None);
        let resp = dispatch_with_session(&req, &session);
        assert!(resp.error.is_some());
    }

    #[test]
    fn dispatch_preserves_request_id() {
        let mut req = JsonRpcRequest::new("webb.health", None);
        req.id = serde_json::json!(42);
        let resp = dispatch(&req);
        assert_eq!(resp.id, serde_json::json!(42));
    }

    #[test]
    fn dispatch_returns_jsonrpc_2_0() {
        let req = JsonRpcRequest::new("webb.health", None);
        let resp = dispatch(&req);
        assert_eq!(resp.jsonrpc, "2.0");
    }

    #[test]
    fn new_shared_session_is_none() {
        let session = new_shared_session();
        assert!(session.lock().unwrap().is_none());
    }

    #[test]
    fn health_version_dispatches() {
        let req = JsonRpcRequest::new("health.version", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let primal = resp
            .result
            .as_ref()
            .and_then(|r| r.get("primal"))
            .and_then(Value::as_str);
        assert_eq!(primal, Some("esotericwebb"));
    }

    #[test]
    fn health_drain_dispatches() {
        let req = JsonRpcRequest::new("health.drain", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let ack = resp
            .result
            .as_ref()
            .and_then(|r| r.get("acknowledged"))
            .and_then(Value::as_bool);
        assert_eq!(ack, Some(true));
    }

    #[test]
    fn primal_announce_dispatches() {
        let req = JsonRpcRequest::new(
            "primal.announce",
            Some(serde_json::json!({"primal": "toadstool", "version": "0.5.0"})),
        );
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let accepted = resp
            .result
            .as_ref()
            .and_then(|r| r.get("accepted"))
            .and_then(Value::as_bool);
        assert_eq!(accepted, Some(true));
    }

    #[test]
    fn primal_info_dispatches() {
        let req = JsonRpcRequest::new("primal.info", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let domain = resp
            .result
            .as_ref()
            .and_then(|r| r.get("domain"))
            .and_then(Value::as_str);
        assert_eq!(domain, Some("narrative"));
    }
}
