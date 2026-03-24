<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Bounded Infinite Architecture — Combinatorial State on Finite Topology

**Status**: Active  
**Date**: March 23, 2026  
**Depends on**: Esoteric Webb narrative model; ludoSpring RPGPT specifications (planes, voices, NPCs)  
**License**: AGPL-3.0-or-later (code); documentation under [CC-BY-SA-4.0](../LICENSE-CC-BY-SA)

---

## Core thesis

**Finite authored topology** plus a **combinatorial state space** yields
**near-infinite traversal** from the player’s perspective, while keeping
**authored content bounded** and **shippable**.

- The **world’s narrative skeleton** is a finite directed graph (typically a
  DAG) of scenes and conditional transitions.
- **Traversal** is not merely “which edge next?” — it is “which edge is
  **enabled** given the current high-dimensional state?”
- The **product** of narrative choices with state dimensions explodes the
  *experienced* path space without requiring an exponentially growing dialogue
  tree.

---

## Narrative topology

**NarrativeGraph** is an **authored DAG** (generalizations may allow controlled
cycles; the default mental model is acyclic beats):

- **Nodes** — scenes (or beat-level units): authored locations for dialogue,
  investigation, tactical resolution, etc.
- **Edges** — **conditional** transitions: guards depend on predicates over
  world state (inventory, flags, trust, knowledge, plane, arc phase, …).

The graph is **small** relative to the number of distinct **plays** because the
same edge can mean different things depending on accumulated state.

---

## State dimensions

Each dimension **multiplies** the traversal space when combined with edge
reevaluation:

| Dimension | Role |
|-----------|------|
| **Knowledge** | Facts the player or NPCs have activated; gates dialogue and investigation |
| **Trust** (per NPC) | Unlocks or locks content; changes risk and revelation |
| **Inventory** | Items and keys to predicates |
| **Conditions** | Timers, wounds, environmental states |
| **Arcs** (per NPC) | Phases of relationship or plot thread |
| **Plane** | Active ruleset / interaction mode (see RPGPT planes) |
| **Flags** | Global and local booleans and enums for authored macros |

Together they form a **combinatorial** layer: the DAG stays fixed; the
**feasible path** through it varies enormously.

---

## Emergence mechanism

**Abilities** (skills, powers, tools) declare **preconditions** and **effects**
on state. Authors author **local** rules; **global** behavior emerges when:

1. Effects update state (knowledge, trust, flags, …).
2. The **NarrativeGraph** reevaluates outgoing edges and eligible scenes.
3. **ludoSpring**-aligned metrics (flow, engagement, DDA) may influence
   tension or optional scaffolding.

Authors cannot enumerate every interaction — that is the point. **Emergence**
is **bounded** by the DAG and certificate/rule constraints, not unbounded LLM
fan-out.

---

## Bounded endings

**Ending nodes** are a **finite** set in the NarrativeGraph. There are not
infinitely many conclusion *types* authored.

What *is* unbounded in feel is the **path**: prior state accumulation makes
each arrival at an ending **distinct in meaning** — who was trusted, what was
known, which plane transitions occurred, which internal voices dominated.

---

## NPC depth without infinite dialogue

Full voice libraries for every branch are infeasible. Depth comes from **structured
bounds** and **assembly**:

- **Knowledge bounds** — per fact or topic: `knows`, `suspects`, `lies_about`,
  `does_not_know` (and related patterns; see ludoSpring NPC specifications).
- **Trust gates** — thresholds that reveal, misdirect, or close topics.
- **Arc phases** — same NPC, different predicate sets as arcs advance.
- **Memory assembly** — retrieval and summarization (e.g. via Squirrel) within
  **certified** personality and knowledge bounds, not unbounded improvisation.

**Reactivity** is the illusion produced by **state × authored lines × gating**,
not infinite bespoke lines.

---

## DAG traceability and provenance

Every action that matters is eligible to become a **provenance vertex** (rhizoCrypt
session DAG, with ruleset and attribution hooks via loamSpine and sweetGrass).

Properties:

- **Replay** — reconstruct *why* the narrative engine chose a transition.
- **Analysis** — debug pacing, dead ends, and metric correlations.
- **Creative attribution** — credit authorship and evolution of content.

This aligns Esoteric Webb with **open, inspectable** story machinery rather
than opaque generative soup.

---

## Comparison to traditional CRPG branching

| Traditional branching tree | Bounded DAG + combinatorial state |
|----------------------------|-----------------------------------|
| Often exponential growth in authored dialogue | Authored **nodes + edges** grow ~linearly in beats |
| Many redundant nodes for similar outcomes | Shared beats; **state** differentiates |
| Hard to maintain consistency | **Predicates and certificates** enforce consistency |
| Replay debugging is narrative-only | **DAG traceability** ties story to mechanics |

The DAG is not “smaller story” — it is **denser** use of authored content.

---

## Mathematical framing (order-of-magnitude)

Let:

- \(N\) = number of narrative nodes (beats/scenes).
- \(S_k, S_t, S_i, S_c, S_a, \ldots\) = effective sizes of state dimensions:
  knowledge, trust, inventory, conditions, arcs (per NPC aggregates or
  representative counts as defined by your content model).

**Authored content** scales roughly as:

\[
O(N) + O\big(\sum_j S_j\big)
\]

— nodes/edges plus **per-dimension** authoring cost (not every combination
explicitly written).

**Experienced traversal space** (distinct feasible histories at coarse
resolution) scales roughly as:

\[
O\big(N \cdot \prod_j f_j(S_j)\big)
\]

where \(f_j\) captures how each dimension participates in edge enablement
(often sublinear per dimension if many values are unreachable together).

This is **not** a formal proof of gameplay depth; it is a **design lens**:
invest in **state** and **predicates** on a **fixed** narrative skeleton.

---

## Science derivation (for reference, not dependency)

This architecture was informed by RPGPT specifications from the game science
spring. These are cited as derivation — Webb does not import or depend on
their source trees. The game science primal produced by that spring is
consumed via IPC:

- RPGPT deep system design — planes, substrate, ruleset certs, pipeline
- RPGPT NPC personality spec — NPC certificates, memory, personality
- RPGPT planes schema — plane definitions and transitions
- RPGPT internal voices spec — internal voices and mechanical triggers
