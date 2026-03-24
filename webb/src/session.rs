// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game session — stateful wrapper for AI-as-player and human-as-player.
//!
//! A session holds the loaded content, director position, and world state.
//! It exposes a JSON-friendly API so both the text REPL and the IPC server
//! can drive the same game engine. An AI agent sends `act()` with a
//! choice; a human picks from `available_actions()` in the terminal.

use serde::{Deserialize, Serialize};

use crate::content::ContentBundle;
use crate::director::{DirectorOutcome, GameDirector, PlayerInput};
use crate::ipc::bridge::PrimalBridge;
use crate::state::WorldState;

/// A running game session.
pub struct GameSession {
    bundle: ContentBundle,
    director: GameDirector,
    state: WorldState,
    history: Vec<ActionRecord>,
    turn: u32,
    /// Optional primal bridge for AI narration, rendering, etc.
    bridge: Option<PrimalBridge>,
}

/// One recorded action in the session history.
#[derive(Debug, Clone, Serialize)]
pub struct ActionRecord {
    /// Turn number when this action was taken (1-based after first act).
    pub turn: u32,
    /// Human-readable description of the action (e.g. `kind:id`).
    pub action: String,
    /// Outcome or narration text returned by the director.
    pub outcome: String,
    /// Narrative node id after the action resolved.
    pub node_after: String,
}

/// A possible action the player (human or AI) can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableAction {
    /// Action category: `exit`, `talk`, `ability`, or `examine`.
    pub kind: String,
    /// Target identifier (node id, NPC id, ability id, etc.).
    pub id: String,
    /// Short label shown in the UI or action list.
    pub label: String,
    /// Optional extra text (e.g. ability description or blocked reason).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Full game state snapshot — everything an AI player needs to decide.
#[derive(Debug, Clone, Serialize)]
pub struct GameStateSnapshot {
    /// Whether the session is still active (always true from [`GameSession::snapshot`]).
    pub session_active: bool,
    /// Current turn count.
    pub turn: u32,
    /// Current narrative node id.
    pub current_node: String,
    /// Text description of the current scene.
    pub scene_description: String,
    /// NPC ids present in the current scene.
    pub scene_npcs: Vec<String>,
    /// Whether the current node is an ending.
    pub is_ending: bool,
    /// Knowledge keys the player has gained, sorted.
    pub knowledge: Vec<String>,
    /// Inventory item ids, sorted.
    pub inventory: Vec<String>,
    /// Active flag names, sorted.
    pub flags: Vec<String>,
    /// Trust values per NPC or entity id.
    pub trust: std::collections::HashMap<String, i32>,
    /// Actions the player may take next.
    pub available_actions: Vec<AvailableAction>,
}

/// Context for AI narration generation.
#[derive(Debug, Clone, Serialize)]
pub struct NarrationContext {
    /// Scene description before the action (for continuity).
    pub scene_description: String,
    /// NPCs that were in the scene before the action.
    pub scene_npcs: Vec<String>,
    /// Encoded player action (`kind:id`).
    pub player_action: String,
    /// Director outcome or narration text for this step.
    pub outcome_text: String,
    /// Knowledge keys after the action, sorted.
    pub knowledge: Vec<String>,
    /// Active flags after the action, sorted.
    pub active_flags: Vec<String>,
    /// Turn number after this action.
    pub turn: u32,
    /// Hints from abilities in the bundle (for model guidance).
    pub narration_hints: Vec<String>,
}

impl GameSession {
    /// Start a new game session from a content directory.
    ///
    /// # Errors
    ///
    /// Returns an error if content fails to load or validate.
    /// Create a new session (standalone, no primal bridge).
    pub fn new(content_path: &str) -> Result<Self, String> {
        Self::with_bridge(content_path, None)
    }

    /// Create a new session with an optional primal bridge.
    ///
    /// When a bridge is provided and primals are connected, the session
    /// can use Squirrel for AI narration and PetalTongue for rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if content fails to load or validate.
    pub fn with_bridge(content_path: &str, bridge: Option<PrimalBridge>) -> Result<Self, String> {
        let bundle = ContentBundle::load(content_path).map_err(|e| format!("load: {e}"))?;
        let issues = bundle.validate();
        if !issues.is_empty() {
            return Err(format!(
                "{} validation issue(s): {}",
                issues.len(),
                issues.join("; ")
            ));
        }
        let director = GameDirector::new(&bundle).map_err(|e| format!("director: {e}"))?;
        Ok(Self {
            bundle,
            director,
            state: WorldState::new(),
            history: Vec::new(),
            turn: 0,
            bridge,
        })
    }

    /// Get a reference to the primal bridge, if connected.
    pub const fn bridge(&self) -> Option<&PrimalBridge> {
        self.bridge.as_ref()
    }

    /// Get the full game state snapshot.
    pub fn snapshot(&self) -> GameStateSnapshot {
        let mut knowledge: Vec<String> = self.state.knowledge.iter().cloned().collect();
        knowledge.sort();
        let mut inventory: Vec<String> = self.state.inventory.iter().cloned().collect();
        inventory.sort();
        let mut flags: Vec<String> = self.state.flags.iter().cloned().collect();
        flags.sort();

        GameStateSnapshot {
            session_active: true,
            turn: self.turn,
            current_node: self.director.current_node_id().to_owned(),
            scene_description: self.director.current_scene_description(&self.bundle),
            scene_npcs: self.current_scene_npcs(),
            is_ending: self.director.is_at_ending(&self.bundle),
            knowledge,
            inventory,
            flags,
            trust: self.state.trust.clone(),
            available_actions: self.available_actions(),
        }
    }

    /// List all available actions from the current state.
    pub fn available_actions(&self) -> Vec<AvailableAction> {
        let mut actions = Vec::new();

        for edge in self.director.available_exits(&self.bundle, &self.state) {
            actions.push(AvailableAction {
                kind: "exit".to_owned(),
                id: edge.target.clone(),
                label: edge.label.as_deref().unwrap_or(&edge.target).to_owned(),
                detail: None,
            });
        }

        for npc_id in &self.current_scene_npcs() {
            actions.push(AvailableAction {
                kind: "talk".to_owned(),
                id: npc_id.clone(),
                label: format!("Talk to {npc_id}"),
                detail: None,
            });
        }

        for ability in self.bundle.abilities.values() {
            let can_use = ability.preconditions.iter().all(|p| self.state.evaluate(p));
            actions.push(AvailableAction {
                kind: "ability".to_owned(),
                id: ability.id.clone(),
                label: ability.name.clone(),
                detail: Some(if can_use {
                    ability.description.clone()
                } else {
                    format!("[blocked] {}", ability.description)
                }),
            });
        }

        actions.push(AvailableAction {
            kind: "examine".to_owned(),
            id: "examine".to_owned(),
            label: "Examine surroundings".to_owned(),
            detail: None,
        });

        actions
    }

    /// Execute an action by kind + id. Returns the outcome text and narration context.
    ///
    /// # Errors
    ///
    /// Returns an error if `kind` is not a recognized action kind.
    pub fn act(&mut self, kind: &str, id: &str) -> Result<(String, NarrationContext), String> {
        let input = match kind {
            "exit" => PlayerInput::ChooseExit(id.to_owned()),
            "talk" => PlayerInput::Talk(id.to_owned()),
            "ability" => PlayerInput::UseAbility(id.to_owned()),
            "examine" => PlayerInput::Examine,
            _ => return Err(format!("unknown action kind: {kind}")),
        };

        let scene_before = self.director.current_scene_description(&self.bundle);
        let npcs_before = self.current_scene_npcs();

        let outcome = self.director.process(&input, &mut self.state, &self.bundle);
        self.turn += 1;

        let outcome_text = match &outcome {
            DirectorOutcome::SceneChange { narration, .. } => narration.clone(),
            DirectorOutcome::Narration(s) | DirectorOutcome::NoEffect(s) => s.clone(),
        };

        let action_desc = format!("{kind}:{id}");
        self.history.push(ActionRecord {
            turn: self.turn,
            action: action_desc.clone(),
            outcome: outcome_text.clone(),
            node_after: self.director.current_node_id().to_owned(),
        });

        let mut knowledge: Vec<String> = self.state.knowledge.iter().cloned().collect();
        knowledge.sort();
        let mut active_flags: Vec<String> = self.state.flags.iter().cloned().collect();
        active_flags.sort();

        let narration_hints: Vec<String> = self
            .bundle
            .abilities
            .values()
            .filter_map(|a| a.narration_hint.clone())
            .collect();

        let ctx = NarrationContext {
            scene_description: scene_before,
            scene_npcs: npcs_before,
            player_action: action_desc,
            outcome_text: outcome_text.clone(),
            knowledge,
            active_flags,
            turn: self.turn,
            narration_hints,
        };

        Ok((outcome_text, ctx))
    }

    /// Get the session history.
    pub fn history(&self) -> &[ActionRecord] {
        &self.history
    }

    /// Whether the game has reached an ending.
    pub fn is_ended(&self) -> bool {
        self.director.is_at_ending(&self.bundle)
    }

    /// Build a narration context for the current scene — used by AI-as-generator.
    ///
    /// This gives an AI narrator everything it needs to produce rich,
    /// contextual prose without knowing the engine internals.
    pub fn narration_context(&self) -> NarrationContext {
        let mut knowledge: Vec<String> = self.state.knowledge.iter().cloned().collect();
        knowledge.sort();
        let mut active_flags: Vec<String> = self.state.flags.iter().cloned().collect();
        active_flags.sort();

        let narration_hints: Vec<String> = self
            .bundle
            .abilities
            .values()
            .filter_map(|a| a.narration_hint.clone())
            .collect();

        let last_action = self
            .history
            .last()
            .map_or_else(|| "(session start)".to_owned(), |r| r.action.clone());
        let last_outcome = self
            .history
            .last()
            .map_or(String::new(), |r| r.outcome.clone());

        NarrationContext {
            scene_description: self.director.current_scene_description(&self.bundle),
            scene_npcs: self.current_scene_npcs(),
            player_action: last_action,
            outcome_text: last_outcome,
            knowledge,
            active_flags,
            turn: self.turn,
            narration_hints,
        }
    }

    /// Build a DAG overlay from the current session state.
    ///
    /// This captures three overlapping views:
    /// - **Narrative DAG**: the full authored graph (implicit in the bundle)
    /// - **Live DAG**: current position, available exits, gated paths
    /// - **Played DAG**: visited nodes and edges taken
    pub fn dag_overlay(&self) -> crate::narrative::DagOverlay {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut edges_taken = HashSet::new();

        visited.insert(
            self.bundle
                .narrative
                .start_node()
                .map_or_else(String::new, |n| n.id.clone()),
        );

        let mut prev_node = visited.iter().next().cloned().unwrap_or_default();
        for record in &self.history {
            if record.action.starts_with("exit:") {
                edges_taken.insert((prev_node.clone(), record.node_after.clone()));
            }
            visited.insert(record.node_after.clone());
            prev_node.clone_from(&record.node_after);
        }

        let current_id = self.director.current_node_id();
        let available_exits = self.director.available_exits(&self.bundle, &self.state);
        let available_targets: HashSet<String> =
            available_exits.iter().map(|e| e.target.clone()).collect();

        let mut gated_targets = HashSet::new();
        if let Some(node) = self.bundle.narrative.get(current_id) {
            for edge in &node.exits {
                if !available_targets.contains(&edge.target) {
                    gated_targets.insert((current_id.to_owned(), edge.target.clone()));
                }
            }
        }

        crate::narrative::DagOverlay {
            visited,
            edges_taken,
            current_node: Some(current_id.to_owned()),
            available_targets,
            gated_targets,
        }
    }

    /// Render the narrative DAG as DOT with the current session state overlaid.
    pub fn to_dot(&self) -> String {
        self.bundle.narrative.to_dot_overlay(&self.dag_overlay())
    }

    /// Get the content bundle (for external inspection).
    pub const fn bundle(&self) -> &ContentBundle {
        &self.bundle
    }

    fn current_scene_npcs(&self) -> Vec<String> {
        self.bundle
            .narrative
            .get(self.director.current_node_id())
            .and_then(|node| self.bundle.scenes.get(&node.content_ref))
            .map(|scene| scene.npcs.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::content::{AbilityDef, NpcDef, SceneContent, WorldMeta};
    use crate::narrative::effect::StateEffect;
    use crate::narrative::predicate::StatePredicate;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use std::collections::HashMap;

    #[allow(clippy::too_many_lines)]
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
        let exit_count = actions.iter().filter(|a| a.kind == "exit").count();
        let ability_count = actions.iter().filter(|a| a.kind == "ability").count();
        let examine_count = actions.iter().filter(|a| a.kind == "examine").count();
        assert_eq!(exit_count, 1); // only room is accessible (ending gated)
        assert_eq!(ability_count, 1);
        assert_eq!(examine_count, 1);
    }

    #[test]
    fn act_exit_transitions_scene() {
        let mut s = session_from_bundle(test_bundle());
        let (text, ctx) = s.act("exit", "room").unwrap();
        assert!(!text.is_empty());
        assert_eq!(ctx.turn, 1);
        assert_eq!(s.snapshot().current_node, "room");
    }

    #[test]
    fn act_ability_applies_effects() {
        let mut s = session_from_bundle(test_bundle());
        let (_, _) = s.act("ability", "insight").unwrap();
        assert!(s.snapshot().flags.contains(&"seen".to_owned()));
    }

    #[test]
    fn full_playthrough_to_ending() {
        let mut s = session_from_bundle(test_bundle());
        s.act("exit", "room").unwrap();
        s.act("exit", "start").unwrap();
        s.act("exit", "ending").unwrap();
        assert!(s.is_ended());
        assert_eq!(s.history().len(), 3);
    }

    #[test]
    fn narration_context_includes_hints() {
        let mut s = session_from_bundle(test_bundle());
        let (_, ctx) = s.act("ability", "insight").unwrap();
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
}
