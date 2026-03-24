// SPDX-License-Identifier: AGPL-3.0-or-later
//! State effects — typed mutations applied to the world state.
//!
//! Every game action that modifies state goes through a `StateEffect`.
//! This is what keeps the DAG consistent and the game bounded — there are
//! no freeform mutations, only typed operations.

use serde::{Deserialize, Serialize};

/// A typed mutation to the game state.
///
/// Applied when entering narrative nodes, using abilities, or as
/// consequences of NPC interactions and skill checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum StateEffect {
    /// Add a knowledge fragment to the player's knowledge store.
    AddKnowledge(String),
    /// Remove a knowledge fragment.
    RemoveKnowledge(String),
    /// Modify trust with an NPC by a delta.
    ModifyTrust(String, i32),
    /// Set trust to an exact value.
    SetTrust(String, i32),
    /// Add an item to inventory.
    AddItem(String),
    /// Remove an item from inventory.
    RemoveItem(String),
    /// Apply a condition with optional duration (turns, 0 = permanent).
    ApplyCondition(String, u32),
    /// Remove a condition.
    RemoveCondition(String),
    /// Advance an NPC's arc to a new phase.
    AdvanceArc(String, String),
    /// Transition to a different plane.
    TransitionPlane(String),
    /// Set a flag.
    SetFlag(String),
    /// Clear a flag.
    ClearFlag(String),
    /// Composite: apply multiple effects in sequence.
    Batch(Vec<Self>),
}

impl StateEffect {
    /// Human-readable description of this effect.
    pub fn describe(&self) -> String {
        match self {
            Self::AddKnowledge(k) => format!("learn '{k}'"),
            Self::RemoveKnowledge(k) => format!("forget '{k}'"),
            Self::ModifyTrust(npc, delta) => format!("trust with '{npc}' {delta:+}"),
            Self::SetTrust(npc, val) => format!("trust with '{npc}' = {val}"),
            Self::AddItem(i) => format!("gain '{i}'"),
            Self::RemoveItem(i) => format!("lose '{i}'"),
            Self::ApplyCondition(c, dur) => {
                if *dur == 0 {
                    format!("apply '{c}' (permanent)")
                } else {
                    format!("apply '{c}' for {dur} turns")
                }
            }
            Self::RemoveCondition(c) => format!("remove '{c}'"),
            Self::AdvanceArc(npc, phase) => format!("'{npc}' arc -> '{phase}'"),
            Self::TransitionPlane(p) => format!("transition to '{p}'"),
            Self::SetFlag(f) => format!("set flag '{f}'"),
            Self::ClearFlag(f) => format!("clear flag '{f}'"),
            Self::Batch(effects) => {
                let descs: Vec<String> = effects.iter().map(Self::describe).collect();
                format!("[{}]", descs.join("; "))
            }
        }
    }

    /// Count the number of atomic effects (expanding batches).
    pub fn count(&self) -> usize {
        match self {
            Self::Batch(effects) => effects.iter().map(Self::count).sum(),
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_serialization_roundtrip() {
        let effect = StateEffect::ModifyTrust("maren".to_owned(), 2);
        let yaml = serde_yaml::to_string(&effect).unwrap_or_default();
        let parsed: StateEffect =
            serde_yaml::from_str(&yaml).unwrap_or(StateEffect::SetFlag(String::new()));
        assert_eq!(effect, parsed);
    }

    #[test]
    fn batch_effect_count() {
        let batch = StateEffect::Batch(vec![
            StateEffect::AddKnowledge("a".to_owned()),
            StateEffect::ModifyTrust("b".to_owned(), 1),
            StateEffect::Batch(vec![
                StateEffect::AddItem("c".to_owned()),
                StateEffect::SetFlag("d".to_owned()),
            ]),
        ]);
        assert_eq!(batch.count(), 4);
    }

    #[test]
    fn describe_all_variants() {
        let variants = vec![
            StateEffect::AddKnowledge("k".to_owned()),
            StateEffect::RemoveKnowledge("k".to_owned()),
            StateEffect::ModifyTrust("n".to_owned(), -1),
            StateEffect::SetTrust("n".to_owned(), 5),
            StateEffect::AddItem("i".to_owned()),
            StateEffect::RemoveItem("i".to_owned()),
            StateEffect::ApplyCondition("c".to_owned(), 3),
            StateEffect::ApplyCondition("c".to_owned(), 0),
            StateEffect::RemoveCondition("c".to_owned()),
            StateEffect::AdvanceArc("n".to_owned(), "p".to_owned()),
            StateEffect::TransitionPlane("tactical".to_owned()),
            StateEffect::SetFlag("f".to_owned()),
            StateEffect::ClearFlag("f".to_owned()),
            StateEffect::Batch(vec![]),
        ];
        for v in &variants {
            assert!(!v.describe().is_empty());
        }
    }
}
