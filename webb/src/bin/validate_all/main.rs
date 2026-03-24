// SPDX-License-Identifier: AGPL-3.0-or-later
//! `validate_all` — meta-runner for all Esoteric Webb experiments.
//!
//! Runs each experiment crate in sequence, collects pass/fail/skip,
//! and prints a summary.

use std::process::Command;

/// All experiment package names, in order.
const EXPERIMENTS: &[&str] = &[
    "esotericwebb-exp001",
    "esotericwebb-exp002",
    "esotericwebb-exp003",
    "esotericwebb-exp004",
    "esotericwebb-exp005",
];

fn main() {
    println!("=== Esoteric Webb — validate_all ===");
    println!();

    let json_mode = std::env::var("ESOTERICWEBB_JSON")
        .ok()
        .is_some_and(|v| v == "1" || v == "true");

    let mut passed = 0u32;
    let mut failed = 0u32;
    let total = EXPERIMENTS.len();

    for &pkg in EXPERIMENTS {
        println!("--- {pkg} ---");

        let mut cmd = Command::new("cargo");
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

    println!("=== SUMMARY ===");
    println!("  {passed}/{total} passed, {failed} failed");

    if failed > 0 {
        std::process::exit(1);
    }
}
