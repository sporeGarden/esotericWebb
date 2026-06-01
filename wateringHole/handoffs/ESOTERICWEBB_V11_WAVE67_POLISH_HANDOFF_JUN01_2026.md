<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V11 — Wave 67 Polish

- **Date**: 2026-06-01
- **Wave**: 67
- **Direction**: Outbound (esotericWebb → ecosystem)
- **Gate**: ironGate

## Summary

Wave 67 polish pass for esotericWebb. No direct Wave 67 asks for gardens —
this is self-initiated debt resolution and vocabulary alignment.

## Changes

### Dead code removal

- **Deleted `ipc/provenance.rs`** — entirely superseded module. Old `provenance.*`
  method constants and unused `ProvenanceClient`/`ProvenanceVertex` types had zero
  importers. Production provenance flows through signal-first `nest.store`/`nest.commit`
  (V8 architecture) with `METHOD_DAG_*` constants from `ipc/mod.rs`.

### Vocabulary alignment

- Ecosystem vocabulary evolved `signal` → `composition` in Wave 67 (primalSpring v0.9.31).
  Wire names (`signal.dispatch`, `signal_tiers` JSON field) preserved as biomeOS contract.
  Updated: doc comments, `capability_registry.toml` descriptions, `EVOLUTION_GAPS.md`,
  README. Internal code references now say "composition" where describing architecture.

### Safety escalation

- `#![forbid(unsafe_code)]` added to `lib.rs`. Aligns with Wave 66-67 ecosystem standard
  (`#![forbid(unsafe)]` on all 88 spring crate roots). Webb already had zero unsafe blocks —
  this is compile-time enforcement.

### Ecosystem sync

- Registry reference updated: 458 → **490 methods** (primalSpring v0.9.31).
- `EVOLUTION_GAPS.md` GAP-004 and GAP-024 updated to reflect current architecture.
- Noted `s_nest_commit_live` scenario availability for GAP-024 validation.

## Metrics

| Metric | V10 | V11 |
|--------|-----|-----|
| Tests | 357 | 355 (2 dead tests removed with provenance.rs) |
| Rust files | 44 | 43 |
| Capabilities | 24 | 24 |
| Open gaps | 12 | 12 |
| Clippy | 0 warnings | 0 warnings |
| Unsafe | zero (unchecked) | zero (`#![forbid]` enforced) |
| Registry ref | 458 | 490 |

## Still-open upstream dependencies

| Dependency | Owner | Status |
|------------|-------|--------|
| 6 `game.*` IPC methods | ludoSpring | Open (Wave 55/63 ask) |
| `capability.call` RPC | biomeOS | P0 glacial blocker |
| Neural API healthy in benchScale | biomeOS | GAP-017, open |
| `nest.store`/`nest.commit` E2E | biomeOS | GAP-024, `s_nest_commit_live` available |

## For primalSpring audit

- Webb freshness hash `f514535` is now stale — new commit supersedes it.
- Webb remains ironGate's sole owned garden. No active impulses targeting Webb.
- Wave 67 glacial cutover items (TLS, auth, relay, sporePrint, Forgejo) are
  ironGate infrastructure — no esotericWebb code changes required.
