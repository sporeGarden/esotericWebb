<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Esoteric Webb V10 — Wave 46 Absorption Handoff

**Date:** May 23, 2026
**From:** esotericWebb (garden)
**To:** primalSpring (coordination), upstream springs, sibling gardens
**Context:** Webb absorbed Wave 46 patterns: env_keys centralization, typed
error validation, deploy graph Dark Forest metadata, primal.announce hint
alignment.

---

## What Changed in V10

### 1. env_keys Centralization

New `src/env_keys.rs` module (17 constants) mirrors primalSpring convention:

| Category | Constants |
|----------|-----------|
| Identity | `FAMILY_ID`, `BIOMEOS_FAMILY_ID` |
| XDG / OS | `XDG_RUNTIME_DIR`, `USER`, `UID` |
| Socket | `BIOMEOS_SOCKET_DIR`, `NEURAL_API_SOCKET` |
| Per-primal | `_ADDRESS`, `_JSONRPC_PORT`, `_HTTP_ADDRESS` suffixes |
| Webb-specific | `ESOTERICWEBB_SOCK`, `_IPC_TIMEOUT_SECS`, `_READINESS_TIMEOUT_SECS`, `_PORT_BASE`, `_SUMMARY_LIMIT`, `_JSON`, `_IPC_RETRY_*`, `_IPC_CB_*` |
| Deployment | `ECOPRIMALS_PLASMID_BIN`, `BIOMEOS_PLASMID_BIN_DIR` |

All 20+ bare `std::env::var("...")` calls across 9 files rewired to constants.

### 2. Deploy Graph Wave 46 Metadata

All 8 deploy graphs now carry:
- `secure_by_default = true` (Dark Forest Gate standard)
- `[graph.metadata]` with `owner`, `domain`, `wave` fields

Graphs versioned from 0.1.0 → 0.1.1 (1.0 → 1.1 for full composition).

### 3. primal.announce Wave 45 Alignment

`announce_self()` now includes:
```json
{
  "cost_hints": { "session.act": "low", "session.start": "medium", ... },
  "latency_estimates": { "session.act": "< 10ms", "session.start": "< 50ms", ... }
}
```

Aligned with Songbird/BearDog announce schema (Wave 45). Enables biomeOS
routing weight decisions for esotericWebb methods.

### 4. Typed Error System Validation

Webb's `IpcError` confirmed already `thiserror`-derived with:
- Semantic variants: `PrimalNotFound`, `ConnectionRefused`, `ConnectionReset`,
  `Timeout`, `ProtocolError`, `MethodNotFound`, `ApplicationError`, `Serialization`
- Classification: `is_retriable()`, `is_recoverable()`, `is_method_not_found()`,
  `is_connection_error()`
- `classify_io_error()` normalizes raw I/O errors

No further evolution needed — aligned with `PhasedIpcError` pattern since V5.

---

## Status vs Wave 46 Downstream Cleared

| Wave 46 Item | Webb Status |
|-------------|-------------|
| Typed error system | Already compliant (thiserror, semantic classification) |
| env_keys centralization | **DONE** V10 — 17 constants, zero bare strings |
| Deploy graph patterns | **DONE** V10 — 8/8 graphs with metadata |
| primal.announce hints | **DONE** V10 — cost_hints + latency_estimates |
| Registry 458 methods | Webb consumes (24 exposed, 458 ecosystem known) |
| NUCLEUS compositions | Webb validates Tower + Nest compositions locally |

## Items NOT yet available (acknowledged per audit)

| Item | Owner | Webb Impact |
|------|-------|-------------|
| FlockGate cross-WAN | cellMembrane | Blocks GAP-024 E2E validation |
| NeuralBridge BTSP auth | biomeOS | Webb uses api mode workaround |
| ludoSpring 6 IPC methods | ludoSpring | N/A — Webb absorbed game science locally (V6) |

---

## Resolved Gaps (V10)

| GAP | Description |
|-----|-------------|
| GAP-031 | env_keys centralization (20+ bare strings → env_keys.rs) |
| GAP-032 | Deploy graph Wave 46 metadata (secure_by_default, graph.metadata) |
| GAP-033 | primal.announce Wave 45 hints (cost_hints, latency_estimates) |

---

## Files Changed

| File | Change |
|------|--------|
| `webb/src/env_keys.rs` | New: centralized env var constants |
| `webb/src/lib.rs` | Added `pub mod env_keys` |
| `webb/src/niche.rs` | Rewired to env_keys constants |
| `webb/src/ipc/client.rs` | Rewired timeout to env_keys |
| `webb/src/ipc/bridge/mod.rs` | Rewired summary limit to env_keys |
| `webb/src/ipc/bridge/domains.rs` | announce_self() + cost_hints/latency_estimates |
| `webb/src/ipc/discovery.rs` | Rewired socket/plasmid/uid to env_keys |
| `webb/src/ipc/launcher.rs` | Rewired readiness/port/plasmid to env_keys |
| `webb/src/ipc/resilience.rs` | Rewired retry/CB config to env_keys |
| `webb/src/experiment.rs` | Rewired JSON mode to env_keys |
| `webb/src/bin/commands/mod.rs` | Rewired JSON mode to env_keys |
| `webb/src/bin/validate_all/main.rs` | Rewired JSON mode to env_keys |
| `graphs/*.toml` (8 files) | secure_by_default + [graph.metadata] |
| `EVOLUTION_GAPS.md` | 3 new gaps resolved (GAP-031–033) |
| `CHANGELOG.md` | V10 entry |
| `README.md` | V10 metrics |
