<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V5.1 — Audit Evolution, Module Refactoring, Primal Feedback

- **Date**: 2026-03-29
- **Source**: esotericWebb V5.1 (sporeGarden / ecoPrimals / gardens)
- **Audience**: All primal teams, all spring teams, ecosystem coordination
- **Type**: Evolution feedback + code quality patterns + primal-specific requests

---

## Executive Summary

V5.1 is a deep audit evolution pass that took V5 (329 tests, 90.84% coverage)
and resolved remaining technical debt: all `#[allow]` migrated to `#[expect]`,
two modules approaching 1000 LOC refactored smartly, hardcoded test ports
replaced with dynamic allocation, tautological experiment assertions fixed,
TCP E2E test suite expanded, and all documentation aligned to current state.

**Result:** 335 tests, 37 Rust files (~12.5k LOC), all 5 quality gates clean,
zero `#[allow]` in production, zero TODO/FIXME, zero unsafe.

---

## What Changed (V5 → V5.1)

### Code Quality

1. **`#[expect]` migration** — every lint suppression now has a mandatory
   `reason` string. Dead suppressions removed (not converted). Three categories:
   - Test modules: `#[expect(clippy::unwrap_used, reason = "test code")]`
   - Legitimate production: `#[expect(clippy::needless_pass_by_value, reason = "...")]`
   - Eliminated at source: `listener.rs` refactored to accept `&TcpStream`
     instead of owned value, removing the lint entirely

2. **Smart module refactoring:**
   - `content/mod.rs` (967 LOC) → `content/types.rs` (109) + `content/mod.rs` (873)
   - `ipc/bridge.rs` (943 LOC) → `bridge/mod.rs` (565) + `bridge/domains.rs` (396)
   - Bridge split uses Rust child-module visibility: domains.rs calls private
     call helpers from parent without widening visibility

3. **Dynamic port allocation** — experiments and integration tests use
   `TcpListener::bind("127.0.0.1:0")` for OS-assigned ephemeral ports

4. **Tautological assertion fixes** — `exp005` autoplay termination and
   `exp002` discovery registry checks corrected from always-true to genuine
   validation

### Testing

5. **TCP E2E suite** — 5 new tests exercising the real TCP listener path:
   `e2e_tcp_health`, `e2e_tcp_identity`, `e2e_tcp_capabilities`,
   `e2e_tcp_multiple_requests`, `e2e_tcp_session_lifecycle`

6. **Capability registry cross-validation** — test iterates all methods from
   `capability_registry.toml` and verifies none return "method not found"

### Documentation

7. All root docs, specs, CHANGELOG, README, CONTRIBUTING aligned to V5.1 state
8. `PAPER_REVIEW_QUEUE.md` created (ecosystem compliance)
9. experiments/README date corrected

---

## Primal-Specific Feedback

### petalTongue (visualization domain)

**GAP-002 remains open.** Webb defines `DialogueTreeScene` payloads but cannot
validate them against petalTongue's rendering capabilities. The `render_scene`
bridge method fires-and-forgets. When petalTongue gains CRPG dialogue tree
rendering (choice highlighting, voice interjection panels, skill check
display), Webb is ready to exercise it immediately.

**Request:** Confirm `visualization.render.scene` accepts a `scene_type:
"dialogue_tree"` payload with `choices`, `voice_notes`, and `skill_checks`
fields.

### Squirrel (AI domain)

**GAP-003 remains open.** Webb's NPC personality certs define hard constraints
(knowledge bounds, trust gates, lies with detection DCs). Current bridge calls
`ai.chat` with these as context, but enforcement is prompt-based, not
mechanical. Webb validates responses client-side as workaround.

**GAP-007 remains open.** `esotericwebb preview` cannot simulate voice
interjections without a running Squirrel instance. Offline voice simulation
using placeholder text from personality parameters is planned but self-owned.

**Request:** Accept NPC personality cert as structured constraint parameter on
`ai.chat` or `game.npc_dialogue`, with hard enforcement of knowledge bounds.

### rhizoCrypt / loamSpine / sweetGrass (provenance trio)

**GAP-004 — wiring complete, live validation pending.** All DAG lifecycle
methods (`dag.session.create`, `dag.event.append`, `dag.session.complete`,
`dag.frontier.get`, `dag.merkle.root`, `dag.query.vertices`) are wired into
the session pipeline. `certificate.mint` is bridge-ready for loamSpine.

**sweetGrass honesty note:** Webb's `domain::PROVENANCE` is discovered via
`primal_names` but no bridge methods exercise `attribution.record` yet. The
README and VISION_AND_EVOLUTION docs have been corrected to say "Discovered,
not yet exercised" rather than "Bridge ready."

**Request:** When provenance trio binaries are stable in plasmidBin, notify
Webb for live end-to-end integration testing. Next: `dag.slice.checkout` for
save/load and `dag.event.append_batch` for bulk import.

### Songbird (discovery domain)

**GAP-006 remains open.** Webb's `PrimalRegistry::discover()` uses filesystem
probes (tiers 1-4) but does not call `discovery.query` for tier-5 lookup.
This is logged as degraded but functional.

**Request:** Confirm `discovery.query` response format for capability-filtered
queries so Webb can implement tier-5 discovery.

### ludoSpring (game science domain)

**GAP-009 remains open.** Webb loads `rulesets/*.yaml` as opaque documents.
No structural validation against ludoSpring's RulesetCert schema exists.

**Request:** Publish RulesetCert JSON Schema or validation endpoint
(`game.ruleset_validate`) so Webb can validate ruleset YAML at content
authoring time.

### toadStool / nestGate / biomeOS

Bridge-ready, optional in deploy graphs. No new feedback from V5.1.

**GAP-010 remains open.** plasmidBin deployment automation is the primary
blocker for gen4 adoption — Webb can compose primals but cannot automatically
deploy them.

---

## Patterns Available for Absorption

These patterns emerged or matured in V5.1 and are available for adoption by
any primal producer or consumer:

1. **`#[expect]` over `#[allow]`** — dead suppression detection, mandatory reasons
2. **Directory modules for growing impl blocks** — child modules access parent
   private methods, new domains extend `domains.rs` without growing core
3. **Dynamic port allocation** — `bind("127.0.0.1:0")` in test infrastructure
4. **Capability registry cross-validation** — test that iterates all registered
   methods and verifies dispatch

---

## Ecosystem Handoff

A corresponding ecosystem-wide handoff has been filed at:
`ecoPrimals/infra/wateringHole/handoffs/ESOTERICWEBB_V51_AUDIT_EVOLUTION_PRIMAL_TEAM_HANDOFF_MAR29_2026.md`
