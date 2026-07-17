// SPDX-License-Identifier: AGPL-3.0-or-later
//! Text-mode interactive preview engine.
//!
//! Drives the game director in a terminal loop without any primal IPC.
//! Used by `esotericwebb preview` for content authoring and testing.

use std::io::Write;

use esoteric_webb::content::ContentBundle;
use esoteric_webb::director::{DirectorOutcome, GameDirector, PlayerInput};
use esoteric_webb::state::WorldState;

/// Run the interactive preview loop until the player quits or reaches an ending.
pub(super) fn run(director: &mut GameDirector, state: &mut WorldState, bundle: &ContentBundle) {
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

    let scene_npcs = super::current_scene_npcs(bundle, director);
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
