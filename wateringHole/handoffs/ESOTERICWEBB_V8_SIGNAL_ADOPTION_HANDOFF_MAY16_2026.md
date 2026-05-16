<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V8 — Wave 17 Signal Adoption + Deep Debt Resolution

- **Date**: 2026-05-16
- **Source**: esotericWebb V8 (garden)
- **Direction**: Outbound (Webb → primal/spring teams)
- **Audience**: primalSpring, biomeOS, Squirrel, petalTongue, ludoSpring, Songbird, rhizoCrypt, loamSpine, sweetGrass, NestGate

---

## Summary

Esoteric Webb V8 absorbs Wave 17 signal adoption from primalSpring, wires
signal dispatch into the enrichment pipeline, completes `primal.announce`
outbound self-registration, and performs deep debt resolution including
smart refactoring, niche capability expansion, and documentation cleanup.

357 tests pass. Zero clippy warnings. Zero unsafe. Zero `#[allow()]`.
All production files under 800 LOC. `cargo deny check` PASS.

---

## What Webb Absorbed

### Signal Dispatch (Wave 17)

Webb's enrichment pipeline now routes provenance through signal dispatch:

| Operation | V7 (direct) | V8 (signal-first) |
|-----------|-------------|-------------------|
| Record vertex | `bridge.dag_event_append()` | `bridge.nest_store()` → falls back to `dag.event.append` |
| Complete session | `bridge.dag_session_complete()` | `bridge.nest_commit()` → falls back to `dag.session.complete` |

When biomeOS Neural API is available, `nest.store` collapses 4 IPC calls
(content.put → dag.event.append → spine.seal → braid.create) into one
atomic signal. When unavailable, the existing direct path executes.

### Self-Announcement (primal.announce)

`cmd_serve` now calls `primal.announce` at startup, broadcasting 24
capabilities and the UDS socket path to biomeOS. This enables push-pull
discovery — other primals can find Webb without filesystem probing.

### Lifecycle Handlers

Webb now handles inbound:
- `health.version` — detailed version, build target, signal tier
- `health.drain` — acknowledges graceful shutdown
- `primal.announce` — accepts registration from other primals
- `primal.info` — returns niche metadata

### Capability Expansion

`niche::CAPABILITIES` expanded from 20 to 24 methods. Cross-validated
against `capability_registry.toml` via automated test.

---

## What Webb Learned (Feedback for Upstream)

### For biomeOS

1. **Signal dispatch E2E not yet validated** (GAP-024). Webb has the
   signal-first code paths but hasn't exercised them against a live
   biomeOS Neural API. The fallback path is proven (all 357 tests pass
   in standalone). Request: exercise the `nest.store` and `nest.commit`
   signal graphs against ironGate NUCLEUS to validate the full chain.

2. **Neural API socket discovery**. Webb resolves via
   `NEURAL_API_SOCKET` env → `neural-api-{family_id}.sock` in XDG dirs.
   Confirm this matches biomeOS's actual socket naming on ironGate.

### For Squirrel

3. **Mechanical constraint passing** (P1). Webb's enrichment pipeline
   sends AI prompts without structured game mechanical context. The
   narration ignores dice results, ability costs, and cooldowns.
   Request: accept a `context.mechanical` field in `ai.query` params
   containing `resolved_predicates`, `ability_costs`, `dice_results`.

4. **Post-narration voice check**. Once ludoSpring exposes
   `game.voice_check`, Webb wants to validate Squirrel's narration
   against game state. This is the agentic DM pattern — AI generates,
   game science validates.

### For petalTongue

5. **DialogueTree scene type** (GAP-002, still open). Webb pushes
   `SceneType::Dialogue` scenes but petalTongue doesn't render branching
   NPC conversation trees. Request: `SceneType::DialogueTree` with
   branch visualization, NPC portraits, ability check indicators.

### For ludoSpring

6. **6-method IPC expansion** (ludoSpring's P0). Webb's bridge code is
   wired for: `game.narrate_action`, `game.npc_dialogue`,
   `game.voice_check`, `game.push_scene`, `game.begin_session`,
   `game.complete_session`. Once ludoSpring exposes these, the connection
   activates automatically via capability discovery.

### For Songbird

7. **Filtered discovery** (GAP-006). Webb currently discovers primals via
   filesystem probing + env vars. Request: `discovery.query({ capabilities:
   ["game.*"] })` for capability-filtered discovery. This is tier-5 in
   Webb's discovery chain.

### For rhizoCrypt / loamSpine / sweetGrass (Provenance Trio)

8. **Provenance E2E session** (P0). The full storytelling provenance loop
   (`session.create → event.append per scene → lineage.branch on player
   choice → braid.create for AI narration → lineage.certify on session
   end`) can now be exercised on ironGate against live NUCLEUS. Webb has
   the bridge wiring; the primals just need to be running.

### For primalSpring (Ecosystem)

9. **Signal adoption is transparent**. The signal-first-with-fallback
   pattern works cleanly. No changes to error handling or degradation
   needed. Other gardens should adopt the same `nest_store()`/`nest_commit()`
   pattern once biomeOS is validated.

10. **Self-announcement at startup is trivial**. The `primal.announce`
    pattern (list methods + socket path, call via Neural API, degrade
    silently) took 30 lines. Every garden/primal serving IPC should adopt.

---

## Composition Patterns for NUCLEUS Deployment

### The Signal-First Pattern

```rust
pub fn nest_store(&mut self, payload: &serde_json::Value) -> Result<bool, IpcError> {
    if self.has_neural_api() {
        // Atomic signal — biomeOS orchestrates the full 4-call sequence
        self.dispatch_signal(SIGNAL_NEST_STORE, payload)?;
        return Ok(true);
    }
    // Fallback — direct domain call
    self.dag_event_append(payload)?;
    Ok(false)
}
```

### The Announce-at-Startup Pattern

```rust
fn announce_to_biomeos(bridge: &mut PrimalBridge, sock: &Path) {
    const METHODS: &[&str] = &["session.start", "session.act", ...];
    bridge.announce_self(&sock.display().to_string(), METHODS);
}
```

### The Enrichment Pipeline (6 phases, all degrading)

```
player action → director resolves outcome
  → Phase 1: AI narration (Squirrel ai.query — fallback: placeholder)
  → Phase 2: NPC dialogue (Squirrel ai.query — fallback: none)
  → Phase 3: Flow evaluation (local science — no IPC)
  → Phase 4: Scene push (petalTongue — fallback: silent)
  → Phase 5: nest.store (provenance vertex — fallback: dag.event.append)
  → Phase 6: nest.commit (session finalize — fallback: dag.session.complete)
→ narration context returned to caller
```

---

## Quality Metrics (V8)

| Metric | V7 | V8 |
|--------|----|----|
| Tests | 342 | 357 |
| Production LOC max | 891 | 764 |
| Capabilities | 20 | 24 |
| Bridge methods | 19 | 22 |
| Signal methods | 0 | 3 (nest.store, nest.commit, announce_self) |
| GAPs resolved | — | GAP-025 |
| GAPs filed | — | GAP-024 |

---

## Files Changed (V8)

```
CHANGELOG.md                        — V8 entry
EVOLUTION_GAPS.md                   — GAP-024 (new), GAP-025 (resolved)
webb/capability_registry.toml       — 4 new methods
webb/src/bin/commands/mod.rs         — announce_to_biomeos, mut bridge
webb/src/content/mod.rs              — test extraction (873→290 LOC)
webb/src/content/tests.rs            — extracted tests (new file)
webb/src/ipc/bridge/domains.rs       — nest_store, nest_commit, announce_self
webb/src/ipc/bridge/mod.rs           — tests for signal dispatch
webb/src/ipc/handlers/lifecycle.rs   — health.version, health.drain, primal.announce, primal.info
webb/src/ipc/handlers/mod.rs         — dispatch routing for new methods
webb/src/ipc/mod.rs                  — signal + lifecycle constants
webb/src/narrative/mod.rs            — clippy fix (sort_by_key)
webb/src/niche.rs                    — 24 capabilities, cross-validation test
webb/src/session/mod.rs              — test extraction (891→425 LOC)
webb/src/session/tests.rs            — extracted tests (new file)
webb/src/session/enrichment.rs       — nest_store/nest_commit pipeline
```
