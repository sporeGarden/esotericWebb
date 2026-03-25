// SPDX-License-Identifier: AGPL-3.0-or-later
//! Autoplay engine — heuristic traversal of the narrative graph.
//!
//! Provides a deterministic exploration strategy that exercises all
//! reachable paths: unexplored exits first, then unused abilities,
//! then NPC conversations (building trust), then examine, then
//! rotate through known exits to discover state-gated paths.
//!
//! Living in the library (not the binary) so it can be unit-tested
//! and reused by experiments and integration tests.

use std::collections::{HashMap, HashSet};

use crate::session::{ActionKind, AvailableAction, GameSession};

/// Configuration for autoplay behavior — replaces magic numbers.
#[derive(Debug, Clone)]
pub struct AutoplayConfig {
    /// Maximum turns before the autoplay halts.
    pub max_turns: u32,
    /// Maximum consecutive turns without novel discovery before halting.
    pub stale_limit: u32,
    /// Maximum talk actions per NPC before moving on.
    pub max_talks_per_npc: u32,
}

impl Default for AutoplayConfig {
    fn default() -> Self {
        Self {
            max_turns: 100,
            stale_limit: 12,
            max_talks_per_npc: 8,
        }
    }
}

/// Result of a completed autoplay run.
#[derive(Debug, Clone)]
pub struct AutoplayResult {
    /// Whether the session reached an ending.
    pub ended: bool,
    /// Total turns taken.
    pub turns: u32,
    /// Unique nodes visited.
    pub nodes_visited: usize,
    /// Whether autoplay halted due to stale state.
    pub stale_halt: bool,
}

/// Tracks exploration state for the heuristic.
#[derive(Debug, Default)]
pub struct HeuristicTracker {
    /// Nodes the autoplay has visited.
    pub visited: HashSet<String>,
    used_abilities: HashSet<String>,
    talk_count: HashMap<String, u32>,
    examined_at: HashSet<String>,
    stale_count: u32,
    last_knowledge_count: usize,
    exit_rotation: usize,
}

impl HeuristicTracker {
    /// Record whether a turn produced novel state, for stale detection.
    pub fn record_novelty(&mut self, kind: ActionKind, id: &str, knowledge_now: usize) {
        let novel = match kind {
            ActionKind::Ability => self.used_abilities.insert(id.to_owned()),
            ActionKind::Talk => knowledge_now > self.last_knowledge_count,
            ActionKind::Examine => self.examined_at.insert(id.to_owned()),
            ActionKind::Exit => !self.visited.contains(id),
        };
        self.last_knowledge_count = knowledge_now;
        if novel {
            self.stale_count = 0;
        } else {
            self.stale_count += 1;
        }
    }

    /// Pick the next action using the heuristic priority:
    /// 1. Unexplored exits
    /// 2. Unused abilities (not blocked)
    /// 3. Talk to NPCs (up to cap)
    /// 4. Examine (once per node)
    /// 5. Rotate through known exits (state may have unlocked new paths)
    pub fn pick(
        &mut self,
        actions: &[AvailableAction],
        current_node: &str,
        config: &AutoplayConfig,
    ) -> Option<(ActionKind, String)> {
        if self.stale_count > config.stale_limit {
            return None;
        }
        for a in actions {
            if a.kind == ActionKind::Exit && !self.visited.contains(&a.id) {
                return Some((a.kind, a.id.clone()));
            }
        }
        for a in actions {
            if a.kind == ActionKind::Ability
                && !self.used_abilities.contains(&a.id)
                && !a.detail.as_deref().unwrap_or("").starts_with("[blocked]")
            {
                return Some((a.kind, a.id.clone()));
            }
        }
        for a in actions {
            if a.kind == ActionKind::Talk {
                let count = self.talk_count.get(&a.id).copied().unwrap_or(0);
                if count < config.max_talks_per_npc {
                    *self.talk_count.entry(a.id.clone()).or_insert(0) += 1;
                    return Some((a.kind, a.id.clone()));
                }
            }
        }
        if !self.examined_at.contains(current_node) {
            self.examined_at.insert(current_node.to_owned());
            return Some((ActionKind::Examine, "examine".to_owned()));
        }
        let exits: Vec<_> = actions
            .iter()
            .filter(|a| a.kind == ActionKind::Exit)
            .collect();
        if !exits.is_empty() {
            let idx = self.exit_rotation % exits.len();
            self.exit_rotation += 1;
            let a = &exits[idx];
            return Some((a.kind, a.id.clone()));
        }
        None
    }

    /// Whether the autoplay has stalled.
    #[must_use]
    pub const fn is_stale(&self, config: &AutoplayConfig) -> bool {
        self.stale_count > config.stale_limit
    }
}

/// Run a full autoplay session with the given configuration.
///
/// # Errors
///
/// Returns an error if the session encounters an unrecoverable state.
pub fn run(session: &mut GameSession, config: &AutoplayConfig) -> Result<AutoplayResult, String> {
    let mut tracker = HeuristicTracker::default();
    tracker.visited.insert(session.snapshot().current_node);

    for _ in 0..config.max_turns {
        if session.is_ended() {
            break;
        }

        let actions = session.available_actions();
        let node = session.snapshot().current_node.clone();
        let choice = tracker.pick(&actions, &node, config);

        let Some((kind, id)) = choice else {
            break;
        };

        let (_outcome_text, _ctx) = session.act(kind, &id).map_err(|e| format!("act: {e}"))?;
        let snap_after = session.snapshot();
        let knowledge_count =
            snap_after.knowledge.len() + snap_after.flags.len() + snap_after.inventory.len();
        tracker.record_novelty(kind, &id, knowledge_count);
        tracker.visited.insert(snap_after.current_node.clone());
    }

    Ok(AutoplayResult {
        ended: session.is_ended(),
        turns: session.snapshot().turn,
        nodes_visited: tracker.visited.len(),
        stale_halt: tracker.is_stale(config),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests {
    use super::*;
    use crate::content::{ContentBundle, SceneContent, WorldMeta};
    use crate::director::GameDirector;
    use crate::narrative::effect::StateEffect;
    use crate::narrative::predicate::StatePredicate;
    use crate::narrative::{
        NarrativeEdge, NarrativeGraph, NarrativeNode, SceneType, TransitionType,
    };
    use crate::state::WorldState;
    use std::collections::HashMap;

    fn test_bundle() -> ContentBundle {
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
                        target: "room".to_owned(),
                        conditions: vec![],
                        priority: 0,
                        transition_type: TransitionType::SamePlane,
                        label: Some("Enter room".to_owned()),
                    },
                    NarrativeEdge {
                        target: "ending".to_owned(),
                        conditions: vec![StatePredicate::HasKnowledge("secret".to_owned())],
                        priority: 1,
                        transition_type: TransitionType::SamePlane,
                        label: Some("Confront".to_owned()),
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
                description: "A threshold.".to_owned(),
                npcs: vec![],
                items: vec![],
            },
        );

        nodes.insert(
            "room".to_owned(),
            NarrativeNode {
                id: "room".to_owned(),
                scene_type: SceneType::Dialogue,
                content_ref: "room".to_owned(),
                preconditions: vec![],
                effects: vec![StateEffect::AddKnowledge("secret".to_owned())],
                exits: vec![NarrativeEdge {
                    target: "start".to_owned(),
                    conditions: vec![],
                    priority: 0,
                    transition_type: TransitionType::SamePlane,
                    label: Some("Return".to_owned()),
                }],
                is_start: false,
                is_ending: false,
                label: None,
            },
        );
        scenes.insert(
            "room".to_owned(),
            SceneContent {
                id: "room".to_owned(),
                description: "A dark room.".to_owned(),
                npcs: vec![],
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
                label: None,
            },
        );
        scenes.insert(
            "ending".to_owned(),
            SceneContent {
                id: "ending".to_owned(),
                description: "The end.".to_owned(),
                npcs: vec![],
                items: vec![],
            },
        );

        ContentBundle {
            meta: WorldMeta {
                name: "Test".to_owned(),
                author: "test".to_owned(),
                version: "0.1.0".to_owned(),
                description: "Test world.".to_owned(),
            },
            narrative: NarrativeGraph { nodes },
            worlds: HashMap::new(),
            npcs: HashMap::new(),
            abilities: HashMap::new(),
            scenes,
            rulesets: HashMap::new(),
            load_warnings: Vec::new(),
        }
    }

    fn session_from_bundle(bundle: ContentBundle) -> GameSession {
        let director = GameDirector::new(&bundle).unwrap();
        GameSession::from_parts(bundle, director, WorldState::new(), None)
    }

    #[test]
    fn autoplay_reaches_ending() {
        let mut session = session_from_bundle(test_bundle());
        let config = AutoplayConfig::default();
        let result = run(&mut session, &config).unwrap();
        assert!(result.ended);
        assert!(result.turns > 0);
        assert!(result.nodes_visited >= 3);
    }

    #[test]
    fn autoplay_respects_max_turns() {
        let mut session = session_from_bundle(test_bundle());
        let config = AutoplayConfig {
            max_turns: 1,
            ..AutoplayConfig::default()
        };
        let result = run(&mut session, &config).unwrap();
        assert!(result.turns <= 1);
    }

    #[test]
    fn heuristic_prefers_unexplored_exits() {
        use crate::session::ActionKind;
        let mut tracker = HeuristicTracker::default();
        let config = AutoplayConfig::default();
        let actions = vec![
            AvailableAction {
                kind: ActionKind::Exit,
                id: "known".to_owned(),
                label: "Known".to_owned(),
                detail: None,
            },
            AvailableAction {
                kind: ActionKind::Exit,
                id: "new".to_owned(),
                label: "New".to_owned(),
                detail: None,
            },
        ];
        tracker.visited.insert("known".to_owned());
        let choice = tracker.pick(&actions, "here", &config);
        assert_eq!(choice, Some((ActionKind::Exit, "new".to_owned())));
    }

    #[test]
    fn stale_detection_halts_autoplay() {
        use crate::session::ActionKind;
        let config = AutoplayConfig {
            stale_limit: 2,
            ..AutoplayConfig::default()
        };
        let mut tracker = HeuristicTracker {
            stale_count: 3,
            ..HeuristicTracker::default()
        };
        assert!(tracker.is_stale(&config));
        let actions = vec![AvailableAction {
            kind: ActionKind::Examine,
            id: "examine".to_owned(),
            label: "Examine".to_owned(),
            detail: None,
        }];
        assert!(tracker.pick(&actions, "here", &config).is_none());
    }

    #[test]
    fn novelty_resets_stale_counter() {
        use crate::session::ActionKind;
        let mut tracker = HeuristicTracker {
            stale_count: 5,
            ..HeuristicTracker::default()
        };
        tracker.record_novelty(ActionKind::Ability, "new_ability", 0);
        assert_eq!(tracker.stale_count, 0);
    }
}
