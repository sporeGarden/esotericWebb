// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp001: Narrative graph reachability and structure validation.
//!
//! Validates that the bundled content has a well-formed narrative graph:
//! all nodes reachable from start, at least one ending reachable, no
//! dangling edge targets, and BFS depth computation is consistent.

fn main() {
    use esoteric_webb::content::ContentBundle;
    use esoteric_webb::experiment::{check_bool, check_skip, exit};

    println!("exp001: narrative reachability");

    let bundle = match ContentBundle::load("content") {
        Ok(b) => b,
        Err(e) => {
            check_skip(&format!("content load failed: {e}"));
            exit("exp001_narrative_reachability");
        }
    };

    let issues = bundle.validate();
    check_bool("content validates cleanly", issues.is_empty());

    let graph = &bundle.narrative;
    check_bool("has start node", graph.start_node().is_some());
    check_bool("has at least one ending", !graph.endings().is_empty());
    check_bool("node count > 0", !graph.nodes.is_empty());

    let depths = graph.bfs_depths();
    let start_id = graph
        .start_node()
        .map_or("(none)", |n| n.id.as_str());
    check_bool(
        &format!("BFS reaches start node '{start_id}'"),
        depths.contains_key(start_id),
    );
    check_bool(
        "BFS depth of start is 0",
        depths.get(start_id).copied() == Some(0),
    );

    let max_depth = depths.values().copied().max().unwrap_or(0);
    check_bool("max depth > 0 (non-trivial graph)", max_depth > 0);

    let reachable = depths.len();
    let total = graph.nodes.len();
    check_bool(
        &format!("all nodes reachable ({reachable}/{total})"),
        reachable == total,
    );

    for ending in graph.endings() {
        check_bool(
            &format!("ending '{}' reachable from start", ending.id),
            depths.contains_key(ending.id.as_str()),
        );
    }

    exit("exp001_narrative_reachability");
}
