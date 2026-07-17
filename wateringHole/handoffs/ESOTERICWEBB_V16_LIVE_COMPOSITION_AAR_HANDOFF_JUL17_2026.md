<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: V16 — Live Primal Composition AAR on flockGate

- **Date**: 2026-07-17
- **From**: esotericWebb (flockGate team)
- **Wave**: 147c
- **Version**: V16

## Summary

V16 validated live primal composition on flockGate for the first time.
Webb now discovers and composes with **6/9 primals** in real-time,
exercising the full BYOB pipeline: discovery -> session -> act ->
enrichment -> scene push. This handoff documents findings for upstream
primal teams.

## Metrics

| Metric | V15 | V16 |
|--------|-----|-----|
| Tests | 469 | 471 |
| Primals connected (flockGate) | 4/9 | 6/9 |
| Primals discovered | 4/9 | 8/9 |
| Experiments | 5 | 6 |
| Live composition validated | No | Yes |

## What V16 delivered

### Discovery reverse-mapping

`probe_directory()` now does two-pass lookup: domain name first, then
primal slug reverse-mapped via `DOMAIN_PRIMAL_MAP`. Unlocks primals
that register by name instead of domain.

**Unlocked**: rhizoCrypt (dag), loamSpine (lineage), toadStool (compute).

### Health check hardening

- `health_liveness()` sends `{}` instead of `null` params
- `-32602` (invalid params) treated as fallback trigger alongside `-32601`

**Unlocked**: squirrel (ai) — was alive but rejected `null` params.

### exp006_live_composition

New experiment: 19 pass, 0 fail, 3 skip. Exercises live discovery,
session with bridge, examine, navigate, enrichment pipeline.

### cmd_serve end-to-end validation

`session.start`, `session.state`, `session.act`, `session.history` all
validated over TCP IPC. Enrichment fires — petalTongue receives scenes,
flow score computed, knowledge gained through gameplay.

## AAR findings for upstream teams

### 1. Socket naming inconsistency (ALL primal teams)

**Issue**: Some primals register domain-named sockets (`visualization.sock`,
`ai.sock`), others register primal-named sockets (`rhizocrypt.sock`,
`loamspine.sock`, `toadstool.sock`). Consumers doing filesystem discovery
must handle both naming conventions.

**Webb workaround**: Two-pass lookup in `probe_directory()`.

**Recommendation**: Ecosystem convention should converge. Either all
primals use domain names, or all use primal names, or both symlinks are
created. `gate.enroll` could enforce the convention.

**GAP**: GAP-036

### 2. squirrel params strictness (squirrel team)

**Issue**: `health.liveness` with `null` params returns `-32602` (invalid
params). JSON-RPC 2.0 allows `null` for "no params" but squirrel's
handler requires a structured value (object or array).

**Webb workaround**: Send `{}` for health checks; treat `-32602` as
fallback trigger.

**Recommendation**: Accept `null` params for health methods, or document
the requirement for structured params.

### 3. songBird HTTP transport (songBird team)

**Issue**: songBird on TCP 7780 speaks HTTP, not raw NDJSON JSON-RPC.
Webb's `PrimalClient` (and the ecosystem-standard sourDough pattern)
sends newline-delimited JSON over raw TCP. Sending NDJSON to songBird
returns `HTTP/1.1 400 Bad Request`.

**Webb workaround**: songBird mesh domain degrades gracefully. Env var
`SONGBIRD_JSONRPC_PORT=7780` enables discovery but not health/calls.

**Recommendation**: Expose a raw JSON-RPC endpoint alongside HTTP, per
sourDough convention. All other primals (squirrel, petaltongue, nestgate,
sweetgrass, beardog, loamspine) work with NDJSON.

**GAP**: GAP-037

### 4. Stale UDS sockets (rhizoCrypt, toadStool teams)

**Issue**: `rhizocrypt.sock` and `toadstool.sock` exist on disk but
`connect()` returns ECONNREFUSED. Primal processes are not running but
socket files were not cleaned up.

**Impact**: Webb discovers them as "found" but correctly classifies as
unhealthy. No false positives, but wasted discovery I/O.

**Recommendation**: Trap SIGTERM/SIGINT and unlink socket on shutdown.
Or have biomeOS gc sockets for non-running primals.

**GAP**: GAP-038

## Connected primals (flockGate, V16)

| Domain | Primal | Transport | Health |
|--------|--------|-----------|--------|
| ai | squirrel | UDS (`ai.sock`) | healthy (V16 fix) |
| visualization | petaltongue | UDS (`petaltongue.sock`) | healthy |
| storage | nestgate | UDS (`nestgate.sock`) | healthy |
| lineage | loamspine | UDS (`loamspine.sock`) | healthy |
| provenance | sweetgrass | UDS (`sweetgrass.sock`) | healthy |
| crypto | beardog | UDS (`beardog.sock`) | healthy |
| dag | rhizocrypt | UDS (`rhizocrypt.sock`) | stale socket |
| compute | toadstool | UDS (`toadstool.sock`) | stale socket |
| mesh | songbird | TCP 7780 (HTTP) | transport mismatch |

## Files changed

- `webb/src/ipc/discovery.rs` — probe_directory reverse-mapping
- `webb/src/ipc/client.rs` — health_liveness params + invalid params fallback
- `experiments/exp006_live_composition/` — new live composition experiment
- `Cargo.toml` — workspace member
- `CHANGELOG.md`, `README.md`, `EVOLUTION_GAPS.md` — V16 updates
