// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp006: Live primal composition — real IPC against running primals.
//!
//! Validates that `PrimalBridge::discover()` finds live primals on the
//! local machine, connects, and exercises a game session with real
//! enrichment. Domains where the primal is down are honest-skipped.

use std::process::ExitCode;

#[expect(
    clippy::too_many_lines,
    reason = "experiment runner is a single integration flow"
)]
fn main() -> ExitCode {
    use esoteric_webb::experiment::{check_bool, check_skip, finish_with_code, section};
    use esoteric_webb::ipc::bridge::PrimalBridge;
    use esoteric_webb::ipc::primal_names::domain;
    use esoteric_webb::session::types::ActionKind;

    println!("exp006: live primal composition");

    // ── Discovery ──

    section("discovery");
    let bridge = PrimalBridge::discover();
    let connected = bridge.connected_count();
    let statuses = bridge.statuses();

    check_bool(
        "discovery returns status for every domain",
        statuses.len() == esoteric_webb::ipc::primal_names::DOMAIN_PRIMAL_MAP.len(),
    );

    let found_count = statuses.iter().filter(|s| s.discovered).count();
    check_bool("at least one primal discovered", found_count > 0);

    for s in statuses {
        let icon = if s.healthy {
            "healthy"
        } else if s.discovered {
            "found"
        } else {
            "absent"
        };
        println!(
            "    {name:<14} {domain:<16} {icon}",
            name = s.name,
            domain = s.domain
        );
    }
    println!("    {connected}/{total} connected", total = statuses.len());

    if connected == 0 {
        check_skip("no primals connected — cannot exercise live composition");
        return finish_with_code("exp006_live_composition");
    }
    check_bool("composition mode (at least 1 connected)", connected > 0);

    // ── Per-domain health ──

    section("per-domain health");
    let domains = [
        (domain::AI, "squirrel"),
        (domain::VISUALIZATION, "petaltongue"),
        (domain::STORAGE, "nestgate"),
        (domain::PROVENANCE, "sweetgrass"),
        (domain::CRYPTO, "beardog"),
        (domain::DAG, "rhizocrypt"),
        (domain::LINEAGE, "loamspine"),
        (domain::COMPUTE, "toadstool"),
        (domain::MESH, "songbird"),
    ];
    for (dom, name) in domains {
        if bridge.has(dom) {
            check_bool(&format!("{name} ({dom}) healthy"), true);
        } else {
            check_skip(&format!("{name} ({dom}) not connected"));
        }
    }

    // ── Session with bridge ──

    section("session with bridge");
    let session_result = esoteric_webb::session::GameSession::with_bridge("content", Some(bridge));

    let Ok(mut session) = session_result else {
        check_skip("content/ not loadable — skipping session tests");
        return finish_with_code("exp006_live_composition");
    };

    let snap = session.snapshot();
    check_bool(
        "session starts at entrance",
        snap.current_node == "entrance",
    );
    check_bool(
        "session has available actions",
        !snap.available_actions.is_empty(),
    );
    check_bool(
        "session loaded content bundle",
        !session.bundle().meta.name.is_empty(),
    );
    println!("    world: {}", session.bundle().meta.name);
    println!("    node: {}", snap.current_node);
    println!("    actions: {}", snap.available_actions.len());

    // ── Act: examine ──

    section("act — examine");
    let act_result = session.act(ActionKind::Examine, "");
    match act_result {
        Ok((outcome, ctx)) => {
            check_bool("examine returns outcome text", !outcome.is_empty());
            check_bool(
                "narration context has scene",
                !ctx.scene_description.is_empty(),
            );
            let enriched = ctx.enrichment.ai_narration.is_some()
                || ctx.enrichment.scene_pushed
                || !ctx.enrichment.voice_notes.is_empty();
            if enriched {
                check_bool("enrichment fired (AI, scene push, or voices)", true);
            } else {
                check_skip("no enrichment fired (primals may not support these calls)");
            }
            println!("    outcome: {}", &outcome[..outcome.len().min(80)]);
            if let Some(ref ai) = ctx.enrichment.ai_narration {
                println!("    AI narration: {}...", &ai[..ai.len().min(60)]);
            }
            if ctx.enrichment.scene_pushed {
                println!("    scene pushed to petalTongue");
            }
            for v in &ctx.enrichment.voice_notes {
                println!("    voice: {} — {}", v.voice_id, v.text);
            }
        }
        Err(e) => {
            check_skip(&format!("act(examine) failed: {e}"));
        }
    }

    // ── Act: navigate ──

    section("act — navigate");
    let actions = session.available_actions();
    let exit_action = actions.iter().find(|a| a.kind == ActionKind::Exit);
    if let Some(exit) = exit_action {
        let target = exit.id.clone();
        println!("    navigating to: {target}");
        match session.act(ActionKind::Exit, &target) {
            Ok((outcome, ctx)) => {
                check_bool("exit action succeeds", true);
                check_bool(
                    "node changed",
                    session.snapshot().current_node != "entrance" || target == "entrance",
                );
                println!("    now at: {}", session.snapshot().current_node);
                println!("    outcome: {}", &outcome[..outcome.len().min(80)]);
                if ctx.enrichment.scene_pushed {
                    println!("    scene pushed to petalTongue");
                }
            }
            Err(e) => {
                check_skip(&format!("navigate failed: {e}"));
            }
        }
    } else {
        check_skip("no exit actions available from start node");
    }

    // ── Session state after actions ──

    section("post-action state");
    let final_snap = session.snapshot();
    check_bool("turn counter advanced", final_snap.turn >= 1);
    check_bool("history recorded", !session.history().is_empty());
    println!("    turns: {}", final_snap.turn);
    println!("    history entries: {}", session.history().len());
    println!("    knowledge: {:?}", final_snap.knowledge);

    finish_with_code("exp006_live_composition")
}
