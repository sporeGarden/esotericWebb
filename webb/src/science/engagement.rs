// SPDX-License-Identifier: AGPL-3.0-or-later
//! Engagement metrics — quantifying fun.
//!
//! "Fun" is measurable. Session length, action density, exploration breadth,
//! and challenge-seeking behavior are all observable signals. This module
//! provides the measurement framework.
//!
//! # References
//! - Lazzaro, N. (2004). "Why We Play Games: Four Keys to More Emotion
//!   Without Story." GDC '04.
//! - Yannakakis, G.N. & Togelius, J. (2018). *Artificial Intelligence and
//!   Games.* Springer.

use serde::{Deserialize, Serialize};

const SECONDS_PER_MINUTE: f64 = 60.0;
const MIN_SESSION_MINUTES: f64 = 0.01;
const ENGAGEMENT_APM_CEILING: f64 = 60.0;
const ENGAGEMENT_EXPLORATION_CEILING: f64 = 5.0;
const ENGAGEMENT_WEIGHT: f64 = 0.2;

/// A snapshot of player behavior over a time window.
#[derive(Debug, Clone, Default)]
pub struct EngagementSnapshot {
    /// Session duration in seconds.
    pub session_duration_s: f64,
    /// Number of meaningful actions taken.
    pub action_count: u64,
    /// Number of distinct areas/states explored.
    pub exploration_breadth: u32,
    /// Number of voluntary difficulty increases (player chose harder path).
    pub challenge_seeking: u32,
    /// Number of times the player repeated a failed attempt.
    pub retry_count: u32,
    /// Number of voluntary pauses (player stopped to think, not frustrated).
    pub deliberate_pauses: u32,
}

/// IPC-shaped engagement result for bridge compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementResult {
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration ratio (new areas per minute, normalized).
    pub exploration_ratio: f64,
    /// Overall engagement score (0.0–1.0).
    pub engagement_score: f64,
}

/// Derived engagement metrics (full detail).
#[derive(Debug, Clone)]
pub struct EngagementMetrics {
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration rate (new areas per minute).
    pub exploration_rate: f64,
    /// Challenge appetite (challenge-seeking / total actions).
    pub challenge_appetite: f64,
    /// Persistence (retry rate).
    pub persistence: f64,
    /// Deliberation rate (pauses per action).
    pub deliberation: f64,
    /// Composite engagement score (0.0–1.0).
    pub composite: f64,
}

/// Compute engagement metrics from a behavior snapshot.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "action_count is realistically small; fits in f64 mantissa"
)]
pub fn compute_engagement(snap: &EngagementSnapshot) -> EngagementMetrics {
    let minutes = (snap.session_duration_s / SECONDS_PER_MINUTE).max(MIN_SESSION_MINUTES);

    let apm = snap.action_count as f64 / minutes;
    let exploration_rate = f64::from(snap.exploration_breadth) / minutes;
    let challenge_appetite = if snap.action_count > 0 {
        f64::from(snap.challenge_seeking) / snap.action_count as f64
    } else {
        0.0
    };
    let persistence = if snap.action_count > 0 {
        f64::from(snap.retry_count) / snap.action_count as f64
    } else {
        0.0
    };
    let deliberation = if snap.action_count > 0 {
        f64::from(snap.deliberate_pauses) / snap.action_count as f64
    } else {
        0.0
    };

    let components = [
        (apm / ENGAGEMENT_APM_CEILING).min(1.0),
        (exploration_rate / ENGAGEMENT_EXPLORATION_CEILING).min(1.0),
        challenge_appetite.min(1.0),
        persistence.min(1.0),
        deliberation.min(1.0),
    ];
    let raw: f64 = components.iter().map(|c| c * ENGAGEMENT_WEIGHT).sum();

    EngagementMetrics {
        actions_per_minute: apm,
        exploration_rate,
        challenge_appetite,
        persistence,
        deliberation,
        composite: raw.clamp(0.0, 1.0),
    }
}

/// Compute simplified engagement result from a snapshot (bridge-compatible).
#[must_use]
pub fn engagement_result(snap: &EngagementSnapshot) -> EngagementResult {
    let m = compute_engagement(snap);
    EngagementResult {
        actions_per_minute: m.actions_per_minute,
        exploration_ratio: m.exploration_rate,
        engagement_score: m.composite,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_player_scores_high() {
        let snap = EngagementSnapshot {
            session_duration_s: 300.0,
            action_count: 200,
            exploration_breadth: 15,
            challenge_seeking: 10,
            retry_count: 20,
            deliberate_pauses: 15,
        };
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite > 0.2);
        assert!(metrics.actions_per_minute > 30.0);
    }

    #[test]
    fn idle_player_scores_low() {
        let snap = EngagementSnapshot {
            session_duration_s: 300.0,
            action_count: 2,
            exploration_breadth: 1,
            challenge_seeking: 0,
            retry_count: 0,
            deliberate_pauses: 0,
        };
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite < 0.1);
    }

    #[test]
    fn zero_duration_doesnt_panic() {
        let snap = EngagementSnapshot::default();
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite.is_finite());
    }

    #[test]
    fn engagement_result_matches_full() {
        let snap = EngagementSnapshot {
            session_duration_s: 120.0,
            action_count: 50,
            exploration_breadth: 5,
            challenge_seeking: 3,
            retry_count: 2,
            deliberate_pauses: 4,
        };
        let full = compute_engagement(&snap);
        let simple = engagement_result(&snap);
        assert!((full.composite - simple.engagement_score).abs() < f64::EPSILON);
    }
}
