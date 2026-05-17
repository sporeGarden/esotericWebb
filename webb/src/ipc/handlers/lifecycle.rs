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

/// `health.version` — detailed version, build target, and signal tier info.
pub(super) fn handle_health_version() -> Value {
    serde_json::json!({
        "primal": "esotericwebb",
        "version": env!("CARGO_PKG_VERSION"),
        "build_target": option_env!("TARGET").unwrap_or("unknown"),
        "edition": "2024",
        "signal_tiers": ["nest", "meta"],
    })
}

/// `health.drain` — acknowledge graceful shutdown intent.
pub(super) fn handle_health_drain() -> Value {
    tracing::info!("health.drain received — shutdown acknowledged");
    serde_json::json!({
        "acknowledged": true,
        "primal": "esotericwebb",
    })
}

/// `primal.announce` — accept inbound registration from another primal.
pub(super) fn handle_primal_announce(params: Option<&Value>) -> Value {
    let primal = params
        .and_then(|p| p.get("primal"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let version = params
        .and_then(|p| p.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    tracing::info!(primal, version, "primal.announce received");
    serde_json::json!({
        "accepted": true,
        "primal": primal,
    })
}

/// `primal.info` — return Webb's niche metadata (identity, capabilities, signal tiers).
pub(super) fn handle_primal_info() -> Value {
    let registry_toml = include_str!("../../../capability_registry.toml");
    let method_count = registry_toml.matches("method = ").count();
    serde_json::json!({
        "primal": "esotericwebb",
        "version": env!("CARGO_PKG_VERSION"),
        "domain": "narrative",
        "capabilities": method_count,
        "signal_tiers": ["nest", "meta"],
        "guidestone_level": 0,
    })
}

/// `capabilities.list` — canonical Wave 20 envelope.
///
/// Returns `{ capabilities, count, primal }` per the ecosystem schema standard
/// (`primalSpring/ecoPrimal/src/validation/scenarios/s_schema_standard.rs`).
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

    let count = capabilities.len();
    serde_json::json!({
        "capabilities": capabilities,
        "count": count,
        "primal": "esotericwebb",
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
    fn capabilities_list_canonical_envelope() {
        let v = handle_capabilities_list();
        let caps = v["capabilities"].as_array().unwrap();
        assert!(!caps.is_empty());
        assert_eq!(v["count"].as_u64().unwrap(), caps.len() as u64);
        assert_eq!(v["primal"], "esotericwebb");
    }

    #[test]
    fn health_version_returns_build_info() {
        let v = handle_health_version();
        assert_eq!(v["primal"], "esotericwebb");
        assert!(v.get("version").is_some());
        assert!(v.get("build_target").is_some());
        let tiers = v["signal_tiers"].as_array().unwrap();
        assert!(tiers.contains(&Value::from("nest")));
        assert!(tiers.contains(&Value::from("meta")));
    }

    #[test]
    fn health_drain_acknowledges() {
        let v = handle_health_drain();
        assert_eq!(v["acknowledged"], true);
        assert_eq!(v["primal"], "esotericwebb");
    }

    #[test]
    fn primal_announce_accepts_inbound() {
        let params = serde_json::json!({
            "primal": "squirrel",
            "version": "1.2.3",
            "capabilities": ["ai"],
        });
        let v = handle_primal_announce(Some(&params));
        assert_eq!(v["accepted"], true);
        assert_eq!(v["primal"], "squirrel");
    }

    #[test]
    fn primal_announce_handles_missing_params() {
        let v = handle_primal_announce(None);
        assert_eq!(v["accepted"], true);
        assert_eq!(v["primal"], "unknown");
    }

    #[test]
    fn primal_info_returns_metadata() {
        let v = handle_primal_info();
        assert_eq!(v["primal"], "esotericwebb");
        assert_eq!(v["domain"], "narrative");
        assert!(v["capabilities"].as_u64().unwrap() > 0);
        let tiers = v["signal_tiers"].as_array().unwrap();
        assert!(tiers.contains(&Value::from("nest")));
    }
}
