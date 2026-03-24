# Esoteric Webb

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

| Primal capability | Role | IPC methods consumed |
|-------------------|------|---------------------|
| Game science | RPGPT evaluation, flow, DDA, engagement | `game.*` |
| AI | Narration, NPC voices, inference | `ai.chat`, `ai.inference`, `ai.summarize` |
| Visualization | Game UI, scene rendering | `visualization.render.scene`, `ui.render`, `interaction.*` |
| Session DAG | Provenance vertices | `provenance.session_create`, `provenance.vertex_*` |
| Certificates | NPC/ruleset certs | `certificate.mint`, `certificate.query` |
| Attribution | Creative attribution | `attribution.record`, `attribution.query` |
| Crypto | Signing, hashing | `crypto.sign`, `crypto.hash` |
| Discovery | Capability-based primal lookup | `discovery.query` |

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
webb/              Main Rust crate (narrative engine + IPC + director)
content/           YAML game content (authored by creative teams)
graphs/            biomeOS deploy graphs
niches/            BYOB niche definitions
deploy/            Composition fragment for biomeOS/primalSpring
specs/             Design specifications
```

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
