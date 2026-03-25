// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game session lifecycle handlers.

use serde_json::Value;

use crate::ipc::envelope::JsonRpcError;
use crate::session::{ActionKind, GameSession};

use super::SharedSession;

/// `session.start` — initialize a new game session from content path.
///
/// Preserves the [`PrimalBridge`] from any previous session so primal
/// composition capabilities survive session restarts.
pub(super) fn handle_session_start(
    params: Option<&Value>,
    session: &SharedSession,
) -> Result<Value, JsonRpcError> {
    let content_path = params
        .and_then(|p| p.get("content_path"))
        .and_then(Value::as_str)
        .unwrap_or("content");

    let mut guard = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let bridge = guard.as_mut().and_then(GameSession::take_bridge);

    let mut game = GameSession::with_bridge(content_path, bridge).map_err(|e| JsonRpcError {
        code: -32000,
        message: format!("session start failed: {e}"),
        data: None,
    })?;

    game.initialize_provenance();

    let snap = game.snapshot();
    *guard = Some(game);
    drop(guard);

    Ok(serde_json::to_value(snap).unwrap_or(Value::Null))
}

/// `session.state` — full game state snapshot.
pub(super) fn handle_session_state(session: &SharedSession) -> Result<Value, JsonRpcError> {
    with_session(session, |s| {
        Ok(serde_json::to_value(s.snapshot()).unwrap_or(Value::Null))
    })
}

/// `session.actions` — list available actions.
pub(super) fn handle_session_actions(session: &SharedSession) -> Result<Value, JsonRpcError> {
    with_session(session, |s| {
        Ok(serde_json::to_value(s.available_actions()).unwrap_or(Value::Null))
    })
}

/// `session.act` — perform an action, returning outcome and narration context.
pub(super) fn handle_session_act(
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
        let mut guard = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let s = guard.as_mut().ok_or_else(|| JsonRpcError {
            code: -32000,
            message: "no active session".to_owned(),
            data: None,
        })?;

        let action_kind = ActionKind::parse(kind).map_err(|e| JsonRpcError {
            code: -32602,
            message: e,
            data: None,
        })?;

        let (outcome_text, narration_ctx) = s.act(action_kind, id).map_err(|e| JsonRpcError {
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

/// `session.history` — full action log.
pub(super) fn handle_session_history(session: &SharedSession) -> Result<Value, JsonRpcError> {
    with_session(session, |s| {
        Ok(serde_json::to_value(s.history()).unwrap_or(Value::Null))
    })
}

/// `session.narrate` — narration context for AI-as-generator.
pub(super) fn handle_session_narrate(session: &SharedSession) -> Result<Value, JsonRpcError> {
    with_session(session, |s| {
        Ok(serde_json::to_value(s.narration_context()).unwrap_or(Value::Null))
    })
}

/// `session.graph` — DOT graph with live session overlay.
pub(super) fn handle_session_graph(session: &SharedSession) -> Result<Value, JsonRpcError> {
    with_session(session, |s| Ok(Value::String(s.to_dot())))
}

/// Helper: lock session, require it exists, run closure.
fn with_session<F>(session: &SharedSession, f: F) -> Result<Value, JsonRpcError>
where
    F: FnOnce(&crate::session::GameSession) -> Result<Value, JsonRpcError>,
{
    let guard = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.as_ref().map_or_else(
        || {
            Err(JsonRpcError {
                code: -32000,
                message: "no active session — call session.start first".to_owned(),
                data: None,
            })
        },
        f,
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn empty_session() -> SharedSession {
        std::sync::Arc::new(std::sync::Mutex::new(None))
    }

    #[test]
    fn session_state_without_session_errors() {
        let result = handle_session_state(&empty_session());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32000);
        assert!(err.message.contains("no active session"));
    }

    #[test]
    fn session_actions_without_session_errors() {
        let result = handle_session_actions(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_act_missing_kind_errors() {
        let session = empty_session();
        {
            let mut guard = session.lock().unwrap();
            *guard = None;
        }
        let params = serde_json::json!({"id": "room"});
        let result = handle_session_act(Some(&params), &session);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("kind"));
    }

    #[test]
    fn session_act_missing_id_errors() {
        let params = serde_json::json!({"kind": "exit"});
        let result = handle_session_act(Some(&params), &empty_session());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("id"));
    }

    #[test]
    fn session_act_invalid_kind_errors() {
        let session = empty_session();
        {
            let mut guard = session.lock().unwrap();
            let game = crate::session::GameSession::new("content");
            if let Ok(g) = game {
                *guard = Some(g);
            }
        }
        let params = serde_json::json!({"kind": "invalid_kind", "id": "room"});
        let result = handle_session_act(Some(&params), &session);
        if session.lock().unwrap().is_some() {
            assert!(result.is_err());
        }
    }

    #[test]
    fn session_history_without_session_errors() {
        let result = handle_session_history(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_narrate_without_session_errors() {
        let result = handle_session_narrate(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_graph_without_session_errors() {
        let result = handle_session_graph(&empty_session());
        assert!(result.is_err());
    }
}
