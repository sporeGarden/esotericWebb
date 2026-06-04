// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph visualization — DOT, JSON, and session overlay rendering.
//!
//! Separated from the core graph structure so that visualization
//! concerns (colors, layout, overlay state) don't mix with graph
//! traversal and predicate logic.

use std::collections::HashSet;

use super::{NarrativeEdge, NarrativeGraph, NarrativeNode};

/// Overlay data for rendering a DAG with session state.
///
/// Three conceptual views on the same structure:
/// - **Narrative DAG**: the full authored graph (the `NarrativeGraph` itself)
/// - **Live DAG**: current position + available/gated edges (during play)
/// - **Played DAG**: visited nodes + edges taken (after a session)
#[derive(Debug, Clone, Default)]
pub struct DagOverlay {
    /// Nodes the player has visited.
    pub visited: HashSet<String>,
    /// Edges the player has traversed (source, target).
    pub edges_taken: HashSet<(String, String)>,
    /// Current node (if in a live session).
    pub current_node: Option<String>,
    /// Edges currently available from the current node.
    pub available_targets: HashSet<String>,
    /// Edges that exist but are gated (conditions not met).
    pub gated_targets: HashSet<(String, String)>,
}

impl DagOverlay {
    /// Build a played-DAG overlay from autoplay JSON history.
    ///
    /// Expects a JSON value with `"history"` (array of `{action, node_after}`)
    /// and `"final_node"`.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON structure is unexpected.
    pub fn from_history_json(
        json: &serde_json::Value,
        graph: &NarrativeGraph,
    ) -> crate::error::Result<Self> {
        let history = json
            .get("history")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| crate::error::WebbError::Other("missing 'history' array".into()))?;

        let start_id = graph
            .start_node()
            .map_or_else(String::new, |n| n.id.clone());

        let mut visited = HashSet::new();
        let mut edges_taken = HashSet::new();

        visited.insert(start_id.clone());
        let mut prev = start_id;

        for entry in history {
            let action = entry
                .get("action")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let node_after = entry
                .get("node_after")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");

            if action.starts_with("exit:") && !node_after.is_empty() {
                edges_taken.insert((prev.clone(), node_after.to_owned()));
            }
            if !node_after.is_empty() {
                visited.insert(node_after.to_owned());
                node_after.clone_into(&mut prev);
            }
        }

        let final_node = json
            .get("final_node")
            .and_then(serde_json::Value::as_str)
            .map(std::borrow::ToOwned::to_owned);

        Ok(Self {
            visited,
            edges_taken,
            current_node: final_node,
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        })
    }
}

impl NarrativeGraph {
    /// Generate DOT format for graph visualization.
    #[must_use]
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;

        let mut dot = String::from("digraph NarrativeGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for node in self.nodes.values() {
            let label = node.label.as_deref().unwrap_or(&node.id);
            let shape = if node.is_start {
                "doubleoctagon"
            } else if node.is_ending {
                "doublecircle"
            } else {
                "box"
            };
            let _ = writeln!(
                dot,
                "  \"{id}\" [label=\"{label}\" shape={shape}];",
                id = node.id
            );
        }
        dot.push('\n');

        for node in self.nodes.values() {
            for edge in &node.exits {
                let label = edge.label.as_deref().unwrap_or("");
                let _ = writeln!(
                    dot,
                    "  \"{src}\" -> \"{tgt}\" [label=\"{label}\"];",
                    src = node.id,
                    tgt = edge.target,
                );
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Classify an edge relative to BFS depth layers.
    pub(crate) const fn edge_kind(src_depth: usize, tgt_depth: usize) -> &'static str {
        match tgt_depth {
            d if d > src_depth => "forward",
            d if d < src_depth => "back",
            _ => "lateral",
        }
    }

    /// Export the graph as structured JSON with BFS depth layers and edge
    /// classification.  When an overlay is provided, each node and edge
    /// carries session state (visited, taken, current, available).
    #[must_use]
    pub fn to_graph_json(&self, overlay: Option<&DagOverlay>) -> serde_json::Value {
        let depths = self.bfs_depths();
        let max_depth = depths.values().copied().max().unwrap_or(0);

        let nodes: Vec<serde_json::Value> = self
            .nodes
            .values()
            .map(|n| {
                let depth = depths.get(&n.id).copied().unwrap_or(0);
                let status = overlay.map_or("neutral", |ov| {
                    if ov.current_node.as_deref() == Some(&*n.id) {
                        "current"
                    } else if ov.visited.contains(&n.id) {
                        "visited"
                    } else if ov.available_targets.contains(&n.id) {
                        "available"
                    } else {
                        "unexplored"
                    }
                });
                serde_json::json!({
                    "id": n.id,
                    "label": n.label.as_deref().unwrap_or(&n.id),
                    "scene_type": n.scene_type,
                    "is_start": n.is_start,
                    "is_ending": n.is_ending,
                    "depth": depth,
                    "status": status,
                })
            })
            .collect();

        let mut edges: Vec<serde_json::Value> = Vec::new();
        for n in self.nodes.values() {
            let src_depth = depths.get(&n.id).copied().unwrap_or(0);
            for e in &n.exits {
                let tgt_depth = depths.get(&e.target).copied().unwrap_or(0);
                let kind = Self::edge_kind(src_depth, tgt_depth);
                let key = (n.id.clone(), e.target.clone());
                let taken = overlay.is_some_and(|ov| ov.edges_taken.contains(&key));
                let available = overlay
                    .is_some_and(|ov| ov.visited.contains(&n.id) && !ov.edges_taken.contains(&key));
                let gated = overlay.is_some_and(|ov| ov.gated_targets.contains(&key));
                edges.push(serde_json::json!({
                    "source": n.id,
                    "target": e.target,
                    "label": e.label,
                    "edge_type": kind,
                    "taken": taken,
                    "available": available,
                    "gated": gated,
                }));
            }
        }

        serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "max_depth": max_depth,
        })
    }

    /// Generate DOT with a session overlay — visited nodes, edges taken,
    /// current position, gated edges, and unexplored paths.
    #[must_use]
    pub fn to_dot_overlay(&self, overlay: &DagOverlay) -> String {
        use std::fmt::Write;

        let mut dot = String::from("digraph NarrativeGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  bgcolor=\"#1a1a2e\";\n");
        dot.push_str("  node [shape=box fontname=\"Helvetica\" fontcolor=\"#e0e0e0\" color=\"#444466\" style=filled];\n");
        dot.push_str("  edge [fontname=\"Helvetica\" fontsize=9 fontcolor=\"#888888\" color=\"#444466\"];\n\n");

        dot.push_str("  // Legend\n");
        dot.push_str("  subgraph cluster_legend {\n");
        dot.push_str("    label=\"\" style=invis;\n");
        dot.push_str("    legend [shape=note fillcolor=\"#1a1a2e\" fontcolor=\"#888888\" label=<");
        dot.push_str("<b>Legend</b><br/>");
        dot.push_str("<font color=\"#50fa7b\">■</font> visited  ");
        dot.push_str("<font color=\"#ff79c6\">■</font> current  ");
        dot.push_str("<font color=\"#ffb86c\">■</font> available  ");
        dot.push_str("<font color=\"#444466\">■</font> unexplored  ");
        dot.push_str("<font color=\"#6272a4\">---</font> gated");
        dot.push_str(">];\n");
        dot.push_str("  }\n\n");

        for node in self.nodes.values() {
            let label = node.label.as_deref().unwrap_or(&node.id);
            let (shape, fill, border) = node_style(node, overlay);
            let _ = writeln!(
                dot,
                "  \"{id}\" [label=\"{label}\" shape={shape} fillcolor=\"{fill}\" color=\"{border}\"];",
                id = node.id,
            );
        }
        dot.push('\n');

        for node in self.nodes.values() {
            for edge in &node.exits {
                let label = edge.label.as_deref().unwrap_or("");
                let (color, style, penwidth) = edge_style(node, edge, overlay);
                let _ = writeln!(
                    dot,
                    "  \"{src}\" -> \"{tgt}\" [label=\"{label}\" color=\"{color}\" style={style} penwidth={penwidth}];",
                    src = node.id,
                    tgt = edge.target,
                );
            }
        }

        dot.push_str("}\n");
        dot
    }
}

fn node_style(node: &NarrativeNode, ov: &DagOverlay) -> (&'static str, &'static str, &'static str) {
    let shape = if node.is_start {
        "doubleoctagon"
    } else if node.is_ending {
        "doublecircle"
    } else {
        "box"
    };

    if ov.current_node.as_deref() == Some(&node.id) {
        (shape, "#ff79c6", "#ff79c6")
    } else if ov.visited.contains(&node.id) {
        (shape, "#2d4a3e", "#50fa7b")
    } else if ov.available_targets.contains(&node.id) {
        (shape, "#3d3520", "#ffb86c")
    } else {
        (shape, "#1e1e30", "#444466")
    }
}

fn edge_style(
    source: &NarrativeNode,
    edge: &NarrativeEdge,
    ov: &DagOverlay,
) -> (&'static str, &'static str, &'static str) {
    let key = (source.id.clone(), edge.target.clone());
    if ov.edges_taken.contains(&key) {
        ("#50fa7b", "bold", "2.5")
    } else if ov.gated_targets.contains(&key) {
        ("#6272a4", "dashed", "1.0")
    } else if ov.visited.contains(&source.id) {
        ("#ffb86c", "solid", "1.0")
    } else {
        ("#444466", "dotted", "0.5")
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::narrative::predicate::StatePredicate;
    use crate::narrative::{NarrativeEdge, NarrativeNode, SceneType, TransitionType};

    fn sample_graph() -> NarrativeGraph {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_owned(),
            NarrativeNode {
                id: "start".to_owned(),
                scene_type: SceneType::Exploration,
                content_ref: "scenes/intro.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![
                    NarrativeEdge {
                        target: "parlor".to_owned(),
                        conditions: vec![],
                        priority: 0,
                        transition_type: TransitionType::SamePlane,
                        label: Some("enter parlor".to_owned()),
                    },
                    NarrativeEdge {
                        target: "garden".to_owned(),
                        conditions: vec![StatePredicate::HasItem("key".to_owned())],
                        priority: 1,
                        transition_type: TransitionType::SamePlane,
                        label: Some("unlock garden".to_owned()),
                    },
                ],
                is_start: true,
                is_ending: false,
                label: Some("Entrance".to_owned()),
            },
        );
        nodes.insert(
            "parlor".to_owned(),
            NarrativeNode {
                id: "parlor".to_owned(),
                scene_type: SceneType::Dialogue,
                content_ref: "scenes/parlor.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![],
                is_start: false,
                is_ending: false,
                label: None,
            },
        );
        nodes.insert(
            "garden".to_owned(),
            NarrativeNode {
                id: "garden".to_owned(),
                scene_type: SceneType::Exploration,
                content_ref: "scenes/garden.yaml".to_owned(),
                preconditions: vec![],
                effects: vec![],
                exits: vec![],
                is_start: false,
                is_ending: true,
                label: Some("Garden Ending".to_owned()),
            },
        );
        NarrativeGraph { nodes }
    }

    #[test]
    fn dot_output_contains_nodes() {
        let g = sample_graph();
        let dot = g.to_dot();
        assert!(dot.contains("start"));
        assert!(dot.contains("parlor"));
        assert!(dot.contains("garden"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn edge_kind_classification() {
        assert_eq!(NarrativeGraph::edge_kind(0, 1), "forward");
        assert_eq!(NarrativeGraph::edge_kind(2, 0), "back");
        assert_eq!(NarrativeGraph::edge_kind(1, 1), "lateral");
    }

    #[test]
    fn to_graph_json_structure() {
        let g = sample_graph();
        let json = g.to_graph_json(None);
        assert!(json.get("nodes").unwrap().as_array().unwrap().len() >= 3);
        assert!(json.get("edges").unwrap().as_array().unwrap().len() >= 2);
        assert!(json.get("max_depth").unwrap().as_u64().unwrap() > 0);
    }

    #[test]
    fn to_graph_json_with_overlay() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned()].into(),
            edges_taken: [("start".to_owned(), "parlor".to_owned())].into(),
            current_node: Some("parlor".to_owned()),
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        };
        let json = g.to_graph_json(Some(&overlay));
        let nodes = json.get("nodes").unwrap().as_array().unwrap();
        let current = nodes
            .iter()
            .find(|n| n.get("id").unwrap().as_str() == Some("parlor"))
            .unwrap();
        assert_eq!(current.get("status").unwrap().as_str(), Some("current"));
    }

    #[test]
    fn dag_overlay_from_history_json() {
        let g = sample_graph();
        let history = serde_json::json!({
            "history": [
                {"action": "exit:parlor", "node_after": "parlor"},
            ],
            "final_node": "parlor"
        });
        let overlay = DagOverlay::from_history_json(&history, &g).unwrap();
        assert!(overlay.visited.contains("start"));
        assert!(overlay.visited.contains("parlor"));
        assert!(
            overlay
                .edges_taken
                .contains(&("start".to_owned(), "parlor".to_owned()))
        );
        assert_eq!(overlay.current_node.as_deref(), Some("parlor"));
    }

    #[test]
    fn dag_overlay_from_history_json_missing_history_errors() {
        let g = sample_graph();
        let bad = serde_json::json!({"final_node": "x"});
        let result = DagOverlay::from_history_json(&bad, &g);
        assert!(result.is_err());
    }

    #[test]
    fn to_dot_overlay_contains_legend_and_styles() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned()].into(),
            edges_taken: HashSet::new(),
            current_node: Some("start".to_owned()),
            available_targets: ["parlor".to_owned()].into(),
            gated_targets: [("start".to_owned(), "garden".to_owned())].into(),
        };
        let dot = g.to_dot_overlay(&overlay);
        assert!(dot.contains("Legend"));
        assert!(dot.contains("#ff79c6"));
        assert!(dot.contains("#ffb86c"));
        assert!(dot.contains("dashed"));
        assert!(dot.contains("digraph"));
    }

    #[test]
    fn empty_graph_dot_is_valid() {
        let g = NarrativeGraph {
            nodes: HashMap::new(),
        };
        let dot = g.to_dot();
        assert!(dot.starts_with("digraph"));
        assert!(dot.ends_with("}\n"));
        assert!(!dot.contains("->"));
    }

    #[test]
    fn empty_graph_json_has_zero_depth() {
        let g = NarrativeGraph {
            nodes: HashMap::new(),
        };
        let json = g.to_graph_json(None);
        assert_eq!(json["max_depth"], 0);
        assert!(json["nodes"].as_array().unwrap().is_empty());
        assert!(json["edges"].as_array().unwrap().is_empty());
    }

    #[test]
    fn dot_start_node_has_doubleoctagon() {
        let g = sample_graph();
        let dot = g.to_dot();
        assert!(dot.contains("doubleoctagon"));
    }

    #[test]
    fn dot_ending_node_has_doublecircle() {
        let g = sample_graph();
        let dot = g.to_dot();
        assert!(dot.contains("doublecircle"));
    }

    #[test]
    fn json_nodes_have_expected_fields() {
        let g = sample_graph();
        let json = g.to_graph_json(None);
        let nodes = json["nodes"].as_array().unwrap();
        for node in nodes {
            assert!(node.get("id").is_some());
            assert!(node.get("label").is_some());
            assert!(node.get("scene_type").is_some());
            assert!(node.get("depth").is_some());
            assert!(node.get("status").is_some());
            assert_eq!(node["status"].as_str(), Some("neutral"));
        }
    }

    #[test]
    fn json_edges_have_expected_fields() {
        let g = sample_graph();
        let json = g.to_graph_json(None);
        let edges = json["edges"].as_array().unwrap();
        for edge in edges {
            assert!(edge.get("source").is_some());
            assert!(edge.get("target").is_some());
            assert!(edge.get("edge_type").is_some());
            assert!(!edge["taken"].as_bool().unwrap());
            assert!(!edge["gated"].as_bool().unwrap());
        }
    }

    #[test]
    fn overlay_visited_node_status() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned(), "parlor".to_owned()]
                .into_iter()
                .collect(),
            edges_taken: HashSet::new(),
            current_node: None,
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        };
        let json = g.to_graph_json(Some(&overlay));
        let nodes = json["nodes"].as_array().unwrap();
        let start = nodes
            .iter()
            .find(|n| n["id"].as_str() == Some("start"))
            .unwrap();
        assert_eq!(start["status"].as_str(), Some("visited"));
    }

    #[test]
    fn overlay_unexplored_node_status() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned()].into(),
            edges_taken: HashSet::new(),
            current_node: Some("start".to_owned()),
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        };
        let json = g.to_graph_json(Some(&overlay));
        let nodes = json["nodes"].as_array().unwrap();
        let garden = nodes
            .iter()
            .find(|n| n["id"].as_str() == Some("garden"))
            .unwrap();
        assert_eq!(garden["status"].as_str(), Some("unexplored"));
    }

    #[test]
    fn overlay_edge_taken_flag() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned(), "parlor".to_owned()]
                .into_iter()
                .collect(),
            edges_taken: [("start".to_owned(), "parlor".to_owned())].into(),
            current_node: Some("parlor".to_owned()),
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        };
        let json = g.to_graph_json(Some(&overlay));
        let edges = json["edges"].as_array().unwrap();
        let taken_edge = edges
            .iter()
            .find(|e| {
                e["source"].as_str() == Some("start") && e["target"].as_str() == Some("parlor")
            })
            .unwrap();
        assert!(taken_edge["taken"].as_bool().unwrap());
    }

    #[test]
    fn overlay_gated_edge_flag() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned()].into(),
            edges_taken: HashSet::new(),
            current_node: Some("start".to_owned()),
            available_targets: HashSet::new(),
            gated_targets: [("start".to_owned(), "garden".to_owned())].into(),
        };
        let json = g.to_graph_json(Some(&overlay));
        let edges = json["edges"].as_array().unwrap();
        let gated_edge = edges
            .iter()
            .find(|e| {
                e["source"].as_str() == Some("start") && e["target"].as_str() == Some("garden")
            })
            .unwrap();
        assert!(gated_edge["gated"].as_bool().unwrap());
    }

    #[test]
    fn dot_overlay_taken_edge_is_bold() {
        let g = sample_graph();
        let overlay = DagOverlay {
            visited: ["start".to_owned(), "parlor".to_owned()]
                .into_iter()
                .collect(),
            edges_taken: [("start".to_owned(), "parlor".to_owned())].into(),
            current_node: Some("parlor".to_owned()),
            available_targets: HashSet::new(),
            gated_targets: HashSet::new(),
        };
        let dot = g.to_dot_overlay(&overlay);
        assert!(dot.contains("style=bold"));
        assert!(dot.contains("#50fa7b"));
    }

    #[test]
    fn history_with_non_exit_actions_ignored_for_edges() {
        let g = sample_graph();
        let history = serde_json::json!({
            "history": [
                {"action": "examine", "node_after": "start"},
                {"action": "exit:parlor", "node_after": "parlor"},
            ],
            "final_node": "parlor"
        });
        let overlay = DagOverlay::from_history_json(&history, &g).unwrap();
        assert_eq!(overlay.edges_taken.len(), 1);
        assert!(
            overlay
                .edges_taken
                .contains(&("start".to_owned(), "parlor".to_owned()))
        );
    }

    #[test]
    fn history_with_empty_entries_handled() {
        let g = sample_graph();
        let history = serde_json::json!({
            "history": [
                {"action": "", "node_after": ""},
            ],
            "final_node": "start"
        });
        let overlay = DagOverlay::from_history_json(&history, &g).unwrap();
        assert!(overlay.edges_taken.is_empty());
        assert_eq!(overlay.current_node.as_deref(), Some("start"));
    }
}
