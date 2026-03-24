// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared validation harness for experiments.
//!
//! Provides `check_bool`, `check_skip`, and `finish` for consistent
//! pass/fail/skip reporting across all `expNNN_*` binaries. Mirrors
//! the pattern used by `primalSpring`'s experiment framework.
//!
//! ## JSON output
//!
//! Set `ESOTERICWEBB_JSON=1` to emit results as a JSON object instead
//! of human-readable text.

use std::sync::Mutex;

/// Global results accumulator.
static RESULTS: Mutex<Vec<CheckResult>> = Mutex::new(Vec::new());

/// A single check result.
#[derive(Debug, Clone)]
struct CheckResult {
    label: String,
    outcome: Outcome,
}

/// Outcome of a single check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pass,
    Fail,
    Skip,
}

/// Record a boolean check. Prints immediately and accumulates.
pub fn check_bool(label: &str, passed: bool) {
    let outcome = if passed { Outcome::Pass } else { Outcome::Fail };
    let icon = if passed { "PASS" } else { "FAIL" };
    if !json_mode() {
        println!("  [{icon}] {label}");
    }
    push(label, outcome);
}

/// Record a skipped check (honest scaffolding — primal unavailable).
pub fn check_skip(label: &str) {
    if !json_mode() {
        println!("  [SKIP] {label}");
    }
    push(label, Outcome::Skip);
}

/// Print a summary and return the exit code (0 = all pass, 1 = any fail).
pub fn finish(experiment_name: &str) -> i32 {
    let results = RESULTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let pass = results
        .iter()
        .filter(|r| r.outcome == Outcome::Pass)
        .count();
    let fail = results
        .iter()
        .filter(|r| r.outcome == Outcome::Fail)
        .count();
    let skip = results
        .iter()
        .filter(|r| r.outcome == Outcome::Skip)
        .count();

    let checks: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "label": r.label,
                "outcome": match r.outcome {
                    Outcome::Pass => "pass",
                    Outcome::Fail => "fail",
                    Outcome::Skip => "skip",
                }
            })
        })
        .collect();
    drop(results);

    if json_mode() {
        let output = serde_json::json!({
            "experiment": experiment_name,
            "pass": pass,
            "fail": fail,
            "skip": skip,
            "checks": checks,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!();
        println!("--- {experiment_name} ---");
        println!("  {pass} passed, {fail} failed, {skip} skipped");
        if fail > 0 {
            println!("  RESULT: FAIL");
        } else {
            println!("  RESULT: OK");
        }
    }

    i32::from(fail > 0)
}

/// Convenience: call `finish` and `std::process::exit`.
pub fn exit(experiment_name: &str) -> ! {
    std::process::exit(finish(experiment_name));
}

fn push(label: &str, outcome: Outcome) {
    if let Ok(mut results) = RESULTS.lock() {
        results.push(CheckResult {
            label: label.to_owned(),
            outcome,
        });
    }
}

fn json_mode() -> bool {
    std::env::var("ESOTERICWEBB_JSON")
        .ok()
        .is_some_and(|v| v == "1" || v == "true")
}
