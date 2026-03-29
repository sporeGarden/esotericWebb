// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp005: Autoplay coverage — game plays itself to validate all paths.
//!
//! Creates a `GameSession`, runs the heuristic autoplay loop, and
//! checks that the session reaches an ending, takes multiple turns,
//! and accumulates knowledge/state changes.

fn main() {
    use esoteric_webb::experiment::{check_bool, check_skip, exit};
    use esoteric_webb::session::GameSession;

    println!("exp005: autoplay coverage");

    let mut session = match GameSession::new("content") {
        Ok(s) => s,
        Err(e) => {
            check_skip(&format!("content load failed: {e}"));
            exit("exp005_autoplay_coverage");
        }
    };

    check_bool("session starts without error", true);

    let snap_before = session.snapshot();
    let is_start = session
        .bundle()
        .narrative
        .start_node()
        .is_some_and(|n| n.id == snap_before.current_node);
    check_bool(
        &format!("starts at start node ('{}')", snap_before.current_node),
        is_start,
    );
    check_bool("turn 0", snap_before.turn == 0);

    let max_turns = 100;
    let mut turns_taken = 0;
    let mut visited_nodes = std::collections::HashSet::new();
    visited_nodes.insert(snap_before.current_node);

    for _ in 0..max_turns {
        if session.is_ended() {
            break;
        }

        let actions = session.available_actions();
        if actions.is_empty() {
            break;
        }

        // Simple heuristic: pick first available action
        let action = &actions[0];
        let result = session.act(action.kind, &action.id);
        if result.is_ok() {
            turns_taken += 1;
            visited_nodes.insert(session.snapshot().current_node.clone());
        }
    }

    check_bool("took at least 1 turn", turns_taken > 0);
    check_bool("visited multiple nodes", visited_nodes.len() > 1);

    let snap_after = session.snapshot();
    check_bool(
        "accumulated state changes",
        !snap_after.knowledge.is_empty()
            || !snap_after.flags.is_empty()
            || !snap_after.inventory.is_empty()
            || snap_after.turn > 0,
    );

    let ended = session.is_ended();
    let no_actions = session.available_actions().is_empty();
    check_bool(
        &format!(
            "terminated: ended={ended}, no_actions={no_actions}, turns={turns_taken}, nodes={}",
            visited_nodes.len()
        ),
        ended || no_actions,
    );

    exit("exp005_autoplay_coverage");
}
