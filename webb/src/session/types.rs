// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public types for game session state, actions, and narration context.
//!
//! Extracted from `session/mod.rs` for ergonomic imports and to keep
//! individual modules under the 1000 line ceiling.

use serde::{Deserialize, Serialize};

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
    pub fn parse(s: &str) -> crate::error::Result<Self> {
        match s {
            "exit" => Ok(Self::Exit),
            "talk" => Ok(Self::Talk),
            "ability" => Ok(Self::Ability),
            "examine" => Ok(Self::Examine),
            _ => Err(crate::error::WebbError::Other(format!(
                "unknown action kind: {s}"
            ))),
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
    /// Whether the session is still active (always true from [`super::GameSession::snapshot`]).
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

/// Enrichments from primal composition applied during [`super::GameSession::act`].
///
/// All fields are best-effort — absent primals result in `None` / empty
/// values. Gameplay is never blocked by missing primals.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PrimalEnrichment {
    /// AI-generated narration text (via Squirrel `ai.query`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_narration: Option<String>,
    /// NPC dialogue response (for talk actions, via Squirrel `ai.query`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npc_dialogue: Option<String>,
    /// Internal voice interjections (via local `science/` module).
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

/// A voice interjection from the local game science module.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceEnrichment {
    /// Voice identifier (e.g. "logic", "empathy").
    pub voice_id: String,
    /// The voice's interjection text.
    pub text: String,
}

/// Session-level engagement metrics for game science and DDA (V13).
///
/// Computed from session history — lightweight, no persistent state needed.
/// Useful for AI narration pacing awareness and flow evaluation calibration.
#[derive(Debug, Clone, Serialize)]
pub struct SessionMetrics {
    /// Total turns played in this session.
    pub turns_played: u32,
    /// Unique narrative nodes visited.
    pub nodes_visited: u32,
    /// Total nodes in the narrative graph.
    pub nodes_total: u32,
    /// Exploration ratio (visited / total), clamped 0.0–1.0.
    pub exploration_ratio: f64,
    /// Times the player returned to a previously-visited node.
    pub backtrack_count: u32,
    /// Total NPC interactions (talk actions).
    pub npc_interactions: u32,
    /// Total ability uses.
    pub ability_uses: u32,
    /// Total examine actions.
    pub examine_count: u32,
    /// Average actions per unique node (pacing indicator).
    pub actions_per_node: f64,
    /// Whether the session reached an ending.
    pub reached_ending: bool,
}
