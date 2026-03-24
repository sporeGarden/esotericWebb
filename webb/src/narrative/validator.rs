// SPDX-License-Identifier: AGPL-3.0-or-later
//! Narrative graph validation.
//!
//! Validates structural integrity: no orphans, all edge targets resolve,
//! exactly one start node, at least one ending, all endings reachable.

use std::collections::{HashSet, VecDeque};

use super::NarrativeGraph;

/// Validate the narrative graph and return a list of issues.
pub fn validate(graph: &NarrativeGraph) -> Vec<String> {
    let mut issues = Vec::new();

    let start_count = graph.nodes.values().filter(|n| n.is_start).count();
    if start_count == 0 {
        issues.push("no start node defined (set is_start: true on exactly one node)".to_owned());
    } else if start_count > 1 {
        issues.push(format!(
            "{start_count} start nodes defined — exactly one required"
        ));
    }

    let endings: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.is_ending)
        .map(|n| n.id.as_str())
        .collect();
    if endings.is_empty() {
        issues
            .push("no ending nodes defined (set is_ending: true on at least one node)".to_owned());
    }

    for node in graph.nodes.values() {
        for edge in &node.exits {
            if !graph.nodes.contains_key(&edge.target) {
                issues.push(format!(
                    "node '{}': edge target '{}' does not exist",
                    node.id, edge.target
                ));
            }
        }
    }

    let referenced: HashSet<&str> = graph
        .nodes
        .values()
        .flat_map(|n| n.exits.iter().map(|e| e.target.as_str()))
        .collect();
    for node in graph.nodes.values() {
        if !node.is_start && !referenced.contains(node.id.as_str()) {
            issues.push(format!(
                "node '{}' is orphaned — no edges point to it and it is not the start",
                node.id
            ));
        }
    }

    if let Some(start) = graph.start_node() {
        let reachable = reachable_from(graph, &start.id);
        for ending_id in &endings {
            if !reachable.contains(*ending_id) {
                issues.push(format!(
                    "ending '{ending_id}' is not reachable from start node",
                ));
            }
        }
    }

    issues
}

/// BFS to find all nodes reachable from a given start.
fn reachable_from(graph: &NarrativeGraph, start_id: &str) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start_id.to_owned());
    visited.insert(start_id.to_owned());

    while let Some(current) = queue.pop_front() {
        if let Some(node) = graph.nodes.get(&current) {
            for edge in &node.exits {
                if visited.insert(edge.target.clone()) {
                    queue.push_back(edge.target.clone());
                }
            }
        }
    }

    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narrative::{NarrativeEdge, NarrativeNode, SceneType, TransitionType};
    use std::collections::HashMap;

    fn minimal_valid_graph() -> NarrativeGraph {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_owned(),
            NarrativeNode {
                id: "start".to_owned(),
                scene_type: SceneType::Exploration,
                content_ref: "scenes/start.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![NarrativeEdge {
                    target: "end".to_owned(),
                    conditions: vec![],
                    priority: 0,
                    transition_type: TransitionType::SamePlane,
                    label: None,
                }],
                is_start: true,
                is_ending: false,
                label: None,
            },
        );
        nodes.insert(
            "end".to_owned(),
            NarrativeNode {
                id: "end".to_owned(),
                scene_type: SceneType::Ending,
                content_ref: "scenes/end.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![],
                is_start: false,
                is_ending: true,
                label: None,
            },
        );
        NarrativeGraph { nodes }
    }

    #[test]
    fn valid_graph_has_no_issues() {
        let g = minimal_valid_graph();
        let issues = validate(&g);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn no_start_node() {
        let mut g = minimal_valid_graph();
        if let Some(n) = g.nodes.get_mut("start") {
            n.is_start = false;
        }
        let issues = validate(&g);
        assert!(issues.iter().any(|i| i.contains("no start node")));
    }

    #[test]
    fn no_ending_node() {
        let mut g = minimal_valid_graph();
        if let Some(n) = g.nodes.get_mut("end") {
            n.is_ending = false;
        }
        let issues = validate(&g);
        assert!(issues.iter().any(|i| i.contains("no ending nodes")));
    }

    #[test]
    fn dangling_edge_target() {
        let mut g = minimal_valid_graph();
        if let Some(n) = g.nodes.get_mut("start") {
            n.exits.push(NarrativeEdge {
                target: "nonexistent".to_owned(),
                conditions: vec![],
                priority: 0,
                transition_type: TransitionType::SamePlane,
                label: None,
            });
        }
        let issues = validate(&g);
        assert!(issues.iter().any(|i| i.contains("nonexistent")));
    }

    #[test]
    fn unreachable_ending() {
        let mut g = minimal_valid_graph();
        g.nodes.insert(
            "unreachable_end".to_owned(),
            NarrativeNode {
                id: "unreachable_end".to_owned(),
                scene_type: SceneType::Ending,
                content_ref: "scenes/x.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![],
                is_start: false,
                is_ending: true,
                label: None,
            },
        );
        let issues = validate(&g);
        assert!(issues.iter().any(|i| i.contains("unreachable_end")));
    }
}
