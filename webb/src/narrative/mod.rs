// SPDX-License-Identifier: AGPL-3.0-or-later
//! NarrativeGraph engine — bounded space, infinite exploration.
//!
//! The core architecture: finite authored topology (DAG of scenes, beats,
//! transitions) combined with combinatorial state space (knowledge, trust,
//! conditions, inventory, arc phases) produces near-infinite traversal
//! with bounded, meaningful endings.

pub mod effect;
pub mod predicate;
pub mod validator;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use self::effect::StateEffect;
use self::predicate::StatePredicate;

/// The type of scene a narrative node represents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneType {
    /// Open dialogue with NPC(s).
    Dialogue,
    /// Free movement and ambient discovery.
    Exploration,
    /// Clue gathering and deduction.
    Investigation,
    /// Grid/zone combat.
    Tactical,
    /// Plane transition (meta-node).
    Transition,
    /// Game ending.
    Ending,
}

/// How a narrative edge transitions between scenes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    /// Same plane, different scene.
    SamePlane,
    /// Cross-plane transition (e.g. dialogue -> tactical).
    CrossPlane,
    /// Temporal (passage of time).
    Temporal,
}

/// An authored scene/beat in the narrative DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeNode {
    /// Unique node identifier.
    pub id: String,
    /// Type of scene.
    pub scene_type: SceneType,
    /// Reference to the YAML scene content file.
    pub content_ref: String,
    /// Preconditions that must be met to enter this node.
    #[serde(default)]
    pub preconditions: Vec<StatePredicate>,
    /// State effects applied when this node is entered.
    #[serde(default)]
    pub effects: Vec<StateEffect>,
    /// Outgoing edges to other nodes.
    #[serde(default)]
    pub exits: Vec<NarrativeEdge>,
    /// Whether this is the start node.
    #[serde(default)]
    pub is_start: bool,
    /// Whether this is an ending node.
    #[serde(default)]
    pub is_ending: bool,
    /// Human-readable label for graph visualization.
    #[serde(default)]
    pub label: Option<String>,
}

/// A directed edge between narrative nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEdge {
    /// Target node ID.
    pub target: String,
    /// Conditions that must be met to traverse this edge.
    #[serde(default)]
    pub conditions: Vec<StatePredicate>,
    /// Priority for disambiguation when multiple edges are valid.
    #[serde(default)]
    pub priority: i32,
    /// Transition type.
    #[serde(default = "default_transition_type")]
    pub transition_type: TransitionType,
    /// Human-readable label.
    #[serde(default)]
    pub label: Option<String>,
}

const fn default_transition_type() -> TransitionType {
    TransitionType::SamePlane
}

/// The full narrative DAG, loaded from YAML content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NarrativeGraph {
    /// All nodes in the graph, keyed by ID.
    pub nodes: HashMap<String, NarrativeNode>,
}

impl NarrativeGraph {
    /// Find the start node (the node with `is_start = true`).
    pub fn start_node(&self) -> Option<&NarrativeNode> {
        self.nodes.values().find(|n| n.is_start)
    }

    /// Find all ending nodes.
    pub fn endings(&self) -> Vec<&NarrativeNode> {
        self.nodes.values().filter(|n| n.is_ending).collect()
    }

    /// Get a node by ID.
    pub fn get(&self, id: &str) -> Option<&NarrativeNode> {
        self.nodes.get(id)
    }

    /// Get the valid exits from a node given the current state predicates.
    ///
    /// Returns edges sorted by priority (highest first).
    pub fn valid_exits<F>(&self, node_id: &str, evaluate: F) -> Vec<&NarrativeEdge>
    where
        F: Fn(&StatePredicate) -> bool,
    {
        let Some(node) = self.nodes.get(node_id) else {
            return Vec::new();
        };
        let mut exits: Vec<&NarrativeEdge> = node
            .exits
            .iter()
            .filter(|edge| edge.conditions.iter().all(&evaluate))
            .collect();
        exits.sort_by(|a, b| b.priority.cmp(&a.priority));
        exits
    }

    /// Generate DOT format for graph visualization.
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

    /// Total number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.nodes.values().map(|n| n.exits.len()).sum()
    }

    /// BFS shortest-path depth from the start node to every reachable node.
    ///
    /// Depth 0 = start, depth N = N forward hops minimum.  Unreachable nodes
    /// are absent from the map.
    pub fn bfs_depths(&self) -> HashMap<String, usize> {
        use std::collections::VecDeque;

        let mut depths = HashMap::new();
        let Some(start) = self.start_node() else {
            return depths;
        };
        let mut queue = VecDeque::new();
        queue.push_back((start.id.clone(), 0usize));
        depths.insert(start.id.clone(), 0);

        while let Some((id, depth)) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&id) {
                for edge in &node.exits {
                    depths.entry(edge.target.clone()).or_insert_with(|| {
                        queue.push_back((edge.target.clone(), depth + 1));
                        depth + 1
                    });
                }
            }
        }
        depths
    }

    /// Classify an edge relative to BFS depth layers.
    const fn edge_kind(src_depth: usize, tgt_depth: usize) -> &'static str {
        match tgt_depth {
            d if d > src_depth => "forward",
            d if d < src_depth => "back",
            _ => "lateral",
        }
    }

    /// Export the graph as structured JSON with BFS depth layers and edge
    /// classification.  When an overlay is provided, each node and edge
    /// carries session state (visited, taken, current, available).
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

/// Overlay data for rendering a DAG with session state.
///
/// Three conceptual views on the same structure:
/// - **Narrative DAG**: the full authored graph (the NarrativeGraph itself)
/// - **Live DAG**: current position + available/gated edges (during play)
/// - **Played DAG**: visited nodes + edges taken (after a session)
#[derive(Debug, Clone, Default)]
pub struct DagOverlay {
    /// Nodes the player has visited.
    pub visited: std::collections::HashSet<String>,
    /// Edges the player has traversed (source, target).
    pub edges_taken: std::collections::HashSet<(String, String)>,
    /// Current node (if in a live session).
    pub current_node: Option<String>,
    /// Edges currently available from the current node.
    pub available_targets: std::collections::HashSet<String>,
    /// Edges that exist but are gated (conditions not met).
    pub gated_targets: std::collections::HashSet<(String, String)>,
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
    ) -> Result<Self, String> {
        let history = json
            .get("history")
            .and_then(serde_json::Value::as_array)
            .ok_or("missing 'history' array")?;

        let start_id = graph
            .start_node()
            .map_or_else(String::new, |n| n.id.clone());

        let mut visited = std::collections::HashSet::new();
        let mut edges_taken = std::collections::HashSet::new();

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
            available_targets: std::collections::HashSet::new(),
            gated_targets: std::collections::HashSet::new(),
        })
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
        (shape, "#ff79c6", "#ff79c6") // pink: current position
    } else if ov.visited.contains(&node.id) {
        (shape, "#2d4a3e", "#50fa7b") // green: visited
    } else if ov.available_targets.contains(&node.id) {
        (shape, "#3d3520", "#ffb86c") // orange: reachable now
    } else {
        (shape, "#1e1e30", "#444466") // dark: unexplored
    }
}

fn edge_style(
    source: &NarrativeNode,
    edge: &NarrativeEdge,
    ov: &DagOverlay,
) -> (&'static str, &'static str, &'static str) {
    let key = (source.id.clone(), edge.target.clone());
    if ov.edges_taken.contains(&key) {
        ("#50fa7b", "bold", "2.5") // green bold: path taken
    } else if ov.gated_targets.contains(&key) {
        ("#6272a4", "dashed", "1.0") // blue dashed: gated
    } else if ov.visited.contains(&source.id) {
        ("#ffb86c", "solid", "1.0") // orange: available but not taken
    } else {
        ("#444466", "dotted", "0.5") // dim: unexplored territory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn start_node_found() {
        let g = sample_graph();
        let start = g.start_node();
        assert!(start.is_some());
        assert_eq!(start.map(|n| n.id.as_str()), Some("start"));
    }

    #[test]
    fn endings_found() {
        let g = sample_graph();
        let endings = g.endings();
        assert_eq!(endings.len(), 1);
        assert_eq!(endings[0].id, "garden");
    }

    #[test]
    fn valid_exits_no_conditions() {
        let g = sample_graph();
        let exits = g.valid_exits("start", |_| true);
        assert_eq!(exits.len(), 2);
        assert_eq!(exits[0].target, "garden"); // priority 1 first
    }

    #[test]
    fn valid_exits_filtered_by_predicate() {
        let g = sample_graph();
        let exits = g.valid_exits("start", |pred| !matches!(pred, StatePredicate::HasItem(_)));
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].target, "parlor");
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
    fn edge_count() {
        let g = sample_graph();
        assert_eq!(g.edge_count(), 2);
    }
}
