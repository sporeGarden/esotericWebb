// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content data model — YAML-authored types for worlds, scenes, NPCs, and abilities.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::narrative::effect::StateEffect;
use crate::narrative::predicate::StatePredicate;

/// World metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldMeta {
    /// World name.
    pub name: String,
    /// Author or team name.
    pub author: String,
    /// Content version.
    pub version: String,
    /// Short description.
    pub description: String,
}

/// A scene's content (loaded from YAML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContent {
    /// Scene identifier (matches narrative node `content_ref`).
    pub id: String,
    /// Scene description text.
    pub description: String,
    /// NPCs present in this scene (by ID).
    #[serde(default)]
    pub npcs: Vec<String>,
    /// Items available in this scene (by ID).
    #[serde(default)]
    pub items: Vec<String>,
}

/// An ability/spell definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDef {
    /// Ability identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Preconditions to use this ability.
    #[serde(default)]
    pub preconditions: Vec<StatePredicate>,
    /// Effects when the ability is used.
    #[serde(default)]
    pub effects: Vec<StateEffect>,
    /// Hint for AI narration.
    #[serde(default)]
    pub narration_hint: Option<String>,
}

/// An NPC personality certificate — trust thresholds, knowledge, and dialogue hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDef {
    /// NPC identifier (short form, e.g. "maren").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Narrative role.
    #[serde(default)]
    pub role: String,
    /// Knowledge this NPC possesses (for reference / AI prompting).
    #[serde(default)]
    pub knows: Vec<String>,
    /// Starting trust value.
    #[serde(default)]
    pub trust_initial: i32,
    /// Trust threshold rewards — keyed by threshold level.
    #[serde(default)]
    pub trust_rewards: std::collections::BTreeMap<i32, NpcTrustReward>,
    /// Topics this NPC lies about.
    #[serde(default)]
    pub lies_about: HashMap<String, LieInfo>,
    /// Arc description (for AI narration context).
    #[serde(default)]
    pub arc: String,
}

/// What an NPC reveals at a given trust threshold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpcTrustReward {
    /// Human-readable description of what happens.
    #[serde(default)]
    pub description: String,
    /// Knowledge granted to the player.
    #[serde(default)]
    pub grants_knowledge: Vec<String>,
    /// Items given to the player.
    #[serde(default)]
    pub grants_items: Vec<String>,
    /// Flags set on the game state.
    #[serde(default)]
    pub sets_flags: Vec<String>,
}

/// Information about a topic an NPC lies about.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LieInfo {
    /// Difficulty class to detect the lie.
    #[serde(default)]
    pub detection_dc: i32,
}
