<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: Esoteric Webb V6 — Primal Capability Gaps for Spring Evolution

- **Date**: 2026-03-29
- **Source**: esotericWebb V6 (sporeGarden / ecoPrimals / gardens)
- **Audience**: All primal teams, all spring teams
- **Type**: Capability gap analysis — plasmidBin metadata vs Webb composition expectations
- **Identity**: Webb is a **composition for deployment** (gen4 consumer). Gaps discovered
  here are evolution pressure for the springs that produce each primal.

---

## Context

V6 removed all spring runtime dependencies. Webb now composes **directly**
from deployed primals via JSON-RPC IPC and biomeOS capability routing. This
handoff documents every gap found by comparing `plasmidBin/*/metadata.toml`
advertised capabilities against what Webb needs for its CRPG composition.

Each gap is owned by the spring that produces the primal. Resolution means
updating the primal's metadata (if capabilities exist but aren't advertised)
or evolving the primal (if capabilities are genuinely missing).

---

## Gap Matrix: Metadata Advertised vs Webb Expected

| Primal | Domain | Webb Expects | Metadata Advertises | Gap |
|--------|--------|-------------|--------------------|----|
| **Squirrel** | ai | `ai.query`, `ai.suggest`, `ai.analyze` | `ai.complete`, `ai.embed`, `ai.tools`, `mcp.serve` | **Naming mismatch** — biomeOS translates but metadata doesn't reflect semantic methods |
| **petalTongue** | visualization | `visualization.render.scene` | `visualization.scene` | **String mismatch** — `.render.scene` vs `.scene` |
| **petalTongue** | interaction | `interaction.poll` | (not listed) | **Missing capability** |
| **rhizoCrypt** | dag | `dag.session.complete` | `dag.session.create`, `.get`, `.list`, `.discard` | **Missing method** — complete not in metadata |
| **rhizoCrypt** | dag | `dag.query.vertices` (per CONTEXT.md) | `dag.vertex.query` | **Naming mismatch** — query.vertices vs vertex.query |
| **BearDog** | crypto | `crypto.hash` | `crypto.identity`, `.keys`, `.sign`, `.verify` | **Missing capability** — hash not advertised |
| **Songbird** | discovery | `discovery.query` | `discovery.announce`, `.resolve`, `mesh.relay`, `.onion` | **Missing capability** — query not advertised |
| **biomeOS** | orchestration | `ConditionalDag`, `Pipeline`, `ContinuousExecutor` RPCs | `orchestration.deploy`, `.federation`, `genome.manage`, `lifecycle.atomic` | **Missing RPCs** (GAP-018) |

---

## Per-Primal Detail

### Squirrel (AI) — Spring: Squirrel team

**Issue**: Webb V6 calls `ai.query`, `ai.suggest`, `ai.analyze` (aligned with
biomeOS capability registry). Squirrel's `metadata.toml` advertises `ai.complete`,
`ai.embed`, `ai.tools`, `mcp.serve`. biomeOS translates between the two
namespaces at runtime, but the metadata doesn't reflect the semantic methods
that consumers actually discover and call.

**What springs should do**:
1. **Squirrel**: Document the mapping between native methods (`complete`, `embed`,
   `tools`) and biomeOS semantic methods (`query`, `suggest`, `analyze`) in
   metadata or a companion doc.
2. **biomeOS**: Consider whether `metadata.toml` should advertise both native
   and semantic method names, or if the capability registry is the source of
   truth for consumer-facing names.

**Webb gap ref**: GAP-022 (resolved in code; metadata alignment is the remaining work).

---

### petalTongue (Visualization) — Spring: petalTongue team

**Issue 1**: Webb calls `visualization.render.scene`; metadata advertises
`visualization.scene`. One of these strings is wrong, or biomeOS needs a
translation entry.

**Issue 2**: Webb expects `interaction.poll` for player input events during
rendered scenes. This capability is not listed in petalTongue metadata.

**What springs should do**:
1. Confirm canonical capability string for scene rendering (`.render.scene` or `.scene`).
2. Confirm whether `interaction.poll` exists or is planned. If not, Webb needs
   to know the intended input pattern for rendered scenes.

**Webb gap ref**: GAP-002 (visualization dialogue tree).

---

### rhizoCrypt (DAG) — Spring: rhizoCrypt team

**Issue 1**: Webb calls `dag.session.complete` to seal a finished playthrough.
Metadata lists `dag.session.create`, `.get`, `.list`, `.discard` — no `.complete`.

**Issue 2**: Webb's CONTEXT.md references `dag.query.vertices`; metadata lists
`dag.vertex.query`. These are likely the same operation with different string
conventions.

**What springs should do**:
1. Confirm whether `dag.session.complete` exists in the binary but is missing from
   metadata, or whether this method needs to be built.
2. Align vertex query method name and update metadata.

**Webb gap ref**: GAP-004 (provenance trio E2E).

---

### BearDog (Crypto) — Spring: BearDog team

**Issue**: Webb needs `crypto.hash` for DAG merkle root computation and content
integrity. Metadata advertises `crypto.identity`, `.keys`, `.sign`, `.verify`
but not `.hash`.

**What springs should do**:
1. Confirm whether hashing is exposed via an existing method (e.g. part of `.sign`)
   or needs a dedicated `crypto.hash` endpoint.
2. If dedicated: add `crypto.hash` to the binary's IPC surface and metadata.

**Webb gap ref**: GAP-019 (beardog crypto domain unwired).

---

### Songbird (Discovery) — Spring: Songbird team

**Issue**: Webb needs `discovery.query` with capability filters for tier-5
primal lookup. Metadata advertises `discovery.announce`, `.resolve`,
`mesh.relay`, `.onion` — no `.query`.

**What springs should do**:
1. Confirm whether capability-filtered discovery is handled via `.resolve` with
   parameters, or whether a dedicated `.query` method is needed.
2. If `.resolve` already accepts capability filters, document the parameter
   schema so consumers can use it.

**Webb gap ref**: GAP-006 (discovery capability-filtered queries).

---

### biomeOS (Orchestration) — Spring: biomeOS team

**Issue**: Webb's storytelling loop is a continuous execution graph. biomeOS has
`ConditionalDag`, `Pipeline`, and `ContinuousExecutor` in code but they aren't
exposed as JSON-RPC methods. `PathwayLearner` is also internal-only.

**What springs should do**:
1. Expose `ConditionalDag`, `Pipeline`, and `ContinuousExecutor` as JSON-RPC
   methods (or document why they remain internal).
2. Confirm whether `PathwayLearner` is intended for consumer use or remains
   an internal optimization.
3. Fix neural-api health in benchScale topologies (GAP-017).

**Webb gap ref**: GAP-017 (neural-api health), GAP-018 (executors not on RPC).

---

## Cross-Cutting: Metadata Convention

Multiple gaps stem from the same root cause: **capability strings in
`metadata.toml` don't match the strings that consumers discover and call**
via biomeOS routing.

**Recommendation for ecosystem coordination (primalSpring / wateringHole)**:

1. Define whether `metadata.toml` advertises **native** method names (what the
   binary's JSON-RPC handler recognizes) or **semantic** names (what biomeOS
   routes to the binary).
2. If native: biomeOS needs a consumer-facing registry that maps semantic names
   to native names, and consumers should discover from that registry.
3. If semantic: primals should advertise the names that consumers actually use.
4. Either way, a `primalSpring validate-metadata` tool that cross-checks
   `metadata.toml` against the binary's actual `capabilities.list` response
   would catch drift.

---

## What Webb Needs Next (Priority Order)

1. **Squirrel metadata alignment** — unblocks confident AI composition testing
2. **rhizoCrypt `dag.session.complete` + vertex naming** — unblocks provenance E2E
3. **biomeOS neural-api health** — unblocks graph-based orchestration
4. **petalTongue scene capability string + interaction.poll** — unblocks rendered play
5. **BearDog `crypto.hash`** — unblocks signed provenance
6. **Songbird `discovery.query`** — unblocks tier-5 discovery
7. **biomeOS executor RPCs** — unblocks continuous storytelling graphs

---

## How This Feeds Back

```
This handoff
  -> spring team reviews capability gap
  -> primal binary updated or metadata corrected
  -> new version deployed to plasmidBin/
  -> Webb's capability discovery picks up the change
  -> composition test validates the gap is closed
  -> EVOLUTION_GAPS.md entry marked resolved
  -> next gap surfaces
```

This is the ecosystem's evolution loop. Webb exists to trigger it.
