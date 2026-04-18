<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V7 — Composition Patterns for Primal and Spring Evolution

- **Date**: 2026-04-17
- **Source**: esotericWebb V7 (sporeGarden / ecoPrimals / gardens)
- **Audience**: All primal teams, all spring teams, biomeOS orchestration, primalSpring coordination
- **Type**: Composition pattern evidence + per-primal learnings + NUCLEUS deployment patterns
- **Identity**: Webb is a gen4 **composition for deployment** — a CRPG that consumes primals via JSON-RPC IPC with zero Rust crate dependencies on any spring or primal.

---

## Executive Summary

V7 completes the deploy artifact alignment that V6 started. All stale
ludoSpring references have been removed from deploy graphs, niche
definitions, launch profiles, and source comments. The downstream manifest
entry now accurately reflects Webb's full composition surface: all four
NUCLEUS fragments, all 9 primal dependencies, and 10 validated capabilities
across all consumed domains.

This handoff captures what Webb learned about composing primals into a
product — patterns that every spring and garden will encounter as they
evolve toward NUCLEUS deployment via biomeOS.

---

## The Validation Ladder (What We Proved)

```
Layer 1: Python baselines (peer-reviewed science → golden JSON)
    ↓ ludoSpring validated these
Layer 2: Rust validation (barraCuda primitives → match Python within tolerances)
    ↓ ludoSpring proved 790+ tests, 3 validate_* binaries
Layer 3: IPC composition (JSON-RPC calls → match Rust library within tolerances)
    ↓ ludoSpring proved via validate_composition + composition_targets.json
Layer 4: Pure composition (gardens consume proven primals, validate composition)
    ↓ esotericWebb proves this — primals are invisible infrastructure
```

Gardens don't re-prove numerical correctness. That was done by the springs.
Gardens prove that the primal stack composes, degrades gracefully, and
delivers capabilities to product code.

---

## Per-Primal Learnings

### Squirrel (AI)

- **What works**: `ai.query` dispatches reliably via biomeOS; context injection
  (NPC personality, scene state) produces coherent narration; `ai.suggest` and
  `ai.analyze` are wired but untested at scale.
- **Gap**: `plasmidBin/squirrel/metadata.toml` advertises `ai.complete`,
  `ai.embed`, `ai.tools`, `mcp.serve` — not the biomeOS semantic method names
  Webb calls (`ai.query`, `ai.suggest`, `ai.analyze`). biomeOS translates, but
  metadata should document the semantic surface.
- **Action for neuralSpring**: Update Squirrel metadata to list both native
  and semantic method names. Consider a `capabilities.semantic_aliases` field.

### petalTongue (Visualization)

- **What works**: `visualization.render.scene` fires successfully when
  petalTongue is present; degradation to text-only mode is seamless.
- **Gap**: Metadata advertises `visualization.scene`, not
  `visualization.render.scene` (string mismatch). `interaction.poll` not
  listed in metadata at all.
- **Action for ludoSpring/petalTongue team**: Align capability strings in
  `plasmidBin/petaltongue/metadata.toml` with the `PRIMAL_IPC_PROTOCOL.md`
  naming convention.

### rhizoCrypt (DAG / Provenance)

- **What works**: `dag.session.create`, `dag.event.append`, `dag.frontier.get`,
  `dag.merkle.root` all exercise correctly in TCP composition tests.
- **Gap**: `dag.session.complete` not in metadata (but exists in implementation).
  `dag.query.vertices` vs `dag.vertex.query` naming mismatch.
- **Action for primalSpring/rhizoCrypt team**: Standardize method names per
  `SEMANTIC_METHOD_NAMING_STANDARD_V2.md`. Publish `dag.session.complete` in
  metadata.

### loamSpine (Lineage / Certificates)

- **What works**: `certificate.mint` is bridge-ready.
- **Gap**: Not yet exercised in production composition — only in exp004 TCP
  round-trip. No NPC personality certificate minting tested end-to-end.
- **Action**: Low priority. When Nest atomic matures, loamSpine certificates
  should integrate with rhizoCrypt session DAGs.

### sweetGrass (Attribution)

- **What works**: Bridge wiring is in place.
- **Gap**: `attribution.record` and `attribution.query` not yet exercised in
  any Webb experiment. The provenance trio TCP test (exp004) skips when
  plasmidBin binaries are absent.
- **Action**: Low priority. Wait for Nest atomic trio maturation.

### NestGate (Storage)

- **What works**: `storage.store` and `storage.retrieve` bridge-ready.
- **Gap**: Not exercised in production — game save/load uses local filesystem.
  NestGate integration planned for when content-addressed save states are needed.
- **Action**: Medium priority when save/load moves from filesystem to NestGate.

### ToadStool (Compute)

- **What works**: `compute.dispatch.submit` bridge-ready.
- **Gap**: No GPU compute path exercised in Webb — all game science is pure math
  at human-interaction speed. ToadStool would matter for procedural generation
  at scale or AI model dispatch.
- **Action**: Low priority. Monitor ludoSpring TensorSession evolution for
  patterns to absorb.

### BearDog (Crypto / Security)

- **What works**: Tower atomic forms the base of all deploy graphs; BearDog is
  the Phase 1 node in every composition.
- **Gap**: GAP-019 — `crypto.hash` not advertised in metadata; the `crypto`
  domain is not wired into Webb's PrimalBridge (bridge has 7 domains, crypto
  is not one of them because Tower handles it implicitly).
- **Action**: Clarify whether crypto should be a bridge domain or remains
  implicit via Tower. Update metadata either way.

### Songbird (Discovery)

- **What works**: Capability-based discovery works via XDG socket dirs and
  biomeOS convention. Phase 2 in all deploy graphs.
- **No gaps**: Discovery is the most mature primal from Webb's perspective.

---

## Composition Patterns for NUCLEUS Deployment

### Pattern 1: Fragment-Based Composition

Every deployment graph should compose from NUCLEUS fragments, not enumerate
individual primals. Webb's `downstream_manifest.toml` entry:

```toml
fragments = ["tower_atomic", "node_atomic", "nest_atomic", "meta_tier"]
depends_on = ["beardog", "songbird", "squirrel", "nestgate", "petaltongue",
              "toadstool", "rhizocrypt", "loamspine", "sweetgrass"]
```

Fragments provide the canonical primal set for each atomic tier. The
`depends_on` list is the explicit dependency surface — biomeOS uses it
for ordering and health-check sequencing.

### Pattern 2: Graceful Degradation per Domain

Webb's `PrimalBridge` tries each domain in order: direct TCP client →
UDS client → Neural API fallback. If all fail, the domain degrades silently.
This means:

- Tower primals (BearDog, Songbird) are **required** — Phase 1 must succeed
- Everything else is **optional** — the product works in text-only mode
  with local science if every primal is absent

This is the right model for gardens. Springs need stricter validation (they
must prove the math), but gardens should always be runnable standalone.

### Pattern 3: Neural API as Universal Fallback

When biomeOS's `neural-api` is present, Webb routes any unresolved
capability through `capability.call`. This means:

- New primals become available to Webb without code changes
- AI capabilities evolve transparently (neuralSpring → Squirrel → WGSL ML)
- The garden never needs to know which primal implements a capability

### Pattern 4: Local Science as Temporary Ownership

Webb absorbed flow, engagement, and DDA locally from ludoSpring patterns.
This is intentional — not tech debt. When a game-science primal emerges
(GAP-021), these local modules become IPC calls with zero API change
(the types already match IPC-shaped JSON-RPC responses).

This pattern applies to any garden: absorb what you need locally, shape the
types for IPC compatibility from day one, and the swap to primal composition
is mechanical when the primal matures.

### Pattern 5: Deploy Graph Phase Ordering

biomeOS executes graphs in phase order. The pattern that emerged:

1. **Tower** (crypto + discovery) — always Phase 1
2. **Node** (compute) — Phase 2 (optional for gardens)
3. **Nest** (storage + provenance) — Phase 3 (optional, fallback=skip)
4. **Meta-tier** (AI, viz) — Phase 4
5. **Product** (garden/spring) — last phase, depends_on everything it consumes
6. **Validation** — Phase 99 (health check on full stack)

---

## Actions for Receiving Teams

### primalSpring (coordination)

1. Update `downstream_manifest.toml` esotericwebb entry — **done in V7**
2. Add esotericwebb to `spring_validate_manifest.toml` as a garden entry
   (gardens validate composition, not science methods)
3. Document the garden validation pattern in `NICHE_STARTER_PATTERNS.md`

### All primal-producing springs

1. Audit `plasmidBin/*/metadata.toml` — ensure capability strings match the
   semantic naming standard and include all implemented methods
2. The V6 gap matrix (previous handoff) lists every naming mismatch Webb found

### neuralSpring

1. Update Squirrel metadata to reflect semantic method names alongside native
2. When neural-api health stabilizes (GAP-017), gardens gain transparent AI

### biomeOS

1. The fragment-based composition model works. `resolve = true` with fragment
   inheritance is the right pattern for both springs and gardens.
2. Consider a garden-specific graph loader that assumes optional dependencies
   and degradation by default.

---

## Downstream Manifest Alignment

```toml
# primalSpring/graphs/downstream/downstream_manifest.toml — esotericwebb entry
[[downstream]]
spring_name = "esotericwebb"
owner = "esotericWebb"
domain = "crpg"
particle_profile = "balanced"
fragments = ["tower_atomic", "node_atomic", "nest_atomic", "meta_tier"]
depends_on = ["beardog", "songbird", "squirrel", "nestgate", "petaltongue",
              "toadstool", "rhizocrypt", "loamspine", "sweetgrass"]
validation_capabilities = [
    "inference.complete", "inference.embed",
    "storage.store", "storage.retrieve",
    "crypto.hash",
    "dag.session.create", "dag.event.append",
    "certificate.mint",
    "visualization.render.scene",
    "compute.dispatch.submit",
]
```

---

*Filed from esotericWebb V7. Next handoff when Nest atomic trio matures or
GAP-021 (game-science primal) surfaces.*
