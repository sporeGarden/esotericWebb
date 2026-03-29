# Evolution Gaps

Living document tracking gaps discovered by Esoteric Webb as it exercises
the primal stack. Each gap references the **primal** that needs to evolve
(not the spring — springs produce primals; Webb consumes primals from
`plasmidBin/`). Gaps feed back to the owning spring via wateringHole handoffs.

## How this works

```
Webb exercises primal composition -> discovers gap in a primal capability
  -> logged here with evidence
  -> crafted into wateringHole handoff for the spring that produces the primal
  -> spring evolves, rebuilds primal -> new genomeBin deployed to plasmidBin/
  -> Webb absorbs via capability discovery
  -> next gap surfaces
```

## Gap template

```markdown
### GAP-NNN: <short description>

- **Primal**: <primal capability domain affected>
- **Spring (producer)**: <spring that builds the primal — for handoff routing>
- **Severity**: critical / high / medium / low
- **Evidence**: <what Webb tried to do and what happened>
- **Expected**: <what capability or behavior is needed>
- **Workaround**: <graceful degradation path Webb uses>
- **Handoff**: <link to wateringHole handoff once filed>
- **Status**: open / filed / absorbed
```

---

## Open gaps

### GAP-002: Visualization primal lacks CRPG dialogue tree scene type

- **Primal**: visualization (`visualization.render.scene`)
- **Spring (producer)**: petalTongue
- **Severity**: medium
- **Evidence**: Webb defines `DialogueTreeScene` payloads but the
  `visualization.render.scene` capability has not confirmed support for
  dialogue tree rendering with choice highlighting, voice interjection
  panels, or skill check result display.
- **Expected**: The visualization primal accepts a `DialogueTreeScene`
  payload and renders it as an interactive dialogue UI with choices,
  voice notes, and skill checks.
- **Workaround**: Webb uses text-mode preview (`esotericwebb preview`)
  which renders to stdout without the visualization primal.
- **Handoff**: File to wateringHole when the visualization primal reaches
  RPGPT UI phase.
- **Status**: open

### GAP-003: AI primal NPC dialogue constraint enforcement

- **Primal**: AI (`ai.chat`, `ai.inference`)
- **Spring (producer)**: Squirrel
- **Severity**: medium
- **Evidence**: Webb's NPC personality certs define knowledge bounds, trust
  gates, lies with detection DCs, and voice constraints. When Webb calls
  `game.npc_dialogue` (which the game science primal delegates to the AI
  primal), the AI primal needs to respect these constraints mechanically —
  not just as system prompt guidance.
- **Expected**: The AI primal accepts an NPC personality cert and enforces
  knowledge bounds, lies, and trust gates as hard constraints on generated
  dialogue, not soft prompt suggestions.
- **Workaround**: Webb validates NPC responses client-side and can
  reject/retry responses that violate constraints. GameDirector enforces
  knowledge bounds independently of AI generation.
- **Handoff**: File to wateringHole when the AI primal reaches RPGPT
  personality constraint phase.
- **Status**: open

### GAP-004: Provenance trio session DAG not wired end-to-end

- **Primal**: provenance (`dag.*`), lineage (`spine.*`, `entry.*`), attribution (`braid.*`, `provenance.*`)
- **Spring (producer)**: rhizoCrypt, loamSpine, sweetGrass
- **Severity**: low (structure ready; blocked on `provenance-trio-types` shared crate)
- **Evidence**: Webb uses a local `ProvenanceClient` fallback that records
  vertices in-memory. exp005 validates vertex recording works locally.
  Webb now has BFS depth layers (`NarrativeGraph::bfs_depths()`) and edge
  classification (forward/back/lateral) that serve as the local projection
  of what rhizoCrypt does at runtime — the cyclic navigation graph gets
  projected onto an acyclic temporal trace. The BFS engine is the test
  surface for validating against rhizoCrypt's `dag.event.append` /
  `dag.vertex.get` / `dag.frontier.get` once the primal is available.
- **Expected**: Full provenance cycle mapping to trio responsibilities:
  - **rhizoCrypt** (branching engine): `dag.session.create` at game start,
    `dag.event.append` per player action (each action = new vertex, even
    revisiting the same room — cyclic navigation becomes acyclic temporal
    DAG), `dag.frontier.get` for save points, `dag.slice.checkout` for
    load, `dag.merkle.root/proof` for anti-cheat/integrity.
  - **loamSpine** (lineage tracker): `spine.create` per session,
    `entry.append` tracking causal chain (which action caused which state
    change), `certificate.mint` for NPC personality certs,
    `session.commit` to seal a completed playthrough.
  - **sweetGrass** (attribution/story): `braid.create` linking creative
    contributions (authored content, AI-generated narration, player
    choices), `attribution.chain` for crediting content authors,
    `provenance.graph` for the full story of a playthrough, exportable
    as PROV-O.
- **Local readiness**: Webb's BFS depth layers, edge classification
  (forward/back/lateral), `DagOverlay`, and `to_graph_json()` provide
  the structural vocabulary that maps directly to rhizoCrypt operations.
  The local `ProvenanceClient` vertex log is exportable for batch import
  via `dag.event.append_batch` when the primal is deployed.
- **Blocker**: ~~`provenance-trio-types` shared crate~~ — resolved. The
  shared types crate was an interconnect relic from the compile-time
  coupling era. All three primals have evolved to standalone projects
  (phase2/) that build independently and communicate over IPC. No shared
  Rust crate dependency exists in any Cargo.toml.
- **Deployment**: All three trio binaries built and harvested to
  `ecoPrimals/plasmidBin/` (2026-03-24):
  - `rhizocrypt` v0.14.0-dev (5.7M, domain: dag)
  - `loamspine` v0.9.13 (8.3M, domain: lineage)
  - `sweetgrass` v0.7.27 (12M, domain: provenance)
- **Progress (V4)**: Full provenance lifecycle wired into GameSession.
  `initialize_provenance()` calls `dag.session.create` on session start and
  stores the real session_id in WorldState. Every `act()` appends a vertex
  via `dag.event.append` with the real session_id. `complete_provenance_if_ended()`
  calls `dag.session.complete` when an ending is reached. PrimalBridge now
  has `dag_session_complete()` and `dag_query_vertices()`.
- **Next**: Integration test against live rhizoCrypt binary from plasmidBin,
  `dag.slice.checkout` for save/load, `dag.event.append_batch` for bulk import.
- **Status**: wiring complete (V4), live end-to-end validation pending

### GAP-006: Discovery primal capability-filtered queries

- **Primal**: discovery (`discovery.query`)
- **Spring (producer)**: Songbird
- **Severity**: medium
- **Evidence**: Webb's `PrimalRegistry::discover()` probes filesystem socket
  directories but does not call the discovery primal's `discovery.query`
  for tier-5 lookup. In a composed niche, the discovery primal is the
  canonical mechanism.
- **Expected**: After filesystem probe, Webb queries the discovery primal
  for any primals not found locally, using `discovery.query` with
  capability filters.
- **Workaround**: Filesystem probe covers tiers 1-4. Tier-5 is logged as
  degraded but functional.
- **Handoff**: File when the discovery primal confirms response format for
  capability-filtered queries.
- **Status**: open

### GAP-007: Voice interjection preview without live AI primal

- **Primal**: AI (`ai.chat`)
- **Spring (producer)**: esotericWebb (self) + Squirrel
- **Severity**: medium
- **Evidence**: Creators profiled in CREATOR_PROFILES_AND_SYSTEM_DESIGN.md
  (ZA/UM, Cliche Studio) need to preview which internal voices fire during
  scene transitions while authoring. Currently `esotericwebb preview` shows
  scene descriptions but cannot simulate voice interjections without a
  running AI primal.
- **Expected**: Offline voice simulation: given authored VoiceId triggers in
  narrative.yaml and NPC certs, show which voices would fire and with what
  priority, using placeholder text that reflects personality parameters.
- **Workaround**: Creators mentally trace voice triggers from YAML. No
  automated preview.
- **Handoff**: Self-owned for offline simulation; AI primal spring for live
  personality-constrained generation.
- **Status**: open

### GAP-008: Creative content pack format for distribution

- **Primal**: N/A (internal tooling)
- **Spring (producer)**: esotericWebb (self)
- **Severity**: low
- **Evidence**: The solo author profile (CREATOR_PROFILES_AND_SYSTEM_DESIGN.md)
  identifies the need to ship content without a publisher. Currently content
  is a loose directory of YAML files with no packaging, versioning, or
  signature format for distribution.
- **Expected**: A content pack format (zip or tar of content directory) with
  manifest, version, author attribution, and optional crypto primal
  signature for integrity verification. `esotericwebb validate --pack`
  validates a pack.
- **Workaround**: Distribute as git repository or zip by hand.
- **Handoff**: N/A (self-owned)
- **Status**: open

### GAP-009: RulesetCert YAML authoring and per-plane validation

- **Primal**: game science (`game.ruleset_validate`)
- **Spring (producer)**: esotericWebb (self) + ludoSpring
- **Severity**: medium
- **Evidence**: Cliche Studio's creative DNA (transparent dice, multi-plane
  play) requires RulesetCert definitions per plane (Investigation, Dialogue,
  Tactical, Crafting). The CONTENT_AUTHORING_SPEC defines a rulesets/
  directory but the content loader does not yet parse or validate RulesetCert
  YAML against the game science primal's expected schema.
- **Expected**: YAML rulesets/ loaded, validated against a schema compatible
  with the game science primal's RulesetCert structure. `esotericwebb
  validate` reports ruleset errors. The game science primal's
  `game.ruleset_validate` confirms compatibility at composition time.
- **Workaround**: Rulesets loaded as opaque YAML documents. No structural
  validation beyond well-formedness.
- **Handoff**: Self-owned for loader; game science primal spring for
  validation endpoint.
- **Status**: open

### GAP-010: plasmidBin population and deployment automation

- **Primal**: all (deployment infrastructure)
- **Spring (producer)**: ecosystem (biomeOS, primalSpring)
- **Severity**: medium
- **Evidence**: `ecoPrimals/plasmidBin/` has been established as the primal
  deployment surface but is not yet populated with actual genomeBin/ecoBin
  artifacts. Webb's BYOB deploy graph references primals by capability but
  cannot resolve them until binaries land in `plasmidBin/`.
- **Expected**: CI pipelines or `genome fetch` tooling populate `plasmidBin/`
  with versioned, checksummed, PIE-verified primal binaries. A
  `manifest.lock` tracks deployed state.
- **Workaround**: Webb operates in offline/preview mode. Primals are
  discovered locally if manually started.
- **Handoff**: biomeOS/primalSpring for deployment tooling.
- **Status**: open

---

## Absorbed gaps

### GAP-001: IPC clients are degradation stubs → RESOLVED (V4, 2026-03-24)

All primal domains wired into `GameSession::act()` via `PrimalBridge` with
23 bridge methods, retry + circuit breaker resilience, and graceful
degradation. Full composition pipeline: AI narration → NPC dialogue → flow
evaluation → scene push → provenance lifecycle. IPC handler split, MCP
JSON Schema, sourDough compliance all complete.

### GAP-005: Content YAML format alignment → RESOLVED (V3, 2026-03-24)

YAML roundtrip tests added for all content types (`WorldMeta`, `SceneContent`,
`AbilityDef`, `NpcDef`). Scaffold-then-load roundtrip verified. Content
loader and serde types fully aligned.

### GAP-011: Semantic IpcError classification → RESOLVED (V5, 2026-03-25)

`IpcError` refactored from flat variants to ecosystem-aligned semantic
classification (`ConnectionRefused`, `Timeout`, `MethodNotFound`,
`ProtocolError`, `ApplicationError`, `PrimalNotFound`) with helper methods
`is_retriable()`, `is_recoverable()`, `is_connection_error()`,
`is_method_not_found()`. Aligns with primalSpring error handling patterns.
`classify_io_error()` normalizes OS-level errors to semantic types.

### GAP-012: Primal name duplication → RESOLVED (V5, 2026-03-25)

Created canonical `ipc/primal_names.rs` module as single source of truth for
all primal slugs, display names, domains, and domain→primal mappings.
Eliminated duplicate KNOWN_PRIMALS and PRIMAL_DOMAINS constants across
discovery, bridge, and handlers. All consumers now reference one canonical list.

### GAP-013: Session module exceeds 1000-line limit → RESOLVED (V5, 2026-03-25)

Smart refactoring of `session.rs` (1192 lines) into three logical modules:
`session/mod.rs` (891 lines, core session logic), `session/types.rs` (data
structures), `session/enrichment.rs` (6-phase primal composition pipeline).
Preserves cohesion while meeting the 1000-line quality gate.

### GAP-014: println logging → RESOLVED (V5, 2026-03-25)

All `println!`/`eprintln!` in production code replaced with `tracing::info!`
and `tracing::warn!` for structured observability. Affects launcher.rs,
commands/mod.rs, and listener.rs.

### GAP-015: No TCP listener for UniBin v1.2 → RESOLVED (V5, 2026-03-25)

Added `serve --listen addr:port` and `serve --port N` CLI arguments. New
`serve_tcp()` and `handle_tcp_connection()` functions in listener.rs provide
TCP IPC alongside existing UDS. Compliant with UniBin v1.2 `--listen`
specification.
