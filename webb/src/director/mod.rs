// SPDX-License-Identifier: AGPL-3.0-or-later
//! `GameDirector` — the runtime coordinator for Esoteric Webb.
//!
//! Processes the game loop:
//! 1. Receive player input (visualization primal poll or text-mode stdin)
//! 2. Resolve against current scene and valid exits
//! 3. Evaluate state predicates against the current world state
//! 4. Apply state effects
//! 5. Record provenance vertex
//! 6. Request narration (game science + AI primals)
//! 7. Push scene (visualization primal)
//! 8. Evaluate metrics (flow, engagement, DDA via game science primal)

use crate::content::ContentBundle;
use crate::narrative::NarrativeEdge;
use crate::narrative::predicate::StatePredicate;
use crate::state::WorldState;

/// Player input that the director resolves.
#[derive(Debug, Clone)]
pub enum PlayerInput {
    /// Choose a specific exit by target node ID.
    ChooseExit(String),
    /// Use an ability by ID.
    UseAbility(String),
    /// Examine the current scene.
    Examine,
    /// Talk to an NPC in the current scene.
    Talk(String),
}

/// The result of processing a single input.
#[derive(Debug)]
pub enum DirectorOutcome {
    /// Pure narration (no state change).
    Narration(String),
    /// Moved to a new scene.
    SceneChange {
        /// Target node we moved to.
        node_id: String,
        /// Narration for the transition.
        narration: String,
    },
    /// Input had no effect.
    NoEffect(String),
}

/// The game director runtime — holds only position, borrows everything else.
pub struct GameDirector {
    current_node: String,
}

impl GameDirector {
    /// Create a new director from a loaded content bundle.
    ///
    /// # Errors
    ///
    /// Returns an error if the content has no start node.
    pub fn new(bundle: &ContentBundle) -> crate::error::Result<Self> {
        let start = bundle
            .narrative
            .start_node()
            .ok_or(crate::error::WebbError::NoStartNode)?;
        Ok(Self {
            current_node: start.id.clone(),
        })
    }

    /// Get the current narrative node ID.
    #[must_use]
    pub fn current_node_id(&self) -> &str {
        &self.current_node
    }

    /// Get the available exits from the current node given the state.
    #[must_use]
    pub fn available_exits<'a>(
        &self,
        bundle: &'a ContentBundle,
        state: &WorldState,
    ) -> Vec<&'a NarrativeEdge> {
        bundle
            .narrative
            .valid_exits(&self.current_node, |pred| state.evaluate(pred))
    }

    /// Check if the current node is an ending.
    #[must_use]
    pub fn is_at_ending(&self, bundle: &ContentBundle) -> bool {
        bundle
            .narrative
            .get(&self.current_node)
            .is_some_and(|n| n.is_ending)
    }

    /// Get the description for the current scene.
    #[must_use]
    pub fn current_scene_description(&self, bundle: &ContentBundle) -> String {
        let content_ref = bundle
            .narrative
            .get(&self.current_node)
            .map_or("", |n| n.content_ref.as_str());
        bundle.scenes.get(content_ref).map_or_else(
            || format!("[no scene content for '{}']", self.current_node),
            |scene| scene.description.clone(),
        )
    }

    /// Process a player input and advance the game state.
    pub fn process(
        &mut self,
        input: &PlayerInput,
        state: &mut WorldState,
        bundle: &ContentBundle,
    ) -> DirectorOutcome {
        match input {
            PlayerInput::ChooseExit(target) => self.process_exit(target, state, bundle),
            PlayerInput::UseAbility(ability_id) => self.process_ability(ability_id, state, bundle),
            PlayerInput::Examine => {
                DirectorOutcome::Narration(self.current_scene_description(bundle))
            }
            PlayerInput::Talk(npc_id) => self.process_talk(npc_id, state, bundle),
        }
    }

    fn process_exit(
        &mut self,
        target: &str,
        state: &mut WorldState,
        bundle: &ContentBundle,
    ) -> DirectorOutcome {
        let exits = self.available_exits(bundle, state);
        let valid = exits.iter().any(|e| e.target == target);
        if !valid {
            return DirectorOutcome::NoEffect(format!("Cannot go to '{target}' from here."));
        }

        if let Some(target_node) = bundle.narrative.get(target) {
            for effect in &target_node.effects {
                state.apply(effect);
            }
        }

        target.clone_into(&mut self.current_node);
        state.current_node.clone_from(&self.current_node);
        state.turn += 1;
        state.tick_conditions();

        let narration = self.current_scene_description(bundle);
        DirectorOutcome::SceneChange {
            node_id: target.to_owned(),
            narration,
        }
    }

    fn process_ability(
        &self,
        ability_id: &str,
        state: &mut WorldState,
        bundle: &ContentBundle,
    ) -> DirectorOutcome {
        let Some(ability) = bundle.abilities.get(ability_id) else {
            return DirectorOutcome::NoEffect(format!("Unknown ability: {ability_id}"));
        };

        let can_use = ability.preconditions.iter().all(|p| state.evaluate(p));
        if !can_use {
            let failed: Vec<String> = ability
                .preconditions
                .iter()
                .filter(|p| !state.evaluate(p))
                .map(StatePredicate::describe)
                .collect();
            return DirectorOutcome::NoEffect(format!(
                "Cannot use '{}' — unmet: {}",
                ability.name,
                failed.join(", ")
            ));
        }

        for effect in &ability.effects {
            state.apply(effect);
        }

        let hint = ability
            .narration_hint
            .as_deref()
            .unwrap_or("The effect takes hold.");

        let new_exits = self.available_exits(bundle, state).len();
        let paths_note = if new_exits > 0 {
            format!(" [{new_exits} path(s) now available]")
        } else {
            String::new()
        };

        DirectorOutcome::Narration(format!("You use {}. {hint}{paths_note}", ability.name))
    }

    fn process_talk(
        &self,
        npc_id: &str,
        state: &mut WorldState,
        bundle: &ContentBundle,
    ) -> DirectorOutcome {
        let in_scene = bundle
            .narrative
            .get(&self.current_node)
            .and_then(|node| bundle.scenes.get(&node.content_ref))
            .is_some_and(|scene| scene.npcs.iter().any(|n| n == npc_id));

        if !in_scene {
            return if bundle.npcs.contains_key(npc_id) {
                DirectorOutcome::NoEffect(format!("{npc_id} is not in this scene."))
            } else {
                DirectorOutcome::NoEffect(format!("There is no one called '{npc_id}' here."))
            };
        }

        let Some(npc) = bundle.npcs.get(npc_id) else {
            return DirectorOutcome::NoEffect(format!("There is no one called '{npc_id}' here."));
        };

        let old_trust = state
            .trust
            .get(npc_id)
            .copied()
            .unwrap_or(npc.trust_initial);
        let new_trust = old_trust + 1;
        state.trust.insert(npc_id.to_owned(), new_trust);

        let mut narration_parts: Vec<String> = vec![format!("You speak with {}.", npc.name)];

        for (&threshold, reward) in &npc.trust_rewards {
            if old_trust < threshold && new_trust >= threshold {
                if !reward.description.is_empty() {
                    narration_parts.push(reward.description.clone());
                }
                for k in &reward.grants_knowledge {
                    state.knowledge.insert(k.clone());
                    narration_parts.push(format!("[Learned: {k}]"));
                }
                for item in &reward.grants_items {
                    state.inventory.insert(item.clone());
                    narration_parts.push(format!("[Received: {item}]"));
                }
                for flag in &reward.sets_flags {
                    state.flags.insert(flag.clone());
                    narration_parts.push(format!("[{flag}]"));
                }
            }
        }

        if narration_parts.len() == 1 {
            narration_parts.push(format!(
                "{} regards you with {}.",
                npc.name,
                trust_demeanor(new_trust)
            ));
        }

        DirectorOutcome::Narration(narration_parts.join(" "))
    }
}

const fn trust_demeanor(trust: i32) -> &'static str {
    match trust {
        i32::MIN..=0 => "hostility",
        1 => "wariness",
        2 => "cautious interest",
        3 => "growing warmth",
        4..=5 => "openness",
        _ => "deep trust",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{AbilityDef, ContentBundle, NpcDef, SceneContent, WorldMeta};
    use crate::narrative::effect::StateEffect;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use std::collections::HashMap;

    #[expect(clippy::too_many_lines, reason = "test fixture construction")]
    fn test_content() -> ContentBundle {
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
                        target: "parlor".to_owned(),
                        conditions: vec![],
                        priority: 0,
                        transition_type: TransitionType::SamePlane,
                        label: Some("enter parlor".to_owned()),
                    },
                    NarrativeEdge {
                        target: "ending".to_owned(),
                        conditions: vec![StatePredicate::HasKnowledge("truth".to_owned())],
                        priority: 1,
                        transition_type: TransitionType::SamePlane,
                        label: Some("confront truth".to_owned()),
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
                description: "You stand at the entrance of The Weaver's Parlor.".to_owned(),
                npcs: vec![],
                items: vec![],
            },
        );

        nodes.insert(
            "parlor".to_owned(),
            NarrativeNode {
                id: "parlor".to_owned(),
                scene_type: SceneType::Dialogue,
                content_ref: "parlor".to_owned(),
                preconditions: vec![],
                effects: vec![StateEffect::AddKnowledge("truth".to_owned())],
                exits: vec![NarrativeEdge {
                    target: "start".to_owned(),
                    conditions: vec![],
                    priority: 0,
                    transition_type: TransitionType::SamePlane,
                    label: Some("return".to_owned()),
                }],
                is_start: false,
                is_ending: false,
                label: None,
            },
        );
        scenes.insert(
            "parlor".to_owned(),
            SceneContent {
                id: "parlor".to_owned(),
                description: "The parlor is filled with antique curiosities.".to_owned(),
                npcs: vec!["maren".to_owned()],
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
                label: Some("Justice Ending".to_owned()),
            },
        );
        scenes.insert(
            "ending".to_owned(),
            SceneContent {
                id: "ending".to_owned(),
                description: "The truth is laid bare. Justice is served.".to_owned(),
                npcs: vec![],
                items: vec![],
            },
        );

        let mut abilities = HashMap::new();
        abilities.insert(
            "read_aura".to_owned(),
            AbilityDef {
                id: "read_aura".to_owned(),
                name: "Read Aura".to_owned(),
                description: "Perceive the emotional state of those nearby.".to_owned(),
                preconditions: vec![],
                effects: vec![StateEffect::AddKnowledge("aura_sight".to_owned())],
                narration_hint: Some("Colors bloom around nearby figures.".to_owned()),
            },
        );

        let mut npcs = HashMap::new();
        npcs.insert(
            "maren".to_owned(),
            NpcDef {
                id: "maren".to_owned(),
                name: "Maren".to_owned(),
                role: "proprietor".to_owned(),
                knows: vec![],
                trust_initial: 0,
                trust_rewards: std::collections::BTreeMap::new(),
                lies_about: HashMap::new(),
                arc: String::new(),
            },
        );

        ContentBundle {
            meta: WorldMeta {
                name: "The Weaver's Parlor".to_owned(),
                author: "Esoteric Webb".to_owned(),
                version: "0.1.0".to_owned(),
                description: "A test scenario.".to_owned(),
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

    #[test]
    fn director_starts_at_start_node() {
        let content = test_content();
        let director = GameDirector::new(&content);
        assert!(director.is_ok());
        let d = director.unwrap_or_else(|_| unreachable!());
        assert_eq!(d.current_node_id(), "start");
    }

    #[test]
    fn examine_returns_scene_description() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();
        let outcome = d.process(&PlayerInput::Examine, &mut state, &content);
        assert!(
            matches!(outcome, DirectorOutcome::Narration(ref s) if s.contains("Weaver's Parlor"))
        );
    }

    #[test]
    fn choose_exit_transitions() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();
        let outcome = d.process(
            &PlayerInput::ChooseExit("parlor".to_owned()),
            &mut state,
            &content,
        );
        assert!(matches!(outcome, DirectorOutcome::SceneChange { .. }));
        assert_eq!(d.current_node_id(), "parlor");
        assert!(state.knowledge.contains("truth"));
    }

    #[test]
    fn gated_exit_becomes_available_after_knowledge() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();

        let exits = d.available_exits(&content, &state);
        assert_eq!(exits.len(), 1);

        d.process(
            &PlayerInput::ChooseExit("parlor".to_owned()),
            &mut state,
            &content,
        );
        d.process(
            &PlayerInput::ChooseExit("start".to_owned()),
            &mut state,
            &content,
        );

        let exits = d.available_exits(&content, &state);
        assert_eq!(exits.len(), 2);
    }

    #[test]
    fn use_ability_applies_effects() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();
        let outcome = d.process(
            &PlayerInput::UseAbility("read_aura".to_owned()),
            &mut state,
            &content,
        );
        assert!(matches!(outcome, DirectorOutcome::Narration(_)));
        assert!(state.knowledge.contains("aura_sight"));
    }

    #[test]
    fn unknown_ability_fails() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();
        let outcome = d.process(
            &PlayerInput::UseAbility("nonexistent".to_owned()),
            &mut state,
            &content,
        );
        assert!(
            matches!(outcome, DirectorOutcome::NoEffect(ref s) if s.contains("Unknown ability"))
        );
    }

    #[test]
    fn ending_detected() {
        let content = test_content();
        let mut d = GameDirector::new(&content).unwrap_or_else(|_| unreachable!());
        let mut state = WorldState::new();
        d.process(
            &PlayerInput::ChooseExit("parlor".to_owned()),
            &mut state,
            &content,
        );
        d.process(
            &PlayerInput::ChooseExit("start".to_owned()),
            &mut state,
            &content,
        );
        d.process(
            &PlayerInput::ChooseExit("ending".to_owned()),
            &mut state,
            &content,
        );
        assert!(d.is_at_ending(&content));
    }
}
