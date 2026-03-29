<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Creator Profiles and System Design

**Status**: Active (V6)
**Date**: March 29, 2026
**Purpose**: Profile the creative teams that Esoteric Webb is built for, map their design DNA to system capabilities, and define the dual-surface (developer + creative) architecture.
**License**: AGPL-3.0-or-later (code); documentation under [CC-BY-SA-4.0](../LICENSE-CC-BY-SA)
**Science derived from**: Game science spring quality profiles and RPGPT research (derivation, not dependency)
**Identity**: sporeGarden composition — deploys primal compositions, no spring runtime dependencies

---

## Who This System Is For

Esoteric Webb exists at the intersection of two user populations:

1. **Developers** — build engine capabilities, IPC integrations, state systems,
   and the BYOB composition pipeline. They work in Rust and TOML deploy graphs.

2. **Creatives** — narrative designers, game writers, world builders, and the
   small studio leads who ship games like Disco Elysium with a novel's worth of
   dialogue and a playwright's ear for character. They work in YAML, pen, and
   paper.

The system fails if either surface is neglected. A brilliant engine that
creatives cannot author for is a tech demo. A beautiful story in YAML that
the engine cannot trace, gate, and compose is a static document.

---

## Creator Profile: ZA/UM (Disco Elysium)

### Team shape

ZA/UM began as a collective of Estonian and Finnish writers, artists, and
musicians. The studio was small (15-30 during core development). Lead
designer Robert Kurvitz was a novelist. The lead artist Aleksander Rostov
painted. The core team came from literature, theater, and tabletop gaming —
not AAA game development.

### Creative DNA

| Trait | What it means | How Webb supports it |
|-------|--------------|---------------------|
| **Literature-first** | The game was a novel before it was a game. Dialogue is the primary medium. Mechanics serve narrative, not the reverse. | YAML scene authoring with no code requirement. The narrative graph IS the manuscript. |
| **Skills as perspectives** | Each skill (Logic, Empathy, Electrochemistry, Inland Empire) is a character with opinions, not a number to optimize. | `VoiceId` system (derived from RPGPT internal voices spec). Voices have personality parameters, temperature, forbidden topics. Webb's `StatePredicate` triggers voice interjections based on game state, not random rolls. |
| **Failure as content** | Failed skill checks produce unique dialogue, not "try again." Failing a Rhetoric check gives you embarrassing lines that advance the story differently. | `StateEffect` mutations on both success and failure branches. The narrative graph has edges for BOTH outcomes. No dead ends from failure. |
| **World as character** | Martinaise (the district) has opinions, moods, weather that responds to time. The space is small but dense. | World state as first-class `WorldState` dimension. Location descriptions are state-gated — the same room reads differently at trust level 0 vs trust level 5 with a specific NPC. |
| **No combat** | DE deliberately chose to have zero combat. Every conflict resolves through dialogue, introspection, or environmental interaction. | Planes are optional. A DE-style game uses Dialogue + Investigation + Exploration planes and ignores Tactical entirely. `RulesetCert` defines which planes are active. |
| **Political and personal** | The game does not shy from ideology, identity, mental health, addiction, or failure. It trusts the player to encounter difficult content. | Content authoring is unconstrained. The system does not censor authored content. Pathogen detection (pattern derived from game science research) identifies exploitative mechanics, not provocative themes. Art is a human medium. |

### What ZA/UM needed that didn't exist

- A way to author 1,000,000+ words of branching dialogue without drowning
  in version control hell. → **YAML narrative graph with CLI validation**.
- Voice interjections that fire contextually, not randomly. → **StatePredicate-driven
  voice system with priority caps and personality constraints**.
- NPCs who remember, lie, and evolve without infinite bespoke lines. →
  **Knowledge bounds + trust gates + arc phases + memory assembly**.
- The ability to test "does this ending remain reachable if the player does X
  in act 2?" → **NarrativeGraph validator with reachability analysis**.
- A system where one writer's changes don't silently break another's quest
  line. → **Cross-reference validation: all NPC refs resolve, all scene refs
  exist, all predicates reference valid state keys**.

---

## Creator Profile: Cliche Studio (Esoteric Ebb)

### Team shape

Small independent studio. Merged Disco Elysium's voice/skill system with
genuine tabletop mechanics (dice pools, degrees of success, visible DCs).
Creative direction driven by a desire to prove that narrative depth and
mechanical crunch are not mutually exclusive.

### Creative DNA

| Trait | What it means | How Webb supports it |
|-------|--------------|---------------------|
| **Voices + dice** | Internal voices comment on mechanical outcomes, not just narrative beats. A Logic voice might note that the DC was unusually high, implying deception. | Voice triggers can reference mechanical state: `ConditionActive("high_dc_detected")`, `FlagSet("deception_suspected")`. Squirrel narrates within personality constraints. |
| **Transparent mechanics** | Players see dice, DCs, modifiers. No hidden rolls. Risk assessment is part of the gameplay. | `RulesetCert` defines dice systems per plane. Voice analysis via Squirrel (`ai.analyze`) evaluates passive checks. Scene DTOs include `DiceResultScene` with optional BearDog signatures. |
| **Multi-plane play** | Investigation, dialogue, and tactical play coexist. Moving from a conversation to a fight preserves world state — the NPC remembers what you said. | `PlaneTransition` as first-class narrative nodes. `WorldStateSnapshot` for condition mapping. Trust, knowledge, and inventory persist across plane boundaries. |
| **Mechanical storytelling** | Game mechanics tell stories. A critical failure on a Rhetoric check isn't just "you fail" — it's "you said something so catastrophically wrong that the NPC's arc advances in a direction you didn't intend." | `StateEffect::AdvanceArc` on failure branches. The narrative graph encodes mechanical consequences as state mutations, not just text flavor. |

### What Cliche Studio needed

- Ruleset definitions that travel with the game, not hardcoded in engine. →
  **YAML `RulesetCert` per plane, validated at load time**.
- A way to test that plane transitions preserve state invariants. →
  **Transition validator (inspired by ludoSpring exp075)**.
- Mechanical outcomes that feed back into narrative branching without
  requiring a programmer for every new interaction. → **`StateEffect` and
  `StatePredicate` are composable by writers in YAML**.

---

## Creator Profile: The Solo Author

### Team shape

One person. Maybe two. Four years in a room with a vision.

This is Eric Barone (Stardew Valley). This is Lucas Pope (Return of the Obra
Dinn). This is the person the Lysogeny catalog (from game science research)
was built for.

### What they need

| Need | How Webb delivers |
|------|-------------------|
| **No engine team** | BYOB composition means the engine IS the primals. Write YAML, run `esotericwebb preview`, iterate. No Unity license, no Unreal royalties, no C++ compilation. |
| **No AI team** | Squirrel handles inference, narration, summarization. The solo author writes personality certs and knowledge bounds. Squirrel generates within constraints. |
| **No QA team** | `esotericwebb validate` catches broken references, unreachable endings, orphan nodes, and predicate inconsistencies. The CI pipeline catches what manual testing misses. |
| **Provenance from day one** | Every playtest session is a DAG. The solo author can replay, analyze pacing, identify dead paths, and attribute creative evolution. |
| **Ship without a publisher** | AGPL + ORC + CC-BY-SA. The tools are free. The math is open. The content belongs to the creator. |

---

## Dual-Surface Architecture

Esoteric Webb maintains two interaction surfaces:

### Developer surface

| Tool | Purpose |
|------|---------|
| Rust crate (`webb/`) | Narrative engine, IPC clients, GameDirector, state machine, content loader |
| Deploy graphs (`graphs/`) | BYOB composition of primals into a running niche |
| IPC server | Webb's own JSON-RPC surface for health, scene status, MCP tools |
| Validation experiments | Structural tests: reachability, emergence, NPC depth, provenance trace |
| `EVOLUTION_GAPS.md` | Living gap tracker feeding spring evolution |

### Creative surface

| Tool | Purpose |
|------|---------|
| YAML content (`content/`) | Worlds, NPCs, abilities, scenes, narrative graph — authored by writers |
| `esotericwebb validate` | Lint content for broken refs, unreachable endings, orphan nodes |
| `esotericwebb preview` | Text-mode game preview — no primals required, immediate iteration |
| `esotericwebb graph` | Visualize narrative DAG as DOT/SVG — see the story structure |
| `esotericwebb new-world` | Scaffold a blank content directory with template YAML |
| `CONTENT_AUTHORING_SPEC.md` | Full YAML format reference with worked examples |

### Player surface (future)

| Feature | Purpose |
|---------|---------|
| `esotericwebb serve` | Full BYOB niche with rendered scenes, AI narration, internal voices |
| Provenance replay | Step through past sessions, see branching points, understand endings |
| Mod support | Players create content packs (new worlds, NPCs, abilities) using the same YAML system creators use |
| Community attribution | sweetGrass tracks who contributed what — mods carry creator lineage |

---

## Design Principles From Case Studies

These principles are extracted from studying the creator profiles above:

### 1. Mechanics serve narrative

Never add a mechanic that doesn't serve the story being told. If a game has
no combat, it needs no Tactical plane. If a game's horror comes from the
unreliability of rules, the `RulesetCert` should encode that unreliability
as a feature, not a bug.

### 2. Failure is content

Every branch in the narrative graph should have edges for failure outcomes.
A failed skill check is not a dead end — it's a different story. The
`StateEffect` system encodes failure consequences as first-class mutations.

### 3. NPCs are bounded, not infinite

The temptation with AI narration is to let NPCs say anything. This produces
chatbot swill. NPCs must have knowledge bounds, trust gates, lies with
mechanical detection DCs, secrets with reveal conditions, and arc phases.
Depth comes from authored constraints + state combinatorics, not from
unbounded generation.

### 4. Transparent when chosen

Some games benefit from visible dice and DCs (Esoteric Ebb). Others benefit
from hidden mechanics (Disco Elysium). Both are valid. The `RulesetCert`
and scene DTOs support both modes — the choice belongs to the creator.

### 5. The creative owns the art

The system does not censor, filter, or "align" creative content. Content
moderation is the creator's responsibility. The Pathogen detector identifies
exploitative game mechanics (loot boxes, Skinner loops), not provocative
themes. Storytelling and art are human mediums that the system exists to
expand, not constrain.

### 6. Ship without permission

AGPL + ORC + CC-BY-SA means no publisher veto, no storefront lock-in, no
royalty negotiation for engine access. The solo author ships when the work
is done.

---

## System Capabilities Mapped to Creator Needs

| Creator need | System capability | Status |
|-------------|-------------------|--------|
| Author branching narrative without code | YAML `narrative.yaml` + NarrativeGraph engine | Implemented (V1) |
| Define NPC depth without infinite dialogue | `npcs/*.yaml` with knowledge bounds, trust, arcs | Implemented (V1) |
| Create abilities that interact emergently | `abilities/*.yaml` with StatePredicate/StateEffect | Implemented (V1) |
| Validate content correctness | `esotericwebb validate` CLI | Implemented (V1) |
| Preview without full stack | `esotericwebb preview` (text mode) | Implemented (V1) |
| Visualize narrative structure | `esotericwebb graph` (DOT output) | Implemented (V1) |
| Scaffold new worlds | `esotericwebb new-world` | Implemented (V1) |
| AI-generated narration within constraints | AI primal IPC with personality certs | Degradation stub (GAP-003) |
| Rendered game UI | Visualization primal scene rendering | Degradation stub (GAP-002) |
| Session provenance and replay | Provenance primal vertex DAG | Local fallback (GAP-004) |
| Live BYOB composition | `esotericwebb serve` with full primal stack | Wired (V4+), degrades when primals absent |
| Internal voice interjections | Squirrel `ai.analyze` voice check + `ai.query` narration | IPC wired (V6), personality constraints enforced via certs |
| Multi-plane transitions | Plane transition nodes in narrative graph | Narrative engine supports; science metrics via local `science/` module |
| Transparent/hidden dice modes | RulesetCert per plane | YAML format defined; runtime evaluation via local science + future primal (GAP-021) |

---

## Roadmap: Creator-Facing Evolution

### V1 (current) — Offline authoring and validation

Creatives can author, validate, preview, and visualize without any running
primals. The Weaver's Parlor demonstrates the full content format.

### V2 — AI-assisted authoring

- AI primal integration: generate NPC dialogue samples within personality cert
  constraints for preview.
- Voice interjection preview: simulate which voices would fire given authored
  trigger conditions.
- Narration style preview: show how the AI would describe scene transitions.

### V3 — Live play and testing

- Full `esotericwebb serve` with visualization primal rendering.
- Live DDA feedback: local science module evaluates pacing during playtesting.
- Session recording and replay via provenance trio.

### V4 — Community and modding

- Content pack format (zip of YAML content directories).
- Mod validation (same `esotericwebb validate` on community content).
- Attribution tracking for derivative works via the attribution primal.

---

## References

### Webb specs

- `BOUNDED_INFINITE_ARCHITECTURE.md` (DAG + state theory)
- `CONTENT_AUTHORING_SPEC.md` (YAML format reference)
- `ESOTERIC_WEBB_DESIGN.md` (architecture overview)

### Science derivation (for reference, not dependency)

- Game science spring: quality profiles, RPGPT deep system design, NPC personality, internal voices, Lysogeny catalog
- AI spring: inference, personality-constrained generation
- Provenance trio: session DAG, certificates, attribution
