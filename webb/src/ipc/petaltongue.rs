// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the visualization primal (`visualization.*`, `ui.*`, `interaction.*`).
//!
//! Consumes scene rendering and player input polling capabilities.
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency. Design derived from petalTongue's visualization architecture.
//!
//! When a live [`PrimalClient`] connection exists, methods send real
//! JSON-RPC calls. Otherwise they degrade silently.

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::envelope::IpcError;

/// Visualization primal method names.
pub const METHOD_RENDER_SCENE: &str = "visualization.render.scene";
/// Generic UI render.
pub const METHOD_UI_RENDER: &str = "ui.render";
/// Subscribe to input events.
pub const METHOD_INTERACTION_SUBSCRIBE: &str = "interaction.subscribe";
/// Poll for input events.
pub const METHOD_INTERACTION_POLL: &str = "interaction.poll";

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

/// Client for visualization primal capabilities (resolved from `plasmidBin/`).
#[derive(Debug)]
pub struct PetalTongueClient {
    available: bool,
}

impl PetalTongueClient {
    /// Create a new client.
    #[must_use]
    pub const fn new(available: bool) -> Self {
        Self { available }
    }

    /// Whether the visualization primal was discovered and is healthy.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Push a scene payload for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn push_scene(
        &self,
        scene: &serde_json::Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<(), IpcError> {
        if let Some(c) = client {
            let _ = c.call(METHOD_RENDER_SCENE, scene.clone())?;
            return Ok(());
        }

        if self.available {
            tracing::debug!("pushing scene to visualization primal");
        } else {
            tracing::debug!("visualization primal unavailable — scene not rendered: {scene}");
        }
        Ok(())
    }

    /// Poll for player input events.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn poll_input(
        &self,
        client: Option<&mut PrimalClient>,
    ) -> Result<Vec<InputEvent>, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_INTERACTION_POLL, serde_json::Value::Null)?;
            if let Some(result) = resp.result {
                if let Ok(events) = serde_json::from_value::<Vec<InputEvent>>(result) {
                    return Ok(events);
                }
            }
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_push_succeeds_silently() {
        let client = PetalTongueClient::new(false);
        let result = client.push_scene(&serde_json::json!({"type": "dialogue"}), None);
        assert!(result.is_ok());
    }

    #[test]
    fn unavailable_poll_returns_empty() {
        let client = PetalTongueClient::new(false);
        let events = client.poll_input(None);
        assert!(events.is_ok());
        assert!(events.unwrap_or_default().is_empty());
    }
}
