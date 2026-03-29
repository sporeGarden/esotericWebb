// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal composition enrichment pipeline for [`GameSession`].
//!
//! Six-phase best-effort pipeline that runs after mechanical action
//! resolution. All primal calls degrade silently — gameplay is never
//! blocked by missing or slow primals.
//!
//! Phases:
//! 1. AI narration (Squirrel via `ai.suggest` / `ai.query`)
//! 2. NPC dialogue (talk actions only, Squirrel via `ai.query`)
//! 3. Flow evaluation (local science — no IPC)
//! 4. Scene push to UI (petalTongue via `render_scene`)
//! 5. Provenance vertex append (rhizoCrypt DAG)
//! 6. Session completion check (DAG close on ending)

use super::GameSession;
use super::types::{ActionKind, PrimalEnrichment, VoiceEnrichment};
use crate::science::FLOW_CHANNEL_WIDTH;
use crate::science::flow::flow_channel_metrics;

impl GameSession {
    /// Best-effort enrichment via primal composition.
    ///
    /// Calls AI primals directly (no ludoSpring mediation) and uses
    /// local science for flow evaluation. All calls degrade silently.
    pub(crate) fn enrich_action(
        &mut self,
        kind: ActionKind,
        id: &str,
        outcome_text: &str,
    ) -> PrimalEnrichment {
        let mut enrichment = PrimalEnrichment::default();
        let turn = self.turn;
        let action_str = format!("{kind}:{id}");

        let Some(bridge) = self.bridge.as_mut() else {
            self.enrich_flow_locally(&mut enrichment);
            return enrichment;
        };

        // Phase 1: AI narration via Squirrel (ai.suggest / ai.query)
        if bridge.has(crate::ipc::domain::AI) {
            let params = serde_json::json!({
                "action": action_str,
                "outcome": outcome_text,
                "turn": turn,
            });
            if let Ok(chat) = bridge.narrate_action(&params) {
                if chat.model != "none" {
                    enrichment.ai_narration = Some(chat.text);
                }
            }
        }

        if enrichment.ai_narration.is_none() && bridge.has(crate::ipc::domain::AI) {
            let prompt =
                format!("Narrate this RPG moment. Action: {action_str}. Outcome: {outcome_text}");
            if let Ok(chat) = bridge.ai_narrate(&prompt) {
                if chat.model != "none" {
                    enrichment.ai_narration = Some(chat.text);
                }
            }
        }

        // Phase 2: NPC dialogue via Squirrel (ai.query with NPC context)
        if kind == ActionKind::Talk && bridge.has(crate::ipc::domain::AI) {
            let params = serde_json::json!({
                "npc_id": id,
                "context": outcome_text,
                "turn": turn,
            });
            if let Ok(dialogue) = bridge.npc_dialogue(&params) {
                if !dialogue.degraded {
                    enrichment.npc_dialogue = Some(dialogue.text);
                }
                enrichment.voice_notes = dialogue
                    .voice_notes
                    .into_iter()
                    .map(|v| VoiceEnrichment {
                        voice_id: v.voice_id,
                        text: v.text,
                    })
                    .collect();
            }
        }

        // Phase 3: Flow evaluation (local science — no IPC)
        self.enrich_flow_locally(&mut enrichment);

        enrichment
    }

    /// Evaluate flow state using local science (no primal IPC required).
    ///
    /// Derives challenge from turn progress and skill from action diversity.
    /// These are heuristic estimates; a dedicated game-science primal would
    /// provide more sophisticated models.
    fn enrich_flow_locally(&self, enrichment: &mut PrimalEnrichment) {
        let challenge = self.estimated_challenge();
        let skill = self.estimated_skill();
        let result = flow_channel_metrics(challenge, skill, FLOW_CHANNEL_WIDTH);
        enrichment.flow_score = Some(result.flow_score);
        enrichment.in_flow = Some(result.in_flow);
    }

    /// Heuristic challenge estimate based on turn progression.
    fn estimated_challenge(&self) -> f64 {
        let progress = (f64::from(self.turn) / 50.0).min(1.0);
        0.3 + progress * 0.4
    }

    /// Heuristic skill estimate based on action diversity.
    #[expect(
        clippy::cast_precision_loss,
        reason = "history length is realistically small"
    )]
    fn estimated_skill(&self) -> f64 {
        if self.history.is_empty() {
            return 0.5;
        }
        let unique_nodes: std::collections::HashSet<&str> =
            self.history.iter().map(|h| h.node_after.as_str()).collect();
        let diversity = unique_nodes.len() as f64 / self.history.len() as f64;
        diversity.mul_add(0.4, 0.3).min(1.0)
    }

    /// Push current scene state to petalTongue for rendering.
    pub(crate) fn push_scene_to_ui(&mut self) -> bool {
        let npcs = self.current_scene_npcs();
        let scene_desc = self.director.current_scene_description(&self.bundle);
        let node_id = self.director.current_node_id().to_owned();
        let turn = self.turn;
        let is_ending = self.director.is_at_ending(&self.bundle);

        let Some(bridge) = self.bridge.as_mut() else {
            return false;
        };

        let scene = serde_json::json!({
            "node": node_id,
            "description": scene_desc,
            "npcs": npcs,
            "turn": turn,
            "is_ending": is_ending,
        });
        match bridge.render_scene(&scene) {
            Ok(()) => true,
            Err(e) => {
                tracing::debug!("scene push degraded: {e}");
                false
            }
        }
    }

    /// Complete the provenance session if the game has reached an ending.
    pub(crate) fn complete_provenance_if_ended(&mut self) {
        if !self.director.is_at_ending(&self.bundle) {
            return;
        }
        let session_id = self.state.session_id.clone();
        if session_id.is_empty() {
            return;
        }
        let turn = self.turn;

        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        let params = serde_json::json!({
            "session_id": session_id,
            "turns": turn,
            "completed": true,
        });
        if let Err(e) = bridge.dag_session_complete(&params) {
            tracing::debug!("provenance session completion degraded: {e}");
        }
    }

    /// Record a provenance vertex via the DAG primal if connected.
    ///
    /// Provenance is best-effort: failures degrade silently so gameplay
    /// is never blocked by an unavailable or slow primal.
    pub(crate) fn record_provenance_vertex(&mut self, action: &str, node_after: &str) {
        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        if !bridge.has(crate::ipc::domain::DAG) {
            return;
        }
        let parent_ids: Vec<&str> = self
            .history
            .len()
            .checked_sub(2)
            .and_then(|i| self.history.get(i))
            .map(|prev| vec![prev.node_after.as_str()])
            .unwrap_or_default();

        let vertex = serde_json::json!({
            "session_id": self.state.session_id,
            "data": {
                "type": "player_action",
                "action": action,
                "node_after": node_after,
                "turn": self.turn,
            },
            "parents": parent_ids,
        });
        if let Err(e) = bridge.dag_event_append(&vertex) {
            tracing::debug!("provenance append degraded: {e}");
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use crate::content::{AbilityDef, ContentBundle, NpcDef, SceneContent, WorldMeta};
    use crate::director::GameDirector;
    use crate::ipc::bridge::PrimalBridge;
    use crate::narrative::effect::StateEffect;
    use crate::narrative::predicate::StatePredicate;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use crate::session::GameSession;
    use crate::session::types::ActionKind;
    use crate::state::WorldState;
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
                        label: Some("End".to_owned()),
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
                description: "Start.".to_owned(),
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
                description: "A room.".to_owned(),
                npcs: vec!["npc".to_owned()],
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

    fn session_with_standalone_bridge() -> GameSession {
        let bundle = test_bundle();
        let director = GameDirector::new(&bundle).unwrap();
        let bridge = PrimalBridge::standalone();
        GameSession::from_parts(bundle, director, WorldState::new(), Some(bridge))
    }

    #[test]
    fn enrich_exit_action_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        let enrichment = s.enrich_action(ActionKind::Exit, "room", "You enter the room.");
        assert!(enrichment.ai_narration.is_none());
        assert!(enrichment.npc_dialogue.is_none());
        assert!(enrichment.voice_notes.is_empty());
        // Flow is always computed locally (no IPC needed).
        assert!(enrichment.flow_score.is_some());
    }

    #[test]
    fn enrich_talk_action_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        let enrichment = s.enrich_action(ActionKind::Talk, "npc", "You speak to NPC.");
        assert!(enrichment.ai_narration.is_none());
        assert!(enrichment.npc_dialogue.is_none());
    }

    #[test]
    fn push_scene_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        let pushed = s.push_scene_to_ui();
        // Standalone bridge has no visualization domain, but call_fire
        // returns Ok(()) when domain is absent — push succeeds silently.
        assert!(pushed);
    }

    #[test]
    fn complete_provenance_with_standalone_bridge_at_ending() {
        let mut s = session_with_standalone_bridge();
        s.act(ActionKind::Exit, "room").unwrap();
        s.act(ActionKind::Exit, "start").unwrap();
        s.act(ActionKind::Exit, "ending").unwrap();
        assert!(s.is_ended());
        s.complete_provenance_if_ended();
    }

    #[test]
    fn complete_provenance_with_standalone_bridge_not_ending() {
        let mut s = session_with_standalone_bridge();
        s.complete_provenance_if_ended();
    }

    #[test]
    fn record_provenance_vertex_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        s.record_provenance_vertex("exit:room", "room");
    }

    #[test]
    fn record_provenance_vertex_with_history() {
        let mut s = session_with_standalone_bridge();
        s.act(ActionKind::Exit, "room").unwrap();
        s.record_provenance_vertex("exit:start", "start");
    }

    #[test]
    fn full_act_pipeline_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        let (text, ctx) = s.act(ActionKind::Exit, "room").unwrap();
        assert!(!text.is_empty());
        assert!(ctx.enrichment.ai_narration.is_none());
        assert!(ctx.enrichment.scene_pushed);
    }

    #[test]
    fn talk_act_pipeline_with_standalone_bridge() {
        let mut s = session_with_standalone_bridge();
        s.act(ActionKind::Exit, "room").unwrap();
        let (_, ctx) = s.act(ActionKind::Talk, "npc").unwrap();
        assert!(ctx.enrichment.npc_dialogue.is_none());
    }
}
