<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Esoteric Webb — Vision and Evolution

**Status**: Active
**Date**: March 29, 2026
**Related specs**: [ESOTERIC_WEBB_DESIGN.md](ESOTERIC_WEBB_DESIGN.md),
[BOUNDED_INFINITE_ARCHITECTURE.md](BOUNDED_INFINITE_ARCHITECTURE.md),
[EVOLUTION_GAPS.md](../EVOLUTION_GAPS.md)

---

## What Esoteric Webb is today

Webb is a ~12,500 LOC Rust project that composes deployed primals from
`plasmidBin/` into a Disco Elysium-inspired CRPG. It is the first **gen4
consumer** — a working tool that exercises the full ecoPrimals stack via
BYOB composition and feeds gaps back into the springs that need to evolve.

Webb is intentionally **not a spring**. Springs are development workspaces
that contain science, experiments, and specs. Springs produce primals. Webb
consumes primals — the same way a real product would.

### The engine

| Layer | What exists | State |
|-------|------------|-------|
| Content | YAML-authored worlds, NPCs, abilities, scenes, narrative graphs; scaffold, load, validate | Solid |
| Narrative | Directed graph engine with typed edges, predicates, effects, BFS depths, DOT/JSON visualization | Strong |
| State | `WorldState` with knowledge, inventory, flags, trust, turn counter | Complete for scope |
| Director | Scene traversal, exit resolution, ability application, predicate gating | Functional |
| Session | Stateful game loop — `act()` pipeline with 6-stage primal enrichment | Tested |
| Autoplay | Heuristic AI-as-player with novelty tracking, stale detection | Solid |
| IPC | JSON-RPC 2.0 over UDS/TCP, capability discovery, circuit breakers, retry | Architecturally mature |
| Bridge | Runtime coordinator for 8 primal domains with graceful degradation | V5 |
| CLI | UniBin: `serve`, `validate`, `preview`, `autoplay`, `graph`, `new-world`, `status` | Complete |

### Quality metrics (V5)

- 37 source files, 335 tests (316 unit + 18 e2e integration + 1 validation)
- 90.84% line coverage (`cargo llvm-cov`)
- Zero clippy warnings (pedantic + nursery), zero unsafe, `forbid(unsafe_code)`
- All `#[allow]` migrated to `#[expect]` with reasons; zero TODO/FIXME in production
- 5 experiment suites, signal handling via `signal-hook`
- AGPL-3.0-or-later + CC-BY-SA-4.0 + ORC, SPDX headers on all files

### Primal consumption

Webb has IPC clients and bridge methods for 8 capability domains:

| Domain | Primal | Key methods | Status |
|--------|--------|-------------|--------|
| ai | Squirrel | `ai.chat`, `ai.summarize` | Bridge ready, degrades to placeholder |
| game | ludoSpring | `game.*` (flow, engagement, DDA, dialogue, narration, voice) | Bridge ready, degrades |
| visualization | petalTongue | `visualization.render_scene`, `interaction.poll` | Bridge ready, degrades |
| dag | rhizoCrypt | `dag.session.*`, `dag.event.*`, `dag.frontier.*`, `dag.merkle.*` | Bridge ready, exp004 validates live |
| lineage | loamSpine | `certificate.mint` | Bridge ready |
| compute | toadStool | `compute.dispatch.submit` | Bridge ready |
| storage | nestGate | `storage.store`, `storage.retrieve` | Bridge ready |
| provenance | sweetGrass | `attribution.record` (planned) | Discovered, not yet exercised |

In standalone mode every method degrades gracefully. In composition mode
primals enrich gameplay with AI narration, NPC dialogue, flow science,
provenance, and visualization.

---

## The ecosystem we live in

### Springs (ecoSprings/)

| Spring | Domain |
|--------|--------|
| ludoSpring | Game/interaction science — RPGPT, Webb's primary science partner |
| primalSpring | Ecosystem coordination — deploy graphs, Neural API, composition validation |
| neuralSpring | ML/surrogates/scholarly reproduction |
| hotSpring | Computational physics |
| wetSpring | Life science/analytical chemistry |
| healthSpring | PK-PD/microbiome/biosignal |
| groundSpring | Measurement noise, inverse problems |
| airSpring | Ecological/agricultural sciences |
| esotericWebb | Us (consumer, not spring) |

### Foundation primals (phase1)

bearDog (crypto), songbird (discovery), squirrel (AI/MCP),
toadStool (hardware), nestGate (storage)

### Post-NUCLEUS primals (phase2)

biomeOS (orchestration), sweetGrass (attribution), loamSpine (permanence),
rhizoCrypt (ephemeral DAG)

### Standalone primals

barraCuda (GPU math), coralReef (shader compiler), petalTongue (UI)

### Standards ladder

UniBin (one binary, subcommands) → ecoBin (+ pure Rust, cross-platform IPC)
→ genomeBin (+ deployment wrapper, OS integration)

Binary artifacts deployed to `ecoPrimals/plasmidBin/`. Consumers discover
via Songbird, filesystem probe, or TCP env vars. See
[PLASMIBIN_DISTRIBUTION.md](PLASMIBIN_DISTRIBUTION.md) for the public
distribution strategy.

---

## What Esoteric Webb could become

Six evolution vectors, each grounded in what already exists and what gaps
remain.

### 1. The living proof that composition works

primalSpring validates the *infrastructure* of composition (deploy graphs,
atomics, bonding). Webb validates the *experience* of composition — what
happens when a player walks into a room and six primals enrich that moment.

**Current foundation**: 7 domain bridge with full degradation paths, deploy
graph launcher with topological wave ordering, capability-based discovery.

**Evolution path**:

- Webb as the canonical BYOB niche reference — the niche YAML and deploy
  graphs in `niches/` and `graphs/` become the ecosystem's standard example
  of "here is what a real composed system looks like."
- Exercise all five Neural API coordination patterns in a real product
  context: pipeline streaming for narration, continuous ticks for game
  science telemetry, conditional DAG for primal-dependent enrichment paths.
- Every gap Webb hits is a gap the ecosystem must close. The gap feedback
  loop (EVOLUTION_GAPS.md) is the ecosystem's immune system — Webb is the
  organism that triggers it.

**Unlocked by**: GAP-006 (Songbird discovery queries), GAP-010 (plasmidBin
deployment automation)

### 2. The game that teaches itself

The RPGPT pipeline (ludoSpring science) is consumed via `game.*` methods.
The architecture allows Webb to become a closed-loop learning system.

**Current foundation**: Flow evaluation, engagement metrics, DDA
recommendations all wired through bridge. Provenance DAG records every
action.

**Evolution path**:

- **Flow-driven DDA**: Feed DDA adjustments back into director behavior —
  adjust predicate thresholds, reveal hidden exits, alter NPC trust dynamics
  in real-time based on flow score.
- **Provenance-informed narration**: Include provenance context in narrate
  calls — "the player has explored 12 turns without finding the secret,
  visited 3/5 rooms" — letting Squirrel generate narration that
  acknowledges the player's journey.
- **Deep voice system**: Evolve `VoiceNote` interjections into a full
  internal monologue subsystem. Voices that remember, contradict each
  other, have opinions about NPC trust levels. Disco Elysium's voices are
  its soul; Webb's voice system is the scaffold for that depth.

**Unlocked by**: GAP-003 (AI constraint enforcement), GAP-007 (voice
preview offline), GAP-009 (RulesetCert validation)

### 3. A real creative tool

The content authoring spec and `new-world` scaffolding exist. Webb could
become the standard creative surface for the ecosystem.

**Current foundation**: YAML content with validation, scaffold-then-load
roundtrip, `preview` command for playtesting, `graph` for visualization.

**Evolution path**:

- **Content packs as genomeBins**: A world authored in YAML packaged as a
  distributable content pack with its own provenance certificate
  (loamSpine), checksums (bearDog), and deployment manifest. `esotericwebb
  validate --pack` for verification.
- **Visual authoring via petalTongue**: The graph visualization produces DOT
  and JSON. petalTongue could render the narrative DAG as an interactive
  editor with drag-and-drop node placement, live predicate testing, and
  "play from here" preview.
- **Multi-world composition**: The `worlds` HashMap in `ContentBundle` is
  populated but functionally unused. Multiple worlds composed into campaigns
  with cross-world state persistence via nestGate storage.

**Unlocked by**: GAP-002 (visualization dialogue tree), GAP-008 (content
pack format)

### 4. Multiplayer provenance

The provenance trio (rhizoCrypt + loamSpine + sweetGrass) is wired for
single-player sessions. The architecture is inherently multi-agent.

**Current foundation**: DAG sessions with append, frontier, merkle.
`initialize_provenance()`, per-action vertex append, session completion.

**Evolution path**:

- **Shared DAGs**: rhizoCrypt sessions are identified by ID. Multiple
  players (or AI agents) could append to the same session DAG, creating a
  multiplayer provenance graph where actions by different players are DAG
  vertices with independent parents.
- **Attribution braids**: sweetGrass records who contributed what. In a
  multiplayer context this becomes a credit system — who discovered the
  secret, who changed the world state, who triggered the ending.
- **Replays as first-class objects**: `cmd_replay` is a stub today. With a
  complete provenance DAG, replays become navigable — scrub through any
  player's perspective, fork from any vertex, compare divergent
  playthroughs.

**Unlocked by**: GAP-004 (provenance trio E2E)

### 5. Sovereign game infrastructure

Webb runs on the ecosystem's sovereign stack — no cloud, no Docker, no
external services.

**Current foundation**: All IPC is local (UDS/TCP to localhost), signal
handling for clean shutdown, capability discovery from local filesystem.

**Evolution path**:

- **Gate-local AI**: Squirrel runs on your hardware. Narration and NPC
  dialogue happen locally with full privacy. No API keys, no rate limits,
  no telemetry to third parties.
- **Gate-local compute**: toadStool + barraCuda + coralReef mean GPU compute
  (procedural generation, physics, shader compilation) happens on sovereign
  hardware.
- **Cross-gate federation**: biomeOS supports Plasmodium (multi-gate
  collective). A Webb game could span gates — your gate runs the game
  engine, a friend's gate runs AI inference, provenance is replicated
  across both via RootPulse.

**Unlocked by**: GAP-006 (Songbird discovery), GAP-010 (plasmidBin
deployment automation), biomeOS Plasmodium evolution

### 6. The gap feedback engine

Every gap Webb discovers is evolution pressure on the primal that needs to
improve. This is not a side effect — it is a primary function.

**Current foundation**: EVOLUTION_GAPS.md with structured gap template,
evidence, workaround, handoff tracking. Two gaps absorbed (V3, V4), eight
open.

**Evolution path**:

- Automate gap detection: when a bridge method degrades, log structured
  telemetry that can be diff'd against previous runs to identify new gaps.
- Gap severity auto-classification based on frequency of degradation hits
  during autoplay sessions.
- Cross-reference with primalSpring's composition validation to identify
  gaps that affect multiple consumers, not just Webb.

---

## Gap-to-vector map

How the open gaps in EVOLUTION_GAPS.md map to the evolution vectors above.

| Gap | Description | Vectors unlocked |
|-----|-------------|-----------------|
| GAP-002 | Visualization dialogue tree | 3 (creative tool) |
| GAP-003 | AI constraint enforcement | 2 (closed-loop game) |
| GAP-004 | Provenance trio E2E | 4 (multiplayer provenance) |
| GAP-006 | Songbird discovery queries | 1 (composition proof), 5 (sovereign) |
| GAP-007 | Voice preview offline | 2 (closed-loop game), 3 (creative tool) |
| GAP-008 | Content pack format | 3 (creative tool) |
| GAP-009 | RulesetCert validation | 2 (closed-loop game) |
| GAP-010 | plasmidBin deployment | 1 (composition proof), 5 (sovereign) |

The highest-leverage gaps are GAP-004 (provenance trio E2E) and GAP-010
(plasmidBin deployment) — they unlock the most vectors and have the widest
ecosystem impact. GAP-010 is addressed by the plasmidBin public
distribution strategy in [PLASMIBIN_DISTRIBUTION.md](PLASMIBIN_DISTRIBUTION.md).

---

## Summary

Webb today is a well-architected, test-covered, cleanly degrading game
engine that plays a narrative CRPG with or without primals. It is the only
project in the ecosystem that makes all the abstractions concrete — not
"here is how primals could compose" but "here is what happens when a player
walks into a room and six primals enrich that moment."

What it could be:

1. The reference consumer that proves the ecosystem works (and surfaces
   where it does not)
2. A closed-loop game where AI, flow science, and provenance feed back into
   gameplay dynamically
3. A creative platform where worlds are authored, packaged, distributed,
   and replayed with full provenance
4. A sovereign multiplayer substrate where game sessions span gates, players
   share provenance DAGs, and no cloud is required
5. The first real product that is not infrastructure — a thing people play,
   built entirely from composed primals

The foundation is solid. The architecture degrades gracefully. The test
coverage is real. The gaps are documented. The ecosystem has the primals.
Webb is the place where they all come alive.
