// SPDX-License-Identifier: AGPL-3.0-or-later
//! Flow state model — the science of optimal experience.
//!
//! Csikszentmihalyi's flow model predicts engagement based on the balance
//! between challenge (task difficulty) and skill (player ability). Games
//! that maintain the flow channel keep players engaged; deviation causes
//! either boredom (too easy) or anxiety (too hard).
//!
//! # References
//! - Csikszentmihalyi, M. (1990). *Flow: The Psychology of Optimal Experience*
//! - Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.

use serde::{Deserialize, Serialize};

/// The player's current experience state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    /// Challenge far below skill. Player disengages.
    Boredom,
    /// Challenge slightly below skill. Comfortable but not gripping.
    Relaxation,
    /// Challenge matches skill. Optimal engagement.
    Flow,
    /// Challenge slightly above skill. Stimulating but stressful.
    Arousal,
    /// Challenge far above skill. Player panics or quits.
    Anxiety,
}

impl std::fmt::Display for FlowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FlowState {
    /// Lowercase string representation for JSON serialization.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boredom => "boredom",
            Self::Relaxation => "relaxation",
            Self::Flow => "flow",
            Self::Arousal => "arousal",
            Self::Anxiety => "anxiety",
        }
    }
}

/// Evaluate flow state given normalized challenge and skill (both 0.0–1.0).
///
/// The flow channel is a band around the `challenge == skill` diagonal.
/// `channel_width` controls how wide the band is (default ~0.15).
#[must_use]
pub fn evaluate_flow(challenge: f64, skill: f64, channel_width: f64) -> FlowState {
    let diff = challenge - skill;
    if diff.abs() <= channel_width {
        FlowState::Flow
    } else if diff > channel_width * 2.0 {
        FlowState::Anxiety
    } else if diff > channel_width {
        FlowState::Arousal
    } else if diff < -channel_width * 2.0 {
        FlowState::Boredom
    } else {
        FlowState::Relaxation
    }
}

/// IPC-shaped flow result for bridge compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    /// Continuous flow score in \[0.0, 1.0\].
    pub flow_score: f64,
    /// Whether the player is currently in the flow channel.
    pub in_flow: bool,
}

const NUMERICAL_FLOOR: f64 = 1e-9;
const SPAN_FLOOR: f64 = 1e-6;

/// Continuous flow score in \[0.0, 1.0\] and whether the player sits in the flow channel.
///
/// Score is 1.0 when |challenge − skill| <= `channel_width` (matches [`FlowState::Flow`]),
/// then falls off linearly toward 0 as the gap grows.
#[must_use]
pub fn flow_channel_metrics(challenge: f64, skill: f64, channel_width: f64) -> FlowResult {
    let d = (challenge - skill).abs();
    let w = channel_width.max(NUMERICAL_FLOOR);
    let in_flow = d <= w;
    let flow_score = if in_flow {
        1.0
    } else {
        let excess = d - w;
        let span = (1.0_f64 - w).max(SPAN_FLOOR);
        (1.0 - excess / span).clamp(0.0, 1.0)
    };
    FlowResult {
        flow_score,
        in_flow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_challenge_skill_is_flow() {
        assert_eq!(evaluate_flow(0.5, 0.5, 0.15), FlowState::Flow);
    }

    #[test]
    fn high_challenge_low_skill_is_anxiety() {
        assert_eq!(evaluate_flow(0.9, 0.1, 0.1), FlowState::Anxiety);
    }

    #[test]
    fn low_challenge_high_skill_is_boredom() {
        assert_eq!(evaluate_flow(0.1, 0.9, 0.1), FlowState::Boredom);
    }

    #[test]
    fn slight_challenge_above_skill_is_arousal() {
        assert_eq!(evaluate_flow(0.65, 0.5, 0.1), FlowState::Arousal);
    }

    #[test]
    fn slight_skill_above_challenge_is_relaxation() {
        assert_eq!(evaluate_flow(0.4, 0.55, 0.1), FlowState::Relaxation);
    }

    #[test]
    fn flow_channel_metrics_in_flow() {
        let result = flow_channel_metrics(0.5, 0.5, 0.15);
        assert!((result.flow_score - 1.0).abs() < f64::EPSILON);
        assert!(result.in_flow);
    }

    #[test]
    fn flow_channel_metrics_outside_flow() {
        let result = flow_channel_metrics(0.9, 0.1, 0.1);
        assert!(!result.in_flow);
        assert!(result.flow_score < 0.5);
    }

    #[test]
    fn flow_channel_metrics_edge() {
        let result = flow_channel_metrics(0.5, 0.4, 0.15);
        assert!(result.in_flow);
        assert!((result.flow_score - 1.0).abs() < f64::EPSILON);
    }
}
