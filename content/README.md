<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Content Directory — Author Guide

This directory holds all game content for **The Weaver's Parlor**.

## Structure

```
content/
├── meta.yaml          ← world metadata (name, author, version)
├── narrative.yaml     ← narrative graph (nodes, edges, conditions)
├── scenes/            ← scene descriptions (one YAML per location)
├── npcs/              ← NPC definitions (one YAML per character)
├── abilities/         ← player abilities (one YAML per ability)
└── demos/             ← E2E demo scenarios (operator verification)
```

## Quick start

1. Copy an existing file as a template (e.g. `scenes/parlor.yaml`)
2. Give it a unique `id:` field matching its filename (without `.yaml`)
3. Wire it into `narrative.yaml` by adding a node with `content_ref: <id>`
4. Run `esotericwebb validate --content content/` to check for errors
5. Run `esotericwebb demo --content content/` to verify the full pipeline

## File formats

### Scenes (`scenes/*.yaml`)

```yaml
id: room_name
description: "What the player sees when they enter."
npcs: [npc_id_1, npc_id_2]   # optional — NPCs present in this scene
```

### NPCs (`npcs/*.yaml`)

```yaml
id: npc_name
name: "Display Name"
role: their role
knows: [knowledge_key_1, knowledge_key_2]
trust_initial: 0
trust_rewards:
  <level>:
    description: "What happens at this trust level."
    grants_knowledge: [key]   # optional
    grants_items: [item]      # optional
lies_about:
  <topic>:
    detection_dc: <number>
arc: "phase1 -> phase2 -> phase3"
```

### Abilities (`abilities/*.yaml`)

```yaml
id: ability_name
name: "Display Name"
description: "What this ability does."
preconditions: []             # list of state predicates (or empty)
effects:
  - type: add_knowledge       # or: add_item, set_flag
    value: key_name
narration_hint: "Prose shown when the ability fires."
```

### Demo scenarios (`demos/*.yaml`)

```yaml
name: "Scenario title"
description: "What this demo exercises."
steps:
  - action: { kind: "examine|exit|talk|ability", id: "target" }
    description: "Human-readable step label"
    expected:
      outcome_contains: "substring"   # case-insensitive
verification:
  min_turns: 8
  zero_errors: true
```

## Full spec

See `specs/CONTENT_AUTHORING_SPEC.md` for the complete schema reference,
state predicates, condition types, and narrative graph constraints.

## Validation

```bash
# Lint content for structural issues
esotericwebb validate --content content/

# Run the guided demo (E2E verification)
esotericwebb demo --content content/

# Interactive preview (play as human)
esotericwebb preview --content content/
```
