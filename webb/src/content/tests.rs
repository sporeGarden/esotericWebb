// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(clippy::unwrap_used, reason = "test code")]

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
        lies_about: HashMap::from([("ward_locations".to_owned(), LieInfo { detection_dc: 14 })]),
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

#[test]
fn validate_ruleset_missing_plane_field() {
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "combat".to_owned(),
        serde_json::json!({
            "rules": [{"id": "r1", "effect": "damage"}]
        }),
    );
    let bundle = ContentBundle {
        meta: WorldMeta::default(),
        narrative: NarrativeGraph::default(),
        worlds: HashMap::new(),
        npcs: HashMap::new(),
        abilities: HashMap::new(),
        scenes: HashMap::new(),
        rulesets,
        load_warnings: Vec::new(),
    };
    let issues = bundle.validate();
    assert!(
        issues
            .iter()
            .any(|i| i.contains("missing required 'plane'"))
    );
}

#[test]
fn validate_ruleset_missing_rules_array() {
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "combat".to_owned(),
        serde_json::json!({
            "plane": "combat"
        }),
    );
    let bundle = ContentBundle {
        meta: WorldMeta::default(),
        narrative: NarrativeGraph::default(),
        worlds: HashMap::new(),
        npcs: HashMap::new(),
        abilities: HashMap::new(),
        scenes: HashMap::new(),
        rulesets,
        load_warnings: Vec::new(),
    };
    let issues = bundle.validate();
    assert!(
        issues
            .iter()
            .any(|i| i.contains("missing required 'rules'"))
    );
}

#[test]
fn validate_ruleset_rule_missing_id() {
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "combat".to_owned(),
        serde_json::json!({
            "plane": "combat",
            "rules": [{"effect": "damage"}]
        }),
    );
    let bundle = ContentBundle {
        meta: WorldMeta::default(),
        narrative: NarrativeGraph::default(),
        worlds: HashMap::new(),
        npcs: HashMap::new(),
        abilities: HashMap::new(),
        scenes: HashMap::new(),
        rulesets,
        load_warnings: Vec::new(),
    };
    let issues = bundle.validate();
    assert!(issues.iter().any(|i| i.contains("rule[0] missing 'id'")));
}

#[test]
fn validate_valid_ruleset_no_issues() {
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "combat".to_owned(),
        serde_json::json!({
            "plane": "combat",
            "version": "1.0.0",
            "description": "Combat rules for shadow plane",
            "rules": [
                {"id": "damage_calc", "type": "formula", "formula": "str * weapon_mod"},
                {"id": "initiative", "type": "check", "stat": "dexterity"}
            ]
        }),
    );
    let bundle = ContentBundle {
        meta: WorldMeta::default(),
        narrative: minimal_narrative(),
        worlds: HashMap::new(),
        npcs: HashMap::new(),
        abilities: HashMap::new(),
        scenes: HashMap::new(),
        rulesets,
        load_warnings: Vec::new(),
    };
    let issues = bundle.validate_rulesets();
    assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
}
