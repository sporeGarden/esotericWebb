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

    fn empty_session() -> SharedSession {
        std::sync::Arc::new(std::sync::Mutex::new(None))
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
    fn narrative_status_without_session() {
        let val = handle_narrative_status(&empty_session());
        assert!(val.get("current_node").unwrap().is_null());
        assert_eq!(val.get("vertex_count").unwrap().as_u64(), Some(0));
    }

    #[test]
    fn content_list_without_session() {
        let val = handle_content_list(&empty_session());
        assert!(val.get("worlds").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("npcs").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("abilities").unwrap().as_array().unwrap().is_empty());
        assert!(val.get("scenes").unwrap().as_array().unwrap().is_empty());
    }
}
