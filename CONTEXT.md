# SPDX-License-Identifier: AGPL-3.0-or-later

# Esoteric Webb — Context

## What is this?

Esoteric Webb is the **first gen4 entity** in the ecoPrimals ecosystem — a
working creative tool that composes deployed primals into a playable CRPG.
It is not a spring. Where springs prove that science computes correctly,
Webb proves that primals compose into invisible infrastructure inside a
product someone actually uses.

Webb consumes primals via JSON-RPC IPC and capability-based discovery from
`ecoPrimals/plasmidBin/`. It does not import any spring or primal Rust
crates. All coordination happens at runtime over TCP/UDS.

## Tool vs science

Webb is informed by the science in the springs but does not depend on spring
source trees. Springs produce primals (genomeBin/ecoBin); Webb consumes
the deployed artifacts. This makes Webb leaner and functionally independent
of spring evolution.

## Ecosystem position

- **Generation**: gen4 (composition + creative surface)
- **Organization**: [sporeGarden](https://github.com/sporeGarden) (creative-facing tools)
- **Type**: Cross-evolution substrate (primal composition)
- **Domain**: `narrative` — CRPG direction, DAG-traced storytelling, emergent gameplay
- **Parent**: ecoPrimals / ecoSprings
- **Primal source**: `ecoPrimals/plasmidBin/` (genomeBin/ecoBin deployment surface)
- **License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA-4.0)

## Architecture

- **Main crate**: `esoteric-webb` (library + UniBin binary)
- **Niche module**: `niche.rs` — self-knowledge (identity, capabilities, socket
  resolution, family ID, neural-api discovery). Absorbed from ludoSpring V32 pattern.
- **IPC**: JSON-RPC 2.0 over TCP and Unix domain sockets (newline-delimited)
- **Transport**: TCP (default, platform-agnostic) + UDS; `connect_transport()` parses `unix:`, `tcp:`, and implicit formats; XDG-compliant socket path resolution, capability-based discovery
- **No cross-primal Rust imports**: all coordination via runtime IPC
- **Content**: YAML-authored worlds, NPCs, abilities, scenes, narrative graphs
- **Primal resolution**: `plasmidBin/` filesystem probe, Songbird discovery, XDG/biomeOS sockets

## Capabilities (Webb's own JSON-RPC surface)

sourDough: `health.liveness`, `health.readiness`, `health.check`,
`identity.get`, `capabilities.list`

Health: `webb.health`, `webb.liveness`, `webb.readiness`

Narrative: `webb.scene.current`, `webb.narrative.status`

Content: `webb.content.list`

Session: `session.start`, `session.state`, `session.actions`,
`session.act`, `session.history`, `session.narrate`, `session.graph`

MCP: `tools.list`, `tools.call`

## Consumed primal capabilities (V6 — self-composed)

AI (Squirrel via biomeOS): `ai.query`, `ai.suggest`, `ai.analyze`

Visualization (petalTongue): `visualization.render.scene`, `interaction.poll`

Compute (ToadStool): `compute.dispatch.submit`

Storage (NestGate): `storage.store`, `storage.retrieve`

DAG (rhizoCrypt): `dag.session.create`, `dag.event.append`, `dag.frontier.get`,
`dag.merkle.root`, `dag.session.complete`, `dag.query.vertices`

Lineage (LoamSpine): `certificate.mint`

## Local science (absorbed, no IPC)

Flow evaluation, engagement metrics, dynamic difficulty adjustment — pure math
absorbed from ludoSpring patterns into `science/` module. When a game-science
primal emerges, these can switch from local to IPC (GAP-021).

All primal calls go through `PrimalBridge` with graceful degradation. The
enrichment pipeline: AI narration → NPC dialogue → local flow → scene push →
provenance. Absent primals degrade to mechanical defaults. No spring dependencies.

## Build

```bash
cargo build --workspace
cargo test --workspace
```

## Standards adherence

- wateringHole UNIVERSAL_IPC_STANDARD_V3
- wateringHole PUBLIC_SURFACE_STANDARD
- wateringHole SPRING_AS_NICHE_DEPLOYMENT_STANDARD
- ScyBorg Provenance Trio Guidance
- Semantic Method Naming Standard v2.2
- ECOBIN_ARCHITECTURE_STANDARD (zero C deps, pure Rust, cross-compile ready)
- CAPABILITY_BASED_DISCOVERY_STANDARD (5-tier discovery with degradation)
- PURE_RUST_SOVEREIGN_STACK_GUIDANCE (`#![forbid(unsafe_code)]`)

## Generational context

```
gen1 — Can we build it?        (hardware, cluster, AI-assisted dev)
gen2 — What should we build?   (protocol, philosophy, sovereignty)
gen3 — Does it work?           (science, springs, primals, 12k+ checks)
gen4 — Who uses it?            (composition, creative surface, products)
           ↑
     Esoteric Webb lives here
```

The primals disappear into the product. A player sees a game, not a DAG
engine, not a lineage tracker, not a GPU compute pipeline. This is the
gen3→gen4 boundary: infrastructure becomes invisible.
