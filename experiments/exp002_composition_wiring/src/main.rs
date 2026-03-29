// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp002: Primal composition wiring and graceful degradation.
//!
//! Validates that `PrimalBridge` in standalone mode degrades correctly
//! for every domain, and that the discovery registry handles absent
//! sockets / addresses without panicking.

fn main() {
    use esoteric_webb::experiment::{check_bool, exit};
    use esoteric_webb::ipc::bridge::PrimalBridge;
    use esoteric_webb::ipc::discovery::PrimalRegistry;

    println!("exp002: composition wiring");

    // Standalone bridge — all degradation paths
    let mut bridge = PrimalBridge::standalone();
    check_bool(
        "standalone has 0 connections",
        bridge.connected_count() == 0,
    );
    check_bool(
        "standalone has all domain status entries",
        bridge.statuses().len() == esoteric_webb::ipc::primal_names::DOMAIN_PRIMAL_MAP.len(),
    );

    let narrate =
        bridge
            .ai_narrate("test")
            .unwrap_or_else(|_| esoteric_webb::ipc::squirrel::ChatResponse {
                text: String::new(),
                model: String::new(),
                tokens: 0,
            });
    check_bool(
        "AI narration degrades",
        narrate.text.contains("unavailable"),
    );

    let summary = bridge.ai_summarize("context").unwrap_or_default();
    check_bool("AI summarize degrades", summary.contains("unavailable"));

    check_bool(
        "render_scene is noop",
        bridge.render_scene(&serde_json::json!({})).is_ok(),
    );
    check_bool(
        "compute_submit returns None",
        bridge
            .compute_submit(&serde_json::json!({}))
            .unwrap_or(Some(String::new()))
            .is_none(),
    );
    check_bool(
        "store returns false",
        !bridge.store("k", &serde_json::json!("v")).unwrap_or(true),
    );
    check_bool(
        "retrieve returns None",
        bridge
            .retrieve("k")
            .unwrap_or(Some(serde_json::json!(null)))
            .is_none(),
    );

    // DAG domain degradation
    check_bool(
        "dag_session_create returns None",
        bridge
            .dag_session_create(&serde_json::json!({}))
            .unwrap_or(Some(String::new()))
            .is_none(),
    );
    check_bool(
        "dag_event_append returns None",
        bridge
            .dag_event_append(&serde_json::json!({}))
            .unwrap_or(Some(String::new()))
            .is_none(),
    );

    // Discovery handles empty environment
    let registry = PrimalRegistry::discover();
    let domain_count = registry.by_domain.len();
    check_bool(
        &format!("discovery returns well-formed registry ({domain_count} domains)"),
        registry.by_domain.keys().all(|k| !k.is_empty()),
    );
    check_bool(
        "find_by_capability on unknown returns None",
        registry.find_by_capability("nonexistent.method").is_none(),
    );

    exit("exp002_composition_wiring");
}
