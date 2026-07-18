// SPDX-License-Identifier: AGPL-3.0-or-later
//! E2E demo scenario runner — replays guided walkthrough YAML against
//! a live session with primal composition.

use esoteric_webb::error::WebbError;
use esoteric_webb::ipc::bridge::PrimalBridge;
use esoteric_webb::session::types::ActionKind;
use esoteric_webb::session::GameSession;
use serde::{Deserialize, Serialize};

type Result<T> = esoteric_webb::error::Result<T>;

#[derive(Debug, Deserialize)]
struct DemoScenario {
    name: String,
    #[allow(dead_code)]
    description: String,
    steps: Vec<DemoStep>,
    verification: Verification,
}

#[derive(Debug, Deserialize)]
struct DemoStep {
    action: DemoAction,
    description: String,
    expected: StepExpectation,
}

#[derive(Debug, Deserialize)]
struct DemoAction {
    kind: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct StepExpectation {
    outcome_contains: Option<String>,
    #[serde(default)]
    enrichment: EnrichmentExpectation,
    #[serde(default)]
    state: StateExpectation,
}

#[derive(Debug, Default, Deserialize)]
struct EnrichmentExpectation {
    #[serde(default)]
    scene_pushed: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    voice_notes_possible: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct StateExpectation {
    current_node: Option<String>,
    knowledge_gained: Option<String>,
    knowledge_contains: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Verification {
    min_turns: u32,
    min_history: u32,
    required_knowledge: Vec<String>,
    min_scene_pushes: u32,
    final_node: String,
    zero_errors: bool,
}

#[derive(Debug, Serialize)]
struct DemoResult {
    scenario: String,
    passed: bool,
    steps_run: usize,
    steps_passed: usize,
    steps_failed: usize,
    verification_passed: bool,
    errors: Vec<String>,
    primal_status: Vec<String>,
}

#[expect(clippy::too_many_lines, reason = "linear E2E sequence, clarity over splitting")]
pub(super) fn run(content_path: &str, scenario_path: &str, json: bool) -> Result<()> {
    let yaml = std::fs::read_to_string(scenario_path).map_err(|e| WebbError::Other(
        format!("reading {scenario_path}: {e}")
    ))?;
    let scenario: DemoScenario = serde_yaml::from_str(&yaml)?;

    if !json {
        println!("=== E2E Demo: {} ===", scenario.name);
        println!();
    }

    let bridge = PrimalBridge::discover();
    let primal_status: Vec<String> = bridge
        .statuses()
        .iter()
        .map(|s| {
            let state = if s.healthy {
                "healthy"
            } else if s.discovered {
                "discovered"
            } else {
                "absent"
            };
            format!("  {} ({}): {}", s.domain, s.name, state)
        })
        .collect();

    if !json {
        println!("Primal composition:");
        for s in &primal_status {
            println!("{s}");
        }
        println!();
    }

    let mut session = GameSession::with_bridge(content_path, Some(bridge))?;

    let mut steps_passed = 0;
    let mut steps_failed = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut scene_pushes = 0u32;

    for (i, step) in scenario.steps.iter().enumerate() {
        let step_num = i + 1;
        if !json {
            print!("  Step {step_num}: {} ... ", step.description);
        }

        let kind = match ActionKind::parse(&step.action.kind) {
            Ok(k) => k,
            Err(e) => {
                let msg = format!("step {step_num}: invalid action kind '{}': {e}", step.action.kind);
                errors.push(msg.clone());
                steps_failed += 1;
                if !json {
                    println!("FAIL ({msg})");
                }
                continue;
            }
        };

        match session.act(kind, &step.action.id) {
            Ok((outcome, ctx)) => {
                let mut step_ok = true;

                if let Some(ref needle) = step.expected.outcome_contains {
                    let haystack = outcome.to_lowercase();
                    if !haystack.contains(&needle.to_lowercase()) {
                        let msg = format!(
                            "step {step_num}: outcome missing '{needle}'"
                        );
                        errors.push(msg);
                        step_ok = false;
                    }
                }

                if step.expected.enrichment.scene_pushed == Some(true) && ctx.enrichment.scene_pushed {
                    scene_pushes += 1;
                }

                if let Some(ref node) = step.expected.state.current_node {
                    let actual = session.current_node_id();
                    if actual != node {
                        let msg = format!(
                            "step {step_num}: expected node '{node}', got '{actual}'"
                        );
                        errors.push(msg);
                        step_ok = false;
                    }
                }

                if let Some(ref k) = step.expected.state.knowledge_gained {
                    if !ctx.knowledge.iter().any(|kk| kk.contains(k)) {
                        let msg = format!("step {step_num}: expected knowledge '{k}' not found");
                        errors.push(msg);
                        step_ok = false;
                    }
                }

                if let Some(ref k) = step.expected.state.knowledge_contains {
                    if !ctx.knowledge.iter().any(|kk| kk.contains(k)) {
                        let msg = format!("step {step_num}: expected knowledge '{k}' not found");
                        errors.push(msg);
                        step_ok = false;
                    }
                }

                if step_ok {
                    steps_passed += 1;
                    if !json {
                        println!("PASS");
                    }
                } else {
                    steps_failed += 1;
                    if !json {
                        println!("FAIL");
                    }
                }
            }
            Err(e) => {
                let msg = format!("step {step_num}: action error: {e}");
                errors.push(msg.clone());
                steps_failed += 1;
                if !json {
                    println!("FAIL ({msg})");
                }
            }
        }
    }

    let turn = session.turn();
    let history_len = session.history_len();
    let knowledge = session.knowledge();
    let current_node = session.current_node_id().to_string();

    #[expect(clippy::useless_let_if_seq, reason = "multiple sequential mutations")]
    let mut verification_passed = true;

    if turn < scenario.verification.min_turns {
        errors.push(format!(
            "verification: turn {turn} < min {}",
            scenario.verification.min_turns
        ));
        verification_passed = false;
    }
    if history_len < scenario.verification.min_history as usize {
        errors.push(format!(
            "verification: history {history_len} < min {}",
            scenario.verification.min_history
        ));
        verification_passed = false;
    }
    for k in &scenario.verification.required_knowledge {
        if !knowledge.iter().any(|kk| kk.contains(k.as_str())) {
            errors.push(format!("verification: required knowledge '{k}' not found"));
            verification_passed = false;
        }
    }
    if current_node != scenario.verification.final_node {
        errors.push(format!(
            "verification: expected final node '{}', got '{current_node}'",
            scenario.verification.final_node
        ));
        verification_passed = false;
    }
    if scene_pushes < scenario.verification.min_scene_pushes {
        errors.push(format!(
            "verification: scene_pushes {scene_pushes} < min {}",
            scenario.verification.min_scene_pushes
        ));
        verification_passed = false;
    }
    if scenario.verification.zero_errors && !errors.is_empty() {
        verification_passed = false;
    }

    let passed = steps_failed == 0 && verification_passed;

    let result = DemoResult {
        scenario: scenario.name.clone(),
        passed,
        steps_run: scenario.steps.len(),
        steps_passed,
        steps_failed,
        verification_passed,
        errors: errors.clone(),
        primal_status,
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"))
        );
    } else {
        println!();
        println!("=== Results ===");
        println!(
            "  Steps: {}/{} passed",
            steps_passed,
            scenario.steps.len()
        );
        println!("  Verification: {}", if verification_passed { "PASS" } else { "FAIL" });
        println!("  Scene pushes: {scene_pushes}");
        println!("  Turn: {turn}");
        if !errors.is_empty() {
            println!();
            println!("  Errors:");
            for e in &errors {
                println!("    - {e}");
            }
        }
        println!();
        if passed {
            println!("  DEMO: PASS ✓");
        } else {
            println!("  DEMO: FAIL ✗");
        }
    }

    if passed {
        Ok(())
    } else {
        Err(WebbError::Validation {
            count: errors.len(),
            summary: "demo scenario failed".into(),
        })
    }
}
