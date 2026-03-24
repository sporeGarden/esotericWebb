# Changelog

All notable changes to Esoteric Webb are documented here.

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
