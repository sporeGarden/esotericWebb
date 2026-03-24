# Esoteric Webb

| | |
|---|---|
| **Version** | V4 |
| **Tests** | 166 |
| **Rust files** | 32 (~8.5k LOC) |
| **Experiments** | 5 (exp001–exp005) |
| **MSRV** | 1.87 (edition 2024) |
| **License** | AGPL-3.0 + ORC + CC-BY-SA 4.0 |
| **Unsafe** | `#![forbid(unsafe_code)]` |
| **C deps** | Zero (ecoBin compliant) |
| **Bridge methods** | 23 (all domains, all degrading) |
| **Primals consumed** | 8 domains (ai, game, viz, dag, lineage, compute, storage, provenance) |
| **Last validation** | 2026-03-24 (V4) |

**A [sporeGarden](https://github.com/sporeGarden) project — the primals as a composed CRPG.**

Esoteric Webb is not a spring. It is a standalone cross-evolution substrate
that composes deployed primals (genomeBins/ecoBins from `plasmidBin/`) into
a creator/player game system via BYOB composition. Its dual purpose:

1. **Build a real game** — a Disco Elysium-inspired CRPG with DAG-traced
   narrative, deep NPCs, emergent ability interactions, and meaningful
   bounded endings.
2. **Find every gap** — in rendering, AI, science, provenance, discovery,
   compute, and composition. Gaps feed back as evolution pressure on the
   primals that need to grow.

## Tool vs Science

Webb is a **working tool**, informed by science, anchored in primal
capabilities. Springs are not primals — springs PRODUCE primals. Webb
consumes primals like primalSpring does, via JSON-RPC IPC and capability-based
discovery from `ecoPrimals/plasmidBin/`. This makes Webb leaner than the
springs it draws science from.

```
Springs (science + experiments)  →  produce  →  primals (genomeBin/ecoBin)
                                                       ↓
                                               plasmidBin/ (deployment)
                                                       ↓
                                          Webb discovers + composes via IPC
```

## Architecture

Esoteric Webb sits atop the primal stack as a **narrative direction
engine**. It consumes primals via JSON-RPC IPC — zero Rust crate dependencies
on any spring. Primals are resolved from `plasmidBin/` or discovered via
Songbird at runtime.

| Domain | Primal | Role | Wired in act() | IPC methods |
|--------|--------|------|-----------------|-------------|
| game | ludoSpring | Flow, DDA, engagement, NPC dialogue, narration, voice | Yes (V4) | `game.evaluate_flow`, `game.npc_dialogue`, `game.narrate_action`, `game.voice_check`, `game.push_scene`, `game.begin_session`, `game.complete_session` |
| ai | Squirrel | Narration fallback, summarization | Yes (V4) | `ai.chat`, `ai.summarize` |
| visualization | petalTongue | Scene rendering, input polling | Yes (V4) | `visualization.render.scene`, `interaction.poll` |
| dag | rhizoCrypt | Provenance DAG lifecycle | Yes (V4) | `dag.session.create`, `dag.event.append`, `dag.session.complete`, `dag.frontier.get`, `dag.merkle.root`, `dag.query.vertices` |
| lineage | loamSpine | NPC personality certs | Bridge ready | `certificate.mint` |
| compute | toadStool | GPU compute dispatch | Bridge ready | `compute.dispatch.submit` |
| storage | nestGate | Key-value persistence | Bridge ready | `storage.store`, `storage.retrieve` |
| provenance | sweetGrass | Creative attribution | Bridge ready | `attribution.record` |

## The Core Thesis: Bounded Space, Infinite Exploration

Traditional CRPGs have branching narratives where NPCs can only say so many
things and player input variety is masked. Esoteric Webb inverts this:

- The **narrative topology** (DAG structure) is finite and authored
- The **traversal state** (knowledge, trust, conditions, inventory, arc phases)
  is combinatorially vast
- **Abilities** interact with state in ways authors cannot predict, creating
  emergent paths through authored structure
- **Bounded endings** carry more meaning because they reflect genuine paths
  through a rich state space

## Quick start

```bash
# Build
cargo build --workspace

# Validate content
cargo run --bin esotericwebb -- validate --content content/

# Preview (text mode, no primals required)
cargo run --bin esotericwebb -- preview --content content/

# Full BYOB niche (requires primal stack from plasmidBin/)
cargo run --bin esotericwebb -- serve --content content/
```

## For Creatives

Esoteric Webb is a **dual-surface** system: developers work in Rust,
creatives work in YAML. No Rust, no engine license, no publisher required.

- `esotericwebb validate` — lint your content for broken refs and dead ends
- `esotericwebb preview` — play through in text mode, iterate immediately
- `esotericwebb graph` — visualize your narrative DAG as DOT/SVG
- `esotericwebb new-world` — scaffold a blank world with template YAML

See `specs/CREATOR_PROFILES_AND_SYSTEM_DESIGN.md` for how the system maps
to the creative DNA of teams like ZA/UM (Disco Elysium) and Cliche Studio
(Esoteric Ebb), and `specs/CONTENT_AUTHORING_SPEC.md` for the YAML format.

## Project structure

```
webb/              Main Rust crate (narrative engine + IPC + director + bridge)
  src/bin/         CLI binary (serve, validate, preview, graph, new-world)
  src/ipc/         JSON-RPC client, bridge, discovery, launcher, resilience
  src/narrative/   Graph, validator, predicate, effect, visualization
  src/director/    Game director (outcome evaluation, DDA integration)
  src/content/     YAML content loader, ability/NPC/scene models
  src/state/       World state (knowledge, trust, inventory, flags, conditions)
content/           YAML game content (authored by creative teams)
experiments/       5 standalone validation crates (exp001–exp005)
graphs/            biomeOS deploy graphs (20+ TOML fragments)
niches/            BYOB niche definitions
deploy/            Composition fragment for biomeOS/primalSpring
specs/             Design specifications (4 documents)
wateringHole/      Handoffs to primal and spring teams
config/            Launch profiles for primal composition
```

## gen4 — The First Consumer

Esoteric Webb is the first `gen4` entity in the ecoPrimals ecosystem (see
`ecoPrimals/whitePaper/gen4/`). Where gen3 proved the infrastructure computes
correct science across 7 springs, gen4 asks: can people who didn't build the
primals compose them into tools they care about? Webb answers yes — the
primals become invisible infrastructure inside a creative product.

## Quality gates

```bash
make check    # fmt + clippy + test + doc
make deny     # supply chain audit
```

## License

ScyBorg triple license:
- **AGPL-3.0-or-later** — code (see `LICENSE`)
- **Open RPG Creative (ORC)** — game mechanics (see `LICENSE-ORC`)
- **CC BY-SA 4.0** — documentation and creative content (see `LICENSE-CC-BY-SA`)
