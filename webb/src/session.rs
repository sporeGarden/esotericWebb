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

/// The kind of action a player can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// Traverse an exit edge to another narrative node.
    Exit,
    /// Talk to an NPC in the current scene.
    Talk,
    /// Use a named ability.
    Ability,
    /// Examine the current scene.
    Examine,
}

impl ActionKind {
    /// Parse an action kind from a string (JSON-RPC boundary).
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a recognised action kind.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "exit" => Ok(Self::Exit),
            "talk" => Ok(Self::Talk),
            "ability" => Ok(Self::Ability),
            "examine" => Ok(Self::Examine),
            _ => Err(format!("unknown action kind: {s}")),
        }
    }
}

impl std::fmt::Display for ActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exit => f.write_str("exit"),
            Self::Talk => f.write_str("talk"),
            Self::Ability => f.write_str("ability"),
            Self::Examine => f.write_str("examine"),
        }
    }
}

/// A possible action the player (human or AI) can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableAction {
    /// Action category.
    pub kind: ActionKind,
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
    /// Primal composition enrichments (AI narration, NPC dialogue, game science).
    pub enrichment: PrimalEnrichment,
}

/// Enrichments from primal composition applied during [`GameSession::act`].
///
/// All fields are best-effort — absent primals result in `None` / empty
/// values. Gameplay is never blocked by missing primals.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PrimalEnrichment {
    /// AI-generated narration text (via ludoSpring → Squirrel, or direct Squirrel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_narration: Option<String>,
    /// NPC dialogue response (for talk actions, via ludoSpring → Squirrel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npc_dialogue: Option<String>,
    /// Internal voice interjections (via ludoSpring game science).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub voice_notes: Vec<VoiceEnrichment>,
    /// Flow evaluation score (0.0 = anxiety, 0.5 = flow, 1.0 = boredom).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_score: Option<f64>,
    /// Whether the player is currently in flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_flow: Option<bool>,
    /// Whether the scene was successfully pushed to the UI primal.
    pub scene_pushed: bool,
}

/// A voice interjection from the game science primal.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceEnrichment {
    /// Voice identifier (e.g. "logic", "empathy").
    pub voice_id: String,
    /// The voice's interjection text.
    pub text: String,
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
    /// can use Squirrel for AI narration and `PetalTongue` for rendering.
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

    /// Build a session from pre-constructed parts (for testing and composition).
    #[must_use]
    pub const fn from_parts(
        bundle: ContentBundle,
        director: GameDirector,
        state: WorldState,
        bridge: Option<PrimalBridge>,
    ) -> Self {
        Self {
            bundle,
            director,
            state,
            history: Vec::new(),
            turn: 0,
            bridge,
        }
    }

    /// Get a reference to the primal bridge, if connected.
    #[must_use]
    pub const fn bridge(&self) -> Option<&PrimalBridge> {
        self.bridge.as_ref()
    }

    /// Remove the primal bridge from this session and return it.
    ///
    /// Used when replacing a session to preserve the bridge for the next one.
    pub const fn take_bridge(&mut self) -> Option<PrimalBridge> {
        self.bridge.take()
    }

    /// Initialize provenance session via DAG primal if available.
    ///
    /// Creates a DAG session and stores the returned session ID in
    /// [`WorldState::session_id`] for subsequent event appends.
    /// Degrades silently if the DAG primal is absent.
    pub fn initialize_provenance(&mut self) {
        let world_name = self.bundle.meta.name.clone();
        let world_version = self.bundle.meta.version.clone();

        let session_id = {
            let Some(bridge) = self.bridge.as_mut() else {
                return;
            };
            let params = serde_json::json!({
                "world": world_name,
                "content_version": world_version,
            });
            match bridge.dag_session_create(&params) {
                Ok(Some(id)) => Some(id),
                Ok(None) => {
                    tracing::debug!("DAG primal unavailable — provenance session not created");
                    None
                }
                Err(e) => {
                    tracing::debug!("provenance session creation degraded: {e}");
                    None
                }
            }
        };

        if let Some(id) = session_id {
            tracing::debug!("provenance session created: {id}");
            self.state.session_id = id;
        }
    }

    /// Best-effort enrichment via primal composition.
    ///
    /// Calls AI and game science primals to enrich the mechanical action
    /// outcome. All calls degrade silently — gameplay is never blocked.
    fn enrich_action(
        &mut self,
        kind: ActionKind,
        id: &str,
        outcome_text: &str,
    ) -> PrimalEnrichment {
        let mut enrichment = PrimalEnrichment::default();
        let turn = self.turn;
        let action_str = format!("{kind}:{id}");

        let Some(bridge) = self.bridge.as_mut() else {
            return enrichment;
        };

        if bridge.has(crate::ipc::DOMAIN_GAME) {
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

        if enrichment.ai_narration.is_none() && bridge.has(crate::ipc::DOMAIN_AI) {
            let prompt =
                format!("Narrate this RPG moment. Action: {action_str}. Outcome: {outcome_text}");
            if let Ok(chat) = bridge.ai_narrate(&prompt) {
                if chat.model != "none" {
                    enrichment.ai_narration = Some(chat.text);
                }
            }
        }

        if kind == ActionKind::Talk && bridge.has(crate::ipc::DOMAIN_GAME) {
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

        if bridge.has(crate::ipc::DOMAIN_GAME) {
            let params = serde_json::json!({ "turn": turn, "action_kind": kind.to_string() });
            if let Ok(flow) = bridge.evaluate_flow(&params) {
                enrichment.flow_score = Some(flow.flow_score);
                enrichment.in_flow = Some(flow.in_flow);
            }
        }

        enrichment
    }

    /// Push current scene state to petalTongue for rendering.
    fn push_scene_to_ui(&mut self) -> bool {
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
    fn complete_provenance_if_ended(&mut self) {
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

    /// Get the full game state snapshot.
    #[must_use]
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
    #[must_use]
    pub fn available_actions(&self) -> Vec<AvailableAction> {
        let mut actions = Vec::new();

        for edge in self.director.available_exits(&self.bundle, &self.state) {
            actions.push(AvailableAction {
                kind: ActionKind::Exit,
                id: edge.target.clone(),
                label: edge.label.as_deref().unwrap_or(&edge.target).to_owned(),
                detail: None,
            });
        }

        for npc_id in &self.current_scene_npcs() {
            actions.push(AvailableAction {
                kind: ActionKind::Talk,
                id: npc_id.clone(),
                label: format!("Talk to {npc_id}"),
                detail: None,
            });
        }

        for ability in self.bundle.abilities.values() {
            let can_use = ability.preconditions.iter().all(|p| self.state.evaluate(p));
            actions.push(AvailableAction {
                kind: ActionKind::Ability,
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
            kind: ActionKind::Examine,
            id: "examine".to_owned(),
            label: "Examine surroundings".to_owned(),
            detail: None,
        });

        actions
    }

    /// Execute an action by kind + id. Returns the outcome text and narration context.
    ///
    /// The full primal composition pipeline runs after mechanical resolution:
    /// 1. AI narration enrichment (ludoSpring → Squirrel, or direct Squirrel)
    /// 2. NPC dialogue for talk actions (ludoSpring → Squirrel)
    /// 3. Flow evaluation (ludoSpring game science)
    /// 4. Scene push to UI (`petalTongue` via `render_scene`)
    /// 5. Provenance vertex append (rhizoCrypt DAG)
    /// 6. Session completion check (DAG close on ending)
    ///
    /// All primal calls are best-effort — failures degrade silently.
    ///
    /// # Errors
    ///
    /// Returns an error if the action fails mechanically (e.g. invalid exit).
    pub fn act(
        &mut self,
        kind: ActionKind,
        id: &str,
    ) -> Result<(String, NarrationContext), String> {
        let input = match kind {
            ActionKind::Exit => PlayerInput::ChooseExit(id.to_owned()),
            ActionKind::Talk => PlayerInput::Talk(id.to_owned()),
            ActionKind::Ability => PlayerInput::UseAbility(id.to_owned()),
            ActionKind::Examine => PlayerInput::Examine,
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
        let node_after = self.director.current_node_id().to_owned();

        self.history.push(ActionRecord {
            turn: self.turn,
            action: action_desc.clone(),
            outcome: outcome_text.clone(),
            node_after: node_after.clone(),
        });

        let mut enrichment = self.enrich_action(kind, id, &outcome_text);
        enrichment.scene_pushed = self.push_scene_to_ui();
        self.record_provenance_vertex(&action_desc, &node_after);
        self.complete_provenance_if_ended();

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
            enrichment,
        };

        Ok((outcome_text, ctx))
    }

    /// Record a provenance vertex via the DAG primal if connected.
    ///
    /// Provenance is best-effort: failures degrade silently so gameplay
    /// is never blocked by an unavailable or slow primal.
    fn record_provenance_vertex(&mut self, action: &str, node_after: &str) {
        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        if !bridge.has(crate::ipc::DOMAIN_DAG) {
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

    /// Get the session history.
    #[must_use]
    pub fn history(&self) -> &[ActionRecord] {
        &self.history
    }

    /// Whether the game has reached an ending.
    #[must_use]
    pub fn is_ended(&self) -> bool {
        self.director.is_at_ending(&self.bundle)
    }

    /// Build a narration context for the current scene — used by AI-as-generator.
    ///
    /// This gives an AI narrator everything it needs to produce rich,
    /// contextual prose without knowing the engine internals.
    #[must_use]
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
            enrichment: PrimalEnrichment::default(),
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
    #[must_use]
    pub fn to_dot(&self) -> String {
        self.bundle.narrative.to_dot_overlay(&self.dag_overlay())
    }

    /// Get the content bundle (for external inspection).
    #[must_use]
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
        assert_eq!(exit_count, 1); // only room is accessible (ending gated)
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
        assert!(ctx.enrichment.flow_score.is_none());
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
        assert!(ctx.enrichment.flow_score.is_none());
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
}
