# Changelog

All notable changes to Esoteric Webb are documented here.

## V13 — Wave 75: Session metrics, mesh push propagation, coverage expansion (Jun 3, 2026)

### Session metrics (V13 feature)

- **`session.metrics`** — new IPC method returning engagement analytics for game
  science / DDA: turns played, exploration ratio, backtrack count, NPC interactions,
  ability uses, actions-per-node pacing indicator.
- **`SessionMetrics` struct** — zero-cost on-demand computation from session history.
- **Capability count**: 24 → 25 (`session.metrics` added to stable tier).

### Mesh registration evolution (Songbird w75)

- **Stability tier metadata** included in `route.register` payload — router can
  prioritize propagation of stable methods vs evolving ones.
- **Push propagation signal** — `"propagation": "push"` declares awareness of
  w75 push model.
- **`gate_id()` function** in `niche.rs` — environment-overridable gate identity
  (default: `ironGate`), replaces hardcoded gate string.
- **`BIOMEOS_GATE_ID` env key** centralized in `env_keys.rs`.

### Test coverage expansion (+32 tests)

- **Director module**: 7 → 19 tests. New: `process_talk`, trust mechanics,
  `trust_demeanor` ranges, ability precondition gating, trust reward thresholds,
  invalid exit handling, scene change turn counting, missing content fallback.
- **Visualization module**: 7 → 20 tests. New: empty graph DOT/JSON, node shape
  assertions, JSON field structure, overlay status states, edge taken/gated flags,
  DOT overlay style assertions, history parsing edge cases.
- **Session module**: +6 metrics tests (initial state, navigation, backtrack,
  interactions, actions-per-node, serialization).
- **Niche module**: +1 test (`gate_id` default).
- **Total**: 378 → 410 tests (all passing, clippy clean, fmt clean).

### Metrics

| Metric | V12 | V13 |
|--------|-----|-----|
| Tests | 378 | 410 |
| Capabilities | 24 | 25 |
| Director tests | 7 | 19 |
| Visualization tests | 7 | 20 |
| Wave compliance | 73 | 75 |
| Mesh propagation | degraded | push-ready |

---

## V12 — Wave 72-74: Zero debt, typed constructors, mesh readiness (Jun 3, 2026)

### Method constant consolidation

- **All `METHOD_*` constants** centralized in `ipc/mod.rs` as single source of truth.
- **String literal dispatch eliminated** — `WEBB_METHODS` array replaced with
  `niche::CAPABILITIES`, MCP and client dispatch uses constants exclusively.
- **`METHOD_ROUTE_REGISTER`** added for cross-gate mesh registration (Wave 73).
- **`METHOD_SESSION_*` constants** moved from handlers to `ipc/mod.rs`.

### Typed error constructors

- **`JsonRpcError::application/invalid_params/method_not_found`** constructors
  eliminate verbose struct-literal boilerplate across all handlers.
- **`ERROR_APPLICATION`** constant added — all raw `-32000`/`-32602` literals
  replaced with named constants from `envelope.rs`.
- **`JsonRpcRequest::with_id`** constructor — IPC client no longer manually
  builds request envelopes.

### Idiomatic Rust evolution

- **DRY session helpers** — `sorted_knowledge()`, `sorted_flags()`,
  `narration_hints()` extracted from 3 duplicate sites.
- **Unnecessary String allocations eliminated** — `health_liveness()` fallback
  chain uses `&[&str]` instead of `[String; 4]`, `contains(&x.to_owned())`
  replaced with `iter().any(|n| n == x)`.

### Mesh registration (Wave 73)

- **`route.register`** wired into `announce_self()` — gracefully degrades
  when mesh router unavailable (single-gate mode continues).
- **Vocabulary fix**: `signal_tiers` → `composition_tiers` in announce payload.
- **GAP-034** (mesh route) and **GAP-035** (sporePrint pipeline) documented.

### Test coverage expansion

- **378 tests** (was 355): +9 constructor tests, +9 `WebbError` variant tests,
  +1 error constant verification, +6 session helper/snapshot tests.

### Metrics

| Metric | V11 | V12 |
|--------|-----|-----|
| Tests | 355 | 378 |
| Hardcoded error codes | 15 | 0 |
| Hardcoded method strings | 8 | 0 |
| Wave compliance | 67 | 74 |

## V11 — Wave 67 Polish: dead code removal, vocabulary alignment, safety (Jun 1, 2026)

### Dead code removal

- **`ipc/provenance.rs` deleted** — superseded legacy module with old `provenance.*`
  method constants and unused `ProvenanceClient`/`ProvenanceVertex` types. Production
  code uses signal-first `nest.store`/`nest.commit` architecture (V8) and modern
  `METHOD_DAG_*` constants from `ipc/mod.rs`. 2 dead tests removed (355 remain).

### Vocabulary alignment (Wave 67)

- Ecosystem vocabulary evolved `signal` → `composition` (wire names preserved as
  biomeOS contract). Updated doc comments, `capability_registry.toml` descriptions,
  and `EVOLUTION_GAPS.md` to use "composition" vocabulary while preserving JSON wire
  field `signal_tiers` (frozen contract).

### Safety escalation

- **`#![forbid(unsafe_code)]`** added to `lib.rs` (crate-level enforcement). Aligns
  with primalSpring Wave 66-67 ecosystem standard (`#![forbid(unsafe_code)]` on all
  88 crate roots). Webb already had zero unsafe — this makes it a compile-time
  guarantee.

### Ecosystem sync

- Registry methods updated: 458 → 490 (primalSpring v0.9.31, Wave 67).
- `EVOLUTION_GAPS.md` GAP-004 and GAP-024 updated to reflect current architecture
  (signal-first provenance, `s_nest_commit_live` scenario availability).
- README metrics updated to V11 posture.

## V10 — Wave 46 Absorption: env_keys, deploy graphs, announce hints (May 23, 2026)

### env_keys centralization (Wave 46 pattern)

- **`src/env_keys.rs` created** — single source of truth for all environment
  variable names (17 constants). Aligned with primalSpring `env_keys.rs`
  convention. All 20+ bare `std::env::var("...")` calls rewired to use
  constants. Zero bare env strings remain in production code. GAP-031 resolved.

### Deploy graph evolution (Wave 46 / Dark Forest Gate)

- **`secure_by_default = true`** added to all 8 deploy graphs per
  `DARK_FOREST_GLACIAL_GATE_STANDARD.md`.
- **`[graph.metadata]`** added with `owner`, `domain`, `wave` fields per
  `DOWNSTREAM_PATTERN_GUIDE.md` §4.
- All graph versions bumped to 0.1.1 / 1.1. GAP-032 resolved.

### primal.announce Wave 45 alignment

- **`cost_hints`** and **`latency_estimates`** added to `announce_self()`
  per Songbird/BearDog announce schema (Wave 45). Enables routing weight
  decisions by biomeOS. GAP-033 resolved.

### IPC error system validation

- Webb's `IpcError` already uses `#[derive(thiserror::Error)]` with semantic
  classification (`is_retriable()`, `is_recoverable()`, `classify_io_error()`).
  Aligned with primalSpring `PhasedIpcError` pattern. No further evolution
  needed — typed error system confirmed compliant.

### Metrics

| Metric | V9 | V10 |
|--------|----|----|
| Tests | 357 | 357 |
| Rust files | 43 | 44 (+env_keys.rs) |
| Bare env strings | 20+ | 0 |
| Deploy graphs with metadata | 0/8 | 8/8 |
| Announce schema | v1 (no hints) | v2 (cost_hints + latency_estimates) |
| Wave compliance | 20 | 46 |
| Resolved gaps | 30 | 33 |

## V9 — Wave 20-21 Canonical Schema Absorption + Degradation Contracts (May 17, 2026)

### Wave 20 canonical schema compliance

- **`capabilities.list` canonical envelope** — response now emits
  `{ capabilities, count, primal }` per the Wave 20 schema standard
  (`primalSpring/ecoPrimal/src/validation/scenarios/s_schema_standard.rs`).
  GAP-026 resolved.
- **Bridge envelope normalization** — `PrimalClient::capabilities()` now unwraps
  the Wave 20 envelope or wraps raw arrays from pre-Wave-20 primals into the
  canonical shape. Consumers always see `{ capabilities, count, primal }`.
  GAP-030 resolved.

### Stability tier awareness

- **`capability_registry.toml` annotated** — method groups now carry
  `stability = "stable" | "evolving"` per Wave 20 convention. sourDough,
  lifecycle, session, and domain methods are `stable`. MCP tools are `evolving`.
  GAP-027 resolved.

### Degradation behavior contracts

- **`docs/DEGRADATION_BEHAVIOR.md`** — formal per-domain degradation contracts
  covering all 9 consumed primal domains, signal dispatch fallbacks, trio
  partial completion states, and standalone/composition mode spectrum.
  GAP-028 resolved.

### Trio partial completion tracking

- **`primals_reached` in `WorldState`** — tracks which trio primals responded
  during provenance operations. Follows `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`
  partial completion rules. GAP-029 resolved.
- **`record_provenance_vertex()` populates tracking** — pushes `"dag"` to
  `primals_reached` on successful `nest.store`.

### Documentation

- **`whitePaper/baseCamp/` renamed** — V8 → V9 with 4 new evolution patterns:
  canonical schema consumption, stability tier awareness, degradation contracts,
  trio partial completion.
- **`EVOLUTION_GAPS.md`** — 6 new gaps (GAP-025 through GAP-030) all resolved V9.
  GAP-025 status corrected from conflicting state to absorbed.
- **Root `README.md`** — V9 metrics: Wave 20 schema compliance, degradation
  contracts, trio tracking.

### Metrics

| Metric | V8 | V9 |
|--------|----|----|
| Tests | 357 | 357 |
| Rust files | 43 | 43 |
| Capabilities exposed | 24 | 24 |
| Wave compliance | 17 | 20 |
| Stability tiers | none | annotated |
| Degradation docs | implicit | `docs/DEGRADATION_BEHAVIOR.md` |
| Trio tracking | none | `primals_reached` in `WorldState` |
| Resolved gaps | 25 | 30 |

## V8 — Wave 17 Signal Adoption + Deep Debt Resolution (May 16, 2026)

### Signal dispatch adoption (primalSpring Wave 17)

- **`nest.store` signal dispatch** — atomic provenance step that collapses
  NestGate.content.put → rhizoCrypt.dag.event.append → loamSpine.spine.seal →
  sweetGrass.braid.create into a single biomeOS-routed signal. Falls back
  to direct `dag.event.append` when biomeOS is unavailable.
- **`nest.commit` signal dispatch** — atomic session finalization (dehydrate →
  sign → store → seal). Falls back to `dag.session.complete`.
- **Signal constants declared** — `nest.store`, `nest.commit`, `meta.observe`,
  `meta.intent` ready for biomeOS orchestration collapse.
- **Enrichment pipeline rewired** — `record_provenance_vertex()` now calls
  `bridge.nest_store()` (signal-first with DAG fallback) instead of direct
  `dag.event.append`. `complete_provenance_if_ended()` calls `bridge.nest_commit()`
  instead of direct `dag.session.complete`.

### Lifecycle / Neural API alignment

- **`primal.announce` inbound handler** — accepts registration announcements
  from other ecosystem primals. Backward-compatible with `lifecycle.register`.
- **`primal.announce` outbound wired** — `cmd_serve` now calls
  `announce_to_biomeos()` at startup, broadcasting 24 capabilities via
  `primal.announce` before starting the IPC server (GAP-025 resolved).
- **`primal.info` handler** — returns niche metadata (version, capabilities,
  signal tiers, guidestone level).
- **`health.version` handler** — detailed version, build target, signal tier info.
- **`health.drain` handler** — acknowledges graceful shutdown intent.

### Smart refactoring (>800 LOC files)

- **`session/mod.rs`** 891→425 LOC — extracted 470-line test suite to
  `session/tests.rs` (idiomatic Rust companion file pattern).
- **`content/mod.rs`** 873→290 LOC — extracted 582-line test suite to
  `content/tests.rs`.
- All production files now under 800 LOC. Largest: `launcher.rs` at 764.

### Niche self-knowledge evolved

- **Capability list expanded** — added `health.version`, `health.drain`,
  `primal.announce`, `primal.info` to `niche::CAPABILITIES` (20→24).
- **Cross-validation test** — new test verifies `niche::CAPABILITIES`
  entries are all present in `capability_registry.toml`.

### Deep debt resolution

- **Clippy clean** — fixed `unnecessary_sort_by` (narrative/mod.rs) and
  `map().unwrap_or()` → `map_or()` (exp004, validation_experiments).
- **GAP reference fix** — `cmd_replay` cited wrong gap number (GAP-003 → GAP-004).
- **GAP-022 re-filed** — moved from "Open gaps" to "Absorbed gaps" (was already
  resolved in V6 but misplaced in document structure).
- **External dep audit** — `yaml_serde → libyaml-rs` confirmed pure Rust
  (no C FFI, no `-sys` crate). ecoBin compliant.
- **Hardcoded path audit** — production code uses env-var-driven, XDG-compliant
  discovery with `plasmidBin/` search-order fallback. Already capability-based.
- **Production stub audit** — all degradation patterns are intentional 4-pattern
  graceful degradation per ecosystem standard. Zero mocks outside `#[cfg(test)]`.

### Metrics

- 338 unit tests + 18 E2E + 1 integration (357 total, up from 342)
- Zero clippy warnings (pedantic + nursery)
- Zero unsafe, zero `#[allow()]`, zero `unwrap()`/`expect()` in production
- Zero `TODO`/`FIXME` in production code
- All production files under 800 LOC
- `cargo deny check` PASS
- `cargo doc` PASS (zero warnings)

### Evolution gaps

- **GAP-024** (filed) — signal dispatch not exercised E2E against live biomeOS
- **GAP-025** (resolved) — `primal.announce` outbound now wired into serve startup

---

## V7 — Deploy Artifact Alignment + Composition Handoff (April 17, 2026)

### Deploy graph and niche cleanup

- **`graphs/esotericwebb_full.toml`** — removed stale `germinate_ludospring`
  node (Phase 2); renumbered phases; updated esotericwebb `depends_on` to
  drop ludospring; aligned Squirrel AI methods to V6 (`ai.query`, `ai.suggest`,
  `ai.analyze`); fixed validation targets
- **`graphs/webb_full.toml`** — corrected esotericwebb `by_capability` from
  `"game"` to `"narrative"`; updated Squirrel capability list
- **`graphs/webb_ai_viz.toml`** — aligned Squirrel capabilities to V6 methods
- **`niches/esoteric-webb.yaml`** — removed ludospring organism and
  interactions; aligned Squirrel capabilities; updated features description
- **README.md** — primal domain table updated to V6 (7 active domains,
  game row struck through with GAP-021 pointer)

### Stale code cleanup

- **`config/primal_launch_profiles.toml`** — removed `[profiles.ludospring]`
  (game domain, port 9420); noted GAP-021 for future game-science primal
- **`session/types.rs`** — updated doc comments: ludoSpring→Squirrel references
  corrected to direct Squirrel calls and local `science/` module
- **`session/mod.rs`** — enrichment pipeline docs updated (no ludoSpring
  mediation)
- **`session/enrichment.rs`** — doc comment updated
- **`discovery.rs` tests** — replaced stale `game.evaluate_flow` / `ludospring`
  test fixtures with current `dag.session.create` / `rhizocrypt` examples

### Documentation and handoff

- **`EVOLUTION_GAPS.md`** — filed GAP-023 (stale deploy artifacts, resolved)
- **`whitePaper/baseCamp/`** — evolution document expanded with
  python→rust→primal composition validation story
- **`wateringHole/handoffs/`** — V7 handoff: composition patterns, per-primal
  learnings, NUCLEUS deployment patterns for primal and spring team absorption
- **`downstream_manifest.toml`** — esotericwebb entry updated: added
  `node_atomic` to fragments; expanded `depends_on` and
  `validation_capabilities` to reflect full V6 bridge surface
- 342 tests, all quality gates clean

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
