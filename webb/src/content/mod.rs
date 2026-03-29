// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content loading, validation, and scaffolding.
//!
//! Creative teams author YAML files. This module loads them into a
//! [`ContentBundle`] and validates cross-references, reachability,
//! and predicate consistency.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::narrative::effect::StateEffect;
use crate::narrative::predicate::StatePredicate;
use crate::narrative::{self, NarrativeGraph};

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

/// The fully loaded and validated content bundle.
#[derive(Debug, Clone)]
pub struct ContentBundle {
    /// World metadata.
    pub meta: WorldMeta,
    /// The narrative graph.
    pub narrative: NarrativeGraph,
    /// World definitions (keyed by world ID).
    pub worlds: HashMap<String, serde_json::Value>,
    /// NPC definitions (keyed by NPC ID).
    pub npcs: HashMap<String, NpcDef>,
    /// Ability definitions (keyed by ability ID).
    pub abilities: HashMap<String, AbilityDef>,
    /// Scene content (keyed by `content_ref`).
    pub scenes: HashMap<String, SceneContent>,
    /// Ruleset definitions (keyed by plane name).
    pub rulesets: HashMap<String, serde_json::Value>,
    /// Diagnostics from content loading (non-fatal issues).
    pub load_warnings: Vec<String>,
}

impl ContentBundle {
    /// Load a content bundle from a directory path.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory doesn't exist or required files
    /// are missing/malformed.
    pub fn load(path: &str) -> Result<Self, String> {
        let base = Path::new(path);
        if !base.exists() {
            return Err(format!("content directory not found: {path}"));
        }

        let mut warnings = Vec::new();

        let meta = load_yaml_with_diag::<WorldMeta>(base, "meta.yaml", &mut warnings);
        let narrative =
            load_yaml_with_diag::<NarrativeGraph>(base, "narrative.yaml", &mut warnings);
        let scenes = load_yaml_dir_with_diag::<SceneContent>(base, "scenes", &mut warnings);
        let abilities = load_yaml_dir_with_diag::<AbilityDef>(base, "abilities", &mut warnings);

        let worlds = load_raw_yaml_dir(base, "worlds");
        let npcs = load_yaml_dir_with_diag::<NpcDef>(base, "npcs", &mut warnings);
        let rulesets = load_raw_yaml_dir(base, "rulesets");

        Ok(Self {
            meta,
            narrative,
            worlds,
            npcs,
            abilities,
            scenes,
            rulesets,
            load_warnings: warnings,
        })
    }

    /// Validate the loaded content and return a list of issues.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = self.load_warnings.clone();
        issues.extend(narrative::validator::validate(&self.narrative));

        for node in self.narrative.nodes.values() {
            if !node.content_ref.is_empty() && !self.scenes.contains_key(&node.content_ref) {
                issues.push(format!(
                    "node '{}': content_ref '{}' not found in scenes",
                    node.id, node.content_ref
                ));
            }
        }

        for scene in self.scenes.values() {
            for npc_id in &scene.npcs {
                if !self.npcs.contains_key(npc_id) {
                    issues.push(format!(
                        "scene '{}': NPC '{}' not found in npcs/",
                        scene.id, npc_id
                    ));
                }
            }
        }

        for npc in self.npcs.values() {
            if npc.trust_rewards.is_empty() && npc.role != "spirit" {
                issues.push(format!(
                    "npc '{}': no trust rewards defined — talk will be inert",
                    npc.id
                ));
            }
        }

        for ability in self.abilities.values() {
            for (i, pred) in ability.preconditions.iter().enumerate() {
                let _ = pred.describe();
                if matches!(pred, StatePredicate::All(v) | StatePredicate::Any(v) if v.is_empty()) {
                    issues.push(format!(
                        "ability '{}': precondition {i} is empty compound",
                        ability.id
                    ));
                }
            }
            if ability.effects.is_empty() {
                issues.push(format!("ability '{}': has no effects", ability.id));
            }
        }

        issues
    }
}

/// Scaffold a new content directory with template YAML files.
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn scaffold(output_path: &str) -> Result<(), String> {
    let base = Path::new(output_path);

    create_dir(base)?;
    for subdir in &["worlds", "npcs", "abilities", "scenes", "rulesets"] {
        create_dir(&base.join(subdir))?;
    }

    write_yaml(
        base,
        "meta.yaml",
        &WorldMeta {
            name: "My World".to_owned(),
            author: "Your Name".to_owned(),
            version: "0.1.0".to_owned(),
            description: "A new Esoteric Webb world.".to_owned(),
        },
    )?;

    write_yaml(base, "narrative.yaml", &NarrativeGraph::default())?;

    println!("Scaffolded new world at {output_path}/");
    println!("  meta.yaml         — world metadata");
    println!("  narrative.yaml    — narrative graph (add nodes and edges)");
    println!("  worlds/           — location definitions");
    println!("  npcs/             — NPC personality certs");
    println!("  abilities/        — spell/ability definitions");
    println!("  scenes/           — scene content");
    println!("  rulesets/         — per-plane ruleset certs");

    Ok(())
}

fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("create dir {}: {e}", path.display()))
}

fn write_yaml<T: Serialize>(base: &Path, name: &str, value: &T) -> Result<(), String> {
    let content = serde_yaml::to_string(value).map_err(|e| format!("serialize {name}: {e}"))?;
    std::fs::write(base.join(name), content).map_err(|e| format!("write {name}: {e}"))
}

fn load_yaml_with_diag<T: for<'de> Deserialize<'de> + Default>(
    base: &Path,
    name: &str,
    warnings: &mut Vec<String>,
) -> T {
    let path = base.join(name);
    if !path.exists() {
        warnings.push(format!("{name}: file not found, using defaults"));
        return T::default();
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        warnings.push(format!("{name}: could not read file"));
        return T::default();
    };
    match serde_yaml::from_str(&content) {
        Ok(val) => val,
        Err(e) => {
            warnings.push(format!("{name}: parse error: {e}"));
            T::default()
        }
    }
}

fn load_yaml_dir_with_diag<T: for<'de> Deserialize<'de> + HasId>(
    base: &Path,
    subdir: &str,
    warnings: &mut Vec<String>,
) -> HashMap<String, T> {
    let mut map = HashMap::new();
    let dir = base.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_yaml::from_str::<T>(&content) {
                    Ok(item) => {
                        map.insert(item.get_id().to_owned(), item);
                    }
                    Err(e) => {
                        warnings.push(format!("{subdir}/{fname}: parse error: {e}"));
                    }
                },
                Err(e) => {
                    warnings.push(format!("{subdir}/{fname}: read error: {e}"));
                }
            }
        }
    }
    map
}

fn load_raw_yaml_dir(base: &Path, subdir: &str) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    let dir = base.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(value) = serde_yaml::from_str::<serde_json::Value>(&content) {
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_owned();
                    map.insert(id, value);
                }
            }
        }
    }
    map
}

/// Trait for content items that have an ID field.
trait HasId {
    /// Get the item's identifier.
    fn get_id(&self) -> &str;
}

impl HasId for SceneContent {
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl HasId for AbilityDef {
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl HasId for NpcDef {
    fn get_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::narrative::effect::StateEffect;
    use crate::narrative::predicate::StatePredicate;

    #[test]
    fn scaffold_creates_structure() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_scaffold");
        let _ = std::fs::remove_dir_all(&dir);
        let result = scaffold(dir.to_str().unwrap_or("/tmp/test"));
        assert!(result.is_ok());
        assert!(dir.join("meta.yaml").exists());
        assert!(dir.join("narrative.yaml").exists());
        assert!(dir.join("npcs").is_dir());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_dir_errors() {
        let result = ContentBundle::load("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn empty_content_validates_with_issues() {
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: NarrativeGraph::default(),
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities: HashMap::new(),
            scenes: HashMap::new(),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(issues.iter().any(|i| i.contains("no start node")));
    }

    // ── YAML roundtrip tests (GAP-005) ─────────────────────────────

    #[test]
    fn world_meta_yaml_roundtrip() {
        let meta = WorldMeta {
            name: "Roundtrip Test".to_owned(),
            author: "Test Author".to_owned(),
            version: "0.1.0".to_owned(),
            description: "A test world.".to_owned(),
        };
        let yaml = serde_yaml::to_string(&meta).unwrap();
        let parsed: WorldMeta = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, meta.name);
        assert_eq!(parsed.author, meta.author);
        assert_eq!(parsed.version, meta.version);
    }

    #[test]
    fn scene_content_yaml_roundtrip() {
        let scene = SceneContent {
            id: "parlor".to_owned(),
            description: "A dimly lit parlor.".to_owned(),
            npcs: vec!["maren".to_owned(), "tobias".to_owned()],
            items: vec!["silver_key".to_owned()],
        };
        let yaml = serde_yaml::to_string(&scene).unwrap();
        let parsed: SceneContent = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.id, scene.id);
        assert_eq!(parsed.npcs, scene.npcs);
        assert_eq!(parsed.items, scene.items);
    }

    #[test]
    fn ability_def_yaml_roundtrip() {
        let ability = AbilityDef {
            id: "read_aura".to_owned(),
            name: "Read Aura".to_owned(),
            description: "Sense emotional state.".to_owned(),
            preconditions: vec![StatePredicate::HasKnowledge("psychic_training".to_owned())],
            effects: vec![StateEffect::SetFlag("aura_active".to_owned())],
            narration_hint: Some("Colors bloom.".to_owned()),
        };
        let yaml = serde_yaml::to_string(&ability).unwrap();
        let parsed: AbilityDef = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.id, ability.id);
        assert_eq!(parsed.preconditions.len(), 1);
        assert_eq!(parsed.effects.len(), 1);
        assert_eq!(parsed.narration_hint, ability.narration_hint);
    }

    #[test]
    fn npc_def_yaml_roundtrip() {
        let npc = NpcDef {
            id: "maren".to_owned(),
            name: "Maren".to_owned(),
            role: "proprietor".to_owned(),
            knows: vec!["elder_sign".to_owned(), "ward_locations".to_owned()],
            trust_initial: -1,
            trust_rewards: std::collections::BTreeMap::from([(
                3,
                NpcTrustReward {
                    description: "Maren softens.".to_owned(),
                    grants_knowledge: vec!["maren_secret".to_owned()],
                    grants_items: vec![],
                    sets_flags: vec!["maren_trusts".to_owned()],
                },
            )]),
            lies_about: HashMap::from([(
                "ward_locations".to_owned(),
                LieInfo { detection_dc: 14 },
            )]),
            arc: "suspicious → cautious → open".to_owned(),
        };
        let yaml = serde_yaml::to_string(&npc).unwrap();
        let parsed: NpcDef = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.id, npc.id);
        assert_eq!(parsed.trust_initial, -1);
        assert_eq!(parsed.trust_rewards.len(), 1);
        assert_eq!(parsed.lies_about.len(), 1);
        assert_eq!(parsed.knows.len(), 2);
    }

    #[test]
    fn scaffold_then_load_roundtrip() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_roundtrip_load");
        let _ = std::fs::remove_dir_all(&dir);
        let path_str = dir.to_str().unwrap();
        scaffold(path_str).unwrap();

        let bundle = ContentBundle::load(path_str);
        assert!(bundle.is_ok(), "scaffold output should load cleanly");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Validation edge-case coverage ───────────────────────────────

    fn minimal_narrative() -> crate::narrative::NarrativeGraph {
        use crate::narrative::{NarrativeGraph, NarrativeNode, SceneType};
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_owned(),
            NarrativeNode {
                id: "start".to_owned(),
                scene_type: SceneType::Exploration,
                content_ref: "start".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![],
                is_start: true,
                is_ending: true,
                label: None,
            },
        );
        NarrativeGraph { nodes }
    }

    #[test]
    fn validate_missing_content_ref() {
        let mut narrative = minimal_narrative();
        narrative.nodes.get_mut("start").unwrap().content_ref = "missing_scene".to_owned();
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative,
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities: HashMap::new(),
            scenes: HashMap::new(),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(
            issues
                .iter()
                .any(|i| i.contains("content_ref") && i.contains("missing_scene"))
        );
    }

    #[test]
    fn validate_missing_npc_in_scene() {
        let mut scenes = HashMap::new();
        scenes.insert(
            "start".to_owned(),
            SceneContent {
                id: "start".to_owned(),
                description: "A room.".to_owned(),
                npcs: vec!["ghost".to_owned()],
                items: vec![],
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities: HashMap::new(),
            scenes,
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(issues.iter().any(|i| i.contains("NPC 'ghost' not found")));
    }

    #[test]
    fn validate_npc_without_trust_rewards() {
        let mut npcs = HashMap::new();
        npcs.insert(
            "bob".to_owned(),
            NpcDef {
                id: "bob".to_owned(),
                name: "Bob".to_owned(),
                role: "merchant".to_owned(),
                knows: vec![],
                trust_initial: 0,
                trust_rewards: std::collections::BTreeMap::new(),
                lies_about: HashMap::new(),
                arc: String::new(),
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs,
            abilities: HashMap::new(),
            scenes: HashMap::from([(
                "start".to_owned(),
                SceneContent {
                    id: "start".to_owned(),
                    description: "Start.".to_owned(),
                    npcs: vec![],
                    items: vec![],
                },
            )]),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(
            issues
                .iter()
                .any(|i| i.contains("bob") && i.contains("no trust rewards"))
        );
    }

    #[test]
    fn validate_spirit_npc_skips_trust_warning() {
        let mut npcs = HashMap::new();
        npcs.insert(
            "wisp".to_owned(),
            NpcDef {
                id: "wisp".to_owned(),
                name: "Wisp".to_owned(),
                role: "spirit".to_owned(),
                knows: vec![],
                trust_initial: 0,
                trust_rewards: std::collections::BTreeMap::new(),
                lies_about: HashMap::new(),
                arc: String::new(),
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs,
            abilities: HashMap::new(),
            scenes: HashMap::from([(
                "start".to_owned(),
                SceneContent {
                    id: "start".to_owned(),
                    description: "Start.".to_owned(),
                    npcs: vec![],
                    items: vec![],
                },
            )]),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(
            !issues
                .iter()
                .any(|i| i.contains("wisp") && i.contains("no trust rewards"))
        );
    }

    #[test]
    fn validate_ability_no_effects() {
        let mut abilities = HashMap::new();
        abilities.insert(
            "empty".to_owned(),
            AbilityDef {
                id: "empty".to_owned(),
                name: "Empty".to_owned(),
                description: "No effects.".to_owned(),
                preconditions: vec![],
                effects: vec![],
                narration_hint: None,
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities,
            scenes: HashMap::from([(
                "start".to_owned(),
                SceneContent {
                    id: "start".to_owned(),
                    description: "Start.".to_owned(),
                    npcs: vec![],
                    items: vec![],
                },
            )]),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(
            issues
                .iter()
                .any(|i| i.contains("empty") && i.contains("has no effects"))
        );
    }

    #[test]
    fn validate_ability_empty_compound_precondition() {
        let mut abilities = HashMap::new();
        abilities.insert(
            "compound".to_owned(),
            AbilityDef {
                id: "compound".to_owned(),
                name: "Compound".to_owned(),
                description: "Empty compound.".to_owned(),
                preconditions: vec![StatePredicate::All(vec![])],
                effects: vec![StateEffect::SetFlag("x".to_owned())],
                narration_hint: None,
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities,
            scenes: HashMap::from([(
                "start".to_owned(),
                SceneContent {
                    id: "start".to_owned(),
                    description: "Start.".to_owned(),
                    npcs: vec![],
                    items: vec![],
                },
            )]),
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(
            issues
                .iter()
                .any(|i| i.contains("compound") && i.contains("empty compound"))
        );
    }

    #[test]
    fn validate_propagates_load_warnings() {
        let bundle = ContentBundle {
            meta: WorldMeta::default(),
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities: HashMap::new(),
            scenes: HashMap::from([(
                "start".to_owned(),
                SceneContent {
                    id: "start".to_owned(),
                    description: "Start.".to_owned(),
                    npcs: vec![],
                    items: vec![],
                },
            )]),
            rulesets: HashMap::new(),
            load_warnings: vec!["some warning".to_owned()],
        };
        let issues = bundle.validate();
        assert!(issues.iter().any(|i| i.contains("some warning")));
    }

    #[test]
    fn load_empty_dir_returns_bundle_with_warnings() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_empty_load");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(!bundle.load_warnings.is_empty());
        assert!(bundle.load_warnings.iter().any(|w| w.contains("meta.yaml")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_with_bad_yaml_collects_warning() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_bad_yaml");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("meta.yaml"), "not: [valid: yaml: {{{").unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(
            bundle
                .load_warnings
                .iter()
                .any(|w| w.contains("meta.yaml") && w.contains("parse error"))
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_with_bad_scene_yaml() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_bad_scene");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("scenes")).unwrap();
        std::fs::write(dir.join("scenes/bad.yaml"), "{{{invalid").unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(
            bundle
                .load_warnings
                .iter()
                .any(|w| w.contains("scenes") && w.contains("parse error"))
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_loads_valid_scenes() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_valid_scene");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("scenes")).unwrap();
        let scene = SceneContent {
            id: "tavern".to_owned(),
            description: "A tavern.".to_owned(),
            npcs: vec![],
            items: vec![],
        };
        let yaml = serde_yaml::to_string(&scene).unwrap();
        std::fs::write(dir.join("scenes/tavern.yaml"), yaml).unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(bundle.scenes.contains_key("tavern"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_loads_valid_npcs() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_valid_npc");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs")).unwrap();
        let npc = NpcDef {
            id: "merchant".to_owned(),
            name: "Merchant".to_owned(),
            role: "vendor".to_owned(),
            knows: vec![],
            trust_initial: 0,
            trust_rewards: std::collections::BTreeMap::new(),
            lies_about: HashMap::new(),
            arc: String::new(),
        };
        let yaml = serde_yaml::to_string(&npc).unwrap();
        std::fs::write(dir.join("npcs/merchant.yaml"), yaml).unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(bundle.npcs.contains_key("merchant"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_loads_valid_abilities() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_valid_ability");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("abilities")).unwrap();
        let ability = AbilityDef {
            id: "fireball".to_owned(),
            name: "Fireball".to_owned(),
            description: "Cast fire.".to_owned(),
            preconditions: vec![],
            effects: vec![StateEffect::SetFlag("burned".to_owned())],
            narration_hint: None,
        };
        let yaml = serde_yaml::to_string(&ability).unwrap();
        std::fs::write(dir.join("abilities/fireball.yaml"), yaml).unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(bundle.abilities.contains_key("fireball"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_loads_worlds_and_rulesets() {
        let dir = std::env::temp_dir().join("esoteric_webb_test_worlds_rulesets");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("worlds")).unwrap();
        std::fs::create_dir_all(dir.join("rulesets")).unwrap();
        std::fs::write(dir.join("worlds/prime.yaml"), "name: Prime\ntype: physical").unwrap();
        std::fs::write(
            dir.join("rulesets/combat.yaml"),
            "style: turn_based\ndifficulty: normal",
        )
        .unwrap();

        let bundle = ContentBundle::load(dir.to_str().unwrap()).unwrap();
        assert!(bundle.worlds.contains_key("prime"));
        assert!(bundle.rulesets.contains_key("combat"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_valid_bundle_no_extra_issues() {
        let mut scenes = HashMap::new();
        scenes.insert(
            "start".to_owned(),
            SceneContent {
                id: "start".to_owned(),
                description: "Start.".to_owned(),
                npcs: vec!["wisp".to_owned()],
                items: vec![],
            },
        );
        let mut npcs = HashMap::new();
        npcs.insert(
            "wisp".to_owned(),
            NpcDef {
                id: "wisp".to_owned(),
                name: "Wisp".to_owned(),
                role: "spirit".to_owned(),
                knows: vec![],
                trust_initial: 0,
                trust_rewards: std::collections::BTreeMap::new(),
                lies_about: HashMap::new(),
                arc: String::new(),
            },
        );
        let mut abilities = HashMap::new();
        abilities.insert(
            "glow".to_owned(),
            AbilityDef {
                id: "glow".to_owned(),
                name: "Glow".to_owned(),
                description: "Emit light.".to_owned(),
                preconditions: vec![],
                effects: vec![StateEffect::SetFlag("glowing".to_owned())],
                narration_hint: None,
            },
        );
        let bundle = ContentBundle {
            meta: WorldMeta {
                name: "Test".to_owned(),
                author: "test".to_owned(),
                version: "0.1.0".to_owned(),
                description: "Test.".to_owned(),
            },
            narrative: minimal_narrative(),
            worlds: HashMap::new(),
            npcs,
            abilities,
            scenes,
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        };
        let issues = bundle.validate();
        assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
    }
}
