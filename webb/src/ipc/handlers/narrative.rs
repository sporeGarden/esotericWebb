// SPDX-License-Identifier: AGPL-3.0-or-later
//! Narrative and content query handlers.

use serde_json::Value;

use super::SharedSession;

/// `webb.scene.current` — current game scene metadata.
pub(super) fn handle_scene_current(session: &SharedSession) -> Value {
    let guard = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// `webb.narrative.status` — narrative DAG high-level status.
pub(super) fn handle_narrative_status(session: &SharedSession) -> Value {
    let guard = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// `webb.content.list` — loaded content summary.
pub(super) fn handle_content_list(session: &SharedSession) -> Value {
    let guard = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::content::{AbilityDef, ContentBundle, NpcDef, SceneContent, WorldMeta};
    use crate::director::GameDirector;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use crate::session::GameSession;
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
                effects: vec![],
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
    fn scene_current_without_session() {
        let val = handle_scene_current(&empty_session());
        assert!(val.get("scene").unwrap().is_null());
        assert!(
            val.get("note")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("no active session")
        );
    }

    #[test]
    fn scene_current_with_session() {
        let val = handle_scene_current(&session_with_game());
        assert_eq!(val["scene"], "start");
        assert!(val["description"].as_str().unwrap().contains("Start"));
        assert!(!val["npcs"].as_array().unwrap().is_empty());
        assert_eq!(val["is_ending"], false);
    }

    #[test]
    fn narrative_status_without_session() {
        let val = handle_narrative_status(&empty_session());
        assert!(val.get("current_node").unwrap().is_null());
        assert_eq!(val.get("vertex_count").unwrap().as_u64(), Some(0));
    }

    #[test]
    fn narrative_status_with_session() {
        let val = handle_narrative_status(&session_with_game());
        assert_eq!(val["current_node"], "start");
        assert_eq!(val["turn"], 0);
        assert_eq!(val["is_ending"], false);
        assert!(val["actions_available"].as_u64().unwrap() > 0);
    }

    #[test]
    fn content_list_without_session() {
        let val = handle_content_list(&empty_session());
        assert!(val.get("worlds").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("npcs").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("abilities").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("scenes").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn content_list_with_session() {
        let val = handle_content_list(&session_with_game());
        assert!(!val["npcs"].as_array().unwrap().is_empty());
        assert!(!val["abilities"].as_array().unwrap().is_empty());
        assert!(!val["scenes"].as_array().unwrap().is_empty());
        assert!(!val["narrative_nodes"].as_array().unwrap().is_empty());
    }
}
