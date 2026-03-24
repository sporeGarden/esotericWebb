<!--
SPDX-FileCopyrightText: Esoteric Webb contributors
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Esoteric Webb — Content Authoring Specification

| Field | Value |
|-------|--------|
| **Status** | Active |
| **Date** | March 23, 2026 |
| **License** | CC-BY-SA-4.0 |
| **Audience** | Creative teams authoring YAML game content |

This document is the reference for writing YAML for Esoteric Webb games. It aligns with the `esoteric-webb` crate’s loaders (`WorldMeta`, `NarrativeGraph`, `SceneContent`, `AbilityDef`) and the narrative validator.

---

## 1. Overview

Creative teams author **YAML files only** — no Rust is required to build or iterate on a world. The **`esotericwebb`** CLI loads the bundle, runs structural validation, and supports text preview and graph visualization.

Typical roles:

- **Narrative**: `meta.yaml`, `narrative.yaml`, `scenes/`
- **Characters**: `npcs/` (personality certificates)
- **Systems**: `abilities/`, `rulesets/`, `worlds/`

---

## 2. Directory structure

Place all authored files under a single content root (often `./content/`):

```
content/
  meta.yaml           — world metadata
  narrative.yaml      — narrative graph (nodes, edges)
  worlds/             — location definitions (*.yaml)
  npcs/               — NPC personality certs (*.yaml)
  abilities/          — spell/ability definitions (*.yaml)
  scenes/             — scene content (*.yaml)
  rulesets/           — per-plane ruleset certs (*.yaml)
```

- **`worlds/`** and **`rulesets/`** are loaded as flexible YAML documents (per file). Use consistent `id` or naming conventions within your team so references stay traceable.
- **`npcs/`**, **`abilities/`**, and **`scenes/`** are indexed by each file’s **`id`** field (see below).

---

## 3. `meta.yaml` format

Top-level fields:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name of the world or campaign. |
| `author` | Yes | Author or team name. |
| `version` | Yes | Semantic version string (e.g. `0.1.0`). |
| `description` | Yes | Short blurb for tooling and previews. |

### Worked example

```yaml
name: "The Brass Labyrinth"
author: "Studio Arc"
version: "0.2.0"
description: >
  A clockwork city where secrets are traded and trust is measured in gears.
```

---

## 4. `narrative.yaml` format

The narrative graph is a map of **nodes** keyed by node ID. Each node defines a scene type, a reference to scene content, optional state predicates/effects, and **exits** (edges).

### Node fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique node id (must match the key under `nodes`). |
| `scene_type` | enum | `dialogue`, `exploration`, `investigation`, `tactical`, `transition`, or `ending`. |
| `content_ref` | string | Key used to look up scene YAML (must equal that scene’s `id` — see §7). |
| `preconditions` | list | [`StatePredicate`](#8-statepredicate-reference) — must hold to enter the node. |
| `effects` | list | [`StateEffect`](#9-stateeffect-reference) — applied when the node is entered. |
| `exits` | list | Outgoing [`NarrativeEdge`](#edge-fields). |
| `is_start` | bool | Exactly one node in the graph must have `true`. |
| `is_ending` | bool | At least one ending node is required. |
| `label` | string (optional) | Label for graph export / debugging. |

### Edge fields

| Field | Type | Description |
|-------|------|-------------|
| `target` | string | Destination node id. |
| `conditions` | list | [`StatePredicate`](#8-statepredicate-reference) — all must hold to follow this edge. |
| `priority` | integer | Higher values sort first when multiple edges are valid. |
| `transition_type` | enum | `same_plane`, `cross_plane`, or `temporal`. |
| `label` | string (optional) | Player- or editor-facing label. |

### Worked example (three nodes)

```yaml
nodes:
  opening:
    id: opening
    scene_type: exploration
    content_ref: scenes/opening.yaml
    preconditions: []
    effects: []
    exits:
      - target: parlor_dialogue
        conditions: []
        priority: 0
        transition_type: same_plane
        label: Enter the parlor
    is_start: true
    is_ending: false
    label: Courtyard

  parlor_dialogue:
    id: parlor_dialogue
    scene_type: dialogue
    content_ref: scenes/parlor.yaml
    preconditions: []
    effects:
      - type: add_knowledge
        value: met_the_host
    exits:
      - target: ending_gate
        conditions:
          - type: trust_above
            value:
              - host_cassian
              - 2
        priority: 1
        transition_type: same_plane
        label: Earn enough trust
    is_start: false
    is_ending: false

  ending_gate:
    id: ending_gate
    scene_type: ending
    content_ref: scenes/ending_gate.yaml
    preconditions: []
    effects: []
    exits: []
    is_start: false
    is_ending: true
    label: Departure
```

---

## 5. NPC certificates (`npcs/*.yaml`)

NPC files follow the spirit of the game science primal’s NPC personality specification (design derived from RPGPT research) — structured personality, not a raw prompt. Esoteric Webb content uses a **flattened** YAML profile suitable for authoring and tooling.

### Fields

| Field | Description |
|-------|-------------|
| `id` | Stable id (matches references in scenes, predicates, trust effects). |
| `name` | Display name. |
| `role` | Short role line (occupation, faction, story hook). |
| `appearance` | Free text. |
| `mannerisms` | List of strings. |
| `motivations` | Maslow-style map: `physiological`, `safety`, `belonging`, `esteem`, `self_actualization` — each may include `urgency`, `current_state`, `threat`, `satisfier`, etc. |
| `knowledge_bounds` | `knows`, `suspects`, `lies_about`, `does_not_know` (see RPGPT spec for rich lie/suspect shapes). |
| `voice` | `cadence`, `vocabulary`, `quirks` (lists or strings as you prefer). |
| `secrets` | List of objects with `id`, `description`, `reveal_conditions`. |
| `relationships` | List of entities (name/id, type, strength, notes). |
| `arc` | `phases`: list with `id`, `description`, `conditions` (when this phase applies or advances). |
| `trust_model` | `initial` trust level, `thresholds` (map or list), `level_effects` (what changes at each level). |

### Worked example NPC

```yaml
id: host_cassian
name: "Cassian Voss"
role: "Patron of the Brass Hall — negotiator, not a friend"
appearance: "Silver at the temples, ink-stained fingers, always between two appointments."
mannerisms:
  - "Taps a brass ring on the table when impatient"
  - "Never gives a straight 'yes' on the first pass"
motivations:
  physiological:
    urgency: 0.1
    current_state: "Comfortable; meals and lodging are solved."
  safety:
    urgency: 0.6
    current_state: "Reputation and contracts protect him — until they don't."
  belonging:
    urgency: 0.4
    current_state: "The Hall is his family; outsiders are inventory."
  esteem:
    urgency: 0.7
    current_state: "Being seen as indispensable matters more than being liked."
  self_actualization:
    urgency: 0.3
    current_state: "Wants his name on something that outlasts the season."
knowledge_bounds:
  knows:
    - "Who holds the key to the east gate"
    - "Which guests are in debt to the Hall"
  suspects:
    - topic: "The leak in the guild books"
      belief: "Someone inside is selling routes"
      confidence: 0.55
  lies_about:
    - topic: "His stake in the smeltery"
      surface_claim: "Advisory only"
      truth: "Silent majority partner"
      reason: "Regulatory scrutiny"
  does_not_know:
    - "The player's true employer"
voice:
  cadence: "Measured; leaves room for others to overcommit"
  vocabulary: "Formal mercantile, occasional street slang for effect"
  quirks:
    - "Quotes contract clauses like proverbs"
secrets:
  - id: smeltery_deal
    description: "Controls the smeltery through a shell guild"
    reveal_conditions:
      - "trust >= 4"
      - "Player presents evidence from the docks"
relationships:
  - entity: "Brass Hall"
    type: institutional
    strength: 0.85
arc:
  phases:
    - id: sizing_up
      description: "Tests the player with small asks"
      conditions:
        - "first meeting"
    - id: partnership
      description: "Treats the player as an asset"
      conditions:
        - "trust >= 3"
trust_model:
  initial: 0
  thresholds: [1, 2, 3, 4, 5]
  level_effects:
    0: "Polite, transactional"
    2: "Shares non-critical rumors"
    4: "Admits the smeltery angle if pressed with proof"
```

---

## 6. Abilities (`abilities/*.yaml`)

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique id. |
| `name` | string | Display name. |
| `description` | string | Player-facing text. |
| `preconditions` | list | [`StatePredicate`](#8-statepredicate-reference). |
| `effects` | list | [`StateEffect`](#9-stateeffect-reference). |
| `narration_hint` | string (optional) | Hint for AI or director narration. |

### Worked example

```yaml
id: brass_insight
name: "Brass Insight"
description: "Read the city's hidden incentives in the hum of the forges."
preconditions:
  - type: in_plane
    value: exploration
  - type: has_knowledge
    value: met_the_host
effects:
  - type: add_knowledge
    value: guild_pressure_map
  - type: modify_trust
    value:
      - host_cassian
      - 1
narration_hint: "Focus on sound, heat, and who profits when the tempo shifts."
```

---

## 7. Scenes (`scenes/*.yaml`)

Each file is one scene. The loader indexes scenes by **`id`** (not by filename). **`narrative.yaml` `content_ref` must match this `id` exactly.**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique id — must match `content_ref` on narrative nodes. |
| `description` | string | Text preview / GM description. |
| `npcs` | list of strings | NPC ids present in this scene (must exist under `npcs/`). |
| `items` | list of strings | Item ids available here (must match item ids used in predicates/effects). |

### Worked example

```yaml
id: scenes/parlor.yaml
description: >
  A low-ceilinged room lined with contracts. Cassian gestures to a chair
  that looks comfortable until you sit.
npcs:
  - host_cassian
items:
  - brass_signet
```

---

## 8. `StatePredicate` reference

Predicates are **adjacently tagged** in YAML: `type` (snake_case variant name) and `value` (payload). Single-argument variants use a scalar `value`; multi-argument variants use a **sequence** of arguments in order.

| Variant | Meaning | YAML example |
|---------|---------|----------------|
| `has_knowledge` | Player has knowledge fragment `k`. | `type: has_knowledge` / `value: elder_sign` |
| `lacks_knowledge` | Player does not have `k`. | `type: lacks_knowledge` / `value: elder_sign` |
| `trust_above` | Trust with NPC `n` ≥ `t`. | See below |
| `trust_below` | Trust with NPC `n` < `t`. | `type: trust_below` / `value: [npc_id, 2]` |
| `in_plane` | Current plane is `p`. | `type: in_plane` / `value: dialogue` |
| `has_item` | Player has item `i`. | `type: has_item` / `value: brass_signet` |
| `lacks_item` | Player lacks item `i`. | `type: lacks_item` / `value: cursed_ring` |
| `condition_active` | Condition `c` is active. | `type: condition_active` / `value: blessed` |
| `condition_inactive` | Condition `c` is not active. | `type: condition_inactive` / `value: blessed` |
| `arc_phase_is` | NPC `n` arc phase is `phase`. | `type: arc_phase_is` / `value: [npc_id, phase_id]` |
| `flag_set` | Flag `f` is set. | `type: flag_set` / `value: east_gate_open` |
| `flag_unset` | Flag `f` is not set. | `type: flag_unset` / `value: east_gate_open` |
| `all` | Every sub-predicate is true. | `type: all` / `value: [ ... predicates ... ]` |
| `any` | At least one sub-predicate is true. | `type: any` / `value: [ ... ]` |
| `not` | Inner predicate is false. | `type: not` / `value: { type: has_item, value: ... }` |

Two-argument `value` examples:

```yaml
type: trust_above
value:
  - host_cassian
  - 3
```

```yaml
type: arc_phase_is
value:
  - host_cassian
  - partnership
```

Compound example:

```yaml
type: all
value:
  - type: has_knowledge
    value: clue_routes
  - type: not
    value:
      type: flag_set
      value: alarm_raised
```

---

## 9. `StateEffect` reference

Effects use the same `type` / `value` pattern as predicates.

| Variant | Meaning | YAML example |
|---------|---------|----------------|
| `add_knowledge` | Learn fragment `k`. | `type: add_knowledge` / `value: clue_routes` |
| `remove_knowledge` | Remove `k`. | `type: remove_knowledge` / `value: rumor_false` |
| `modify_trust` | Adjust trust with NPC by delta `d`. | `type: modify_trust` / `value: [npc_id, 1]` |
| `set_trust` | Set trust to exact `t`. | `type: set_trust` / `value: [npc_id, 3]` |
| `add_item` | Gain item `i`. | `type: add_item` / `value: brass_signet` |
| `remove_item` | Lose item `i`. | `type: remove_item` / `value: brass_signet` |
| `apply_condition` | Apply condition `c` for `dur` turns (`0` = permanent). | See below |
| `remove_condition` | Remove condition `c`. | `type: remove_condition` / `value: blessed` |
| `advance_arc` | Set NPC `n` arc to `phase`. | `type: advance_arc` / `value: [npc_id, phase_id]` |
| `transition_plane` | Move to plane `p`. | `type: transition_plane` / `value: tactical` |
| `set_flag` | Set flag `f`. | `type: set_flag` / `value: east_gate_open` |
| `clear_flag` | Clear flag `f`. | `type: clear_flag` / `value: east_gate_open` |
| `batch` | Apply sub-effects in order. | `type: batch` / `value: [ ... effects ... ]` |

```yaml
type: apply_condition
value:
  - blessed
  - 3
```

```yaml
type: batch
value:
  - type: add_knowledge
    value: finale_unlocked
  - type: set_flag
    value: story_complete
```

---

## 10. Validation

Run:

```bash
esotericwebb validate --content ./content/
```

The loader parses YAML; **validation** checks (among other things):

| Check | Description |
|-------|-------------|
| **Narrative structure** | Exactly one `is_start` node; at least one `is_ending` node; every edge `target` exists. |
| **Orphan nodes** | Every node except the start is reachable from at least one incoming edge. |
| **Ending reachability** | Every ending node is reachable from the start node. |
| **Scene references** | For each narrative node with a non-empty `content_ref`, a loaded scene with that **`id`** exists. |
| **NPC references** | For each scene, every id in `npcs` exists in `npcs/*.yaml`. |
| **Predicate consistency** | Authoring discipline: use the same string ids for NPCs, items, knowledge fragments, planes, flags, conditions, and arc phases across predicates, effects, and definitions. Invalid YAML or unresolved structural references will fail the pipeline; keep identifiers consistent so runtime evaluation matches design intent. |

If validation fails, the CLI prints each issue on stderr and exits non-zero.

---

## 11. Workflow

1. **Edit** YAML under your content root.
2. **Validate** with `esotericwebb validate --content ./content/`.
3. **Preview** with `esotericwebb preview --content ./content/` (requires a passing validate for preview).
4. **Iterate** until validation is clean.
5. **Commit** the content tree to version control.

Optional: `esotericwebb graph --content ./content/` exports the narrative graph as **DOT** for visualization.

---

## See also

- Bounded narrative model: `ESOTERIC_WEBB_DESIGN.md` (same directory).
- Full NPC certificate depth: derived from game science spring RPGPT NPC personality research.
