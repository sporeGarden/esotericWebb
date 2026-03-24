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
- **IPC**: JSON-RPC 2.0 over Unix domain sockets (newline-delimited)
- **Transport**: XDG-compliant socket path resolution, capability-based discovery
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

## Consumed primal capabilities (V4 — live wired)

Game science: `game.evaluate_flow`, `game.engagement`,
`game.difficulty_adjustment`, `game.npc_dialogue`, `game.narrate_action`,
`game.voice_check`, `game.push_scene`, `game.begin_session`,
`game.record_action`, `game.complete_session`, `game.query_vertices`,
`game.mint_certificate`

AI: `ai.chat`, `ai.inference`, `ai.summarize`

Visualization: `visualization.render.scene`, `ui.render`,
`interaction.subscribe`, `interaction.poll`

DAG: `dag.session.create`, `dag.event.append`, `dag.frontier.get`,
`dag.merkle.root`, `dag.session.complete`, `dag.query.vertices`

Lineage: `certificate.mint`

Provenance: `provenance.session_create`, `provenance.vertex_append`,
`provenance.vertex_query`, `certificate.query`, `attribution.record`

Tower: `crypto.sign`, `crypto.hash`, `discovery.query`

All game science, AI, visualization, and DAG capabilities are wired into
`GameSession::act()` via `PrimalBridge` with graceful degradation. Each
action runs the full composition pipeline: narrate → dialogue → flow →
scene push → provenance. Absent primals degrade to mechanical defaults.

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
- Semantic Method Naming Standard v2.1
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
