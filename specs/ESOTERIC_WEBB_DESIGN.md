<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Esoteric Webb — Core Design

**Status**: Active (V6)
**Date**: March 29, 2026
**Identity**: sporeGarden composition — deploys primal compositions via biomeOS graph deployments
**Primals consumed**: AI (Squirrel), visualization (petalTongue), compute (ToadStool), storage (NestGate), DAG (rhizoCrypt), lineage (LoamSpine), provenance (sweetGrass) — composed via JSON-RPC IPC
**Local science**: flow, engagement, DDA (absorbed from ludoSpring patterns, no runtime dependency)
**Primal source**: `ecoPrimals/plasmidBin/` (genomeBin/ecoBin deployment surface)
**Science derived from**: ludoSpring, Squirrel, petalTongue, provenance trio — springs produce primals; Webb composes primals
**License**: AGPL-3.0-or-later (code); documentation under [CC-BY-SA-4.0](../LICENSE-CC-BY-SA)

---

## Overview

**Esoteric Webb** is a **composition for deployment** — a gen4 consumer in
the sporeGarden organization that composes deployed primals (genomeBins/ecoBins
resolved from `plasmidBin/`) into a playable creative surface. It is informed
by the science in the springs but anchored in primal capabilities, not spring
source code.

Webb is intentionally **not a spring**. Springs are development workspaces
that contain science, experiments, and specs. Springs **produce** primals.
Webb **composes** primals — via JSON-RPC IPC, biomeOS capability routing,
and local science algorithms — into a self-composed game engine and creative
engine with UI and AI agentics.

**sporeGarden projects** share this identity: they are compositions for
deployment. They consume primals, discover gaps, and hand off evolution
pressure to the springs that own each domain. The gap between what primal
compositions can do today and what a self-composed creative engine needs
drives the entire ecosystem forward.

The end result is a **creator/player game system**: a dual-surface tool
where developers work in Rust and creatives work in YAML, both composing
primal capabilities into deep narrative experiences.

---

## Tool vs science boundary

Webb separates the **working tool** from the **scientific foundation**:

| Layer | What it is | Where it lives |
|-------|-----------|----------------|
| **Working tool** (Webb) | Narrative engine, GameDirector, content authoring, CLI, IPC orchestration, local science | `gardens/esotericWebb/` |
| **Local science** | Flow evaluation, engagement metrics, DDA — pure math absorbed from spring patterns | `webb/src/science/` |
| **Deployed primals** | Compiled genomeBin/ecoBin binaries with IPC capabilities | `ecoPrimals/plasmidBin/` |
| **Scientific foundation** | Validated HCI models, experiments, quality profiles, RPGPT theory | Spring source trees — derivation only |

Webb references the science as **derivation** — "this design was informed
by ludoSpring's flow model" — not as a dependency. V6 absorbed core game
science algorithms locally so Webb is self-sufficient for flow, engagement,
and DDA without any spring or game-science primal at runtime (GAP-021 tracks
the evolution toward a dedicated game-science primal for ecosystem reuse).

This means:

- **Springs evolve independently** — a spring can refactor, rename crates,
  or restructure without breaking Webb.
- **Webb is self-composed** — it carries no spring dependencies, no game
  domain coupling. Just the tool, local science, and direct primal composition.
- **Primals are the contract** — the IPC method signatures and capability
  names are the only coupling. Everything else is derivation.
- **biomeOS is the router** — capability.call routes requests through
  the ecosystem's semantic registry, translating Webb's calls to primal-
  native methods transparently.

---

## Dual purpose

1. **Build a real CRPG** — Disco Elysium-inspired depth: internal voices,
   knowledge bounds, trust dynamics, plane transitions, DAG-traced narrative,
   and endings that remain **bounded** yet **meaning-laden** because paths
   accumulate distinct state.

2. **Find every gap in the primal stack** — rendering, AI, science,
   provenance, discovery, compute, IPC ergonomics, and composition. Each gap
   becomes evolution pressure on the owning primal, documented and closed
   through the gap feedback loop (see below).

---

## Architecture: self-composed via primal composition

Webb coordinates primals **only** through **JSON-RPC 2.0** IPC (TCP default,
UDS for co-located). There are **no cross-primal Rust crate imports** and
**no spring dependencies**: the narrative engine owns orchestration; primals
own their domains. Primal binaries are resolved from `plasmidBin/` or
discovered via Songbird at runtime. biomeOS `capability.call` provides
semantic routing when direct connections are unavailable.

**Composition stack (V6):**

| Layer | Responsibility |
|-------|----------------|
| **Esoteric Webb** | Narrative direction, scene selection, state machine, content validation, `GameDirector`, local science (flow/engagement/DDA) |
| **AI primal** (Squirrel) | Narration via `ai.query`, summarization via `ai.suggest`, analysis via `ai.analyze` — routed through biomeOS capability registry |
| **Visualization primal** (petalTongue) | Scene rendering, UI, interaction (`visualization.*`, `interaction.*`) |
| **Provenance trio** | Session DAG vertices (rhizoCrypt), certificates/rulesets (LoamSpine), creative attribution (sweetGrass) |
| **Infrastructure primals** | Compute (ToadStool), storage (NestGate), crypto (BearDog, GAP-019), discovery (Songbird, GAP-006) |

This is **BYOB** (bring your own binaries): deploy the niche from
`plasmidBin/`, connect sockets, discover capabilities — Webb does not embed
primal implementations.

---

## Key insight: games as compositions of primals

A Webb-shaped game is not "a dialogue system plus combat." It is a **composition
of primals**:

- **Narrative direction** — authored graph + predicates + effects (Webb).
- **Game science** — flow, difficulty, engagement (local `science/` module; future primal via GAP-021).
- **AI** — narration, NPC dialogue, voice analysis via Squirrel (`ai.query`, `ai.suggest`, `ai.analyze`).
- **UI** — scene rendering and interaction surfaces (petalTongue).
- **Provenance** — every meaningful action is a vertex with traceable lineage (provenance trio).

When one primal lags, the whole experience degrades in a **specific** way —
which is why Webb exists: to make those failures visible and attributable.

---

## Case study framing: inspirations

**Disco Elysium** — internal voices as mechanics; skills as perspectives;
conversation as the primary arena; failure and contradiction as content.

**Esoteric Ebb** (internal lineage) — continuity with ecoPrimals narrative
experiments; emphasis on **knowledge bounds**, **trust**, and **weird
topology** without infinite bespoke dialogue.

Webb explicitly targets:

- **Internal voices** — triggered commentary tied to AI analysis and state.
- **Knowledge bounds** — what NPCs know, suspect, lie about, or lack.
- **Trust dynamics** — per-NPC gates that change what is said or offered.
- **Plane transitions** — DAG-recorded shifts of interaction rules while world
  state persists (design derived from RPGPT planes and deep system concepts
  in the game science spring).

---

## The enrichment pipeline

End-to-end play loop (V6):

```
Player input
  -> GameDirector
  -> state evaluation (predicates, arcs, knowledge, trust, plane, flags)
  -> enrichment pipeline:
       1. AI narration (Squirrel via ai.suggest / ai.query)
       2. NPC dialogue (Squirrel via ai.query with NPC context)
       3. Flow evaluation (local science — no IPC)
       4. Scene rendering (petalTongue)
       5. Provenance vertex (rhizoCrypt DAG)
       6. Session completion (rhizoCrypt DAG close)
```

Provenance writes occur at orchestration boundaries so **replay, analysis,
and attribution** remain first-class. All phases degrade gracefully — gameplay
is never blocked by a missing or slow primal.

---

## Creative team story: YAML, CLI, no Rust required

Game content is **YAML-first**: worlds, scenes, narrative graphs, NPCs,
abilities, conditions. Authors use the **CLI** to validate and preview (`validate`, `preview`, `serve` flows).

- **Validation** catches structural and referential errors before runtime.
- **Text preview** allows iteration without the full BYOB stack.
- **No Rust required** for creative roles — only for engine and IPC work.

This keeps the **authoring surface** wide while the **execution surface**
remains strict.

---

## Gap feedback loop

```
Webb exercises composition -> gap discovered
  -> logged in EVOLUTION_GAPS.md with evidence
  -> crafted into wateringHole handoff for owning primal's spring
  -> spring evolves; primal capability appears or hardens
  -> primal redeployed to plasmidBin/
  -> Webb absorbs via capability discovery
  -> next gap surfaces
```

`EVOLUTION_GAPS.md` is the living ledger; handoffs align with wateringHole
standards so evolution is **actionable** and **traceable**.

---

## Primal consumption table (V6)

What Webb **consumes** from each primal via direct IPC or biomeOS
`capability.call` routing. Resolved from `plasmidBin/` or via Songbird
discovery — never from spring source trees:

| Domain | Primal | Webb's use | Representative capabilities |
|--------|--------|------------|----------------------------|
| **AI** | Squirrel | Narration, NPC dialogue, voice analysis | `ai.query`, `ai.suggest`, `ai.analyze` |
| **Visualization** | petalTongue | Scene rendering, input | `visualization.render.scene`, `interaction.poll` |
| **Compute** | ToadStool | GPU dispatch | `compute.dispatch.submit` |
| **Storage** | NestGate | Key-value persistence | `storage.store`, `storage.retrieve` |
| **DAG** | rhizoCrypt | Session provenance | `dag.session.create`, `dag.event.append`, `dag.merkle.root` |
| **Lineage** | LoamSpine | Certificates | `certificate.mint` |
| **Provenance** | sweetGrass | Attribution | `attribution.record` |
| **Local** | (Webb itself) | Flow, engagement, DDA | `science/flow.rs`, `science/engagement.rs`, `science/dda.rs` |

Webb **composes**; it does not duplicate primal internals. Local science is
a stopgap — when a game-science primal emerges (GAP-021), Webb can swap
local calls for IPC without architectural changes.

---

## Phase roadmap (six phases)

1. **Foundations** — YAML content model, narrative graph representation,
   validator CLI, local text preview, no mandatory remote primals.

2. **Director & state** — `GameDirector`, predicate/effect evaluation,
   state dimensions (knowledge, trust, inventory, conditions, arcs, plane, flags),
   pipeline stubs with clear seams for IPC.

3. **IPC spine** — JSON-RPC clients for AI, visualization, and provenance
   primals; health/liveness; capability discovery from `plasmidBin/`;
   graceful degradation when a primal is absent; biomeOS `capability.call`
   fallback routing.

4. **Provenance & certificates** — session DAG/vertex lifecycle,
   certificate queries/mints, attribution on creative outputs; crypto and
   discovery primals wired where required (GAP-019, GAP-006).

5. **Full BYOB play** — `serve` path with rendering, internal voices via
   AI analysis, plane transitions, local science metrics fed back into
   direction and tuning.

6. **Evolution closure** — systematic use of `EVOLUTION_GAPS.md`, wateringHole
   handoffs, absorption of new capabilities, performance and reliability hardening
   for long sessions and large content sets.

---

## Related specifications

- Bounded combinatorial narrative model: `BOUNDED_INFINITE_ARCHITECTURE.md`
- Creator profiles and dual-surface design: `CREATOR_PROFILES_AND_SYSTEM_DESIGN.md`
- Content authoring reference: `CONTENT_AUTHORING_SPEC.md`
- Gap ledger: `../EVOLUTION_GAPS.md`
- Primal deployment surface: `../../../plasmidBin/README.md`

### Science derivation (for reference, not dependency)

Webb's design was informed by these spring-produced specifications. They
are cited as derivation — Webb does not import or depend on their source
trees. The primals produced by these springs are consumed via IPC:

- Game science: RPGPT deep system design, internal voices, NPC personality, planes schema (ludoSpring — patterns absorbed, no runtime dependency)
- Quality profiles: game exemplar and anti-pattern catalog (ludoSpring)
- Provenance: session DAG, certificates, attribution (rhizoCrypt, loamSpine, sweetGrass)
