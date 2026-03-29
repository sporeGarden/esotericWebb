<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

> **ARCHIVE NOTE (V6)**: This handoff describes V5.1 architecture which
> included a ludoSpring GAME domain dependency. V6 decomposed ludoSpring
> entirely — game science is now local (`science/`), AI methods realigned to
> `ai.query`/`ai.suggest`/`ai.analyze`, and no spring runtime dependencies
> remain. GAP-016 is superseded, GAP-022 resolved. See CHANGELOG.md V6 entry.

# HANDOFF: Esoteric Webb V5.1 — Use-Case Gaps, Audit Evolution, Primal Feedback

- **Date**: 2026-03-29 (updated: use-case gap pass)
- **Source**: esotericWebb V5.1 (sporeGarden / ecoPrimals / gardens)
- **Audience**: All primal teams, all spring teams, ecosystem coordination
- **Type**: Use-case failure evidence + evolution feedback + code quality patterns

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

## Use-Case Gaps (Spring Validation Gaps → Primal Debt)

Webb is the use-case layer. When a composition doesn't work, that's a spring
validation gap. Spring validation gaps find primal debt. These gaps were
discovered by attempting real compositions against live primals in benchScale
topologies and by analyzing the ecosystem codebase.

### GAP-016: ludoSpring UDS-only transport blocks container composition

Webb → ludoSpring fails in containers and benchScale. Webb is TCP-first
(UniBin v1.2). ludoSpring V32 only listens on UDS. The "play a storytelling
session" use case fails at transport before method dispatch.

**Severity:** high — blocks all `game.*` composition in non-local deployments.

**Action for ludoSpring team:** Implement `--listen addr:port` (UniBin v1.2 TCP
listener). Webb, beardog, songbird, and all springs in benchScale already
support TCP. ludoSpring is the last holdout.

### GAP-017: biomeOS neural-api fails to start in benchScale

In benchScale `tower-2node`, beardog and songbird come up `LIVE`, biomeOS
`neural-api` is `ZOMBIE`. This blocks all graph-based orchestration use cases.
Webb cannot test composition graphs routed through neural-api.

**Severity:** critical — blocks the entire "biomeOS orchestrates primals" use case.

**Action for biomeOS team:** Investigate neural-api startup failure in Docker.
The ZOMBIE status means it started but failed health check. Likely a socket
bind, dependency, or configuration issue in the container environment.

### GAP-018: neuralAPI executors not on JSON-RPC

Webb's storytelling loop is a continuous execution graph. biomeOS has
`ConditionalDag`, `Pipeline`, `ContinuousExecutor`, and `PathwayLearner`
internally but none are exposed as JSON-RPC methods. Only basic
`graph.execute` → `graph.status` → `graph.result` is available.

**Severity:** high — blocks adaptive/continuous/pipeline compositions.

**Action for biomeOS team:** Expose `ContinuousExecutor` session management,
`ConditionalDag` branching, `Pipeline` chaining, and `PathwayLearner`
learn/suggest on JSON-RPC. These are the gates to "E2E neuralAPI workflows."

### GAP-020: Deploy graph format divergence

Webb ships TOML fragments, biomeOS uses JSON graph definitions internally.
No formal schema or cross-validation tooling exists.

**Severity:** low — currently manual alignment works.

**Action for primalSpring / wateringHole:** Define canonical deploy fragment
schema. `primalSpring validate-graph` would catch mismatches before deployment.

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

**GAP-016 is the critical blocker.** Webb cannot compose with ludoSpring in
any non-local environment because ludoSpring is UDS-only. All `game.*`
enrichment degrades to mechanical defaults in benchScale, Docker, and
cross-host topologies.

**GAP-009 remains open.** Webb loads `rulesets/*.yaml` as opaque documents.
No structural validation against ludoSpring's RulesetCert schema exists.

**Requests:**
1. **TCP listener** (`--listen addr:port`, UniBin v1.2) — this is the P1 gate
2. Publish RulesetCert JSON Schema or `game.ruleset_validate` endpoint

### beardog (crypto domain)

**GAP-019: self-owned.** Webb needs to wire `crypto.sign`, `crypto.verify`,
and `crypto.hash` into PrimalBridge for signed provenance. beardog V4 is
ready with Ed25519, SHA-256, post-quantum, HSM abstraction. No blocker on
beardog's side.

### toadStool / nestGate

Bridge-ready, optional in deploy graphs. No new feedback from V5.1.

### biomeOS

**GAP-017 is the critical blocker.** neural-api doesn't start healthy in
benchScale. Blocks all graph-based orchestration.

**GAP-018 is the next gate.** Once neural-api is healthy, Webb needs
`ContinuousExecutor`, `ConditionalDag`, `Pipeline`, and `PathwayLearner`
on JSON-RPC to build real E2E neuralAPI workflows.

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
5. **`niche.rs` self-knowledge** — centralized identity constants, capability
   arrays, family-scoped socket resolution, neural-api discovery. Absorbed from
   ludoSpring V32 pattern. Every composition and primal should have a single
   niche module with no IPC dependencies.

---

## Ecosystem Handoff

A corresponding ecosystem-wide handoff has been filed at:
`ecoPrimals/infra/wateringHole/handoffs/ESOTERICWEBB_V51_AUDIT_EVOLUTION_PRIMAL_TEAM_HANDOFF_MAR29_2026.md`
