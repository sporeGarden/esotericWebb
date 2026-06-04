// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game session — stateful wrapper for AI-as-player and human-as-player.
//!
//! A session holds the loaded content, director position, and world state.
//! It exposes a JSON-friendly API so both the text REPL and the IPC server
//! can drive the same game engine. An AI agent sends `act()` with a
//! choice; a human picks from `available_actions()` in the terminal.
//!
//! ## Module layout
//!
//! - [`types`]: public data types (`ActionKind`, `ActionRecord`, etc.)
//! - `enrichment`: primal composition pipeline (AI narration, game science, provenance)
//! - This module: `GameSession` struct, core methods, and tests

mod enrichment;
pub mod types;

pub use types::{
    ActionKind, ActionRecord, AvailableAction, GameStateSnapshot, NarrationContext,
    PrimalEnrichment, SessionMetrics, VoiceEnrichment,
};

use crate::content::ContentBundle;
use crate::director::{DirectorOutcome, GameDirector, PlayerInput};
use crate::ipc::bridge::PrimalBridge;
use crate::state::WorldState;

/// A running game session.
pub struct GameSession {
    pub(crate) bundle: ContentBundle,
    pub(crate) director: GameDirector,
    pub(crate) state: WorldState,
    pub(crate) history: Vec<ActionRecord>,
    pub(crate) turn: u32,
    pub(crate) bridge: Option<PrimalBridge>,
}

impl GameSession {
    /// Start a new game session from a content directory.
    ///
    /// # Errors
    ///
    /// Returns an error if content fails to load or validate.
    /// Create a new session (standalone, no primal bridge).
    pub fn new(content_path: &str) -> crate::error::Result<Self> {
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
    pub fn with_bridge(
        content_path: &str,
        bridge: Option<PrimalBridge>,
    ) -> crate::error::Result<Self> {
        let bundle = ContentBundle::load(content_path)?;
        let issues = bundle.validate();
        if !issues.is_empty() {
            return Err(crate::error::WebbError::Validation {
                count: issues.len(),
                summary: issues.join("; "),
            });
        }
        let director = GameDirector::new(&bundle)?;
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

    /// Get the full game state snapshot.
    #[must_use]
    pub fn snapshot(&self) -> GameStateSnapshot {
        let mut inventory: Vec<String> = self.state.inventory.iter().cloned().collect();
        inventory.sort();

        GameStateSnapshot {
            session_active: true,
            turn: self.turn,
            current_node: self.director.current_node_id().to_owned(),
            scene_description: self.director.current_scene_description(&self.bundle),
            scene_npcs: self.current_scene_npcs(),
            is_ending: self.director.is_at_ending(&self.bundle),
            knowledge: self.sorted_knowledge(),
            inventory,
            flags: self.sorted_flags(),
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
    /// 1. AI narration enrichment (Squirrel `ai.query`)
    /// 2. NPC dialogue for talk actions (Squirrel `ai.query`)
    /// 3. Flow evaluation (local `science::flow`)
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
    ) -> crate::error::Result<(String, NarrationContext)> {
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

        let ctx = NarrationContext {
            scene_description: scene_before,
            scene_npcs: npcs_before,
            player_action: action_desc,
            outcome_text: outcome_text.clone(),
            knowledge: self.sorted_knowledge(),
            active_flags: self.sorted_flags(),
            turn: self.turn,
            narration_hints: self.narration_hints(),
            enrichment,
        };

        Ok((outcome_text, ctx))
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
            knowledge: self.sorted_knowledge(),
            active_flags: self.sorted_flags(),
            turn: self.turn,
            narration_hints: self.narration_hints(),
            enrichment: PrimalEnrichment::default(),
        }
    }

    /// Compute session engagement metrics (V13 — game science / DDA).
    ///
    /// Derived entirely from session history and bundle metadata.
    /// Zero allocation cost when not called — metrics are computed on demand.
    #[must_use]
    pub fn metrics(&self) -> SessionMetrics {
        use std::collections::HashSet;

        let mut visited_nodes: HashSet<&str> = HashSet::new();
        let mut backtrack_count: u32 = 0;
        let mut npc_interactions: u32 = 0;
        let mut ability_uses: u32 = 0;
        let mut examine_count: u32 = 0;

        if let Some(start) = self.bundle.narrative.start_node() {
            visited_nodes.insert(&start.id);
        }

        for record in &self.history {
            let was_visited = visited_nodes.contains(record.node_after.as_str());
            visited_nodes.insert(&record.node_after);

            if record.action.starts_with("exit:") && was_visited {
                backtrack_count += 1;
            }
            if record.action.starts_with("talk:") {
                npc_interactions += 1;
            }
            if record.action.starts_with("ability:") {
                ability_uses += 1;
            }
            if record.action.starts_with("examine:") {
                examine_count += 1;
            }
        }

        #[expect(clippy::cast_possible_truncation, reason = "graph sizes <<2^32")]
        let nodes_visited = visited_nodes.len() as u32;
        #[expect(clippy::cast_possible_truncation, reason = "graph sizes <<2^32")]
        let nodes_total = self.bundle.narrative.node_count() as u32;
        let exploration_ratio = if nodes_total > 0 {
            f64::from(nodes_visited) / f64::from(nodes_total)
        } else {
            0.0
        };
        let actions_per_node = if nodes_visited > 0 {
            #[expect(clippy::cast_precision_loss, reason = "action counts fit f64")]
            let actions = self.history.len() as f64;
            actions / f64::from(nodes_visited)
        } else {
            0.0
        };

        SessionMetrics {
            turns_played: self.turn,
            nodes_visited,
            nodes_total,
            exploration_ratio,
            backtrack_count,
            npc_interactions,
            ability_uses,
            examine_count,
            actions_per_node,
            reached_ending: self.is_ended(),
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

    pub(crate) fn current_scene_npcs(&self) -> Vec<String> {
        self.bundle
            .narrative
            .get(self.director.current_node_id())
            .and_then(|node| self.bundle.scenes.get(&node.content_ref))
            .map(|scene| scene.npcs.clone())
            .unwrap_or_default()
    }

    fn sorted_knowledge(&self) -> Vec<String> {
        let mut v: Vec<String> = self.state.knowledge.iter().cloned().collect();
        v.sort();
        v
    }

    fn sorted_flags(&self) -> Vec<String> {
        let mut v: Vec<String> = self.state.flags.iter().cloned().collect();
        v.sort();
        v
    }

    fn narration_hints(&self) -> Vec<String> {
        self.bundle
            .abilities
            .values()
            .filter_map(|a| a.narration_hint.clone())
            .collect()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
