# SPDX-License-Identifier: AGPL-3.0-or-later

# Esoteric Webb — Context

## What is this?

Esoteric Webb is a **working tool** — a cross-evolution substrate that
composes deployed primals into a playable CRPG. It is not a spring; it
consumes primals like primalSpring does, via JSON-RPC IPC and capability-based
discovery from `ecoPrimals/plasmidBin/`.

## Tool vs science

Webb is informed by the science in the springs but does not depend on spring
source trees. Springs produce primals (genomeBin/ecoBin); Webb consumes
the deployed artifacts. This makes Webb leaner and functionally independent
of spring evolution.

## Ecosystem position

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

Health: `webb.health`, `webb.liveness`, `webb.readiness`

Narrative: `webb.scene.current`, `webb.narrative.status`

Content: `webb.content.list`, `webb.content.validate`

MCP: `tools.list`, `tools.call`

## Consumed primal capabilities

Game science: `game.evaluate_flow`, `game.engagement`,
`game.difficulty_adjustment`, `game.npc_dialogue`, `game.narrate_action`,
`game.voice_check`, `game.push_scene`, `game.begin_session`,
`game.record_action`, `game.complete_session`, `game.query_vertices`,
`game.mint_certificate`

AI: `ai.chat`, `ai.inference`, `ai.summarize`

Visualization: `visualization.render.scene`, `ui.render`,
`interaction.subscribe`, `interaction.poll`

Provenance: `provenance.session_create`, `provenance.vertex_append`,
`provenance.vertex_query`, `certificate.mint`, `certificate.query`,
`attribution.record`

Tower: `crypto.sign`, `crypto.hash`, `discovery.query`

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
