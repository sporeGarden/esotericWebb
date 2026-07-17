// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game science — locally absorbed algorithms for game design.
//!
//! These are deterministic, pure-math functions with zero IPC or primal
//! dependencies. They implement well-known game design science (flow theory,
//! engagement metrics, dynamic difficulty adjustment) that Webb uses for
//! real-time player experience evaluation.
//!
//! These algorithms are Webb's own implementations. When a dedicated
//! game-science primal emerges, Webb can swap local calls for IPC —
//! the gap drives primal evolution (GAP-021).

pub mod dda;
pub mod engagement;
pub mod flow;
pub mod voice;

/// Default flow channel half-width (Csikszentmihalyi model).
pub const FLOW_CHANNEL_WIDTH: f64 = 0.15;

/// Default DDA target success rate (Hunicke 2005 sweet spot).
pub const DDA_TARGET_SUCCESS_RATE: f64 = 0.7;
