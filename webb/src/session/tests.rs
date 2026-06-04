// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;
use crate::content::{AbilityDef, NpcDef, SceneContent, WorldMeta};
use crate::narrative::effect::StateEffect;
use crate::narrative::predicate::StatePredicate;
use crate::narrative::{NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType};
use std::collections::HashMap;

#[expect(clippy::too_many_lines, reason = "test fixture construction")]
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
            exits: vec![
                NarrativeEdge {
                    target: "room".to_owned(),
                    conditions: vec![],
                    priority: 0,
                    transition_type: TransitionType::SamePlane,
                    label: Some("Enter room".to_owned()),
                },
                NarrativeEdge {
                    target: "ending".to_owned(),
                    conditions: vec![StatePredicate::HasKnowledge("secret".to_owned())],
                    priority: 1,
                    transition_type: TransitionType::SamePlane,
                    label: Some("Confront".to_owned()),
                },
            ],
            is_start: true,
            is_ending: false,
            label: None,
        },
    );
    scenes.insert(
        "start".to_owned(),
        SceneContent {
            id: "start".to_owned(),
            description: "A threshold.".to_owned(),
            npcs: vec![],
            items: vec![],
        },
    );

    nodes.insert(
        "room".to_owned(),
        NarrativeNode {
            id: "room".to_owned(),
            scene_type: SceneType::Dialogue,
            content_ref: "room".to_owned(),
            preconditions: vec![],
            effects: vec![StateEffect::AddKnowledge("secret".to_owned())],
            exits: vec![NarrativeEdge {
                target: "start".to_owned(),
                conditions: vec![],
                priority: 0,
                transition_type: TransitionType::SamePlane,
                label: Some("Return".to_owned()),
            }],
            is_start: false,
            is_ending: false,
            label: None,
        },
    );
    scenes.insert(
        "room".to_owned(),
        SceneContent {
            id: "room".to_owned(),
            description: "A dark room.".to_owned(),
            npcs: vec!["npc_a".to_owned()],
            items: vec![],
        },
    );

    nodes.insert(
        "ending".to_owned(),
        NarrativeNode {
            id: "ending".to_owned(),
            scene_type: SceneType::Ending,
            content_ref: "ending".to_owned(),
            preconditions: vec![],
            effects: vec![],
            exits: vec![],
            is_start: false,
            is_ending: true,
            label: None,
        },
    );
    scenes.insert(
        "ending".to_owned(),
        SceneContent {
            id: "ending".to_owned(),
            description: "The end.".to_owned(),
            npcs: vec![],
            items: vec![],
        },
    );

    let mut abilities = HashMap::new();
    abilities.insert(
        "insight".to_owned(),
        AbilityDef {
            id: "insight".to_owned(),
            name: "Insight".to_owned(),
            description: "See the truth.".to_owned(),
            preconditions: vec![],
            effects: vec![StateEffect::SetFlag("seen".to_owned())],
            narration_hint: Some("Eyes open.".to_owned()),
        },
    );

    let mut npcs = HashMap::new();
    npcs.insert(
        "npc_a".to_owned(),
        NpcDef {
            id: "npc_a".to_owned(),
            name: "A".to_owned(),
            role: String::new(),
            knows: vec![],
            trust_initial: 0,
            trust_rewards: std::collections::BTreeMap::new(),
            lies_about: HashMap::new(),
            arc: String::new(),
        },
    );

    ContentBundle {
        meta: WorldMeta {
            name: "Test".to_owned(),
            author: "test".to_owned(),
            version: "0.1.0".to_owned(),
            description: "Test world.".to_owned(),
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

fn session_from_bundle(bundle: ContentBundle) -> GameSession {
    let director = GameDirector::new(&bundle).unwrap();
    GameSession {
        bundle,
        director,
        state: WorldState::new(),
        history: Vec::new(),
        turn: 0,
        bridge: None,
    }
}

#[test]
fn snapshot_shows_start_state() {
    let s = session_from_bundle(test_bundle());
    let snap = s.snapshot();
    assert_eq!(snap.current_node, "start");
    assert!(!snap.is_ending);
    assert!(snap.session_active);
    assert!(!snap.available_actions.is_empty());
}

#[test]
fn available_actions_include_exits_and_abilities() {
    let s = session_from_bundle(test_bundle());
    let actions = s.available_actions();
    let exit_count = actions
        .iter()
        .filter(|a| a.kind == ActionKind::Exit)
        .count();
    let ability_count = actions
        .iter()
        .filter(|a| a.kind == ActionKind::Ability)
        .count();
    let examine_count = actions
        .iter()
        .filter(|a| a.kind == ActionKind::Examine)
        .count();
    assert_eq!(exit_count, 1);
    assert_eq!(ability_count, 1);
    assert_eq!(examine_count, 1);
}

#[test]
fn act_exit_transitions_scene() {
    let mut s = session_from_bundle(test_bundle());
    let (text, ctx) = s.act(ActionKind::Exit, "room").unwrap();
    assert!(!text.is_empty());
    assert_eq!(ctx.turn, 1);
    assert_eq!(s.snapshot().current_node, "room");
}

#[test]
fn act_ability_applies_effects() {
    let mut s = session_from_bundle(test_bundle());
    let (_, _) = s.act(ActionKind::Ability, "insight").unwrap();
    assert!(s.snapshot().flags.contains(&"seen".to_owned()));
}

#[test]
fn full_playthrough_to_ending() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    s.act(ActionKind::Exit, "start").unwrap();
    s.act(ActionKind::Exit, "ending").unwrap();
    assert!(s.is_ended());
    assert_eq!(s.history().len(), 3);
}

#[test]
fn narration_context_includes_hints() {
    let mut s = session_from_bundle(test_bundle());
    let (_, ctx) = s.act(ActionKind::Ability, "insight").unwrap();
    assert!(!ctx.narration_hints.is_empty());
    assert!(ctx.narration_hints.iter().any(|h| h.contains("Eyes")));
}

#[test]
fn snapshot_serializes_to_json() {
    let s = session_from_bundle(test_bundle());
    let snap = s.snapshot();
    let json = serde_json::to_string_pretty(&snap);
    assert!(json.is_ok());
}

#[test]
fn act_returns_default_enrichment_without_bridge() {
    let mut s = session_from_bundle(test_bundle());
    let (_, ctx) = s.act(ActionKind::Exit, "room").unwrap();
    assert!(ctx.enrichment.ai_narration.is_none());
    assert!(ctx.enrichment.npc_dialogue.is_none());
    assert!(ctx.enrichment.voice_notes.is_empty());
    assert!(ctx.enrichment.flow_score.is_some());
    assert!(!ctx.enrichment.scene_pushed);
}

#[test]
fn act_enrichment_serializes_to_json() {
    let mut s = session_from_bundle(test_bundle());
    let (_, ctx) = s.act(ActionKind::Exit, "room").unwrap();
    let json = serde_json::to_string(&ctx).unwrap();
    assert!(json.contains("enrichment"));
}

#[test]
fn take_bridge_returns_none_for_standalone() {
    let mut s = session_from_bundle(test_bundle());
    assert!(s.take_bridge().is_none());
}

#[test]
fn narration_context_has_default_enrichment() {
    let s = session_from_bundle(test_bundle());
    let ctx = s.narration_context();
    assert!(ctx.enrichment.ai_narration.is_none());
    assert!(!ctx.enrichment.scene_pushed);
}

#[test]
fn initialize_provenance_is_noop_without_bridge() {
    let mut s = session_from_bundle(test_bundle());
    s.initialize_provenance();
    assert!(s.snapshot().trust.is_empty());
}

#[test]
fn act_talk_returns_narration() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    let (text, ctx) = s.act(ActionKind::Talk, "npc_a").unwrap();
    assert!(!text.is_empty());
    assert_eq!(ctx.turn, 2);
    assert!(ctx.player_action.contains("talk"));
}

#[test]
fn act_examine_returns_scene_description() {
    let mut s = session_from_bundle(test_bundle());
    let (text, ctx) = s.act(ActionKind::Examine, "examine").unwrap();
    assert!(!text.is_empty());
    assert_eq!(ctx.turn, 1);
    assert!(ctx.player_action.contains("examine"));
}

#[test]
fn from_parts_creates_valid_session() {
    let bundle = test_bundle();
    let director = GameDirector::new(&bundle).unwrap();
    let state = WorldState::new();
    let s = GameSession::from_parts(bundle, director, state, None);
    assert_eq!(s.snapshot().current_node, "start");
    assert!(s.bridge().is_none());
}

#[test]
fn dag_overlay_initial_state() {
    let s = session_from_bundle(test_bundle());
    let overlay = s.dag_overlay();
    assert!(overlay.visited.contains("start"));
    assert_eq!(overlay.current_node.as_deref(), Some("start"));
    assert!(overlay.edges_taken.is_empty());
    assert!(!overlay.available_targets.is_empty());
}

#[test]
fn dag_overlay_tracks_movement() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    let overlay = s.dag_overlay();
    assert!(overlay.visited.contains("start"));
    assert!(overlay.visited.contains("room"));
    assert!(
        overlay
            .edges_taken
            .contains(&("start".to_owned(), "room".to_owned()))
    );
    assert_eq!(overlay.current_node.as_deref(), Some("room"));
}

#[test]
fn dag_overlay_shows_gated_paths() {
    let s = session_from_bundle(test_bundle());
    let overlay = s.dag_overlay();
    assert!(
        overlay
            .gated_targets
            .contains(&("start".to_owned(), "ending".to_owned()))
    );
}

#[test]
fn to_dot_produces_valid_output() {
    let s = session_from_bundle(test_bundle());
    let dot = s.to_dot();
    assert!(dot.contains("digraph"));
    assert!(dot.contains("start"));
}

#[test]
fn narration_context_at_session_start() {
    let s = session_from_bundle(test_bundle());
    let ctx = s.narration_context();
    assert!(ctx.player_action.contains("session start"));
    assert!(ctx.outcome_text.is_empty());
    assert_eq!(ctx.turn, 0);
}

#[test]
fn narration_context_after_action() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    let ctx = s.narration_context();
    assert!(ctx.player_action.contains("exit:room"));
    assert!(!ctx.outcome_text.is_empty());
    assert_eq!(ctx.turn, 1);
}

#[test]
fn with_standalone_bridge_enrichment_degrades() {
    let bundle = test_bundle();
    let director = GameDirector::new(&bundle).unwrap();
    let bridge = crate::ipc::bridge::PrimalBridge::standalone();
    let mut s = GameSession::from_parts(bundle, director, WorldState::new(), Some(bridge));
    let (_, ctx) = s.act(ActionKind::Exit, "room").unwrap();
    assert!(ctx.enrichment.ai_narration.is_none());
    assert!(ctx.enrichment.npc_dialogue.is_none());
    assert!(ctx.enrichment.voice_notes.is_empty());
    assert!(ctx.enrichment.flow_score.is_some());
}

#[test]
fn action_kind_display() {
    assert_eq!(ActionKind::Exit.to_string(), "exit");
    assert_eq!(ActionKind::Talk.to_string(), "talk");
    assert_eq!(ActionKind::Ability.to_string(), "ability");
    assert_eq!(ActionKind::Examine.to_string(), "examine");
}

#[test]
fn action_kind_parse_round_trip() {
    for kind in [
        ActionKind::Exit,
        ActionKind::Talk,
        ActionKind::Ability,
        ActionKind::Examine,
    ] {
        let parsed = ActionKind::parse(&kind.to_string()).unwrap();
        assert_eq!(parsed, kind);
    }
}

#[test]
fn action_kind_parse_unknown_errors() {
    assert!(ActionKind::parse("fly").is_err());
    assert!(ActionKind::parse("").is_err());
}

#[test]
fn action_kind_serde_round_trip() {
    let action = AvailableAction {
        kind: ActionKind::Talk,
        id: "npc_a".to_owned(),
        label: "Talk to A".to_owned(),
        detail: Some("details".to_owned()),
    };
    let json = serde_json::to_string(&action).unwrap();
    let back: AvailableAction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.kind, ActionKind::Talk);
    assert_eq!(back.id, "npc_a");
}

#[test]
fn history_is_empty_initially() {
    let s = session_from_bundle(test_bundle());
    assert!(s.history().is_empty());
}

#[test]
fn history_grows_with_actions() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    s.act(ActionKind::Examine, "examine").unwrap();
    assert_eq!(s.history().len(), 2);
    assert_eq!(s.history()[0].turn, 1);
    assert_eq!(s.history()[1].turn, 2);
}

#[test]
fn is_ended_false_initially() {
    let s = session_from_bundle(test_bundle());
    assert!(!s.is_ended());
}

#[test]
fn push_scene_returns_false_without_bridge() {
    let mut s = session_from_bundle(test_bundle());
    assert!(!s.push_scene_to_ui());
}

#[test]
fn complete_provenance_noop_without_bridge() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    s.act(ActionKind::Exit, "start").unwrap();
    s.act(ActionKind::Exit, "ending").unwrap();
    assert!(s.is_ended());
    s.complete_provenance_if_ended();
}

#[test]
fn record_provenance_noop_without_bridge() {
    let mut s = session_from_bundle(test_bundle());
    s.record_provenance_vertex("test", "room");
}

#[test]
fn sorted_knowledge_returns_alphabetical() {
    let mut s = session_from_bundle(test_bundle());
    s.state.knowledge.insert("zebra".to_owned());
    s.state.knowledge.insert("alpha".to_owned());
    s.state.knowledge.insert("middle".to_owned());
    let k = s.sorted_knowledge();
    assert_eq!(k, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn sorted_flags_returns_alphabetical() {
    let mut s = session_from_bundle(test_bundle());
    s.state.flags.insert("z_flag".to_owned());
    s.state.flags.insert("a_flag".to_owned());
    let f = s.sorted_flags();
    assert_eq!(f, vec!["a_flag", "z_flag"]);
}

#[test]
fn narration_hints_from_abilities() {
    let s = session_from_bundle(test_bundle());
    let hints = s.narration_hints();
    assert!(hints.iter().any(|h| h.contains("Eyes")));
}

#[test]
fn narration_context_initial_state() {
    let s = session_from_bundle(test_bundle());
    let ctx = s.narration_context();
    assert_eq!(ctx.player_action, "(session start)");
    assert!(ctx.outcome_text.is_empty());
    assert_eq!(ctx.turn, 0);
    assert!(!ctx.scene_description.is_empty());
}

#[test]
fn narration_context_reflects_last_action() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    let ctx = s.narration_context();
    assert!(ctx.player_action.contains("exit:room"));
    assert_eq!(ctx.turn, 1);
    assert!(!ctx.outcome_text.is_empty());
}

#[test]
fn snapshot_reflects_state() {
    let mut s = session_from_bundle(test_bundle());
    s.state.knowledge.insert("lore".to_owned());
    s.state.inventory.insert("sword".to_owned());
    s.state.flags.insert("quest_started".to_owned());
    let snap = s.snapshot();
    assert!(snap.knowledge.contains(&"lore".to_owned()));
    assert!(snap.inventory.contains(&"sword".to_owned()));
    assert!(snap.flags.contains(&"quest_started".to_owned()));
    assert!(snap.session_active);
}

#[test]
fn metrics_initial_state() {
    let s = session_from_bundle(test_bundle());
    let m = s.metrics();
    assert_eq!(m.turns_played, 0);
    assert_eq!(m.nodes_visited, 1); // start node
    assert!(m.nodes_total >= 2);
    assert!(!m.reached_ending);
    assert_eq!(m.backtrack_count, 0);
    assert_eq!(m.npc_interactions, 0);
    assert_eq!(m.ability_uses, 0);
    assert_eq!(m.examine_count, 0);
    assert!(m.exploration_ratio > 0.0);
    assert!(m.exploration_ratio <= 1.0);
}

#[test]
fn metrics_after_navigation() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    let m = s.metrics();
    assert_eq!(m.turns_played, 1);
    assert_eq!(m.nodes_visited, 2);
    assert_eq!(m.backtrack_count, 0);
}

#[test]
fn metrics_counts_backtrack() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Exit, "room").unwrap();
    s.act(ActionKind::Exit, "start").unwrap();
    let m = s.metrics();
    assert_eq!(m.backtrack_count, 1);
}

#[test]
fn metrics_counts_interactions() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Examine, "examine").unwrap();
    let m = s.metrics();
    assert_eq!(m.examine_count, 1);
}

#[test]
fn metrics_actions_per_node() {
    let mut s = session_from_bundle(test_bundle());
    s.act(ActionKind::Examine, "examine").unwrap();
    s.act(ActionKind::Examine, "examine").unwrap();
    let m = s.metrics();
    assert!(m.actions_per_node >= 2.0);
}

#[test]
fn metrics_serializable() {
    let s = session_from_bundle(test_bundle());
    let m = s.metrics();
    let json = serde_json::to_value(&m).unwrap();
    assert!(json.get("turns_played").is_some());
    assert!(json.get("exploration_ratio").is_some());
    assert!(json.get("reached_ending").is_some());
}
