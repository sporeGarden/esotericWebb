// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the visualization primal (`visualization.*`, `ui.*`, `interaction.*`).
//!
//! Consumes scene rendering and player input polling capabilities.
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency. Design derived from petalTongue's visualization architecture.
//! The `PrimalBridge` handles all IPC with graceful degradation.

use serde::{Deserialize, Serialize};

/// Visualization primal method names.
pub const METHOD_RENDER_SCENE: &str = "visualization.render.scene";
/// Generic UI render — accepts `{type, content}` and renders immediately.
pub const METHOD_UI_RENDER: &str = "ui.render";
/// Subscribe to input events.
pub const METHOD_INTERACTION_SUBSCRIBE: &str = "interaction.subscribe";
/// Poll for input events.
pub const METHOD_INTERACTION_POLL: &str = "interaction.poll";
/// Unsubscribe from input events.
pub const METHOD_INTERACTION_UNSUBSCRIBE: &str = "interaction.unsubscribe";

/// Player input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    /// Event type (e.g. "choice", "ability", "examine", "move").
    pub event_type: String,
    /// Event payload (choice index, ability ID, target, etc.).
    pub payload: serde_json::Value,
    /// Timestamp (milliseconds since session start).
    pub timestamp_ms: u64,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn input_event_serde_round_trip() {
        let event = InputEvent {
            event_type: "choice".to_owned(),
            payload: serde_json::json!({"index": 0}),
            timestamp_ms: 1234,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, "choice");
        assert_eq!(back.timestamp_ms, 1234);
    }
}
