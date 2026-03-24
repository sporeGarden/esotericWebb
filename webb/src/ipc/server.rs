// SPDX-License-Identifier: AGPL-3.0-or-later
//! Webb's own JSON-RPC server.
//!
//! Exposes health, narrative status, content listing, MCP tools, and
//! game session methods so AI agents and human tools can play the game.

use std::sync::{Arc, Mutex, PoisonError};

use serde_json::Value;

use super::envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::{
    METHOD_CAPABILITIES_LIST, METHOD_CONTENT_LIST, METHOD_HEALTH, METHOD_HEALTH_CHECK,
    METHOD_HEALTH_LIVENESS, METHOD_HEALTH_READINESS, METHOD_LIVENESS, METHOD_NARRATIVE_STATUS,
    METHOD_READINESS, METHOD_SCENE_CURRENT, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
};
use crate::session::GameSession;

/// Shared session handle for the IPC server.
pub type SharedSession = Arc<Mutex<Option<GameSession>>>;

/// Create a new shared session handle (initially empty).
pub fn new_shared_session() -> SharedSession {
    Arc::new(Mutex::new(None))
}

/// Session method names.
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

/// Dispatch a JSON-RPC request to the appropriate handler.
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
        | METHOD_HEALTH_CHECK => Ok(handle_health()),
        METHOD_HEALTH_READINESS => Ok(handle_readiness(session)),
        METHOD_CAPABILITIES_LIST => Ok(handle_capabilities_list()),
        METHOD_SCENE_CURRENT => Ok(handle_scene_current(session)),
        METHOD_NARRATIVE_STATUS => Ok(handle_narrative_status(session)),
        METHOD_CONTENT_LIST => Ok(handle_content_list(session)),
        METHOD_TOOLS_LIST => Ok(handle_tools_list()),
        METHOD_TOOLS_CALL => handle_tools_call(request.params.as_ref(), session),

        METHOD_SESSION_START => handle_session_start(request.params.as_ref(), session),
        METHOD_SESSION_STATE => handle_session_state(session),
        METHOD_SESSION_ACTIONS => handle_session_actions(session),
        METHOD_SESSION_ACT => handle_session_act(request.params.as_ref(), session),
        METHOD_SESSION_HISTORY => handle_session_history(session),
        METHOD_SESSION_NARRATE => handle_session_narrate(session),
        METHOD_SESSION_GRAPH => handle_session_graph(session),

        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            result: Some(value),
            error: None,
            id: request.id.clone(),
        },
        Err(err) => JsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            error: Some(err),
            id: request.id.clone(),
        },
    }
}

fn handle_health() -> Value {
    serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn handle_readiness(session: &SharedSession) -> Value {
    let ready = session
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some();
    serde_json::json!({
        "ready": ready,
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn handle_capabilities_list() -> Value {
    let registry_toml = include_str!("../../capability_registry.toml");
    let table: toml::Value =
        toml::from_str(registry_toml).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    let capabilities: Vec<Value> = table
        .get("capabilities")
        .and_then(toml::Value::as_array)
        .map_or_else(Vec::new, |caps| {
            caps.iter()
                .filter_map(|c| {
                    let method = c.get("method")?.as_str()?;
                    let desc = c.get("description")?.as_str()?;
                    Some(serde_json::json!({
                        "method": method,
                        "description": desc,
                    }))
                })
                .collect()
        });

    serde_json::json!({
        "primal": "esotericwebb",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": capabilities,
    })
}

fn handle_scene_current(session: &SharedSession) -> Value {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            serde_json::json!({
                "scene": null,
                "note": "no active session",
            })
        },
        |s| {
            let snap = s.snapshot();
            serde_json::json!({
                "scene": snap.current_node,
                "description": snap.scene_description,
                "npcs": snap.scene_npcs,
                "is_ending": snap.is_ending,
            })
        },
    )
}

fn handle_narrative_status(session: &SharedSession) -> Value {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            serde_json::json!({
                "current_node": null,
                "active_plane": null,
                "vertex_count": 0,
            })
        },
        |s| {
            let snap = s.snapshot();
            serde_json::json!({
                "current_node": snap.current_node,
                "turn": snap.turn,
                "is_ending": snap.is_ending,
                "knowledge_count": snap.knowledge.len(),
                "actions_available": snap.available_actions.len(),
            })
        },
    )
}

fn handle_content_list(session: &SharedSession) -> Value {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            serde_json::json!({
                "worlds": [],
                "npcs": [],
                "abilities": [],
                "scenes": [],
            })
        },
        |s| {
            let b = s.bundle();
            serde_json::json!({
                "npcs": b.npcs.keys().collect::<Vec<_>>(),
                "abilities": b.abilities.keys().collect::<Vec<_>>(),
                "scenes": b.scenes.keys().collect::<Vec<_>>(),
                "worlds": b.worlds.keys().collect::<Vec<_>>(),
                "narrative_nodes": b.narrative.nodes.keys().collect::<Vec<_>>(),
            })
        },
    )
}

fn handle_tools_list() -> Value {
    serde_json::json!({
        "tools": [
            {"name": "health.liveness", "description": "Liveness probe (sourDough)"},
            {"name": "health.readiness", "description": "Readiness probe (sourDough)"},
            {"name": "capabilities.list", "description": "List all capabilities (sourDough)"},
            {"name": "webb.scene.current", "description": "Get the current game scene"},
            {"name": "webb.narrative.status", "description": "Get narrative DAG status"},
            {"name": "webb.content.list", "description": "List loaded content"},
            {"name": "session.start", "description": "Start a new game session"},
            {"name": "session.state", "description": "Get full game state snapshot"},
            {"name": "session.actions", "description": "List available actions"},
            {"name": "session.act", "description": "Perform an action (kind + id)"},
            {"name": "session.history", "description": "Get session action history"},
            {"name": "session.narrate", "description": "Get narration context for AI generator"},
            {"name": "session.graph", "description": "Get DOT graph with live session overlay"},
        ]
    })
}

fn handle_tools_call(
    params: Option<&Value>,
    session: &SharedSession,
) -> Result<Value, JsonRpcError> {
    let tool_name = params
        .and_then(|p| p.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match tool_name {
        "webb.scene.current" => Ok(handle_scene_current(session)),
        "webb.narrative.status" => Ok(handle_narrative_status(session)),
        "webb.content.list" => Ok(handle_content_list(session)),
        _ => Err(JsonRpcError {
            code: -32602,
            message: format!("unknown tool: {tool_name}"),
            data: None,
        }),
    }
}

// ── Session methods ────────────────────────────────────────────────────────

fn handle_session_start(
    params: Option<&Value>,
    session: &SharedSession,
) -> Result<Value, JsonRpcError> {
    let content_path = params
        .and_then(|p| p.get("content_path"))
        .and_then(Value::as_str)
        .unwrap_or("content");

    let game = GameSession::new(content_path).map_err(|e| JsonRpcError {
        code: -32000,
        message: format!("session start failed: {e}"),
        data: None,
    })?;

    let snap = game.snapshot();
    let mut guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    *guard = Some(game);
    drop(guard);

    Ok(serde_json::to_value(snap).unwrap_or(Value::Null))
}

fn handle_session_state(session: &SharedSession) -> Result<Value, JsonRpcError> {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session — call session.start first".to_owned(),
                data: None,
            })
        },
        |s| Ok(serde_json::to_value(s.snapshot()).unwrap_or(Value::Null)),
    )
}

fn handle_session_actions(session: &SharedSession) -> Result<Value, JsonRpcError> {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session".to_owned(),
                data: None,
            })
        },
        |s| Ok(serde_json::to_value(s.available_actions()).unwrap_or(Value::Null)),
    )
}

fn handle_session_act(
    params: Option<&Value>,
    session: &SharedSession,
) -> Result<Value, JsonRpcError> {
    let kind = params
        .and_then(|p| p.get("kind"))
        .and_then(Value::as_str)
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "missing 'kind' parameter".to_owned(),
            data: None,
        })?;

    let id = params
        .and_then(|p| p.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "missing 'id' parameter".to_owned(),
            data: None,
        })?;

    let (outcome_text, narration_ctx, state_after) = {
        let mut guard = session.lock().unwrap_or_else(PoisonError::into_inner);
        let s = guard.as_mut().ok_or_else(|| JsonRpcError {
            code: -32000,
            message: "no active session".to_owned(),
            data: None,
        })?;

        let (outcome_text, narration_ctx) = s.act(kind, id).map_err(|e| JsonRpcError {
            code: -32000,
            message: e,
            data: None,
        })?;

        let state_after = s.snapshot();
        drop(guard);
        (outcome_text, narration_ctx, state_after)
    };

    Ok(serde_json::json!({
        "outcome": outcome_text,
        "narration_context": serde_json::to_value(narration_ctx).unwrap_or(Value::Null),
        "state": serde_json::to_value(state_after).unwrap_or(Value::Null),
    }))
}

fn handle_session_history(session: &SharedSession) -> Result<Value, JsonRpcError> {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session".to_owned(),
                data: None,
            })
        },
        |s| Ok(serde_json::to_value(s.history()).unwrap_or(Value::Null)),
    )
}

fn handle_session_narrate(session: &SharedSession) -> Result<Value, JsonRpcError> {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session".to_owned(),
                data: None,
            })
        },
        |s| Ok(serde_json::to_value(s.narration_context()).unwrap_or(Value::Null)),
    )
}

fn handle_session_graph(session: &SharedSession) -> Result<Value, JsonRpcError> {
    let guard = session.lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session".to_owned(),
                data: None,
            })
        },
        |s| Ok(Value::String(s.to_dot())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::envelope::JsonRpcRequest;

    #[test]
    fn health_returns_ok() {
        let req = JsonRpcRequest::new("webb.health", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let status = resp
            .result
            .as_ref()
            .and_then(|r| r.get("status"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(status, Some("healthy"));
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
    fn tools_list_returns_tools() {
        let req = JsonRpcRequest::new("tools.list", None);
        let resp = dispatch(&req);
        assert!(resp.error.is_none());
        let tools = resp
            .result
            .as_ref()
            .and_then(|r| r.get("tools"))
            .and_then(serde_json::Value::as_array);
        assert!(tools.is_some());
        assert!(!tools.unwrap_or(&vec![]).is_empty());
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
}
