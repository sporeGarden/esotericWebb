// SPDX-License-Identifier: AGPL-3.0-or-later
//! `NarrativeGraph` engine — bounded space, infinite exploration.
//!
//! The core architecture: finite authored topology (DAG of scenes, beats,
//! transitions) combined with combinatorial state space (knowledge, trust,
//! conditions, inventory, arc phases) produces near-infinite traversal
//! with bounded, meaningful endings.

pub mod effect;
pub mod predicate;
pub mod validator;
pub mod visualization;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use visualization::DagOverlay;

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
    /// Total number of narrative nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Find the start node (the node with `is_start = true`).
    #[must_use]
    pub fn start_node(&self) -> Option<&NarrativeNode> {
        self.nodes.values().find(|n| n.is_start)
    }

    /// Find all ending nodes.
    #[must_use]
    pub fn endings(&self) -> Vec<&NarrativeNode> {
        self.nodes.values().filter(|n| n.is_ending).collect()
    }

    /// Get a node by ID.
    #[must_use]
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
        exits.sort_by_key(|e| std::cmp::Reverse(e.priority));
        exits
    }

    /// Total number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.nodes.values().map(|n| n.exits.len()).sum()
    }

    /// BFS shortest-path depth from the start node to every reachable node.
    ///
    /// Depth 0 = start, depth N = N forward hops minimum.  Unreachable nodes
    /// are absent from the map.
    #[must_use]
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
    fn edge_count() {
        let g = sample_graph();
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn bfs_depths_correct() {
        let g = sample_graph();
        let depths = g.bfs_depths();
        assert_eq!(depths.get("start").copied(), Some(0));
        assert_eq!(depths.get("parlor").copied(), Some(1));
        assert_eq!(depths.get("garden").copied(), Some(1));
    }
}
