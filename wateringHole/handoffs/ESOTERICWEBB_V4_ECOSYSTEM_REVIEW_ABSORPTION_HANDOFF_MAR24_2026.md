<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V4 — Ecosystem Review & Absorption Opportunities

- **Date**: 2026-03-24
- **Source**: Ecosystem-wide review of 8 sibling springs + primals
- **Direction**: Inbound — what Webb can absorb from ecosystem evolution

---

## Ecosystem State Snapshot (March 24, 2026)

| Spring | Version | Tests | Key Evolution |
|--------|---------|-------|---------------|
| ludoSpring | V30 | 675 | MCP tools, handler split, 91% coverage, deploy fragment |
| primalSpring | v0.7.0 | 303 | Multi-node bonding, federation, graph overlays, 87/87 gates |
| hotSpring | v0.6.32 | 848+ | Ember swap, GCN5, precision brain, coralReef sovereign compile |
| groundSpring | V118→V120 | 990+ | 8 RPC methods, proptests, PRNG migration, 92%+ coverage |
| neuralSpring | V120→S171 | 1356 | Cross-ecosystem absorption, OnceLock GPU probe, 227 tolerances |
| wetSpring | V132 | 1776 | Anderson hormesis, DeviceCapabilities, ValidationSink |
| airSpring | v0.10.0 | 946+ | 10 MCP ecology tools, NaN-safe floats, Kahan summation |
| healthSpring | V39→V41 | 848 | Toxicology, simulation, cross-spring hormesis, 85 capabilities |

## Patterns to Absorb

### From primalSpring v0.7.0

1. **Graph overlay composition** (`merge_graphs()`): Webb's deploy graph
   model can evolve to support runtime overlay merging — add provenance trio
   as an overlay to a base game graph without modifying the base.
2. **Squirrel cross-primal discovery**: env + XDG_RUNTIME_DIR scan pattern
   for discovering primals without hardcoded socket paths.
3. **`by_capability` on all graph nodes**: Webb's deploy graph nodes already
   use capability-based identity, consistent with primalSpring's model.
4. **5 BondTypes** (covalent/ionic/metallic/hydrogen/van-der-Waals): Webb
   could model primal composition strength using bonding vocabulary.

### From ludoSpring V30

1. **MCP tools with JSON Schema**: Already absorbed in V3.
2. **Deploy fragment** (`deploy/ludospring.toml`): Pattern for declaring
   spring capabilities as biomeOS deploy graph fragments.
3. **Session decomposition + typed transitions**: `TransitionIssue` enum
   for tracking session quality issues. Webb's `GameDirector` could adopt
   this for tracking narrative quality signals.

### From neuralSpring S171

1. **OnceLock GPU probe cache**: Thread-safe cached hardware discovery.
   Webb's `PrimalRegistry::discover()` could cache discovery results.
2. **`extract_rpc_result` helpers**: Utility for pulling typed results from
   JSON-RPC responses. Webb does this manually in bridge methods.
3. **PROVENANCE_REGISTRY**: Centralized provenance documentation for all
   baselines. Webb should document its content authoring provenance.

### From wetSpring V132

1. **`DeviceCapabilities` pattern**: Hardware-aware capability selection.
   Webb could use this for adaptive UI quality based on available hardware.
2. **`ValidationSink` plugin**: Webb's experiment harness already uses
   `check_bool`/`check_skip`; a sink plugin for structured output would
   improve CI integration.
3. **`PROVENANCE_REGISTRY` for baselines**: Documenting the creative
   provenance of content (inspiration sources, game design references).

### From airSpring v0.10.0

1. **NaN-safe float comparisons**: Webb handles flow scores and DDA metrics
   from primals — NaN guards would harden the enrichment pipeline.
2. **Kahan summation**: For accumulating provenance statistics across
   long game sessions without floating-point drift.

### From healthSpring V41

1. **Cross-spring hormesis pattern**: The idea that small perturbations
   strengthen systems. Webb could model this narratively — minor NPC
   betrayals increase player resilience / unlock new dialogue paths.

---

## Primal Phase1/Phase2 Review

### Deployed to plasmidBin/ (from phase2/)

| Primal | Version | Domain | Status |
|--------|---------|--------|--------|
| rhizoCrypt | v0.14.0-dev | dag | Metadata in plasmidBin, binary needs rebuild |
| loamSpine | v0.9.13 | lineage | Metadata in plasmidBin |
| sweetGrass | v0.7.27 | provenance | Metadata in plasmidBin |

### Available in ecosystem (not yet in plasmidBin/)

| Primal | Source | Domain | Webb interest |
|--------|--------|--------|---------------|
| Squirrel | phase1 | ai | High — narration + NPC dialogue |
| BearDog | phase2 | crypto | Medium — anti-cheat signing |
| Songbird | phase1 | discovery | Medium — capability-filtered queries |
| NestGate | phase1 | storage | Medium — game save persistence |
| toadStool | ecoPrimals | compute | Low (no GPU in game loop yet) |
| biomeOS | ecoPrimals | orchestration | Low (direct spawn via PrimalLauncher) |
| petalTongue | phase2 | visualization | High — scene rendering |

---

## What Webb Gives Back

1. **First gen4 validation**: Proof that primals compose into products
   without spring-level complexity.
2. **Gap pressure**: 7 open evolution gaps that drive primal improvement.
3. **Consumer patterns**: Bridge preservation, tiered degradation,
   PrimalEnrichment typing, deploy graph consumer implementation.
4. **Cross-domain validation**: Same DAG operations for game provenance,
   sample custody, and medical records (>80% structural similarity from
   ludoSpring experiments).
5. **Creative surface**: The sporeGarden organizational model for tools
   that compose primals into products.

---

## Action Items (Webb V5 roadmap)

| Priority | Action | Blocked on |
|----------|--------|------------|
| High | Absorb OnceLock discovery cache (neuralSpring pattern) | — |
| High | Add NaN-safe guards to enrichment pipeline (airSpring pattern) | — |
| Medium | Evolve deploy graphs to support overlay merging (primalSpring pattern) | — |
| Medium | Absorb extract_rpc_result helper pattern (neuralSpring) | — |
| Medium | Add content provenance documentation (wetSpring PROVENANCE_REGISTRY) | — |
| Low | Model bonding vocabulary for composition strength | primalSpring docs |
| Low | Narrative hormesis mechanics (healthSpring cross-spring pattern) | — |
