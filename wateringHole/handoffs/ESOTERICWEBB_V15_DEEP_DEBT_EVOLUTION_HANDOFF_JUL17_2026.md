<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V15 — Deep Debt Evolution

- **Date**: 2026-07-17
- **Version**: V15
- **Direction**: Outbound (Webb → ecosystem)
- **Quality gates**: fmt ✓ clippy ✓ doc ✓ test ✓ (354 unit + 18 E2E + 1 validation = 373 total)

## Summary

V15 is a deep debt evolution pass focused on domain wiring, production mock
cleanup, hardcoded constant evolution, and new local science capabilities.
All changes are internal to Webb — no primal API changes requested.

## What changed

### New domain wiring (9 bridge methods added)

| Domain | Primal | Methods | Gap |
|--------|--------|---------|-----|
| crypto | bearDog | `crypto.sign`, `crypto.verify`, `crypto.hash` | GAP-019 resolved |
| mesh | songBird | `discovery.topology`, `discovery.health`, `discovery.query`, `discovery.bonds` | First milestone enabler |
| provenance | sweetGrass | `braid.create`, `braid.query` | Trio completion |

`DOMAIN_PRIMAL_MAP` expanded from 7 to 9 domains. All new methods degrade
gracefully when the target primal is unavailable.

### Hardcoded constant evolution

All `127.0.0.1` occurrences (3 sites in launcher.rs, discovery.rs) replaced
with `ipc::host_port()` / `ipc::default_host()`. Overridable via
`ESOTERICWEBB_DEFAULT_HOST` for container and Graphene deployments.

### Production mock cleanup

- Vestigial `SquirrelClient` struct removed (superseded by PrimalBridge)
- Vestigial `PetalTongueClient` struct removed (superseded by PrimalBridge)
- `LUDOSPRING` constant removed from `primal_names.rs`, replaced with
  `BEARDOG` + `SONGBIRD`
- All ludoSpring-specific doc comment references evolved to ecosystem-generic

### Offline voice interjection engine (GAP-007 partial)

New `science/voice.rs` module fires Disco Elysium-style internal voice
interjections based on game state predicates. Built-in profiles: Logic,
Empathy, Perception. Wired into the enrichment pipeline for every action.

### RulesetCert validation (GAP-009 partial)

`ContentBundle::validate_rulesets()` validates structural correctness of
loaded rulesets (required `plane`, `rules` array, per-rule `id`).

## Metrics delta

| Metric | V10 | V11 |
|--------|-----|-----|
| Tests | 338 | 354 (+16) |
| Domains | 7 | 9 |
| Bridge methods | 23 | 32 |
| Hardcoded localhost | 3 | 0 |
| Production mock structs | 2 | 0 |
| Rust files | 44 | 52 |

## Gaps status

| Gap | Status | Notes |
|-----|--------|-------|
| GAP-019 | **Resolved** | Crypto bridge wired |
| GAP-007 | Partial | Offline engine done, YAML authoring pending |
| GAP-009 | Partial | Structural validation done, typed model pending |

## Upstream action items for primal teams

### bearDog team
- Webb now calls `crypto.sign`, `crypto.verify`, `crypto.hash`. Please
  confirm method signatures align with bearDog's JSON-RPC surface.
- Signed provenance use case: `crypto.sign` on DAG vertices before
  `dag.event.append`.

### songBird team
- Webb now calls `discovery.topology`, `discovery.health`, `discovery.query`,
  `discovery.bonds`. These are the data source for the first milestone
  (static site rendering of ecosystem topology on `primals.eco/webb/`).
- Please confirm response schemas for topology and bond queries.

### sweetGrass team
- Webb now calls `braid.create` and `braid.query` for attribution tracking.
- Please confirm method availability and response format.

### biomeOS team
- GAP-017 (neural-api ZOMBIE) and GAP-018 (executors not exposed) remain
  open blockers for orchestrated composition.

## Open gaps (unchanged from V14)

GAP-002, GAP-003, GAP-004, GAP-006, GAP-008, GAP-010, GAP-017, GAP-018,
GAP-020, GAP-021, GAP-024 — see `EVOLUTION_GAPS.md` for details.
