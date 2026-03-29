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
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::content::{AbilityDef, ContentBundle, NpcDef, SceneContent, WorldMeta};
    use crate::director::GameDirector;
    use crate::narrative::effect::StateEffect;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use crate::state::WorldState;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn empty_session() -> SharedSession {
        Arc::new(Mutex::new(None))
    }

    fn session_with_game() -> SharedSession {
        let bundle = test_bundle();
        let director = GameDirector::new(&bundle).unwrap();
        let session = GameSession::from_parts(bundle, director, WorldState::new(), None);
        Arc::new(Mutex::new(Some(session)))
    }

    fn test_bundle() -> ContentBundle {
        let mut nodes = HashMap::new();
        let mut scenes = HashMap::new();
        nodes.insert(
            "start".to_owned(),
            NarrativeNode {
                id: "start".to_owned(),
                scene_type: SceneType::Exploration,
                content_ref: "start".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![NarrativeEdge {
                    target: "end".to_owned(),
                    conditions: vec![],
                    priority: 0,
                    transition_type: TransitionType::SamePlane,
                    label: Some("Go".to_owned()),
                }],
                is_start: true,
                is_ending: false,
                label: None,
            },
        );
        scenes.insert(
            "start".to_owned(),
            SceneContent {
                id: "start".to_owned(),
                description: "Start.".to_owned(),
                npcs: vec!["npc".to_owned()],
                items: vec![],
            },
        );
        nodes.insert(
            "end".to_owned(),
            NarrativeNode {
                id: "end".to_owned(),
                scene_type: SceneType::Ending,
                content_ref: "end".to_owned(),
                preconditions: vec![],
                effects: vec![StateEffect::SetFlag("done".to_owned())],
                exits: vec![],
                is_start: false,
                is_ending: true,
                label: None,
            },
        );
        scenes.insert(
            "end".to_owned(),
            SceneContent {
                id: "end".to_owned(),
                description: "End.".to_owned(),
                npcs: vec![],
                items: vec![],
            },
        );
        let mut npcs = HashMap::new();
        npcs.insert(
            "npc".to_owned(),
            NpcDef {
                id: "npc".to_owned(),
                name: "NPC".to_owned(),
                role: String::new(),
                knows: vec![],
                trust_initial: 0,
                trust_rewards: std::collections::BTreeMap::new(),
                lies_about: HashMap::new(),
                arc: String::new(),
            },
        );
        let mut abilities = HashMap::new();
        abilities.insert(
            "look".to_owned(),
            AbilityDef {
                id: "look".to_owned(),
                name: "Look".to_owned(),
                description: "Look around.".to_owned(),
                preconditions: vec![],
                effects: vec![],
                narration_hint: None,
            },
        );
        ContentBundle {
            meta: WorldMeta {
                name: "Test".to_owned(),
                author: "test".to_owned(),
                version: "0.1.0".to_owned(),
                description: "Test.".to_owned(),
            },
            narrative: NarrativeGraph { nodes },
            worlds: HashMap::new(),
            npcs,
            abilities,
            scenes,
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        }
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
    fn session_state_with_session_succeeds() {
        let result = handle_session_state(&session_with_game());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["current_node"], "start");
        assert_eq!(val["session_active"], true);
    }

    #[test]
    fn session_actions_without_session_errors() {
        let result = handle_session_actions(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_actions_with_session_succeeds() {
        let result = handle_session_actions(&session_with_game());
        assert!(result.is_ok());
        let val = result.unwrap();
        let actions = val.as_array().unwrap();
        assert!(!actions.is_empty());
    }

    #[test]
    fn session_act_missing_kind_errors() {
        let params = serde_json::json!({"id": "room"});
        let result = handle_session_act(Some(&params), &session_with_game());
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
        let params = serde_json::json!({"kind": "invalid_kind", "id": "end"});
        let result = handle_session_act(Some(&params), &session_with_game());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32602);
    }

    #[test]
    fn session_act_exit_succeeds() {
        let session = session_with_game();
        let params = serde_json::json!({"kind": "exit", "id": "end"});
        let result = handle_session_act(Some(&params), &session);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.get("outcome").is_some());
        assert!(val.get("narration_context").is_some());
        assert!(val.get("state").is_some());
    }

    #[test]
    fn session_act_examine_succeeds() {
        let session = session_with_game();
        let params = serde_json::json!({"kind": "examine", "id": "examine"});
        let result = handle_session_act(Some(&params), &session);
        assert!(result.is_ok());
    }

    #[test]
    fn session_act_talk_succeeds() {
        let session = session_with_game();
        let params = serde_json::json!({"kind": "talk", "id": "npc"});
        let result = handle_session_act(Some(&params), &session);
        assert!(result.is_ok());
    }

    #[test]
    fn session_act_ability_succeeds() {
        let session = session_with_game();
        let params = serde_json::json!({"kind": "ability", "id": "look"});
        let result = handle_session_act(Some(&params), &session);
        assert!(result.is_ok());
    }

    #[test]
    fn session_act_no_session_errors() {
        let params = serde_json::json!({"kind": "exit", "id": "end"});
        let result = handle_session_act(Some(&params), &empty_session());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32000);
    }

    #[test]
    fn session_history_without_session_errors() {
        let result = handle_session_history(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_history_with_session_succeeds() {
        let session = session_with_game();
        let params = serde_json::json!({"kind": "exit", "id": "end"});
        let _ = handle_session_act(Some(&params), &session);
        let result = handle_session_history(&session);
        assert!(result.is_ok());
        let val = result.unwrap();
        let history = val.as_array().unwrap();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn session_narrate_without_session_errors() {
        let result = handle_session_narrate(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_narrate_with_session_succeeds() {
        let result = handle_session_narrate(&session_with_game());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.get("player_action").is_some());
    }

    #[test]
    fn session_graph_without_session_errors() {
        let result = handle_session_graph(&empty_session());
        assert!(result.is_err());
    }

    #[test]
    fn session_graph_with_session_succeeds() {
        let result = handle_session_graph(&session_with_game());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.as_str().unwrap().contains("digraph"));
    }
}
