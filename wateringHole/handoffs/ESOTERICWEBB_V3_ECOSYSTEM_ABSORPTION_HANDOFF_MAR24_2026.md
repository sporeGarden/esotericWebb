<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<!-- Docs/creative content: CC-BY-SA-4.0 -->

> **ARCHIVE NOTE (V6)**: This handoff describes V3 architecture. V6 removed
> all spring runtime dependencies. Retained as evolution fossil record.

# Esoteric Webb V3 — Ecosystem Absorption Handoff

**Date**: 2026-03-24
**Version**: V3
**Previous**: V2 (2026-03-24)
**Author**: esotericWebb team (sporeGarden)

## Summary

V3 absorbs patterns from across the ecoPrimals ecosystem — ludoSpring V30,
primalSpring v0.7.0, neuralSpring V168b, and wetSpring V130 — to mature
Webb's IPC architecture, MCP compliance, and experiment framework.

## Changes

### IPC Handler Split (ludoSpring pattern)

Monolithic `server.rs` (461 LOC) split into domain-focused handler modules:

- `handlers/lifecycle.rs` — health, readiness, identity, capabilities
- `handlers/narrative.rs` — scene, narrative status, content listing
- `handlers/session.rs` — game session lifecycle (start, act, state, ...)
- `handlers/mcp.rs` — MCP tools.list / tools.call with JSON Schema
- `handlers/mod.rs` — thin dispatch entry

`server.rs` retained as backward-compatible re-export shim.

### MCP JSON Schema (ludoSpring pattern)

`tools.list` now returns MCP-compliant descriptors with typed `input_schema`
per tool (JSON Schema). `tools.call` routes all 14 exposed methods through
the same handlers used by JSON-RPC — zero duplicate logic.

### IPC Client Resilience (neuralSpring pattern)

New `resilience.rs` module with:

- `RetryPolicy` — configurable exponential backoff (env: `ESOTERICWEBB_IPC_RETRY_*`)
- `CircuitBreaker` — Closed/Open/HalfOpen with threshold and cooldown
  (env: `ESOTERICWEBB_IPC_CB_*`)
- `is_recoverable()` — classifies transient vs permanent IPC errors

All `PrimalBridge` domain calls now use `resilient_call()` with per-domain
circuit breakers. Degradation behavior unchanged for standalone mode.

### sourDough Compliance

- Added `identity.get` method (returns primal name, version, domain)
- Added `health.check` to capability registry
- Fixed `domain = "narrative"` (was `"game"`) in capability registry
- Aligned `deploy/esotericwebb.toml`, `capability_registry.toml`,
  `CONTEXT.md`, and `graphs/esotericwebb_full.toml` to full parity
- Removed phantom `webb.content.validate` from deploy fragment

### Experiment Harness Evolution (wetSpring Validator pattern)

- Added `section()` for structured output headers
- Added `finish_with_code() -> ExitCode` for clean unwinding
- Added `primal_or_skip()` helper for domain availability checks
- Added zero-test guard (empty suite = FAIL)
- Section markers excluded from pass/fail/skip counts

## Webb's Capability Surface (V3)

| Domain | Methods |
|--------|---------|
| sourDough | `health.liveness`, `health.readiness`, `health.check`, `identity.get`, `capabilities.list` |
| Health | `webb.health`, `webb.liveness`, `webb.readiness` |
| Narrative | `webb.scene.current`, `webb.narrative.status` |
| Content | `webb.content.list` |
| Session | `session.start`, `session.state`, `session.actions`, `session.act`, `session.history`, `session.narrate`, `session.graph` |
| MCP | `tools.list`, `tools.call` |

## Consumed Primal Capabilities

| Domain | Primal | Status | plasmidBin |
|--------|--------|--------|------------|
| ai | squirrel | degraded (IPC stubs) | not deployed |
| visualization | petaltongue | degraded (IPC stubs) | not deployed |
| compute | toadstool | degraded (IPC stubs) | not deployed |
| storage | nestgate | degraded (IPC stubs) | not deployed |
| game | ludospring | degraded (IPC stubs) | not deployed |
| dag | rhizocrypt | wired (session provenance) | v0.14.0-dev |
| lineage | loamspine | wired (session provenance) | v0.9.13 |
| provenance | sweetgrass | wired (session provenance) | v0.7.27 |

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace` | PASS (zero warnings) |
| `cargo test --workspace` | PASS (148 tests) |
| `cargo doc --workspace --no-deps` | PASS |
| `cargo deny check` | PASS |

## Deploy Graphs

8 TOML deploy graphs in `graphs/`, including `esotericwebb_full.toml`
(full BYOB stack with Tower, game, AI, viz, compute, provenance trio).

## Patterns Absorbed From

- **ludoSpring V30**: handler/domain split, MCP JSON Schema, deploy fragment
- **neuralSpring V168b**: `RetryPolicy` + `CircuitBreaker` resilience
- **primalSpring v0.7.0**: BYOB graph conventions, socket nucleation
- **wetSpring V130**: `Validator` structured accumulator, `ExitCode` finishers

## Next Steps

- Populate `plasmidBin/` with squirrel, petaltongue, ludospring, toadstool, nestgate
- Evolve IPC stubs to live integration as primals become available
- Add `tarpc-ipc` mirroring JSON-RPC (ludoSpring V30 pattern)
- Reach 90%+ line coverage (llvm-cov)

---

*ScyBorg Provenance Trio: AGPL-3.0-or-later (code) · ORC (game mechanics) · CC-BY-SA-4.0 (docs/creative)*
