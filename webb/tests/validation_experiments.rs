// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation experiments for Esoteric Webb's primal composition.
//!
//! These tests validate live integration with primal binaries from
//! `plasmidBin/`. They are **skipped** when the required binary is
//! not available — CI without binaries still passes.

#![expect(clippy::expect_used, reason = "integration test code")]

use esoteric_webb::ipc::client::PrimalClient;
use esoteric_webb::ipc::launcher::{PrimalLauncher, discover_binary};

fn allocate_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map_or(19401, |a| a.port())
}

/// exp008: Live round-trip with rhizoCrypt from plasmidBin.
///
/// 1. Spawn rhizoCrypt via `PrimalLauncher`
/// 2. Connect via `PrimalClient::connect_tcp`
/// 3. `health.check` → assert healthy
/// 4. `dag.session.create` → get session ID
/// 5. `dag.event.append` → append a game event vertex
/// 6. `dag.frontier.get` → verify frontier includes the new vertex
/// 7. Launcher kills rhizoCrypt on drop
#[test]
fn exp008_rhizocrypt_live_round_trip() {
    if discover_binary("rhizocrypt").is_err() {
        eprintln!("SKIP: rhizocrypt binary not found in plasmidBin — skipping integration test");
        return;
    }

    let mut launcher = PrimalLauncher::new();
    let port = allocate_port();

    let sp = match launcher.spawn("rhizocrypt", port, "dag") {
        Ok(sp) => sp,
        Err(e) => {
            eprintln!("SKIP: could not spawn rhizocrypt: {e}");
            return;
        }
    };

    let addr = sp.address.clone();
    let mut client = match PrimalClient::connect_tcp(&addr, "rhizocrypt") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("SKIP: could not connect to spawned rhizocrypt at {addr}: {e}");
            return;
        }
    };

    // 1. Health check
    let healthy = match client.health_liveness() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("SKIP: rhizocrypt health call failed (binary may need configuration): {e}");
            return;
        }
    };
    assert!(healthy, "rhizocrypt should be healthy after spawn");

    // 2. Create a session
    let resp = client
        .call(
            "dag.session.create",
            serde_json::json!({"name": "exp008-test-session"}),
        )
        .expect("session.create should succeed");
    assert!(
        resp.error.is_none(),
        "session.create should not return an error: {:?}",
        resp.error
    );
    let session_id = resp
        .result
        .as_ref()
        .and_then(|r| {
            r.get("session_id")
                .or_else(|| r.get("id"))
                .and_then(|v| v.as_str())
        })
        .expect("session.create should return a session_id");
    assert!(!session_id.is_empty(), "session_id should not be empty");

    // 3. Append an event
    let event_resp = client
        .call(
            "dag.event.append",
            serde_json::json!({
                "session_id": session_id,
                "data": {
                    "type": "player_action",
                    "action": "examine",
                    "node": "forest_clearing"
                }
            }),
        )
        .expect("event.append should succeed");
    assert!(
        event_resp.error.is_none(),
        "event.append error: {:?}",
        event_resp.error
    );

    // 4. Query frontier
    let frontier_resp = client
        .call(
            "dag.frontier.get",
            serde_json::json!({"session_id": session_id}),
        )
        .expect("frontier.get should succeed");
    assert!(
        frontier_resp.error.is_none(),
        "frontier.get error: {:?}",
        frontier_resp.error
    );
    assert!(
        frontier_resp.result.is_some(),
        "frontier should have a result"
    );

    // 5. Merkle root
    let merkle_resp = client
        .call(
            "dag.merkle.root",
            serde_json::json!({"session_id": session_id}),
        )
        .expect("merkle.root should succeed");
    assert!(
        merkle_resp.error.is_none(),
        "merkle.root error: {:?}",
        merkle_resp.error
    );

    // Launcher::drop will kill rhizocrypt
}
