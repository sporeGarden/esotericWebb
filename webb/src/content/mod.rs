// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content loading, validation, and scaffolding.
//!
//! Creative teams author YAML files. This module loads them into a
//! [`ContentBundle`] and validates cross-references, reachability,
//! and predicate consistency.
//!
//! Type definitions live in `types`, keeping this module focused on
//! load/validate/scaffold orchestration.

mod types;

pub use types::{AbilityDef, LieInfo, NpcDef, NpcTrustReward, SceneContent, WorldMeta};

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::narrative::predicate::StatePredicate;
use crate::narrative::{self, NarrativeGraph};

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
#[path = "tests.rs"]
mod tests;
