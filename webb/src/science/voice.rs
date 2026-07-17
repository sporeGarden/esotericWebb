// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline voice interjection engine.
//!
//! Fires "internal voice" notes based on game state predicates,
//! emulating Disco Elysium-style skill commentary without requiring
//! the AI primal. When Squirrel is available, these local interjections
//! are supplemented (not replaced) by AI-generated ones.
//!
//! Voice profiles are defined as static data — creative teams will
//! eventually author them in YAML alongside other content. This module
//! provides the evaluation engine and a starter set of built-in voices.

use crate::state::WorldState;

/// A voice profile that can fire interjections.
#[derive(Debug, Clone)]
pub struct VoiceProfile {
    /// Voice identifier (e.g. "logic", "empathy", "perception").
    pub id: &'static str,
    /// Display name shown to the player.
    pub display_name: &'static str,
    /// Triggers that cause this voice to fire.
    pub triggers: &'static [VoiceTrigger],
}

/// A condition under which a voice fires an interjection.
#[derive(Debug, Clone)]
pub struct VoiceTrigger {
    /// What state condition activates this trigger.
    pub condition: TriggerCondition,
    /// The interjection text to display.
    pub text: &'static str,
    /// Priority (lower = shown first when multiple voices fire).
    pub priority: u32,
}

/// Condition types for voice triggers.
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    /// Fires when the player has a specific knowledge entry.
    HasKnowledge(&'static str),
    /// Fires when the player has a specific flag set.
    HasFlag(&'static str),
    /// Fires when trust with an NPC exceeds a threshold.
    TrustAbove(&'static str, i32),
    /// Fires when trust with an NPC is below a threshold.
    TrustBelow(&'static str, i32),
    /// Fires when the player has a specific item.
    HasItem(&'static str),
    /// Fires when the player is on a specific narrative plane.
    OnPlane(&'static str),
}

/// A fired voice interjection ready for enrichment.
#[derive(Debug, Clone)]
pub struct FiredInterjection {
    /// Voice identifier.
    pub voice_id: String,
    /// Interjection text.
    pub text: String,
    /// Priority (lower = more important).
    pub priority: u32,
}

/// Built-in voice profiles — the starter set for offline play.
pub const BUILT_IN_VOICES: &[VoiceProfile] = &[
    VoiceProfile {
        id: "logic",
        display_name: "Logic",
        triggers: &[
            VoiceTrigger {
                condition: TriggerCondition::HasKnowledge("contradiction"),
                text: "Something doesn't add up here. The facts contradict each other.",
                priority: 1,
            },
            VoiceTrigger {
                condition: TriggerCondition::HasFlag("evidence_found"),
                text: "We have evidence now. Time to confront the inconsistencies.",
                priority: 2,
            },
        ],
    },
    VoiceProfile {
        id: "empathy",
        display_name: "Empathy",
        triggers: &[
            VoiceTrigger {
                condition: TriggerCondition::TrustBelow("maren", 0),
                text: "She's pulling away. Something we said hurt her.",
                priority: 1,
            },
            VoiceTrigger {
                condition: TriggerCondition::TrustAbove("maren", 5),
                text: "There's genuine warmth here now. She trusts us.",
                priority: 3,
            },
        ],
    },
    VoiceProfile {
        id: "perception",
        display_name: "Perception",
        triggers: &[
            VoiceTrigger {
                condition: TriggerCondition::HasItem("old_key"),
                text: "That key... it's heavier than it should be. There's something inside.",
                priority: 2,
            },
            VoiceTrigger {
                condition: TriggerCondition::OnPlane("shadow"),
                text: "The shadows are thicker here. Stay alert.",
                priority: 1,
            },
        ],
    },
];

/// Evaluate all voice profiles against the current game state.
///
/// Returns fired interjections sorted by priority (lowest first).
#[must_use]
pub fn evaluate_voices(state: &WorldState, voices: &[VoiceProfile]) -> Vec<FiredInterjection> {
    let mut fired = Vec::new();

    for voice in voices {
        for trigger in voice.triggers {
            if matches_condition(&trigger.condition, state) {
                fired.push(FiredInterjection {
                    voice_id: voice.id.to_owned(),
                    text: trigger.text.to_owned(),
                    priority: trigger.priority,
                });
            }
        }
    }

    fired.sort_by_key(|f| f.priority);
    fired
}

/// Check whether a trigger condition is satisfied by the current state.
fn matches_condition(condition: &TriggerCondition, state: &WorldState) -> bool {
    match condition {
        TriggerCondition::HasKnowledge(k) => state.knowledge.contains(*k),
        TriggerCondition::HasFlag(f) => state.flags.contains(*f),
        TriggerCondition::TrustAbove(npc, threshold) => {
            state.trust.get(*npc).is_some_and(|t| *t > *threshold)
        }
        TriggerCondition::TrustBelow(npc, threshold) => {
            state.trust.get(*npc).is_some_and(|t| *t < *threshold)
        }
        TriggerCondition::HasItem(item) => state.inventory.contains(*item),
        TriggerCondition::OnPlane(plane) => state.active_plane == *plane,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> WorldState {
        let mut state = WorldState::default();
        state.knowledge.insert("contradiction".to_owned());
        state.flags.insert("evidence_found".to_owned());
        state.trust.insert("maren".to_owned(), -2);
        state.inventory.insert("old_key".to_owned());
        state.active_plane = "material".to_owned();
        state
    }

    #[test]
    fn evaluate_fires_matching_triggers() {
        let state = test_state();
        let interjections = evaluate_voices(&state, BUILT_IN_VOICES);
        assert!(
            !interjections.is_empty(),
            "should fire at least one interjection"
        );
        let voice_ids: Vec<&str> = interjections.iter().map(|i| i.voice_id.as_str()).collect();
        assert!(voice_ids.contains(&"logic"), "logic should fire");
        assert!(voice_ids.contains(&"empathy"), "empathy should fire");
        assert!(voice_ids.contains(&"perception"), "perception should fire");
    }

    #[test]
    fn interjections_sorted_by_priority() {
        let state = test_state();
        let interjections = evaluate_voices(&state, BUILT_IN_VOICES);
        for window in interjections.windows(2) {
            assert!(
                window[0].priority <= window[1].priority,
                "interjections should be sorted by priority"
            );
        }
    }

    #[test]
    fn empty_state_fires_nothing() {
        let state = WorldState::default();
        let interjections = evaluate_voices(&state, BUILT_IN_VOICES);
        assert!(interjections.is_empty());
    }

    #[test]
    fn trust_above_triggers_correctly() {
        let mut state = WorldState::default();
        state.trust.insert("maren".to_owned(), 10);
        let interjections = evaluate_voices(&state, BUILT_IN_VOICES);
        assert!(interjections.iter().any(|i| i.voice_id == "empathy"));
    }

    #[test]
    fn plane_trigger_fires_on_match() {
        let state = WorldState {
            active_plane: "shadow".to_owned(),
            ..WorldState::default()
        };
        let interjections = evaluate_voices(&state, BUILT_IN_VOICES);
        assert!(interjections.iter().any(|i| i.voice_id == "perception"));
    }

    #[test]
    fn plane_trigger_silent_on_mismatch() {
        let state = WorldState {
            active_plane: "material".to_owned(),
            ..WorldState::default()
        };
        let voices = &[VoiceProfile {
            id: "test",
            display_name: "Test",
            triggers: &[VoiceTrigger {
                condition: TriggerCondition::OnPlane("shadow"),
                text: "shadow text",
                priority: 1,
            }],
        }];
        let interjections = evaluate_voices(&state, voices);
        assert!(interjections.is_empty());
    }

    #[test]
    fn custom_voice_profiles_work() {
        let voices = &[VoiceProfile {
            id: "custom",
            display_name: "Custom Voice",
            triggers: &[VoiceTrigger {
                condition: TriggerCondition::HasFlag("test_flag"),
                text: "Custom interjection.",
                priority: 0,
            }],
        }];
        let mut state = WorldState::default();
        state.flags.insert("test_flag".to_owned());
        let interjections = evaluate_voices(&state, voices);
        assert_eq!(interjections.len(), 1);
        assert_eq!(interjections[0].voice_id, "custom");
    }
}
