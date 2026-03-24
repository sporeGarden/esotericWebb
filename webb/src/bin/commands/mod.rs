// SPDX-License-Identifier: AGPL-3.0-or-later
//! UniBin subcommand implementations.

use std::io::Write;

use esoteric_webb::content::ContentBundle;
use esoteric_webb::director::{DirectorOutcome, GameDirector, PlayerInput};
use esoteric_webb::state::WorldState;

/// Start the full BYOB niche with game director and IPC server.
///
/// With `--launch`, spawns primal binaries from `plasmidBin/` using the
/// deploy graph before discovering. Without it, connects to running primals.
pub fn cmd_serve(content_path: &str, launch: bool, graph_path: &str) -> Result<(), String> {
    // Launcher must live as long as the server — child processes are killed on Drop.
    #[allow(clippy::collection_is_never_read)]
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

    let shared = esoteric_webb::ipc::server::new_shared_session();
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

/// Visualize the NarrativeGraph as DOT format.
/// Show primal composition status.
#[allow(clippy::unnecessary_wraps)]
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
///
/// The game plays itself using a heuristic strategy:
/// 1. Prefer unexplored exits over visited ones
/// 2. Try abilities that haven't been used yet
/// 3. Talk to NPCs in the scene
/// 4. Fall back to examine
pub fn cmd_autoplay(content_path: &str, max_turns: u32, json_output: bool) -> Result<(), String> {
    use esoteric_webb::session::GameSession;

    let mut session = GameSession::new(content_path)?;
    let mut tracker = HeuristicTracker::default();

    if !json_output {
        let snap = session.snapshot();
        println!("=== AUTOPLAY: {} ===", session.bundle().meta.name);
        println!("{}", snap.scene_description);
        println!();
    }

    tracker.visited.insert(session.snapshot().current_node);

    for _ in 0..max_turns {
        if session.is_ended() {
            break;
        }

        let actions = session.available_actions();
        let node = session.snapshot().current_node.clone();
        let choice = tracker.pick(&actions, &node);

        let Some((kind, id)) = choice else {
            break;
        };

        let (outcome_text, _ctx) = session.act(&kind, &id).map_err(|e| format!("act: {e}"))?;
        let snap_after = session.snapshot();
        let knowledge_count =
            snap_after.knowledge.len() + snap_after.flags.len() + snap_after.inventory.len();
        tracker.record_novelty(&kind, &id, knowledge_count);
        tracker.visited.insert(snap_after.current_node.clone());

        if !json_output {
            println!("[turn {}] {kind}:{id}", session.snapshot().turn);
            println!("> {outcome_text}");
            println!();
        }
    }

    autoplay_summary(&session, json_output);
    Ok(())
}

fn autoplay_summary(session: &esoteric_webb::session::GameSession, json_output: bool) {
    if json_output {
        let output = serde_json::json!({
            "world": session.bundle().meta.name,
            "ended": session.is_ended(),
            "turns": session.snapshot().turn,
            "final_node": session.snapshot().current_node,
            "knowledge": session.snapshot().knowledge,
            "inventory": session.snapshot().inventory,
            "flags": session.snapshot().flags,
            "history": serde_json::to_value(session.history()).unwrap_or_default(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        let snap = session.snapshot();
        println!("=== AUTOPLAY COMPLETE ===");
        println!("Ended: {}", session.is_ended());
        println!("Turns: {}", snap.turn);
        println!("Node: {}", snap.current_node);
        println!("Knowledge: {}", snap.knowledge.join(", "));
        println!("Flags: {}", snap.flags.join(", "));
    }
}

/// Tracks the heuristic's exploration state so it doesn't loop.
#[derive(Default)]
struct HeuristicTracker {
    visited: std::collections::HashSet<String>,
    used_abilities: std::collections::HashSet<String>,
    talk_count: std::collections::HashMap<String, u32>,
    examined_at: std::collections::HashSet<String>,
    stale_count: u32,
    last_knowledge_count: usize,
    exit_rotation: usize,
}

const MAX_TALKS_PER_NPC: u32 = 8;

impl HeuristicTracker {
    fn record_novelty(&mut self, kind: &str, id: &str, knowledge_now: usize) {
        let novel = match kind {
            "ability" => self.used_abilities.insert(id.to_owned()),
            "talk" => knowledge_now > self.last_knowledge_count,
            "examine" => self.examined_at.insert(id.to_owned()),
            "exit" => !self.visited.contains(id),
            _ => false,
        };
        self.last_knowledge_count = knowledge_now;
        if novel {
            self.stale_count = 0;
        } else {
            self.stale_count += 1;
        }
    }

    fn pick(
        &mut self,
        actions: &[esoteric_webb::session::AvailableAction],
        current_node: &str,
    ) -> Option<(String, String)> {
        if self.stale_count > 12 {
            return None;
        }
        // 1. Unexplored exits — highest priority
        for a in actions {
            if a.kind == "exit" && !self.visited.contains(&a.id) {
                return Some((a.kind.clone(), a.id.clone()));
            }
        }
        // 2. Unused abilities that aren't blocked
        for a in actions {
            if a.kind == "ability"
                && !self.used_abilities.contains(&a.id)
                && !a.detail.as_deref().unwrap_or("").starts_with("[blocked]")
            {
                return Some((a.kind.clone(), a.id.clone()));
            }
        }
        // 3. Talk to NPCs (up to MAX per NPC to build trust)
        for a in actions {
            if a.kind == "talk" {
                let count = self.talk_count.get(&a.id).copied().unwrap_or(0);
                if count < MAX_TALKS_PER_NPC {
                    *self.talk_count.entry(a.id.clone()).or_insert(0) += 1;
                    return Some((a.kind.clone(), a.id.clone()));
                }
            }
        }
        // 4. Examine if we haven't at this node
        if !self.examined_at.contains(current_node) {
            self.examined_at.insert(current_node.to_owned());
            return Some(("examine".to_owned(), "examine".to_owned()));
        }
        // 5. Rotate through exits — state may have opened new paths deeper
        let exits: Vec<_> = actions.iter().filter(|a| a.kind == "exit").collect();
        if !exits.is_empty() {
            let idx = self.exit_rotation % exits.len();
            self.exit_rotation += 1;
            let a = &exits[idx];
            return Some((a.kind.clone(), a.id.clone()));
        }
        None
    }
}

/// Scaffold a new content directory with template YAML.
pub fn cmd_new_world(output_path: &str) -> Result<(), String> {
    esoteric_webb::content::scaffold(output_path).map_err(|e| format!("scaffold: {e}"))
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
