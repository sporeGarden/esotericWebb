// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared validation harness for experiments (wetSpring `Validator` pattern).
//!
//! Provides a structured accumulator with section headers, pass/fail/skip
//! semantics, and `ExitCode` finishers for consistent reporting across all
//! `expNNN_*` binaries.
//!
//! ## JSON output
//!
//! Set `ESOTERICWEBB_JSON=1` to emit results as a JSON object instead
//! of human-readable text.

use std::process::ExitCode;
use std::sync::Mutex;

/// Global results accumulator.
static RESULTS: Mutex<Vec<CheckResult>> = Mutex::new(Vec::new());

/// A single check result.
#[derive(Debug, Clone)]
struct CheckResult {
    section: Option<String>,
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

/// Print a section header to structure output.
pub fn section(name: &str) {
    if !json_mode() {
        println!();
        println!("── {name} ──");
    }
    if let Ok(mut results) = RESULTS.lock() {
        results.push(CheckResult {
            section: Some(name.to_owned()),
            label: String::new(),
            outcome: Outcome::Pass,
        });
    }
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

/// Skip the experiment if a primal domain is not available.
///
/// Returns `Some(ExitCode)` if the primal is missing (caller should return
/// it early), `None` if the primal is available.
#[must_use]
pub fn primal_or_skip(domain: &str, bridge: &crate::ipc::bridge::PrimalBridge) -> Option<ExitCode> {
    if bridge.has(domain) {
        None
    } else {
        check_skip(&format!(
            "{domain} primal not available — skipping experiment"
        ));
        Some(finish_with_code("skipped"))
    }
}

/// Print a summary and return the exit code (0 = all pass, 1 = any fail).
///
/// Fails if no checks were recorded (guards against silent empty suites).
#[must_use]
pub fn finish(experiment_name: &str) -> i32 {
    let (pass, fail, skip, checks) = collect_results();

    if json_mode() {
        print_json(experiment_name, pass, fail, skip, &checks);
    } else {
        print_human(experiment_name, pass, fail, skip);
    }

    if pass + fail + skip == 0 {
        if !json_mode() {
            println!("  WARNING: no checks recorded — treating as FAIL");
        }
        return 1;
    }
    i32::from(fail > 0)
}

/// Return an `ExitCode` for use with `fn main() -> ExitCode`.
///
/// Preferred over `finish()` + `std::process::exit()` for cleaner stack
/// unwinding and `Drop` execution.
#[must_use]
pub fn finish_with_code(experiment_name: &str) -> ExitCode {
    let code = finish(experiment_name);
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Convenience: call `finish` and `std::process::exit`.
pub fn exit(experiment_name: &str) -> ! {
    std::process::exit(finish(experiment_name));
}

fn push(label: &str, outcome: Outcome) {
    if let Ok(mut results) = RESULTS.lock() {
        let current_section = results.iter().rev().find_map(|r| r.section.clone());
        results.push(CheckResult {
            section: current_section,
            label: label.to_owned(),
            outcome,
        });
    }
}

fn collect_results() -> (usize, usize, usize, Vec<serde_json::Value>) {
    let results = RESULTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let actual: Vec<_> = results.iter().filter(|r| !r.label.is_empty()).collect();
    let pass = actual.iter().filter(|r| r.outcome == Outcome::Pass).count();
    let fail = actual.iter().filter(|r| r.outcome == Outcome::Fail).count();
    let skip = actual.iter().filter(|r| r.outcome == Outcome::Skip).count();

    let checks: Vec<serde_json::Value> = actual
        .iter()
        .map(|r| {
            serde_json::json!({
                "label": r.label,
                "section": r.section,
                "outcome": match r.outcome {
                    Outcome::Pass => "pass",
                    Outcome::Fail => "fail",
                    Outcome::Skip => "skip",
                }
            })
        })
        .collect();

    drop(results);
    (pass, fail, skip, checks)
}

fn print_json(name: &str, pass: usize, fail: usize, skip: usize, checks: &[serde_json::Value]) {
    let output = serde_json::json!({
        "experiment": name,
        "pass": pass,
        "fail": fail,
        "skip": skip,
        "checks": checks,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}

fn print_human(name: &str, pass: usize, fail: usize, skip: usize) {
    println!();
    println!("--- {name} ---");
    println!("  {pass} passed, {fail} failed, {skip} skipped");
    if fail > 0 {
        println!("  RESULT: FAIL");
    } else {
        println!("  RESULT: OK");
    }
}

fn json_mode() -> bool {
    std::env::var(crate::env_keys::ESOTERICWEBB_JSON)
        .ok()
        .is_some_and(|v| v == "1" || v == "true")
}

#[cfg(test)]
#[expect(clippy::significant_drop_tightening, reason = "test mutex patterns")]
mod tests {
    use super::*;

    /// Serialize experiment tests — they share a global accumulator.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn locked<F: FnOnce()>(f: F) {
        let _guard = TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        RESULTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        f();
    }

    #[test]
    fn check_bool_pass_accumulates() {
        locked(|| {
            check_bool("test pass", true);
            let guard = RESULTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(
                guard
                    .iter()
                    .any(|r| r.outcome == Outcome::Pass && r.label == "test pass")
            );
        });
    }

    #[test]
    fn check_bool_fail_accumulates() {
        locked(|| {
            check_bool("test fail", false);
            let guard = RESULTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(guard.iter().any(|r| r.outcome == Outcome::Fail));
        });
    }

    #[test]
    fn check_skip_accumulates() {
        locked(|| {
            check_skip("primal unavailable");
            let guard = RESULTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(guard.iter().any(|r| r.outcome == Outcome::Skip));
        });
    }

    #[test]
    fn finish_returns_zero_on_all_pass() {
        locked(|| {
            check_bool("a", true);
            check_bool("b", true);
            check_skip("c");
            assert_eq!(finish("test_experiment"), 0);
        });
    }

    #[test]
    fn finish_returns_one_on_failure() {
        locked(|| {
            check_bool("pass", true);
            check_bool("fail", false);
            assert_eq!(finish("test_experiment_fail"), 1);
        });
    }

    #[test]
    fn finish_fails_on_empty_suite() {
        locked(|| {
            assert_eq!(finish("empty_suite"), 1);
        });
    }

    #[test]
    fn section_does_not_count_as_check() {
        locked(|| {
            section("graph tests");
            check_bool("edge count", true);
            let (pass, fail, skip, _) = collect_results();
            assert_eq!(pass, 1);
            assert_eq!(fail, 0);
            assert_eq!(skip, 0);
        });
    }

    #[test]
    fn finish_with_code_returns_success() {
        locked(|| {
            check_bool("ok", true);
            let code = finish_with_code("test_exit_code");
            assert_eq!(code, ExitCode::SUCCESS);
        });
    }

    #[test]
    fn finish_with_code_returns_failure() {
        locked(|| {
            check_bool("bad", false);
            let code = finish_with_code("test_exit_code_fail");
            assert_eq!(code, ExitCode::FAILURE);
        });
    }
}
