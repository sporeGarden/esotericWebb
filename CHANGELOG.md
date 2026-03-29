# Changelog

All notable changes to Esoteric Webb are documented here.

## V6 — ludoSpring Decomposition: Self-Composed via Primal Composition (March 29, 2026)

### Architecture

- **Removed ludoSpring dependency entirely** — Webb no longer routes any calls
  through the `game` domain or ludoSpring. All composition is via direct primal
  calls through biomeOS semantic methods.
- **`science/` module created** — absorbed flow evaluation, engagement metrics,
  and DDA algorithms locally. Pure math, zero IPC. Patterns originated in
  ludoSpring but Webb owns the implementations.
- **AI method alignment** — fixed `ai.chat` → `ai.query`, `ai.summarize` →
  `ai.suggest`, added `ai.analyze`. Aligns with biomeOS capability registry
  and Squirrel's native methods.
- **NPC dialogue routes directly to Squirrel** — Webb formats NPC personality
  context and calls `ai.query` directly, no ludoSpring mediation.
- **Flow evaluation is local** — enrichment pipeline phase 3 now calls local
  `science::flow::flow_channel_metrics()` instead of IPC to ludoSpring.
- **Deleted `ipc/ludospring.rs`** — all `game.*` method constants, the
  `LudoSpringClient` struct, and the 12 ludoSpring JSON-RPC method wrappers
  removed. Types migrated: `FlowResult` → `science/flow.rs`,
  `EngagementResult` → `science/engagement.rs`, `DdaResult` → `science/dda.rs`,
  `DialogueResponse`/`VoiceNote` → `squirrel.rs`.
- **`domain::GAME` removed** from `primal_names.rs` and `DOMAIN_PRIMAL_MAP`.
  7 domains remain: ai, visualization, compute, storage, dag, lineage, provenance.
- **Deploy fragment updated** — `ludospring` removed from
  `deploy/esotericwebb.toml` optional dependencies.
- **3 new gaps filed** — GAP-021 (game science needs a primal), GAP-022
  (AI method alignment, resolved), GAP-016 superseded.
- 342 tests (all passing), all quality gates clean

## V5.1 — Audit Evolution + Use-Case Gap Pass (March 29, 2026)

### Use-Case Gaps (ecosystem review)

- **`niche.rs` self-knowledge module** — absorbed from ludoSpring V32 pattern;
  centralizes `NICHE_NAME`, `NICHE_DOMAIN`, `CAPABILITIES` array, `family_id()`,
  `socket_dirs()`, `resolve_server_socket()`, and `resolve_neural_api_socket()`;
  `listener.rs::socket_path()` now delegates to niche; 6 tests verify identity,
  capability namespacing, and socket resolution
- **Deploy fragment evolved** — `deploy/esotericwebb.toml` added nestgate,
  toadstool, songbird, beardog to optional dependencies (all bridge-ready domains)
- **5 new evolution gaps filed** (GAP-016 through GAP-020):
  - GAP-016: ludoSpring UDS-only transport blocks container composition (high)
  - GAP-017: biomeOS neural-api fails to start in benchScale (critical)
  - GAP-018: neuralAPI executors not exposed on JSON-RPC (high)
  - GAP-019: beardog crypto domain not wired into Webb bridge (medium, self-owned)
  - GAP-020: Deploy graph format divergence (low)
- **Handoffs updated** — use-case gap evidence filed to both local wateringHole
  and `ecoPrimals/infra/wateringHole/` with prioritized action items per team

### Audit evolution (code quality)

- **Zero `#[allow]` in production code** — all suppression attributes migrated to
  `#[expect(…, reason = "…")]` with mandatory justification; dead lints removed
  entirely rather than converted (e.g. `handle_tools_list` no longer triggers
  `too_many_lines`, `validation_experiments.rs` narrowed to `expect_used` only)
- **Smart module refactoring** — `content/mod.rs` (967 LOC) decomposed into
  `content/types.rs` (data model) + `content/mod.rs` (873, load/validate/scaffold);
  `ipc/bridge.rs` (943 LOC) decomposed into `ipc/bridge/mod.rs` (565, core +
  resilience + tests) + `ipc/bridge/domains.rs` (396, domain delegations); both
  well under 1000-line limit with growth headroom for new domains
- **Hardcoded port elimination** — experiment ports (`exp004`, `validation_experiments`)
  evolved from hardcoded values to dynamic OS-assigned ephemeral ports via
  `allocate_port()` helper (bind to `127.0.0.1:0`), preventing parallel test collisions
- **Tautological assertion fixes** — `exp005` autoplay termination check and `exp002`
  discovery registry check corrected from always-true to genuine validation logic
- **TCP E2E test suite** — 5 new TCP E2E tests (`health`, `identity`, `capabilities`,
  `multiple_requests`, `session_lifecycle`) + capability registry cross-validation test
  ensuring all methods in `capability_registry.toml` dispatch without "method not found"
- **Listener signature evolution** — `handle_tcp_connection` and `handle_connection`
  evolved to accept references (`&TcpStream`, `&UnixStream`) instead of owned values,
  eliminating `needless_pass_by_value` lint at the source rather than suppressing
- **Documentation alignment** — all root docs, specs, CHANGELOG, README,
  CONTRIBUTING aligned to current state
- 341 tests, all 5 quality gates clean

## V5 — Deep Debt Resolution + Ecosystem Evolution (March 25, 2026)

- **Coverage gate: 90.84% lines** — enforced via `cargo llvm-cov --fail-under-lines 90`;
  329 total tests (316 unit + 12 E2E + 1 validation), up from 166 in V4
- **Canonical `primal_names` module** — single source of truth for all primal
  slugs, domains, and display names; eliminates duplicate KNOWN_PRIMALS /
  PRIMAL_DOMAINS constants across discovery, bridge, and handlers
- **Semantic `IpcError` classification** — `IpcError` refactored to ecosystem
  pattern (primalSpring alignment): `ConnectionRefused`, `Timeout`,
  `MethodNotFound`, `ProtocolError`, `ApplicationError`, `PrimalNotFound`;
  helper methods `is_retriable()`, `is_recoverable()`, `is_method_not_found()`,
  `is_connection_error()`, and `classify_io_error()` for consistent circuit
  breaker and retry logic across all consumers
- **Transport negotiation** — `PrimalClient::connect_transport()` parses
  `unix:`, `tcp:`, implicit path, and implicit address formats per
  primalSpring transport priority pattern
- **Smart session refactor** — `session.rs` (1192 lines) decomposed into
  `session/mod.rs` (891), `session/types.rs` (data structures),
  `session/enrichment.rs` (primal composition pipeline); all under 1000-line
  limit while preserving logical cohesion
- **Logging modernization** — all `println!`/`eprintln!` replaced with
  `tracing::info!`/`tracing::warn!` for structured observability
- **UniBin v1.2 TCP listener** — `serve --listen addr:port` and `serve --port N`
  for TCP IPC alongside existing UDS; `serve_tcp` and `handle_tcp_connection`
  in `listener.rs`
- **`cmd_replay` evolution** — stub replaced with honest error + guidance
  pointing to `EVOLUTION_GAPS.md` for proper implementation
- **Content validation coverage** — 14 new tests for missing content_ref,
  missing NPC, trust reward warnings, ability effects, compound predicates,
  YAML load paths, worlds/rulesets loading
- **Launcher test suite** — 15 new tests covering topological sort edge cases,
  TOML round-trips, deploy graph diamond/cycle/missing-dep, TCP readiness,
  spawn error paths, struct defaults
- **Discovery test suite** — 8 new tests for metadata ingestion edge cases,
  probe_directory socket scanning, unknown domain fallback, TCP address
  preservation
- **Client test suite** — 7 new tests for capabilities fallback chain, health
  liveness edge cases, Transport debug formatting
- **Handler test expansion** — extensive new tests for session and narrative
  handlers with active GameSession state (act, history, narrate, graph)
- **TCP listener tests** — valid request, parse error, and empty line handling
  for both TCP and UDS connection handlers
- **Enrichment pipeline tests** — 9 new tests exercising the full 6-phase
  enrichment pipeline with standalone bridge
- All 5 quality gates clean: fmt, clippy (pedantic + nursery), test, doc, deny

## V4 — Wire Live Primal Composition (March 24, 2026)

- **Critical fix: session.start bridge preservation** — IPC `session.start` now preserves the
  PrimalBridge from the previous session, fixing a bug where all primal composition capabilities
  were dropped on session restart
- **Full bridge method coverage** — PrimalBridge now exposes all ecosystem capabilities:
  engagement(), npc_dialogue(), narrate_action(), voice_check(), game_push_scene(),
  game_begin_session(), game_complete_session(), dag_session_complete(), dag_query_vertices(),
  mint_certificate(), poll_input() — all with graceful degradation
- **AI narration wired into act()** — each action attempts narration via ludoSpring → Squirrel
  (game-science-enriched), with fallback to direct Squirrel, with mechanical text as final fallback
- **NPC dialogue composition** — talk actions call game.npc_dialogue via ludoSpring → Squirrel,
  returning dialogue text and voice interjections
- **Scene rendering wired into act()** — after each action, scene state is pushed to petalTongue
  via bridge.render_scene() for live UI rendering
- **Game science wired into act()** — flow evaluation (evaluate_flow) runs per-action when
  the game science primal is connected; results included in PrimalEnrichment
- **Provenance lifecycle complete** — dag_session_create on session start stores real session_id
  in WorldState; dag_session_complete fires when an ending is reached; all actions recorded with
  proper session_id
- **PrimalEnrichment type** — new serializable struct captures all primal composition results
  (ai_narration, npc_dialogue, voice_notes, flow_score, in_flow, scene_pushed) in NarrationContext
- **VoiceEnrichment type** — voice interjections from game science mapped to session-level type
- **LudoSpringClient complete** — typed client methods for npc_dialogue, narrate_action,
  voice_check, push_scene, begin_session, complete_session
- **New IPC constants** — METHOD_DAG_SESSION_COMPLETE, METHOD_DAG_QUERY_VERTICES, METHOD_CERT_MINT
- 166 tests passing, all 5 quality gates clean (fmt, clippy, test, doc, deny)

## V3 — Ecosystem Absorption (March 24, 2026)

- IPC handler split: monolithic server.rs (461 LOC) decomposed into handlers/{lifecycle,narrative,session,mcp}.rs (ludoSpring V30 pattern)
- MCP tools.list returns JSON Schema `input_schema` per tool; tools.call routes all 14 methods through shared handlers
- IPC client resilience: RetryPolicy (exponential backoff) + CircuitBreaker (Closed/Open/HalfOpen) for all PrimalBridge domain calls (neuralSpring pattern)
- sourDough compliance: added `identity.get`, `health.check`; fixed domain to "narrative" in registry
- Capability parity: aligned deploy fragment, capability registry, CONTEXT.md, and deploy graphs to identical surface
- Removed phantom `webb.content.validate` from deploy fragment
- Experiment harness evolution: `section()` headers, `finish_with_code() -> ExitCode`, `primal_or_skip()`, zero-test guard (wetSpring Validator pattern)
- 148 tests passing, all 5 quality gates clean (fmt, clippy, test, doc, deny)
- First wateringHole handoff: ESOTERICWEBB_V3_ECOSYSTEM_ABSORPTION_HANDOFF_MAR24_2026.md
- Environment-configurable resilience: ESOTERICWEBB_IPC_RETRY_*, ESOTERICWEBB_IPC_CB_*

## V2 — TCP Primal Composition + Team Scaffold (March 24, 2026)

- PrimalClient dual transport: TCP (platform-agnostic) + UDS via Transport enum
- PrimalEndpoint extended with tcp_addr; PrimalRegistry discovers from env vars and plasmidBin/ metadata
- PrimalBridge::discover() tries TCP first, falls back to UDS; transport field in status
- PrimalLauncher: binary discovery (6-pattern search), process spawn, TCP readiness polling
- Deploy graph support: topological wave ordering (Kahn's algorithm), graph-driven primal spawning
- `graphs/webb_provenance_trio.toml` deploy graph for the provenance trio
- `--launch` and `--graph` flags on `serve` subcommand for local primal spawning
- DAG domain methods wired: dag_session_create, dag_event_append, dag_frontier_get, dag_merkle_root
- PrimalBridge::inject() for launcher-spawned connections
- Experiment framework: shared validation harness (check_bool/check_skip/finish), JSON output mode
- 5 numbered experiments: narrative reachability, composition wiring, state emergence, provenance trio TCP, autoplay coverage
- validate_all meta-runner binary
- Integration test: exp008_rhizocrypt_live_round_trip (graceful skip when binary unavailable)
- deny.toml: supply chain audit with ecoBin C-dependency bans
- .gitignore, CONTRIBUTING.md, wateringHole/ docs structure
- config/primal_launch_profiles.toml for per-primal spawn configuration
- .cargo/config.toml coverage alias

## V1 — Bootstrap (March 23, 2026)

- Repository skeleton: Cargo workspace, Makefile, CI, triple license
- BYOB niche definition and deploy graph
- Core design specs: ESOTERIC_WEBB_DESIGN.md, BOUNDED_INFINITE_ARCHITECTURE.md
- IPC client modules: ludoSpring, Squirrel, petalTongue, provenance, discovery
- IPC server: health, narrative status, content listing, MCP tools
- NarrativeGraph engine: nodes, edges, state predicates, state effects, validator
- GameDirector runtime: input resolution, state evaluation, primal orchestration
- WorldState composite: knowledge, trust, inventory, conditions, arcs, plane, session
- Ability/spell system with emergent interaction evaluation
- Content authoring layer: YAML formats, loader, validator, CLI subcommand
- Content authoring spec with worked examples
- Case study: "The Weaver's Parlor" (8 rooms, 5 NPCs, 8 abilities, 4 endings)
- Validation experiments: composition wiring, reachability, emergence, NPC depth, provenance
- UniBin subcommands: serve, validate, preview, graph, replay, new-world
- EVOLUTION_GAPS.md: living gap tracker for cross-spring feedback
