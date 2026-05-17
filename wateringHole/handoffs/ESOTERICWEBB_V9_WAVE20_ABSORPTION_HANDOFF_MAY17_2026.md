<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Esoteric Webb V9 — Wave 20-21 Absorption Handoff

**Date:** May 17, 2026
**From:** esotericWebb (garden)
**To:** primalSpring (coordination), upstream springs, sibling gardens
**Context:** Webb absorbed Wave 20 canonical schemas, stability tiers,
degradation behavior contracts, and trio partial completion tracking.

---

## What Changed in V9

### 1. Canonical Schema Compliance (Wave 20)

Webb's `capabilities.list` handler now emits the canonical envelope:

```json
{
  "capabilities": [{ "method": "...", "description": "..." }, ...],
  "count": 24,
  "primal": "esotericwebb"
}
```

Webb's `PrimalClient::capabilities()` normalizes responses from all primals
into this shape — wrapping raw arrays from pre-Wave-20 primals transparently.

### 2. Stability Tier Annotations

`capability_registry.toml` now annotates method groups with stability tiers:

| Group | Stability | Methods |
|-------|-----------|---------|
| health | stable | `health.liveness`, `health.readiness`, `health.check`, `health.version`, `health.drain` |
| identity | stable | `identity.get` |
| capabilities | stable | `capabilities.list` |
| lifecycle | stable | `primal.announce`, `primal.info` |
| webb | stable | `webb.health`, `webb.liveness`, `webb.readiness`, `webb.scene.current`, `webb.narrative.status`, `webb.content.list` |
| session | stable | `session.start`, `session.state`, `session.actions`, `session.act`, `session.history`, `session.narrate`, `session.graph` |
| tools | evolving | `tools.list`, `tools.call` |

### 3. Degradation Behavior Documentation

New `docs/DEGRADATION_BEHAVIOR.md` documents per-domain failure modes:

- 9 primal domain degradation contracts (ai, viz, dag, lineage, compute, storage, provenance, orchestration, discovery)
- Signal dispatch fallback table (`nest.store` → `dag.event.append`, `nest.commit` → `dag.session.complete`)
- Trio partial completion state table (Full / DAG+spine / DAG only / None)
- Standalone vs composition mode behavior
- Ecosystem invariant: "Gameplay is never gated behind primal availability"

### 4. Trio Partial Completion Tracking

`WorldState` now includes `primals_reached: Vec<String>` tracking which trio
primals responded during provenance operations. This follows
`PROVENANCE_TRIO_INTEGRATION_GUIDE.md` — no rollback on partial, consumer
decides acceptability.

---

## Feedback for Upstream Teams

### For primalSpring

- Webb's `capability_registry.toml` now mirrors the spring stability tier
  convention. The `[webb]` block in the primalSpring registry should note
  that Webb carries local stability annotations.
- Webb's canonical envelope compliance can be validated by `s_schema_standard`
  scenario expectations.

### For biomeOS

- GAP-017 (neural-api ZOMBIE in benchScale) remains open. Webb's signal
  dispatch fallback works, but E2E collapse is blocked until neural-api
  starts healthy.
- GAP-024 (E2E signal dispatch validation) depends on GAP-017 resolution.

### For trio primals (rhizoCrypt, loamSpine, sweetGrass)

- Webb now tracks `primals_reached` per session. When loamSpine and sweetGrass
  integration matures, Webb will add `"spine"` and `"braid"` to the tracking.
- Trio partial completion is handled per the integration guide — no domain
  logic failure on partial provenance.

### For sibling gardens (lithoSpore, projectFOUNDATION, projectNUCLEUS)

- The Wave 20 canonical envelope normalization pattern (`unwrap_capabilities_envelope`)
  may be useful for any consumer that needs to handle both pre- and post-Wave-20
  primals.
- The degradation behavior documentation format (`docs/DEGRADATION_BEHAVIOR.md`)
  follows the ecosystem ask from lithoSpore's primalSpring evolution handoff.

---

## Resolved Gaps (V9)

| GAP | Description |
|-----|-------------|
| GAP-026 | `capabilities.list` canonical Wave 20 envelope |
| GAP-027 | Stability tier annotations in capability registry |
| GAP-028 | Formal degradation behavior documentation |
| GAP-029 | Trio partial completion tracking in session state |
| GAP-030 | Bridge canonical envelope parsing for pre/post-Wave-20 primals |

---

## Files Changed

| File | Change |
|------|--------|
| `webb/src/ipc/handlers/lifecycle.rs` | `handle_capabilities_list()` emits canonical envelope with `count` |
| `webb/src/ipc/client.rs` | `unwrap_capabilities_envelope()` normalizes all responses |
| `webb/capability_registry.toml` | Stability tier annotations added per method group |
| `webb/src/state/mod.rs` | `primals_reached: Vec<String>` field added to `WorldState` |
| `webb/src/session/enrichment.rs` | `record_provenance_vertex()` tracks DAG reach |
| `docs/DEGRADATION_BEHAVIOR.md` | New: per-domain degradation contracts |
| `EVOLUTION_GAPS.md` | 6 new gaps (GAP-025–030) all resolved |
| `CHANGELOG.md` | V9 entry |
| `README.md` | V9 metrics |
| `whitePaper/baseCamp/ESOTERICWEBB_V9_EVOLUTION_AND_PATTERNS.md` | Patterns 11–14 added |
