// SPDX-License-Identifier: AGPL-3.0-or-later
//! `UniBin` subcommand implementations.

use std::io::Write;

use esoteric_webb::content::ContentBundle;
use esoteric_webb::director::{DirectorOutcome, GameDirector, PlayerInput};
use esoteric_webb::state::WorldState;

/// Start the full BYOB niche with game director and IPC server.
///
/// With `--launch`, spawns primal binaries from `plasmidBin/` using the
/// deploy graph before discovering. Without it, connects to running primals.
pub fn cmd_serve(content_path: &str, launch: bool, graph_path: &str) -> Result<(), String> {
    // Launcher owns child processes and kills them on Drop — must outlive the server.
    #[allow(clippy::collection_is_never_read)] // held for Drop semantics, not read access
    let _launcher: Option<esoteric_webb::ipc::launcher::PrimalLauncher>;

    let bridge = if launch {
        println!("BYOB composition: launching primals from plasmidBin/ ...");
        let mut launcher = esoteric_webb::ipc::launcher::PrimalLauncher::new();

        let graph_exists = std::path::Path::new(graph_path).is_file();
        if graph_exists {
            println!("Deploy graph: {graph_path}");
            launcher
                .spawn_from_graph(graph_path)
                .map_err(|e| format!("launch from graph: {e}"))?;
        } else {
            println!("No deploy graph at {graph_path} — skipping graph-driven launch");
        }

        let mut bridge = esoteric_webb::ipc::bridge::PrimalBridge::discover();

        for sp in launcher.spawned() {
            if !bridge.has(&sp.domain) {
                if let Ok(client) =
                    esoteric_webb::ipc::client::PrimalClient::connect_tcp(&sp.address, &sp.name)
                {
                    bridge.inject(&sp.domain, client, &format!("tcp:{}", sp.address));
                }
            }
        }

        _launcher = Some(launcher);
        bridge
    } else {
        println!("BYOB composition: discovering primals from plasmidBin/ via Songbird...");
        _launcher = None;
        esoteric_webb::ipc::bridge::PrimalBridge::discover()
    };

    let connected = bridge.connected_count();
    for s in bridge.statuses() {
        let icon = if s.healthy { "+" } else { "-" };
        let transport = s.transport.as_deref().unwrap_or("—");
        println!(
            "  [{icon}] {name} ({domain}) {transport}",
            name = s.name,
            domain = s.domain,
        );
    }
    println!(
        "{connected} primal(s) connected — {mode}",
        mode = if connected == 0 {
            "standalone mode"
        } else {
            "composition mode"
        }
    );

    let session = esoteric_webb::session::GameSession::with_bridge(content_path, Some(bridge))?;
    let b = session.bundle();
    println!(
        "Loaded: {} NPC(s), {} ability(ies), {} scene(s), {} narrative node(s)",
        b.npcs.len(),
        b.abilities.len(),
        b.scenes.len(),
        b.narrative.nodes.len(),
    );

    let shared = esoteric_webb::ipc::handlers::new_shared_session();
    {
        let mut guard = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(session);
    }

    let sock = esoteric_webb::ipc::listener::socket_path();
    println!("Starting IPC server on {}", sock.display());
    println!("Session pre-loaded — connect and call session.state to begin");
    esoteric_webb::ipc::listener::serve(&sock, &shared)
}

/// Validate a content directory for correctness.
pub fn cmd_validate(content_path: &str) -> Result<(), String> {
    let bundle = ContentBundle::load(content_path).map_err(|e| format!("load: {e}"))?;
    let issues = bundle.validate();
    if issues.is_empty() {
        println!(
            "Content valid: {} NPC(s), {} ability(ies), {} scene(s), {} narrative node(s)",
            bundle.npcs.len(),
            bundle.abilities.len(),
            bundle.scenes.len(),
            bundle.narrative.nodes.len(),
        );
        Ok(())
    } else {
        for issue in &issues {
            eprintln!("  - {issue}");
        }
        Err(format!("{} validation issue(s)", issues.len()))
    }
}

/// Text-mode interactive game preview (no primals required).
pub fn cmd_preview(content_path: &str) -> Result<(), String> {
    let bundle = ContentBundle::load(content_path).map_err(|e| format!("load: {e}"))?;
    print_load_warnings(&bundle);
    let issues = bundle.validate();
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("  - {issue}");
        }
        return Err(format!(
            "{} validation issue(s) — run `validate` first",
            issues.len()
        ));
    }

    println!("=== Esoteric Webb — {} ===", bundle.meta.name);
    println!("by {}", bundle.meta.author);
    println!("{}", bundle.meta.description);
    println!();

    let mut director = GameDirector::new(&bundle).map_err(|e| format!("director init: {e}"))?;
    let mut state = WorldState::new();

    preview_loop(&mut director, &mut state, &bundle);
    Ok(())
}

fn preview_loop(director: &mut GameDirector, state: &mut WorldState, bundle: &ContentBundle) {
    loop {
        let scene_desc = director.current_scene_description(bundle);
        if !scene_desc.is_empty() {
            println!("{scene_desc}");
            println!();
        }

        if director.is_at_ending(bundle) {
            println!("=== THE END ===");
            println!("Turns taken: {}", state.turn);
            let knowledge: Vec<_> = state.knowledge.iter().cloned().collect();
            println!("Knowledge: {}", knowledge.join(", "));
            break;
        }

        let options = build_action_menu(director, state, bundle);
        println!("--- Actions ---");
        for (i, (label, _)) in options.iter().enumerate() {
            println!("  [{i}] {label}");
        }
        println!();

        let Some(choice) = read_choice(options.len()) else {
            println!("Goodbye.");
            break;
        };

        let (_, input) = &options[choice];
        let outcome = director.process(input, state, bundle);
        println!();
        match outcome {
            DirectorOutcome::Narration(text) => println!("> {text}"),
            DirectorOutcome::SceneChange { node_id, narration } => {
                println!("--- Moving to: {node_id} ---");
                if !narration.is_empty() {
                    println!("> {narration}");
                }
            }
            DirectorOutcome::NoEffect(msg) => println!("({msg})"),
        }
        println!();
    }
}

fn build_action_menu(
    director: &GameDirector,
    state: &WorldState,
    bundle: &ContentBundle,
) -> Vec<(String, PlayerInput)> {
    let mut options: Vec<(String, PlayerInput)> = Vec::new();

    for edge in &director.available_exits(bundle, state) {
        let label = edge.label.as_deref().unwrap_or(&edge.target);
        options.push((
            format!("Go: {label}"),
            PlayerInput::ChooseExit(edge.target.clone()),
        ));
    }

    let scene_npcs = current_scene_npcs(bundle, director);
    for npc_id in &scene_npcs {
        options.push((
            format!("Talk to {npc_id}"),
            PlayerInput::Talk(npc_id.clone()),
        ));
    }

    for ability in bundle.abilities.values() {
        options.push((
            format!("Use: {} — {}", ability.name, ability.description),
            PlayerInput::UseAbility(ability.id.clone()),
        ));
    }

    options.push(("Examine surroundings".to_owned(), PlayerInput::Examine));
    options
}

fn read_choice(max: usize) -> Option<usize> {
    print!("Choose (number, or q to quit): ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return None;
    }
    let input = input.trim();
    if input == "q" || input == "quit" {
        return None;
    }
    input.parse::<usize>().ok().filter(|&i| i < max)
}

/// Show primal composition status.
///
/// Returns `Result` for command dispatch uniformity — currently infallible.
#[allow(clippy::unnecessary_wraps)] // all cmd_* share a uniform Result signature
pub fn cmd_status() -> Result<(), String> {
    println!("Esoteric Webb — primal composition status");
    println!("Discovering primals...\n");

    let bridge = esoteric_webb::ipc::bridge::PrimalBridge::discover();
    let connected = bridge.connected_count();

    for s in bridge.statuses() {
        let disc = if s.discovered { "found" } else { "—" };
        let health = if s.healthy { "healthy" } else { "—" };
        println!(
            "  {name:<14} {domain:<16} {disc:<10} {health}",
            name = s.name,
            domain = s.domain,
        );
    }

    println!(
        "\n{connected}/{total} primal(s) connected",
        total = bridge.statuses().len()
    );

    if connected == 0 {
        println!("Mode: standalone (all degradation paths active)");
    } else {
        println!("Mode: composition");
    }

    Ok(())
}

/// Visualize the narrative DAG.
///
/// Three views:
/// - Bare: the full authored graph (no overlay)
/// - Played: overlay a completed session JSON onto the graph
/// - Live: start a session and show its initial state
///
/// Two formats:
/// - `dot`: graphviz DOT output
/// - `json`: structured 3D graph JSON (nodes with BFS depth, edges with
///   forward/back/lateral classification, session overlay state)
pub fn cmd_graph(
    content_path: &str,
    played_path: Option<&str>,
    live: bool,
    format: &str,
) -> Result<(), String> {
    let bundle = ContentBundle::load(content_path).map_err(|e| format!("load: {e}"))?;
    print_load_warnings(&bundle);

    let overlay = if let Some(path) = played_path {
        let json_str = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
        let json: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("parse JSON: {e}"))?;
        Some(esoteric_webb::narrative::DagOverlay::from_history_json(
            &json,
            &bundle.narrative,
        )?)
    } else if live {
        let session = esoteric_webb::session::GameSession::new(content_path)?;
        Some(session.dag_overlay())
    } else {
        None
    };

    match format {
        "json" => {
            let graph_json = bundle.narrative.to_graph_json(overlay.as_ref());
            println!(
                "{}",
                serde_json::to_string_pretty(&graph_json).unwrap_or_default()
            );
        }
        _ => {
            if let Some(ref ov) = overlay {
                println!("{}", bundle.narrative.to_dot_overlay(ov));
            } else {
                println!("{}", bundle.narrative.to_dot());
            }
        }
    }
    Ok(())
}

/// Replay a provenance-traced session.
pub fn cmd_replay(session_path: &str, content_path: &str) -> Result<(), String> {
    let _bundle = ContentBundle::load(content_path).map_err(|e| format!("load: {e}"))?;
    println!("Replay session: {session_path}");
    println!("(provenance replay not yet wired — see EVOLUTION_GAPS.md)");
    Ok(())
}

/// Automated playthrough — AI-as-player demonstration.
pub fn cmd_autoplay(content_path: &str, max_turns: u32, json_output: bool) -> Result<(), String> {
    use esoteric_webb::autoplay::{self, AutoplayConfig};
    use esoteric_webb::session::GameSession;

    let mut session = GameSession::new(content_path)?;

    if !json_output {
        let snap = session.snapshot();
        println!("=== AUTOPLAY: {} ===", session.bundle().meta.name);
        println!("{}", snap.scene_description);
        println!();
    }

    let config = AutoplayConfig {
        max_turns,
        ..AutoplayConfig::default()
    };
    let result = autoplay::run(&mut session, &config)?;

    if json_output {
        let output = serde_json::json!({
            "world": session.bundle().meta.name,
            "ended": result.ended,
            "turns": result.turns,
            "final_node": session.snapshot().current_node,
            "knowledge": session.snapshot().knowledge,
            "inventory": session.snapshot().inventory,
            "flags": session.snapshot().flags,
            "nodes_visited": result.nodes_visited,
            "stale_halt": result.stale_halt,
            "history": serde_json::to_value(session.history()).unwrap_or_default(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        let snap = session.snapshot();
        println!("=== AUTOPLAY COMPLETE ===");
        println!("Ended: {}", result.ended);
        println!("Turns: {}", snap.turn);
        println!("Node: {}", snap.current_node);
        println!("Knowledge: {}", snap.knowledge.join(", "));
        println!("Flags: {}", snap.flags.join(", "));
        println!("Nodes visited: {}", result.nodes_visited);
    }
    Ok(())
}

/// Scaffold a new content directory with template YAML.
pub fn cmd_new_world(output_path: &str) -> Result<(), String> {
    esoteric_webb::content::scaffold(output_path).map_err(|e| format!("scaffold: {e}"))
}

/// Run all experiment validation suites (meta-runner).
pub fn cmd_validate_all() -> Result<(), String> {
    const EXPERIMENTS: &[&str] = &[
        "esotericwebb-exp001",
        "esotericwebb-exp002",
        "esotericwebb-exp003",
        "esotericwebb-exp004",
        "esotericwebb-exp005",
    ];

    println!("=== Esoteric Webb — validate --all ===\n");

    let json_mode = std::env::var("ESOTERICWEBB_JSON")
        .ok()
        .is_some_and(|v| v == "1" || v == "true");

    let mut passed = 0u32;
    let mut failed = 0u32;

    for &pkg in EXPERIMENTS {
        println!("--- {pkg} ---");
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(["run", "--release", "-p", pkg]);
        if json_mode {
            cmd.env("ESOTERICWEBB_JSON", "1");
        }
        match cmd.status() {
            Ok(status) if status.success() => {
                passed += 1;
                println!("  -> PASS");
            }
            Ok(status) => {
                failed += 1;
                let code = status.code().unwrap_or(-1);
                println!("  -> FAIL (exit {code})");
            }
            Err(e) => {
                failed += 1;
                println!("  -> ERROR: {e}");
            }
        }
        println!();
    }

    let total = EXPERIMENTS.len();
    println!("=== SUMMARY ===");
    println!("  {passed}/{total} passed, {failed} failed");

    if failed > 0 {
        Err(format!("{failed} experiment(s) failed"))
    } else {
        Ok(())
    }
}

fn print_load_warnings(bundle: &ContentBundle) {
    for w in &bundle.load_warnings {
        eprintln!("warning: {w}");
    }
}

fn current_scene_npcs(bundle: &ContentBundle, director: &GameDirector) -> Vec<String> {
    let node_id = director.current_node_id();
    bundle
        .narrative
        .get(node_id)
        .and_then(|node| bundle.scenes.get(&node.content_ref))
        .map(|scene| scene.npcs.clone())
        .unwrap_or_default()
}
