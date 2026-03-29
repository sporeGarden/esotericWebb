// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health, readiness, identity, and capability handlers.

use serde_json::Value;

use super::SharedSession;

/// `health.liveness` / `webb.health` / `health.check` — always alive.
pub(super) fn handle_health() -> Value {
    serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// `health.readiness` — ready iff a session is loaded.
pub(super) fn handle_readiness(session: &SharedSession) -> Value {
    let ready = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some();
    serde_json::json!({
        "ready": ready,
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// `identity.get` — sourDough-required self-identification.
pub(super) fn handle_identity() -> Value {
    serde_json::json!({
        "primal": "esotericwebb",
        "version": env!("CARGO_PKG_VERSION"),
        "domain": "narrative",
    })
}

/// `capabilities.list` — parsed from the embedded capability registry.
pub(super) fn handle_capabilities_list() -> Value {
    let registry_toml = include_str!("../../../capability_registry.toml");
    let table: toml::Value =
        toml::from_str(registry_toml).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    let capabilities: Vec<Value> = table
        .get("capabilities")
        .and_then(toml::Value::as_array)
        .map_or_else(Vec::new, |caps| {
            caps.iter()
                .filter_map(|c| {
                    let method = c.get("method")?.as_str()?;
                    let desc = c.get("description")?.as_str()?;
                    Some(serde_json::json!({
                        "method": method,
                        "description": desc,
                    }))
                })
                .collect()
        });

    serde_json::json!({
        "primal": "esotericwebb",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": capabilities,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn empty_session() -> SharedSession {
        Arc::new(Mutex::new(None))
    }

    #[test]
    fn health_returns_status() {
        let v = handle_health();
        assert_eq!(v["status"], "healthy");
    }

    #[test]
    fn readiness_false_without_session() {
        let v = handle_readiness(&empty_session());
        assert_eq!(v["ready"], false);
    }

    #[test]
    fn identity_returns_primal_info() {
        let v = handle_identity();
        assert_eq!(v["primal"], "esotericwebb");
        assert_eq!(v["domain"], "narrative");
        assert!(v.get("version").is_some());
    }

    #[test]
    fn capabilities_list_is_nonempty() {
        let v = handle_capabilities_list();
        let caps = v["capabilities"].as_array().unwrap();
        assert!(!caps.is_empty());
    }
}
