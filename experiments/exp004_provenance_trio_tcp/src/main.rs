// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp004: Provenance trio TCP round-trip.
//!
//! Spawns rhizoCrypt from `plasmidBin/`, connects via TCP JSON-RPC, and
//! exercises the full DAG session lifecycle: create, append, frontier, merkle.
//! Honestly skips if binaries are not available.

fn allocate_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .unwrap_or(19404)
}

fn main() {
    use esoteric_webb::experiment::{check_bool, check_skip, exit};
    use esoteric_webb::ipc::client::PrimalClient;
    use esoteric_webb::ipc::launcher::{PrimalLauncher, discover_binary};

    println!("exp004: provenance trio TCP");

    // Check binary availability
    if discover_binary("rhizocrypt").is_err() {
        check_skip("rhizocrypt binary not in plasmidBin");
        exit("exp004_provenance_trio_tcp");
    }

    let mut launcher = PrimalLauncher::new();
    let port = allocate_port();
    let sp = match launcher.spawn("rhizocrypt", port, "dag") {
        Ok(sp) => sp,
        Err(e) => {
            check_skip(&format!("could not spawn rhizocrypt: {e}"));
            exit("exp004_provenance_trio_tcp");
        }
    };

    let addr = sp.address.clone();
    let mut client = match PrimalClient::connect_tcp(&addr, "rhizocrypt") {
        Ok(c) => c,
        Err(e) => {
            check_skip(&format!("TCP connect failed: {e}"));
            exit("exp004_provenance_trio_tcp");
        }
    };

    // Health
    let healthy = client.health_liveness().unwrap_or(false);
    check_bool("rhizocrypt healthy over TCP", healthy);

    // Session create
    let resp = client.call(
        "dag.session.create",
        serde_json::json!({"name": "exp004-session"}),
    );
    let session_ok = resp.as_ref().is_ok_and(|r| r.error.is_none());
    check_bool("dag.session.create succeeds", session_ok);

    let session_id = resp
        .ok()
        .and_then(|r| r.result)
        .and_then(|r| {
            r.get("session_id")
                .or_else(|| r.get("id"))
                .and_then(|v| v.as_str().map(str::to_owned))
        })
        .unwrap_or_default();
    check_bool("session_id is non-empty", !session_id.is_empty());

    // Event append
    let append = client.call(
        "dag.event.append",
        serde_json::json!({
            "session_id": session_id,
            "data": {"type": "player_action", "action": "examine"}
        }),
    );
    check_bool(
        "dag.event.append succeeds",
        append.is_ok_and(|r| r.error.is_none()),
    );

    // Frontier
    let frontier = client.call(
        "dag.frontier.get",
        serde_json::json!({"session_id": session_id}),
    );
    check_bool(
        "dag.frontier.get returns result",
        frontier.is_ok_and(|r| r.result.is_some()),
    );

    // Merkle root
    let merkle = client.call(
        "dag.merkle.root",
        serde_json::json!({"session_id": session_id}),
    );
    check_bool(
        "dag.merkle.root returns result",
        merkle.is_ok_and(|r| r.error.is_none()),
    );

    // Launcher kills rhizocrypt on drop
    exit("exp004_provenance_trio_tcp");
}
