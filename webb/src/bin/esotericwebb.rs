// SPDX-License-Identifier: AGPL-3.0-or-later
//! Esoteric Webb — UniBin entry point.
//!
//! Subcommands:
//! - `serve`     — start the full BYOB niche (germinate primals + game director)
//! - `validate`  — lint and validate a content directory
//! - `preview`   — text-mode game preview (human player)
//! - `autoplay`  — automated playthrough (AI-as-player demonstration)
//! - `graph`     — visualize the NarrativeGraph as DOT
//! - `replay`    — replay a provenance-traced session
//! - `new-world` — scaffold a new content directory with template YAML

mod commands;

use clap::{Parser, Subcommand};

/// Esoteric Webb — cross-evolution CRPG substrate.
#[derive(Parser)]
#[command(name = "esotericwebb", version, about)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Command,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Command {
    /// Start the full BYOB niche with game director and IPC server.
    Serve {
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
        /// Spawn primal binaries from plasmidBin before discovering.
        #[arg(long)]
        launch: bool,
        /// Deploy graph TOML for primal spawn ordering (requires --launch).
        #[arg(long, default_value = "graphs/webb_provenance_trio.toml")]
        graph: String,
    },
    /// Validate a content directory for correctness.
    Validate {
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
    },
    /// Text-mode interactive game preview (human player).
    Preview {
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
    },
    /// Automated playthrough — game plays itself via heuristic choices.
    Autoplay {
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
        /// Maximum turns before stopping.
        #[arg(long, default_value = "50")]
        max_turns: u32,
        /// Output the session as JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Visualize the NarrativeGraph as DOT format.
    ///
    /// Three views: bare narrative (default), played session (--played),
    /// or live session state (--live).
    Graph {
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
        /// Overlay a completed session (path to autoplay JSON).
        #[arg(long)]
        played: Option<String>,
        /// Start a live session and show its state.
        #[arg(long)]
        live: bool,
        /// Output format: "dot" (default) or "json" (structured 3D graph data).
        #[arg(long, default_value = "dot")]
        format: String,
    },
    /// Replay a provenance-traced session.
    Replay {
        /// Path to session trace file.
        #[arg(long)]
        session: String,
        /// Path to content directory.
        #[arg(long, default_value = "content")]
        content: String,
    },
    /// Scaffold a new content directory with template YAML.
    NewWorld {
        /// Path for the new content directory.
        #[arg(long)]
        output: String,
    },
    /// Show primal composition status — which primals are discovered and healthy.
    Status,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Serve {
            content,
            launch,
            graph,
        } => commands::cmd_serve(&content, launch, &graph),
        Command::Validate { content } => commands::cmd_validate(&content),
        Command::Preview { content } => commands::cmd_preview(&content),
        Command::Autoplay {
            content,
            max_turns,
            json,
        } => commands::cmd_autoplay(&content, max_turns, json),
        Command::Graph {
            content,
            played,
            live,
            format,
        } => commands::cmd_graph(&content, played.as_deref(), live, &format),
        Command::Replay { session, content } => commands::cmd_replay(&session, &content),
        Command::NewWorld { output } => commands::cmd_new_world(&output),
        Command::Status => commands::cmd_status(),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
