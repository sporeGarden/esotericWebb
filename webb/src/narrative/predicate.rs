// SPDX-License-Identifier: AGPL-3.0-or-later
//! State predicates — composable queries against the world state.
//!
//! Predicates are the mechanism by which authored content gates traversal.
//! They are serialized in YAML content files and evaluated at runtime
//! against the current game state.

use serde::{Deserialize, Serialize};

/// A composable predicate that queries the game state.
///
/// Used in narrative edge conditions, node preconditions, and
/// ability preconditions to determine what is currently possible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum StatePredicate {
    /// Player has acquired a specific knowledge fragment.
    HasKnowledge(String),
    /// Player does NOT have a specific knowledge fragment.
    LacksKnowledge(String),
    /// Trust with an NPC is at or above a threshold.
    TrustAbove(String, i32),
    /// Trust with an NPC is below a threshold.
    TrustBelow(String, i32),
    /// Player is currently in a specific plane.
    InPlane(String),
    /// Player has a specific item in inventory.
    HasItem(String),
    /// Player does not have a specific item.
    LacksItem(String),
    /// A specific condition is currently active.
    ConditionActive(String),
    /// A specific condition is NOT active.
    ConditionInactive(String),
    /// An NPC's arc is at a specific phase.
    ArcPhaseIs(String, String),
    /// A flag has been set.
    FlagSet(String),
    /// A flag has NOT been set.
    FlagUnset(String),
    /// All sub-predicates must be true.
    All(Vec<Self>),
    /// At least one sub-predicate must be true.
    Any(Vec<Self>),
    /// The sub-predicate must be false.
    Not(Box<Self>),
}

impl StatePredicate {
    /// Human-readable description of this predicate.
    pub fn describe(&self) -> String {
        match self {
            Self::HasKnowledge(k) => format!("knows '{k}'"),
            Self::LacksKnowledge(k) => format!("does not know '{k}'"),
            Self::TrustAbove(npc, lvl) => format!("trust with '{npc}' >= {lvl}"),
            Self::TrustBelow(npc, lvl) => format!("trust with '{npc}' < {lvl}"),
            Self::InPlane(p) => format!("in plane '{p}'"),
            Self::HasItem(i) => format!("has item '{i}'"),
            Self::LacksItem(i) => format!("lacks item '{i}'"),
            Self::ConditionActive(c) => format!("condition '{c}' active"),
            Self::ConditionInactive(c) => format!("condition '{c}' inactive"),
            Self::ArcPhaseIs(npc, phase) => format!("'{npc}' arc at '{phase}'"),
            Self::FlagSet(f) => format!("flag '{f}' set"),
            Self::FlagUnset(f) => format!("flag '{f}' unset"),
            Self::All(preds) => {
                let descs: Vec<String> = preds.iter().map(Self::describe).collect();
                format!("all of [{}]", descs.join(", "))
            }
            Self::Any(preds) => {
                let descs: Vec<String> = preds.iter().map(Self::describe).collect();
                format!("any of [{}]", descs.join(", "))
            }
            Self::Not(pred) => format!("not ({})", pred.describe()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_serialization_roundtrip() {
        let pred = StatePredicate::HasKnowledge("elder_sign".to_owned());
        let yaml = serde_yaml::to_string(&pred).unwrap_or_default();
        let parsed: StatePredicate =
            serde_yaml::from_str(&yaml).unwrap_or(StatePredicate::FlagSet(String::new()));
        assert_eq!(pred, parsed);
    }

    #[test]
    fn compound_predicate_all() {
        let pred = StatePredicate::All(vec![
            StatePredicate::HasKnowledge("clue_a".to_owned()),
            StatePredicate::TrustAbove("maren".to_owned(), 3),
        ]);
        let desc = pred.describe();
        assert!(desc.contains("all of"));
        assert!(desc.contains("clue_a"));
    }

    #[test]
    fn compound_predicate_not() {
        let pred = StatePredicate::Not(Box::new(StatePredicate::HasItem("cursed_ring".to_owned())));
        let desc = pred.describe();
        assert!(desc.contains("not"));
        assert!(desc.contains("cursed_ring"));
    }

    #[test]
    fn describe_all_variants() {
        let variants = vec![
            StatePredicate::HasKnowledge("k".to_owned()),
            StatePredicate::LacksKnowledge("k".to_owned()),
            StatePredicate::TrustAbove("n".to_owned(), 1),
            StatePredicate::TrustBelow("n".to_owned(), 1),
            StatePredicate::InPlane("dialogue".to_owned()),
            StatePredicate::HasItem("i".to_owned()),
            StatePredicate::LacksItem("i".to_owned()),
            StatePredicate::ConditionActive("c".to_owned()),
            StatePredicate::ConditionInactive("c".to_owned()),
            StatePredicate::ArcPhaseIs("n".to_owned(), "p".to_owned()),
            StatePredicate::FlagSet("f".to_owned()),
            StatePredicate::FlagUnset("f".to_owned()),
            StatePredicate::Any(vec![]),
        ];
        for v in &variants {
            assert!(!v.describe().is_empty());
        }
    }
}
