#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;

#[test]
fn discover_binary_fails_for_nonexistent() {
    let result = discover_binary("nonexistent_primal_xyz");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("binary not found"));
}

#[test]
fn topological_waves_empty_graph() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "empty".to_owned(),
            node: vec![],
        },
    };
    let waves = topological_waves(&graph).unwrap();
    assert!(waves.is_empty());
}

#[test]
fn topological_waves_linear_chain() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "linear".to_owned(),
            node: vec![
                GraphNode {
                    name: "a".to_owned(),
                    binary: None,
                    order: 1,
                    spawn: true,
                    depends_on: vec![],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "b".to_owned(),
                    binary: None,
                    order: 2,
                    spawn: true,
                    depends_on: vec!["a".to_owned()],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "c".to_owned(),
                    binary: None,
                    order: 3,
                    spawn: true,
                    depends_on: vec!["b".to_owned()],
                    by_capability: None,
                    port: None,
                },
            ],
        },
    };
    let waves = topological_waves(&graph).unwrap();
    assert_eq!(waves.len(), 3);
    assert_eq!(waves[0][0].name, "a");
    assert_eq!(waves[1][0].name, "b");
    assert_eq!(waves[2][0].name, "c");
}

#[test]
fn topological_waves_parallel_tier() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "parallel".to_owned(),
            node: vec![
                GraphNode {
                    name: "a".to_owned(),
                    binary: None,
                    order: 1,
                    spawn: true,
                    depends_on: vec![],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "b".to_owned(),
                    binary: None,
                    order: 2,
                    spawn: true,
                    depends_on: vec![],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "c".to_owned(),
                    binary: None,
                    order: 3,
                    spawn: true,
                    depends_on: vec!["a".to_owned(), "b".to_owned()],
                    by_capability: None,
                    port: None,
                },
            ],
        },
    };
    let waves = topological_waves(&graph).unwrap();
    assert_eq!(waves.len(), 2);
    assert_eq!(waves[0].len(), 2);
    assert_eq!(waves[1].len(), 1);
    assert_eq!(waves[1][0].name, "c");
}

#[test]
fn topological_waves_detects_cycle() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "cyclic".to_owned(),
            node: vec![
                GraphNode {
                    name: "a".to_owned(),
                    binary: None,
                    order: 1,
                    spawn: true,
                    depends_on: vec!["b".to_owned()],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "b".to_owned(),
                    binary: None,
                    order: 2,
                    spawn: true,
                    depends_on: vec!["a".to_owned()],
                    by_capability: None,
                    port: None,
                },
            ],
        },
    };
    let result = topological_waves(&graph);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn launcher_new_has_no_spawned() {
    let launcher = PrimalLauncher::new();
    assert!(launcher.spawned().is_empty());
}

#[test]
fn launcher_default_matches_new() {
    let d = PrimalLauncher::default();
    assert!(d.spawned().is_empty());
}

#[test]
fn launcher_shutdown_on_empty_is_noop() {
    let mut launcher = PrimalLauncher::new();
    launcher.shutdown();
    assert!(launcher.spawned().is_empty());
}

#[test]
fn launcher_drop_is_safe_when_empty() {
    let launcher = PrimalLauncher::new();
    drop(launcher);
}

#[test]
fn readiness_timeout_returns_duration() {
    let t = readiness_timeout();
    assert!(t.as_secs() > 0);
}

#[test]
fn default_port_base_returns_valid_port() {
    let p = default_port_base();
    assert!(p > 0);
}

#[test]
fn topological_waves_missing_dependency() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "missing_dep".to_owned(),
            node: vec![GraphNode {
                name: "a".to_owned(),
                binary: None,
                order: 1,
                spawn: true,
                depends_on: vec!["nonexistent".to_owned()],
                by_capability: None,
                port: None,
            }],
        },
    };
    let result = topological_waves(&graph);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not in the graph"));
}

#[test]
fn topological_waves_single_node() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "single".to_owned(),
            node: vec![GraphNode {
                name: "solo".to_owned(),
                binary: None,
                order: 0,
                spawn: true,
                depends_on: vec![],
                by_capability: Some("dag".to_owned()),
                port: Some(9500),
            }],
        },
    };
    let waves = topological_waves(&graph).unwrap();
    assert_eq!(waves.len(), 1);
    assert_eq!(waves[0][0].name, "solo");
}

#[test]
fn topological_waves_diamond() {
    let graph = DeployGraph {
        graph: GraphSection {
            name: "diamond".to_owned(),
            node: vec![
                GraphNode {
                    name: "a".to_owned(),
                    binary: None,
                    order: 0,
                    spawn: true,
                    depends_on: vec![],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "b".to_owned(),
                    binary: None,
                    order: 1,
                    spawn: true,
                    depends_on: vec!["a".to_owned()],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "c".to_owned(),
                    binary: None,
                    order: 2,
                    spawn: true,
                    depends_on: vec!["a".to_owned()],
                    by_capability: None,
                    port: None,
                },
                GraphNode {
                    name: "d".to_owned(),
                    binary: None,
                    order: 3,
                    spawn: true,
                    depends_on: vec!["b".to_owned(), "c".to_owned()],
                    by_capability: None,
                    port: None,
                },
            ],
        },
    };
    let waves = topological_waves(&graph).unwrap();
    assert_eq!(waves.len(), 3);
    assert_eq!(waves[0].len(), 1);
    assert_eq!(waves[1].len(), 2);
    assert_eq!(waves[2].len(), 1);
    assert_eq!(waves[2][0].name, "d");
}

#[test]
fn deploy_graph_toml_roundtrip() {
    let toml_str = r#"
[graph]
name = "test"

[[graph.node]]
name = "rhizocrypt"
order = 1
depends_on = []
by_capability = "dag"
port = 9410

[[graph.node]]
name = "squirrel"
order = 2
depends_on = ["rhizocrypt"]
"#;
    let graph: DeployGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.graph.name, "test");
    assert_eq!(graph.graph.node.len(), 2);
    assert_eq!(graph.graph.node[0].name, "rhizocrypt");
    assert_eq!(graph.graph.node[0].port, Some(9410));
    assert!(graph.graph.node[0].spawn);
    assert_eq!(graph.graph.node[1].depends_on, vec!["rhizocrypt"]);
}

#[test]
fn spawn_from_graph_bad_toml_file() {
    let dir = std::env::temp_dir().join("esoteric_webb_test_bad_toml");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("graph.toml");
    std::fs::write(&path, "{{{ not toml").unwrap();

    let mut launcher = PrimalLauncher::new();
    let result = launcher.spawn_from_graph(path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn spawn_from_graph_missing_file() {
    let mut launcher = PrimalLauncher::new();
    let result = launcher.spawn_from_graph("/nonexistent/graph.toml");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("io:") || msg.contains("No such file"));
}

#[test]
fn spawn_fails_for_missing_binary() {
    let mut launcher = PrimalLauncher::new();
    let result = launcher.spawn("nonexistent_primal_xyz", 9999, "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("binary not found"));
}

#[test]
fn graph_node_defaults() {
    let toml_str = r#"
[graph]
name = "defaults"

[[graph.node]]
name = "minimal"
"#;
    let graph: DeployGraph = toml::from_str(toml_str).unwrap();
    let node = &graph.graph.node[0];
    assert_eq!(node.order, 0);
    assert!(node.spawn);
    assert!(node.depends_on.is_empty());
    assert!(node.binary.is_none());
    assert!(node.by_capability.is_none());
    assert!(node.port.is_none());
}

#[test]
fn graph_node_spawn_false() {
    let toml_str = r#"
[graph]
name = "nospawn"

[[graph.node]]
name = "observer"
spawn = false
"#;
    let graph: DeployGraph = toml::from_str(toml_str).unwrap();
    assert!(!graph.graph.node[0].spawn);
}

// NOTE: await_tcp_ready timeout test omitted — requires set_var
// which is unsafe in edition 2024, forbidden by workspace lints.

#[test]
fn await_tcp_ready_succeeds_for_bound_port() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let result = await_tcp_ready(&addr);
    assert!(result.is_ok());
}

#[test]
fn spawned_primal_fields() {
    let sp = SpawnedPrimal {
        name: "rhizocrypt".to_owned(),
        pid: 12345,
        address: "127.0.0.1:9410".to_owned(),
        domain: "dag".to_owned(),
    };
    assert_eq!(sp.name, "rhizocrypt");
    assert_eq!(sp.pid, 12345);
    assert_eq!(sp.address, "127.0.0.1:9410");
    assert_eq!(sp.domain, "dag");
}
