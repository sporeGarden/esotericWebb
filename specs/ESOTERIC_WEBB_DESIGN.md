<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Esoteric Webb — Core Design

**Status**: Active  
**Date**: March 23, 2026  
**Primals consumed**: Game science, AI/MCP, visualization, session DAG, certificates, attribution, signing, discovery — composed via JSON-RPC IPC  
**Primal source**: `ecoPrimals/plasmidBin/` (genomeBin/ecoBin deployment surface)  
**Science derived from**: ludoSpring, Squirrel, petalTongue, provenance trio — springs produce primals; Webb consumes primals  
**License**: AGPL-3.0-or-later (code); documentation under [CC-BY-SA-4.0](../LICENSE-CC-BY-SA)

---

## Overview

**Esoteric Webb** is a **working tool** — a cross-evolution substrate that
composes deployed primals (genomeBins/ecoBins resolved from `plasmidBin/`)
into a playable creative surface. It is informed by the science in the
springs but anchored in primal capabilities, not spring source code.

Webb is intentionally **not a spring**. Springs are development workspaces
that contain science, experiments, and specs. Springs **produce** primals.
Webb **consumes** primals — like primalSpring does — via JSON-RPC IPC and
capability-based discovery. This makes Webb leaner than the springs it
draws science from, and functionally independent of their internal evolution.

The end result is a **creator/player game system**: a dual-surface tool
where developers work in Rust and creatives work in YAML, both composing
primal capabilities into deep narrative experiences.

---

## Tool vs science boundary

Webb separates the **working tool** from the **scientific foundation**:

| Layer | What it is | Where it lives |
|-------|-----------|----------------|
| **Working tool** (Webb) | Narrative engine, GameDirector, content authoring, CLI, IPC orchestration | `ecoSprings/esotericWebb/` |
| **Deployed primals** | Compiled genomeBin/ecoBin binaries with IPC capabilities | `ecoPrimals/plasmidBin/` |
| **Scientific foundation** | Validated HCI models, experiments, quality profiles, RPGPT theory | Spring source trees (ludoSpring, etc.) — derivation only |

Webb references the science as **derivation** — "this design was informed
by ludoSpring's flow model" — not as a dependency. The game science primal's
`game.evaluate_flow` capability exists because a spring built it; Webb
discovers and calls it without knowing how the spring is structured.

This means:

- **Springs evolve independently** — a spring can refactor, rename crates,
  or restructure without breaking Webb.
- **Webb is leaner** — it carries no spring experiments, no test fixtures for
  other domains, no scientific apparatus. Just the tool.
- **Primals are the contract** — the IPC method signatures and capability
  names are the only coupling. Everything else is derivation.

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

## Architecture: BYOB composition via JSON-RPC IPC

Webb coordinates primals **only** through **JSON-RPC 2.0** IPC (e.g. Unix
domain sockets, capability-based discovery). There are **no cross-primal Rust
crate imports**: the narrative engine owns orchestration; primals own their
domains. Primal binaries are resolved from `plasmidBin/` or discovered via
Songbird at runtime.

**Composition stack (conceptual):**

| Layer | Responsibility |
|-------|----------------|
| **Esoteric Webb** | Narrative direction, scene selection, state machine, content validation, `GameDirector`, metrics hooks |
| **Game science primal** | RPGPT-oriented evaluation (`game.*`), flow/DDA/engagement, voice checks, session/scene hooks |
| **AI primal** | Narration, inference, summarization (`ai.*`) |
| **Visualization primal** | Scene rendering, UI, interaction (`visualization.*`, `ui.*`, `interaction.*`) |
| **Provenance trio** | Session DAG vertices, certificates/rulesets, creative attribution |
| **Tower primals** | Signing/hashing, discovery — as consumed for crypto and lookup |

This is **BYOB** (bring your own binaries): deploy the niche from
`plasmidBin/`, connect sockets, discover capabilities — Webb does not embed
primal implementations.

---

## Key insight: games as compositions of primals

A Webb-shaped game is not "a dialogue system plus combat." It is a **composition
of primals**:

- **Narrative direction** — authored graph + predicates + effects (Webb).
- **Game science** — flow, difficulty, engagement, NPC dialogue metrics (game science primal).
- **AI** — voices, paraphrase, summarization, inference under cert constraints (AI primal).
- **UI** — scene rendering and interaction surfaces (visualization primal).
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

- **Internal voices** — triggered commentary tied to game science and state.
- **Knowledge bounds** — what NPCs know, suspect, lie about, or lack.
- **Trust dynamics** — per-NPC gates that change what is said or offered.
- **Plane transitions** — DAG-recorded shifts of interaction rules while world
  state persists (design derived from RPGPT planes and deep system concepts
  in the game science spring).

---

## The RPGPT pipeline

End-to-end play loop (conceptual):

```
Player input
  -> GameDirector
  -> state evaluation (predicates, arcs, knowledge, trust, plane, flags)
  -> primal orchestration (game science / AI / visualization / provenance)
  -> narration (AI primal + style constraints)
  -> scene rendering (visualization primal)
  -> metrics (engagement, flow, DDA via game science primal)
```

Provenance writes occur at orchestration boundaries so **replay, analysis,
and attribution** remain first-class.

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

## Primal consumption table

What Webb **consumes** from each primal (illustrative; exact method names
evolve with IPC versions). Webb resolves these from `plasmidBin/` or via
Songbird discovery — never from spring source trees:

| Primal | Webb's use | Representative capabilities |
|--------|------------|----------------------------|
| **Game science** | RPGPT hooks, metrics | `game.evaluate_flow`, engagement, DDA, NPC dialogue evaluation, voice checks, session/scene lifecycle |
| **AI** | Narration, inference, compression of state for prompts | `ai.chat`, `ai.inference`, `ai.summarize` |
| **Visualization** | Scene and UI | `visualization.render.scene`, `ui.render`, `interaction.subscribe`, `interaction.poll` |
| **Session DAG** | Provenance vertices | `provenance.session_create`, `provenance.vertex_append`, `provenance.vertex_query` |
| **Certificates** | Rulesets, NPC/voice certificates | `certificate.mint`, `certificate.query` |
| **Attribution** | Creative attribution | `attribution.record`, `attribution.query` |
| **Crypto** | Signing and hashing | `crypto.sign`, `crypto.hash` |
| **Discovery** | Capability-based primal lookup | `discovery.query` |

Webb **orchestrates**; it does not duplicate primal internals.

---

## Phase roadmap (six phases)

1. **Foundations** — YAML content model, narrative graph representation,
   validator CLI, local text preview, no mandatory remote primals.

2. **Director & state** — `GameDirector`, predicate/effect evaluation,
   state dimensions (knowledge, trust, inventory, conditions, arcs, plane, flags),
   pipeline stubs with clear seams for IPC.

3. **IPC spine** — JSON-RPC clients for game science, AI, and visualization
   primals; health/liveness; capability discovery from `plasmidBin/`;
   graceful degradation when a primal is absent.

4. **Provenance & certificates** — session DAG/vertex lifecycle,
   certificate queries/mints, attribution on creative outputs; crypto and
   discovery primals wired where required.

5. **Full BYOB play** — `serve` path with rendering, internal voices hooks,
   plane transitions coordinated with game science rulesets, metrics fed back
   into direction and tuning.

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

- Game science: RPGPT deep system design, internal voices, NPC personality, planes schema (ludoSpring)
- Quality profiles: game exemplar and anti-pattern catalog (ludoSpring)
- Provenance: session DAG, certificates, attribution (rhizoCrypt, loamSpine, sweetGrass)
