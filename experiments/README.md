# Experiments

Numbered validation experiments for Esoteric Webb. Each is a standalone
binary crate that exercises a specific capability or composition pattern.
Experiments use a **shared validation harness** (`check_bool` / `check_skip`)
and exit with a summary of pass/fail/skip counts.

**Status**: 5 experiments, all passing (V13, Jun 3, 2026)

## Running

```bash
# All experiments (meta-validator)
cargo run --release --bin validate_all

# Single experiment
cargo run --release -p esotericwebb-exp001

# JSON output (for CI)
ESOTERICWEBB_JSON=1 cargo run --release --bin validate_all
```

## Tracks

| Track | Range | Focus |
|-------|-------|-------|
| Narrative | 001–009 | Graph reachability, validator, BFS depth, edge classification |
| Composition | 010–019 | Primal wiring, bridge degradation, TCP/UDS transport |
| State | 020–029 | Emergence, predicate evaluation, combinatorial state space |
| Provenance | 030–039 | Trio TCP round-trips, DAG session lifecycle, Merkle integrity |
| Gameplay | 040–049 | Autoplay coverage, director outcomes, heuristic exploration |
| Content | 050–059 | YAML loading, validation, scaffold, cross-ref integrity |
| DDA/Flow | 060–069 | Game science degradation, flow evaluation, difficulty curves |
| Integration | 070–079 | Full stack composition, launcher, deploy graph ordering |

## Status key

- **local**: validated with Webb's local engines only
- **tcp-wired**: validated with live primal over TCP
- **skip**: primal not available, honestly skipped

## Experiments

| # | Name | Track | Modules exercised | Status |
|---|------|-------|-------------------|--------|
| 001 | `narrative_reachability` | Narrative | `narrative`, `content`, BFS depth engine | local |
| 002 | `composition_wiring` | Composition | `ipc::bridge`, `ipc::discovery`, degradation paths, resilience (retry + circuit breaker) | local |
| 003 | `state_emergence` | State | `state`, `narrative::predicate`, `narrative::effect`, combinatorial space | local |
| 004 | `provenance_trio_tcp` | Provenance | `ipc::client` (TCP), `ipc::launcher`, rhizoCrypt DAG lifecycle | skip (requires plasmidBin trio binaries) |
| 005 | `autoplay_coverage` | Gameplay | `session`, `director`, `autoplay` heuristic engine, primal enrichment | local |

## Next experiments (planned)

| # | Name | Track | Modules | Blocked on |
|---|------|-------|---------|------------|
| 006 | `content_roundtrip` | Content | YAML load/save/validate, scaffold, cross-ref | — |
| 007 | `flow_degradation` | DDA/Flow | Local `science/flow` + `science/dda`, degradation path | — (local science, no primal needed) |
| 008 | `deploy_graph_ordering` | Integration | `PrimalLauncher`, topological waves, readiness poll | plasmidBin binaries |
| 009 | `ai_narration_pipeline` | Composition | `enrich_action()`, Squirrel fallback, voice notes | AI primal |

## Honest scaffolding

Experiments must **never fake a pass**. If a primal is unavailable, use
`check_skip("reason")` — not `check_bool("...", true)`. The `validate_all`
meta-runner treats skips as informational, not failures.

## Adding an experiment

1. Create `experiments/expNNN_descriptive_name/` with `Cargo.toml` + `src/main.rs`
2. Add to `[workspace] members` in root `Cargo.toml`
3. Add to the `EXPERIMENTS` list in `webb/src/bin/validate_all/main.rs`
4. Update the table above
