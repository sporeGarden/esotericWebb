// SPDX-License-Identifier: AGPL-3.0-or-later
//! `WorldState` — composite game state that predicates query and effects mutate.
//!
//! The combinatorial richness of this state (knowledge x trust x inventory x
//! conditions x arcs x plane x flags) is what creates the "near-infinite
//! exploration" within a bounded narrative topology.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::narrative::effect::StateEffect;
use crate::narrative::predicate::StatePredicate;

/// The composite game state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldState {
    /// Knowledge fragments the player has accumulated.
    pub knowledge: HashSet<String>,
    /// Per-NPC trust levels.
    pub trust: HashMap<String, i32>,
    /// Items in inventory.
    pub inventory: HashSet<String>,
    /// Active conditions with remaining duration (0 = permanent).
    pub conditions: HashMap<String, u32>,
    /// Per-NPC arc phases.
    pub arcs: HashMap<String, String>,
    /// Active plane type.
    pub active_plane: String,
    /// Flags (boolean state bits).
    pub flags: HashSet<String>,
    /// Current narrative node ID.
    pub current_node: String,
    /// Provenance session ID.
    pub session_id: String,
    /// Trio primals reached during provenance operations (per `PROVENANCE_TRIO_INTEGRATION_GUIDE`).
    ///
    /// Tracks partial completion: `["dag"]` means only rhizoCrypt responded,
    /// `["dag", "spine", "braid"]` means full trio. Empty means no provenance.
    pub primals_reached: Vec<String>,
    /// Turn counter.
    pub turn: u32,
}

impl WorldState {
    /// Create a new default world state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_plane: "exploration".to_owned(),
            ..Self::default()
        }
    }

    /// Evaluate a predicate against the current state.
    #[must_use]
    pub fn evaluate(&self, predicate: &StatePredicate) -> bool {
        match predicate {
            StatePredicate::HasKnowledge(k) => self.knowledge.contains(k),
            StatePredicate::LacksKnowledge(k) => !self.knowledge.contains(k),
            StatePredicate::TrustAbove(npc, level) => {
                self.trust.get(npc).copied().unwrap_or(0) >= *level
            }
            StatePredicate::TrustBelow(npc, level) => {
                self.trust.get(npc).copied().unwrap_or(0) < *level
            }
            StatePredicate::InPlane(plane) => self.active_plane == *plane,
            StatePredicate::HasItem(item) => self.inventory.contains(item),
            StatePredicate::LacksItem(item) => !self.inventory.contains(item),
            StatePredicate::ConditionActive(cond) => self.conditions.contains_key(cond),
            StatePredicate::ConditionInactive(cond) => !self.conditions.contains_key(cond),
            StatePredicate::ArcPhaseIs(npc, phase) => self.arcs.get(npc) == Some(phase),
            StatePredicate::FlagSet(flag) => self.flags.contains(flag),
            StatePredicate::FlagUnset(flag) => !self.flags.contains(flag),
            StatePredicate::All(preds) => preds.iter().all(|p| self.evaluate(p)),
            StatePredicate::Any(preds) => preds.iter().any(|p| self.evaluate(p)),
            StatePredicate::Not(pred) => !self.evaluate(pred),
        }
    }

    /// Apply a state effect, mutating this state.
    pub fn apply(&mut self, effect: &StateEffect) {
        match effect {
            StateEffect::AddKnowledge(k) => {
                self.knowledge.insert(k.clone());
            }
            StateEffect::RemoveKnowledge(k) => {
                self.knowledge.remove(k);
            }
            StateEffect::ModifyTrust(npc, delta) => {
                let entry = self.trust.entry(npc.clone()).or_insert(0);
                *entry = entry.saturating_add(*delta);
            }
            StateEffect::SetTrust(npc, val) => {
                self.trust.insert(npc.clone(), *val);
            }
            StateEffect::AddItem(item) => {
                self.inventory.insert(item.clone());
            }
            StateEffect::RemoveItem(item) => {
                self.inventory.remove(item);
            }
            StateEffect::ApplyCondition(cond, duration) => {
                self.conditions.insert(cond.clone(), *duration);
            }
            StateEffect::RemoveCondition(cond) => {
                self.conditions.remove(cond);
            }
            StateEffect::AdvanceArc(npc, phase) => {
                self.arcs.insert(npc.clone(), phase.clone());
            }
            StateEffect::TransitionPlane(plane) => {
                self.active_plane.clone_from(plane);
            }
            StateEffect::SetFlag(flag) => {
                self.flags.insert(flag.clone());
            }
            StateEffect::ClearFlag(flag) => {
                self.flags.remove(flag);
            }
            StateEffect::Batch(effects) => {
                for e in effects {
                    self.apply(e);
                }
            }
        }
    }

    /// Tick conditions: decrement durations, remove expired ones.
    pub fn tick_conditions(&mut self) {
        self.conditions.retain(|_, dur| {
            if *dur == 0 {
                return true; // permanent
            }
            *dur = dur.saturating_sub(1);
            *dur > 0
        });
        self.turn = self.turn.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_defaults() {
        let state = WorldState::new();
        assert_eq!(state.current_node, "");
        assert_eq!(state.active_plane, "exploration");
        assert_eq!(state.turn, 0);
        assert!(state.knowledge.is_empty());
    }

    #[test]
    fn knowledge_predicate_and_effect() {
        let mut state = WorldState::new();
        assert!(state.evaluate(&StatePredicate::LacksKnowledge("elder_sign".to_owned())));
        state.apply(&StateEffect::AddKnowledge("elder_sign".to_owned()));
        assert!(state.evaluate(&StatePredicate::HasKnowledge("elder_sign".to_owned())));
    }

    #[test]
    fn trust_predicate_and_effect() {
        let mut state = WorldState::new();
        assert!(state.evaluate(&StatePredicate::TrustBelow("maren".to_owned(), 1)));
        state.apply(&StateEffect::ModifyTrust("maren".to_owned(), 3));
        assert!(state.evaluate(&StatePredicate::TrustAbove("maren".to_owned(), 3)));
        assert!(!state.evaluate(&StatePredicate::TrustAbove("maren".to_owned(), 4)));
    }

    #[test]
    fn inventory_predicate_and_effect() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::AddItem("silver_key".to_owned()));
        assert!(state.evaluate(&StatePredicate::HasItem("silver_key".to_owned())));
        state.apply(&StateEffect::RemoveItem("silver_key".to_owned()));
        assert!(state.evaluate(&StatePredicate::LacksItem("silver_key".to_owned())));
    }

    #[test]
    fn condition_tick_expiry() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::ApplyCondition("frightened".to_owned(), 2));
        assert!(state.evaluate(&StatePredicate::ConditionActive("frightened".to_owned())));
        state.tick_conditions();
        assert!(state.evaluate(&StatePredicate::ConditionActive("frightened".to_owned())));
        state.tick_conditions();
        assert!(state.evaluate(&StatePredicate::ConditionInactive("frightened".to_owned())));
    }

    #[test]
    fn permanent_condition_persists() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::ApplyCondition("blessed".to_owned(), 0));
        for _ in 0..10 {
            state.tick_conditions();
        }
        assert!(state.evaluate(&StatePredicate::ConditionActive("blessed".to_owned())));
    }

    #[test]
    fn arc_phase_tracking() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::AdvanceArc(
            "maren".to_owned(),
            "suspicious".to_owned(),
        ));
        assert!(state.evaluate(&StatePredicate::ArcPhaseIs(
            "maren".to_owned(),
            "suspicious".to_owned()
        )));
    }

    #[test]
    fn plane_transition() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::TransitionPlane("dialogue".to_owned()));
        assert!(state.evaluate(&StatePredicate::InPlane("dialogue".to_owned())));
    }

    #[test]
    fn flag_operations() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::SetFlag("ritual_started".to_owned()));
        assert!(state.evaluate(&StatePredicate::FlagSet("ritual_started".to_owned())));
        state.apply(&StateEffect::ClearFlag("ritual_started".to_owned()));
        assert!(state.evaluate(&StatePredicate::FlagUnset("ritual_started".to_owned())));
    }

    #[test]
    fn compound_predicates() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::AddKnowledge("a".to_owned()));
        state.apply(&StateEffect::AddKnowledge("b".to_owned()));

        let all = StatePredicate::All(vec![
            StatePredicate::HasKnowledge("a".to_owned()),
            StatePredicate::HasKnowledge("b".to_owned()),
        ]);
        assert!(state.evaluate(&all));

        let any = StatePredicate::Any(vec![
            StatePredicate::HasKnowledge("a".to_owned()),
            StatePredicate::HasKnowledge("c".to_owned()),
        ]);
        assert!(state.evaluate(&any));

        let not = StatePredicate::Not(Box::new(StatePredicate::HasKnowledge("c".to_owned())));
        assert!(state.evaluate(&not));
    }

    #[test]
    fn batch_effect() {
        let mut state = WorldState::new();
        state.apply(&StateEffect::Batch(vec![
            StateEffect::AddKnowledge("x".to_owned()),
            StateEffect::ModifyTrust("n".to_owned(), 2),
            StateEffect::SetFlag("f".to_owned()),
        ]));
        assert!(state.evaluate(&StatePredicate::HasKnowledge("x".to_owned())));
        assert!(state.evaluate(&StatePredicate::TrustAbove("n".to_owned(), 2)));
        assert!(state.evaluate(&StatePredicate::FlagSet("f".to_owned())));
    }
}
