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
- **Evidence**: petalTongue v1.6.6 exposes `visualization.render.scene` but
  requires a full `SceneGraph` wire format with `nodes` (map), `edges`,
  `transform` (position/rotation/scale), and other 3D-graph fields. Webb's
  CRPG scene model (node + description + NPCs + turn) does not match this
  schema. The `ui.render` method accepts simpler payloads and works today
  (confirmed: `{"type":"text","content":"..."}` returns `rendered: true`).
- **Expected**: Either (a) petalTongue defines a `SceneGraph` subtype for
  text/narrative scenes (dialogue trees, choices, voice notes), or (b) Webb
  continues using `ui.render` for text content and `visualization.render.scene`
  only when a full scene graph is available.
- **Workaround**: Webb now uses `ui.render` for scene pushes (V21). Scene push
  confirmed working with live petalTongue composition.
- **Handoff**: petalTongue team — define CRPG scene type for
  `visualization.render.scene`, or document `ui.render` as the intended path
  for text-based narrative composition.
- **Status**: partial (workaround shipped, upstream schema TBD)

### GAP-003: AI primal NPC dialogue constraint enforcement

- **Primal**: AI (`ai.query`, `ai.analyze`)
- **Spring (producer)**: Squirrel
- **Severity**: medium
- **Evidence**: Webb's NPC personality certs define knowledge bounds, trust
  gates, lies with detection DCs, and voice constraints. When Webb calls
  `ai.query` with NPC context (direct Squirrel composition, V6), the AI
  primal needs to respect these constraints mechanically — not just as
  system prompt guidance.
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
- **Severity**: low (structure ready; signal-first architecture wired)
- **Evidence**: Webb uses signal-first composition dispatch (`nest.store` /
  `nest.commit`) with automatic fallback to direct `dag.*` domain calls.
  Webb has BFS depth layers (`NarrativeGraph::bfs_depths()`) and edge
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
  Session provenance flows through `nest.store` (composition dispatch)
  with `dag.event.append` fallback when biomeOS is unavailable.
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

- **Primal**: discovery (`discovery.query`, `capability.resolve`, `ipc.resolve`)
- **Spring (producer)**: Songbird
- **Severity**: medium (closer — TransportEndpoint type landed V14)
- **Evidence**: Webb's `PrimalRegistry::discover()` probes filesystem socket
  directories but does not yet call songBird's `ipc.resolve` for tier-5
  topology-aware resolution. In the 4-gate mesh collective (Wave 107),
  songBird returns structured `TransportEndpoint` responses from
  `capability.resolve` and `ipc.resolve` (MeshRelay endpoints for cross-gate).
- **Expected**: After filesystem probe, Webb queries songBird `ipc.resolve`
  for primals not found locally, receiving `TransportEndpoint` (UDS/TCP/MeshRelay)
  responses. The `TransportEndpoint` type (V14) already matches this wire format.
- **Workaround**: Filesystem probe covers tiers 1-4. Tier-5 is logged as
  degraded but functional. `TransportEndpoint` type is ready for consumption.
- **Handoff**: Next step: wire `ipc.resolve` call into discovery after local
  probe fails, deserialize response as `TransportEndpoint`.
- **Progress (V16)**: songBird discovered via `SONGBIRD_JSONRPC_PORT=7780`
  env var on flockGate. However songBird TCP 7780 speaks HTTP, not raw
  NDJSON JSON-RPC — Webb's `PrimalClient` sends NDJSON and gets HTTP 400.
  Either songBird needs a raw JSON-RPC endpoint or Webb needs an HTTP
  transport adapter. Filed in V16 AAR handoff.
- **Status**: type-ready (V14), env-var discovery works (V16), live health blocked by transport mismatch

### GAP-007: Voice interjection preview without live AI primal

- **Primal**: AI (`ai.query`, `ai.analyze`)
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
- **Progress (V11)**: Implemented `science/voice.rs` with offline voice
  interjection engine. Built-in voice profiles (logic, empathy, perception)
  fire based on game state predicates (knowledge, flags, trust, inventory,
  plane). Wired into the enrichment pipeline — voice interjections fire on
  every action, even without the AI primal. Custom voice profiles supported.
- **Next**: YAML-authored voice profiles in content data, preview CLI command.
- **Status**: partial (offline engine complete, content authoring pending)

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

- **Primal**: game science (future `science.ruleset_validate`, GAP-021)
- **Spring (producer)**: esotericWebb (self) + future game-science primal
- **Severity**: medium
- **Evidence**: Cliche Studio's creative DNA (transparent dice, multi-plane
  play) requires RulesetCert definitions per plane (Investigation, Dialogue,
  Tactical, Crafting). The CONTENT_AUTHORING_SPEC defines a rulesets/
  directory but the content loader does not yet parse or validate RulesetCert
  YAML against any schema. V6 absorbed flow/engagement/DDA locally but
  RulesetCert validation remains unimplemented.
- **Expected**: YAML rulesets/ loaded, validated against a schema. `esotericwebb
  validate` reports ruleset errors. When a game-science primal emerges
  (GAP-021), `science.ruleset_validate` confirms compatibility at composition
  time.
- **Progress (V11)**: `ContentBundle::validate_rulesets()` now validates
  structural correctness of loaded rulesets: required `plane` field, required
  `rules` array, per-rule `id` field. Four new tests cover missing fields,
  invalid structure, and valid rulesets. `esotericwebb validate` reports
  ruleset issues alongside other content diagnostics.
- **Next**: Typed `RulesetCert` struct (replace opaque `serde_json::Value`),
  per-rule effect/condition validation, game-science primal integration.
- **Handoff**: Self-owned for loader; future game-science primal for
  validation endpoint (GAP-021).
- **Status**: partial (structural validation complete, typed model pending)

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
- **Status**: **resolved** (Wave 150a — depot operational, 59+ binaries, 4 arch, manifest.toml tracks state)

### GAP-016: ludoSpring UDS-only transport blocks container composition → SUPERSEDED (V6)

- **Status**: superseded — Webb no longer depends on ludoSpring (V6 decomposition).
  Game science (flow, engagement, DDA) absorbed locally. AI delegation routes
  directly to Squirrel via biomeOS semantic methods. This gap is no longer
  relevant to Webb; it may still apply to other ludoSpring consumers.

### GAP-017: biomeOS neural-api fails to start in benchScale

- **Primal**: neural-api (biomeOS orchestration layer)
- **Spring (producer)**: biomeOS
- **Severity**: critical
- **Evidence**: In a benchScale `tower-2node` live run, beardog and songbird
  come up `LIVE`, but biomeOS `neural-api` is `ZOMBIE` (fails health check
  after startup). This blocks the "biomeOS-orchestrated composition" use case
  where graphs are submitted to neural-api and routed to primals. Webb cannot
  test graph-based orchestration until neural-api is healthy.
- **Expected**: biomeOS neural-api starts healthy in benchScale topologies
  and responds to `health.liveness` within the configured timeout.
- **Workaround**: Webb composes directly to primals via PrimalBridge,
  bypassing biomeOS orchestration entirely. All capability routing is
  done by Webb's own discovery + bridge.
- **Handoff**: `ESOTERICWEBB_V51_AUDIT_EVOLUTION_HANDOFF_MAR29_2026.md`
- **Status**: open

### GAP-018: neuralAPI executors not exposed on JSON-RPC

- **Primal**: neural-api (`ConditionalDag`, `Pipeline`, `ContinuousExecutor`)
- **Spring (producer)**: biomeOS
- **Severity**: high
- **Evidence**: Webb's storytelling loop is naturally a continuous execution
  graph: player input → narrate → evaluate flow → push scene → wait for next
  input → repeat. biomeOS has `ConditionalDag`, `Pipeline`, and
  `ContinuousExecutor` in the codebase but they are not exposed as JSON-RPC
  methods. Webb cannot submit a storytelling graph for orchestrated execution.
  The `PathwayLearner` (learns from execution traces to optimize routing) is
  also internal-only. Without these, "E2E neuralAPI workflow" means only basic
  `graph.execute` → `graph.status` → `graph.result` for simple DAGs.
- **Expected**: `ConditionalDag` execution, `Pipeline` chaining, and
  `ContinuousExecutor` sessions available via JSON-RPC methods. PathwayLearner
  exposes `pathway.learn` and `pathway.suggest` for adaptive optimization.
- **Workaround**: Webb drives its own composition loop via PrimalBridge
  sequential calls. No graph-based orchestration.
- **Handoff**: `ESOTERICWEBB_V51_AUDIT_EVOLUTION_HANDOFF_MAR29_2026.md`
- **Status**: open

### GAP-019: beardog crypto domain not wired into Webb bridge → RESOLVED (V11)

- **Status**: resolved — `crypto` domain added to `DOMAIN_PRIMAL_MAP` with
  bearDog as the default primal. Three bridge methods wired: `crypto_sign`,
  `crypto_verify`, `crypto_hash`. Method constants: `crypto.sign`,
  `crypto.verify`, `crypto.hash`. All degrade gracefully when bearDog is
  unavailable (unsigned provenance, trust-on-first-use content). Three
  standalone degradation tests added.

### GAP-020: Deploy graph format divergence (TOML fragments vs biomeOS JSON)

- **Primal**: deployment infrastructure
- **Spring (producer)**: primalSpring / biomeOS
- **Severity**: low
- **Evidence**: Webb ships `deploy/esotericwebb.toml` and `graphs/*.toml`
  composition fragments. biomeOS uses JSON graph definitions internally.
  primalSpring reads TOML fragments. Two conventions exist side by side
  with no formal schema or cross-validation. When biomeOS ingests a
  composition graph, the format translation is opaque.
- **Expected**: Ecosystem-wide deploy fragment schema (TOML canonical,
  JSON derived) with validation tooling. `primalSpring validate-graph`
  checks a composition before deployment.
- **Workaround**: Webb maintains TOML fragments per wateringHole convention.
  Manual verification against primalSpring expectations.
- **Handoff**: primalSpring / wateringHole for schema standardization.
- **Status**: open

### GAP-021: Game science has no standalone primal

- **Primal**: game science (flow, engagement, DDA, WFC, noise, Fitts, accessibility)
- **Spring (producer)**: N/A — no primal offers these capabilities yet
- **Severity**: medium (Webb works with local science; primal would enable ecosystem reuse)
- **Evidence**: ludoSpring bundles 8 pure-science algorithms (flow evaluation,
  Fitts' law, engagement metrics, DDA, WFC, noise generation, UI analysis,
  accessibility scoring) that are deterministic math with zero primal IPC.
  These algorithms are useful to any game or interactive system, not just
  ludoSpring. Webb V6 absorbed flow, engagement, and DDA locally to remove
  the ludoSpring dependency, but the remaining 5 algorithms and the absorbed
  3 are pure math that would benefit from being a reusable primal capability.
- **Expected**: A dedicated game-science primal (or barraCuda extension) that
  exposes `science.evaluate_flow`, `science.engagement`, `science.dda`,
  `science.wfc_step`, `science.generate_noise`, `science.fitts_cost`,
  `science.analyze_ui`, `science.accessibility` via JSON-RPC. This allows
  any consumer (Webb, other gardens, springs) to compose game science without
  absorbing the algorithms locally or depending on ludoSpring.
- **Workaround**: Webb implements flow, engagement, and DDA locally in
  `science/` module (absorbed from ludoSpring patterns). Other science
  (WFC, noise, Fitts, UI analysis, accessibility) deferred until primal
  evolution delivers them.
- **Handoff**: primalSpring / wateringHole for game-science primal design.
- **Status**: open

### GAP-034: Mesh route registration awaiting router availability

- **Primal**: mesh routing (`route.register`)
- **Spring (producer)**: biomeOS / primalSpring
- **Severity**: low (push-model ready; awaiting live mesh router)
- **Evidence**: Wave 73 required cross-gate capability visibility via
  `route.register`. Webb V13 sends the full registration payload including
  stability tiers (stable/evolving method classification) and `"propagation":
  "push"` signal for Songbird w75 compatibility. Gate identity is
  environment-overridable via `BIOMEOS_GATE_ID`. The call gracefully degrades
  to single-gate mode when the mesh router is unavailable.
- **Expected**: Mesh router accepts `route.register` and push-propagates
  Webb's 25 capabilities (23 stable + 2 evolving) to all connected gates.
- **Workaround**: Single-gate operation via `primal.announce` to local
  biomeOS. Cross-gate access requires manual socket configuration.
- **Handoff**: biomeOS / primalSpring for mesh router deployment.
- **Status**: push-model ready (V13), live validation pending

### GAP-035: Content pipeline integration with sporePrint (post-S3)

- **Primal**: sporePrint (sovereign content delivery)
- **Spring (producer)**: sporePrint / primalSpring
- **Severity**: low (current GitHub Pages workflow functional)
- **Evidence**: Wave 73 audit identifies that post-S3 cutover, esotericWebb
  content should flow through the sporePrint sovereign pipeline instead of
  GitHub Pages. Webb already has `.github/workflows/notify-sporeprint.yml`
  as a bridge mechanism.
- **Expected**: Content bundles (YAML worlds, scenes, NPCs, abilities)
  published via sporePrint composition deploy instead of GitHub Pages.
  `sporeprint_composition.toml` routes content artifacts through sovereign
  infrastructure.
- **Workaround**: GitHub Actions notification workflow triggers sporePrint.
  Content served via GitHub Pages until S3 cutover completes.
- **Handoff**: sporePrint / primalSpring for pipeline readiness.
- **Status**: open (blocked on S3 cutover)

### GAP-036: Ecosystem socket naming convention divergence

- **Primal**: all primals registering UDS sockets
- **Spring (producer)**: ecosystem-wide (each spring registers its own socket)
- **Severity**: low (V16 workaround in place)
- **Evidence**: On flockGate (Wave 147c), some primals register domain-named
  sockets (`visualization.sock`, `ai.sock`) while others register primal-named
  sockets (`rhizocrypt.sock`, `loamspine.sock`, `toadstool.sock`). Webb's
  original `probe_directory()` only matched domain names, causing 3 primals
  to be invisible despite running. V16 added reverse-mapping as a workaround.
- **Expected**: Ecosystem convention converges on one naming scheme (domain
  or primal slug) or primals register both. `biomeOS/primalSpring` should
  document the convention and enforce it in `gate.enroll`.
- **Workaround**: Webb V16 `probe_directory()` does two-pass lookup (domain
  first, primal slug reverse-map second). Works for all known primals.
- **Handoff**: V16 AAR handoff for upstream primal teams.
- **Status**: **resolved** (Wave 150a — closed ecosystem-wide)

### GAP-037: songBird uses HTTP transport, not raw JSON-RPC

- **Primal**: mesh (`discovery.topology`, `discovery.health`)
- **Spring (producer)**: songBird
- **Severity**: medium
- **Evidence**: songBird on flockGate listens on TCP 7780 but speaks HTTP
  (returns `HTTP/1.1 400 Bad Request` to NDJSON payloads). Webb's
  `PrimalClient` sends newline-delimited JSON over raw TCP, which is the
  ecosystem standard for all other primals (squirrel, petaltongue, nestgate,
  sweetgrass, loamspine, beardog).
- **Expected**: Either songBird exposes a raw JSON-RPC endpoint (NDJSON
  over TCP, per sourDough convention) or Webb adds an HTTP POST transport
  adapter.
- **Workaround**: songBird mesh domain marked as discovered but unhealthy.
  Mesh bridge methods degrade gracefully.
- **Handoff**: V16 AAR handoff for songBird team.
- **Status**: **resolved** (Wave 148a — songBird shipped `/jsonrpc` endpoint; Webb needs HTTP transport adapter to consume it)

### GAP-038: Stale UDS sockets from crashed primals

- **Primal**: rhizoCrypt, toadStool (and potentially others)
- **Spring (producer)**: ecosystem-wide
- **Severity**: low
- **Evidence**: On flockGate, `rhizocrypt.sock` and `toadstool.sock`
  exist on disk but `connect()` returns ECONNREFUSED. The primal processes
  are not running but their socket files were not cleaned up on exit.
  This causes Webb to discover them as "found" but fail health check.
- **Expected**: Primals clean up their UDS sockets on shutdown (trap
  SIGTERM/SIGINT). Alternatively, `gate.enroll` or biomeOS could gc stale
  sockets.
- **Workaround**: Webb's bridge correctly classifies these as unhealthy
  (discovered but not connected). No false positives in composition.
- **Handoff**: V16 AAR handoff for upstream primal teams.
- **Status**: **resolved** (Wave 150a — closed ecosystem-wide)

### GAP-024: Composition dispatch not yet exercised E2E against live biomeOS

- **Primal**: biomeOS (composition orchestration layer)
- **Spring (producer)**: biomeOS
- **Severity**: low
- **Evidence**: Webb V8 declares `nest.store` and `nest.commit` composition
  dispatch methods with automatic fallback to domain calls. However, biomeOS
  `neural-api` has not been validated as routing these compositions on ironGate.
  The fallback to `dag.event.append` / `dag.session.complete` works, but
  the orchestration collapse (content.put + dag.append + spine.seal + braid.create
  in a single composition) has not been exercised live.
  primalSpring `s_nest_commit_live` scenario validates the wire contract (Wave 67).
- **Expected**: `nest.store` dispatched via biomeOS decomposes into the full
  provenance pipeline. `nest.commit` decomposes into session finalization.
- **Workaround**: Fallback to direct domain calls (functional, just not collapsed).
- **Handoff**: Validate on ironGate once biomeOS neural-api is healthy (GAP-017).
- **Status**: open

---

## Absorbed gaps

### GAP-025: `primal.announce` outbound not wired into serve startup → RESOLVED (V8, 2026-05-16)

`cmd_serve` now calls `announce_to_biomeos()` before starting the IPC
server, broadcasting all 24 capabilities via `primal.announce`. Filesystem
socket probe still works as fallback.

### GAP-026: `capabilities.list` response missing canonical Wave 20 envelope → RESOLVED (V9, 2026-05-17)

Webb's `capabilities.list` handler returned `{ capabilities, primal, version }`
but lacked the `count` field required by the Wave 20 canonical schema
(`primalSpring/ecoPrimal/src/validation/scenarios/s_schema_standard.rs`).
V9 adds `count` and drops `version` from the envelope to match the canonical
shape: `{ capabilities, count, primal }`.

### GAP-027: No stability tier annotations in capability_registry.toml → RESOLVED (V9, 2026-05-17)

All 8 springs annotate method groups with `stability = "stable" | "evolving" |
"internal"` per Wave 20. Webb V9 adds group-level stability annotations to
`capability_registry.toml`. All sourDough, lifecycle, session, and domain
methods are `stable`. MCP tools are `evolving`.

### GAP-028: No formal degradation behavior documentation → RESOLVED (V9, 2026-05-17)

Springs document per-primal degradation in `docs/DEGRADATION_BEHAVIOR.md`.
Webb V9 adds `docs/DEGRADATION_BEHAVIOR.md` documenting per-domain unreachable
behavior, signal dispatch fallbacks, trio partial completion states, and the
standalone/composition mode spectrum.

### GAP-029: Trio partial completion not tracked in session state → RESOLVED (V9, 2026-05-17)

Per `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`, consumers must report which trio
primals were reached. Webb V9 adds `primals_reached: Vec<String>` to
`WorldState` and populates it during provenance operations. Consumers can
inspect partial provenance state.

### GAP-030: Bridge `capabilities.list` parsing assumes raw arrays → RESOLVED (V9, 2026-05-17)

Webb's bridge introspection code parsed `capability.list` responses as flat
arrays. Wave 20 guarantees the canonical envelope `{ capabilities, count, primal }`.
V9 evolves the bridge to unwrap the canonical envelope, with fallback to raw
array parsing for pre-Wave-20 primals.

### GAP-031: Scattered bare env var strings → RESOLVED (V10, 2026-05-23)

20+ `std::env::var("...")` calls scattered across 9 files with string literals.
Wave 46 mandates centralized `env_keys.rs` (per primalSpring convention). V10
creates `src/env_keys.rs` with all env constants and rewires every call site.
Zero bare env strings remain in production code.

### GAP-032: Deploy graphs missing Wave 46 metadata → RESOLVED (V10, 2026-05-23)

All 8 deploy graphs lacked `secure_by_default = true` and `[graph.metadata]`
(owner, domain, wave) required by the Dark Forest Gate standard and
`DOWNSTREAM_PATTERN_GUIDE.md` §4. V10 adds these to all graphs.

### GAP-033: `primal.announce` missing Wave 45 hints → RESOLVED (V10, 2026-05-23)

`announce_self()` sent capabilities/methods/composition_tiers but no `cost_hints`
or `latency_estimates`. Wave 45 (Songbird/BearDog announce schema) requires
these for routing weight decisions. V10 adds per-method cost and latency hints.

### GAP-022: Webb AI method alignment with biomeOS capability registry → RESOLVED (V6, 2026-03-29)

Webb V5 called `ai.chat`, `ai.summarize`, `ai.inference` — none of which
exist in biomeOS's capability registry or Squirrel's native methods. V6
aligned all AI methods: `ai.chat` → `ai.query`, `ai.summarize` →
`ai.suggest`, added `ai.analyze`. NPC dialogue and narration now route
directly to Squirrel via biomeOS semantic methods instead of through
ludoSpring delegation.

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

### GAP-023: Stale deploy graphs and niche YAML still reference ludoSpring → RESOLVED (V6+, 2026-04-17)

`graphs/esotericwebb_full.toml` still contained `germinate_ludospring` as a
required Phase 2 node; `niches/esoteric-webb.yaml` listed ludoSpring as a
required organism with 12 `game.*` capabilities and pre-V6 AI method names
(`ai.chat`, `ai.summarize`); `graphs/webb_full.toml` and `webb_ai_viz.toml`
used stale Squirrel capabilities; `esotericwebb` node's `by_capability` was
`"game"` instead of `"narrative"`. All cleaned: ludoSpring removed from graphs
and niche definitions, AI methods aligned to V6 (`ai.query`, `ai.suggest`,
`ai.analyze`), domain corrected. README primal table updated to V6 state.
