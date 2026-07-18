# Esoteric Webb

| | |
|---|---|
| **Version** | V18 |
| **Tests** | 472 (453 unit + 18 E2E + 1 validation) |
| **Coverage** | ~92% lines (`cargo llvm-cov`) |
| **Rust files** | 50 (~16k LOC) |
| **Experiments** | 6 (exp001–exp006) |
| **MSRV** | 1.87 (edition 2024) |
| **License** | AGPL-3.0 + ORC + CC-BY-SA 4.0 |
| **Unsafe** | `#![forbid(unsafe_code)]` (crate-level) |
| **C deps** | Zero (ecoBin compliant) |
| **Bridge methods** | 32 (all domains, all degrading, composition-first) |
| **Capabilities exposed** | 26 (sourDough + lifecycle + narrative + session + introspection + MCP) |
| **Primals consumed** | 9 domains (ai, viz, dag, lineage, compute, storage, provenance, crypto, mesh) |
| **Composition adoption** | Wave 17 — `nest.store`, `nest.commit`, `primal.announce` |
| **Mesh registration** | Wave 107 — `route.register` with stability tiers + push propagation |
| **Wave compliance** | Wave 107 — zero debt, typed errors, mesh-visible, introspection, TransportEndpoint |
| **Degradation contracts** | Written per-domain in `docs/DEGRADATION_BEHAVIOR.md` |
| **Trio tracking** | `primals_reached` in session state per `PROVENANCE_TRIO_INTEGRATION_GUIDE` |
| **Local science** | flow, engagement, DDA, voice interjections, session metrics (absorbed patterns) |
| **Ecosystem registry** | 490+ methods (primalSpring) |
| **Live primals (flockGate)** | 6/9 connected (squirrel, petaltongue, nestgate, loamspine, sweetgrass, beardog) |
| **E2E demo** | `esotericwebb demo` — guided scenario exercising all connected primals |
| **Last validation** | 2026-07-18 (V18) |

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

| Domain | Primal | Role | Status (V17) | Key IPC methods |
|--------|--------|------|-------------|-----------------|
| ai | Squirrel | AI narration, NPC dialogue, inference | Live on flockGate | `ai.query`, `ai.suggest`, `ai.analyze` |
| visualization | petalTongue | Scene rendering, input polling | Live on flockGate | `visualization.render.scene`, `interaction.poll` |
| dag | rhizoCrypt | Provenance DAG lifecycle | Found (stale socket) | `dag.session.create`, `dag.event.append`, `dag.merkle.root` |
| lineage | loamSpine | NPC personality certs | Live on flockGate | `certificate.mint` |
| compute | toadStool | GPU compute dispatch | Found (stale socket) | `compute.dispatch.submit` |
| storage | nestGate | Key-value persistence | Live on flockGate | `storage.store`, `storage.retrieve` |
| provenance | sweetGrass | Creative attribution | Live on flockGate | `braid.create`, `braid.query` |
| crypto | bearDog | Signing, verification, hashing | Live on flockGate | `crypto.sign`, `crypto.verify`, `crypto.hash` |
| mesh | songBird | Topology, discovery, bonds | TCP only (HTTP transport) | `discovery.topology`, `discovery.health`, `discovery.bonds` |
| orchestration | biomeOS | Neural API, composition dispatch | Lifecycle wired | `primal.announce`, `health.version`, `health.drain` |

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
  src/content/     YAML content loader, ability/NPC/scene models (tests in tests.rs)
  src/session/     Game session, enrichment pipeline, types (tests in tests.rs)
  src/science/     Local game science (flow, engagement, DDA, voice interjections)
  src/niche.rs     Self-knowledge (identity, 26 capabilities, socket resolution)
  src/state/       World state (knowledge, trust, inventory, flags, conditions)
  capability_registry.toml   All 25 exposed JSON-RPC methods
content/           YAML game content (authored by creative teams)
experiments/       5 standalone validation crates (exp001–exp005)
graphs/            biomeOS deploy graphs (8 TOML compositions, secure_by_default)
niches/            BYOB niche definition (esoteric-webb.yaml)
deploy/            Composition fragment for biomeOS/primalSpring
specs/             Design specifications (7 documents)
wateringHole/      Handoffs to primal and spring teams
whitePaper/        baseCamp evolution patterns document
```

## gen4 — The First Consumer

Esoteric Webb is the first `gen4` entity in the ecoPrimals ecosystem (see
`ecoPrimals/whitePaper/gen4/`). Where gen3 proved the infrastructure computes
correct science across 7 springs, gen4 asks: can people who didn't build the
primals compose them into tools they care about? Webb answers yes — the
primals become invisible infrastructure inside a creative product.

## Quality gates

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib --tests
cargo doc --workspace --no-deps
cargo llvm-cov --workspace --lib --fail-under-lines 90  # coverage
```

## License

ScyBorg triple license:
- **AGPL-3.0-or-later** — code (see `LICENSE`)
- **Open RPG Creative (ORC)** — game mechanics (see `LICENSE-ORC`)
- **CC BY-SA 4.0** — documentation and creative content (see `LICENSE-CC-BY-SA`)
