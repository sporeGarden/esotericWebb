// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game science — locally absorbed algorithms for game design.
//!
//! These are deterministic, pure-math functions with zero IPC or primal
//! dependencies. They implement well-known game design science (flow theory,
//! engagement metrics, dynamic difficulty adjustment) that Webb uses for
//! real-time player experience evaluation.
//!
//! The patterns originate in ludoSpring's research but live here as Webb's
//! own implementations. When a dedicated game-science primal emerges,
//! Webb can swap these local calls for IPC — the gap drives primal evolution.

pub mod dda;
pub mod engagement;
pub mod flow;

/// Default flow channel half-width (Csikszentmihalyi model).
pub const FLOW_CHANNEL_WIDTH: f64 = 0.15;

/// Default DDA target success rate (Hunicke 2005 sweet spot).
pub const DDA_TARGET_SUCCESS_RATE: f64 = 0.7;
