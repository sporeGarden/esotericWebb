# Changelog

All notable changes to Esoteric Webb are documented here.

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
