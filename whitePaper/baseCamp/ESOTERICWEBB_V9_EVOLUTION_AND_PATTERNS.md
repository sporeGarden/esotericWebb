<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Esoteric Webb V9 — Evolution Patterns and Ecosystem Learnings

**Date:** May 17, 2026 (updated from May 16, 2026)
**Author:** ecoPrimals / sporeGarden
**Foundation:** V1–V4 bootstrap; V5 deep debt; V5.1 audit; V6 ludoSpring decomposition; V7 deploy alignment; V8 Wave 17 signal adoption + deep debt + smart refactoring; V9 Wave 20-21 canonical schema absorption + stability tiers + degradation contracts + trio tracking
**Coverage:** ~91% lines (357 tests)

---

## What This Is

This document captures patterns, discoveries, and architectural learnings from
Esoteric Webb's evolution from V1 (bootstrap) through V5.1 (audit evolution).
Webb is the first gen4 consumer in the ecoPrimals ecosystem — a CRPG substrate
that composes primals via JSON-RPC IPC without importing any spring or primal
Rust crate.

These learnings feed back to the ecosystem via wateringHole handoffs and are
relevant to any team building primal consumers or evolving primal producers.

---

## Pattern 1: Semantic Error Classification Across IPC Boundaries

**Problem.** Flat error enums (`Io(String)`, `Parse(String)`) lose semantic
meaning at IPC boundaries. Circuit breakers and retry policies cannot
distinguish "server temporarily down" from "method does not exist."

**Solution.** Classify errors by operational meaning:

| Variant | Semantic | Retriable | Recoverable |
|---------|----------|-----------|-------------|
| `ConnectionRefused` | Transport layer, server down | Yes | Yes |
| `Timeout` | Server slow or overloaded | Yes | Yes |
| `MethodNotFound` | Capability gap | No | Yes |
| `ProtocolError` | Wire format mismatch | No | No |
| `ApplicationError` | Server-side logic error | No | Yes |
| `PrimalNotFound` | Discovery failure | Yes | Yes |
| `Serialization` | Local serialization bug | No | No |

**Key insight.** `classify_io_error()` normalizes OS-level `io::Error`
variants (ConnectionRefused, TimedOut, BrokenPipe, etc.) to semantic types
*before* they propagate. This lets every consumer (bridge, resilience, retry
policy) make decisions without pattern-matching on OS error codes.

**Ecosystem alignment.** Mirrors primalSpring's `IpcError` semantic
classification. Any primal consumer should adopt this pattern.

---

## Pattern 2: Single Source of Truth for Primal Names

**Problem.** Multiple modules maintained independent lists of primal names,
domains, and slugs (`KNOWN_PRIMALS`, `PRIMAL_DOMAINS`, `DOMAIN_AI`, etc.).
Changes required coordinated updates across discovery, bridge, and handlers.

**Solution.** A single canonical module (`ipc/primal_names.rs`) that defines:
- All primal slugs as constants
- All domain constants
- A `DOMAIN_PRIMAL_MAP` associating domains to primals
- Utility functions: `display_name()`, `discovery_slug()`, `primal_for_domain()`

All consumers import from this one module. Adding a new primal is a single-line
change.

**Ecosystem relevance.** This pattern should be adopted by any system that
maintains a registry of known primals. primalSpring, biomeOS, Songbird — all
benefit from canonical name modules rather than scattered constants.

---

## Pattern 3: Smart Module Refactoring (Not Just Splitting)

**Problem.** Modules approaching 1000-line quality gates need restructuring
without sacrificing cohesion.

**Solution.** Identify *semantic boundaries* within the module:

**V5 — session.rs (1192 → 3 modules):**
1. **Data structures** → `session/types.rs`
2. **Primal composition pipeline** → `session/enrichment.rs`
3. **Core session logic** → `session/mod.rs`

**V5.1 — content/mod.rs (967 → 2 modules):**
1. **Data model** → `content/types.rs` (109 LOC — pure serde structs)
2. **Load/validate/scaffold** → `content/mod.rs` (873 LOC)

**V5.1 — bridge.rs (943 → directory module):**
1. **Core struct + resilience + call helpers** → `bridge/mod.rs` (565 LOC)
2. **Domain delegations** → `bridge/domains.rs` (396 LOC)

**Key insight.** `bridge/domains.rs` is exclusively 1-3 line delegation methods
that map to generic call helpers. This boundary is *functional*: new primal
domains add lines only to `domains.rs`, never growing the core infrastructure.
Private methods in the parent module are accessible to child modules in Rust,
so no visibility changes were needed.

**Metric.** `pub(crate)` fields allow submodules to operate on shared state
while keeping the public API unchanged. The refactoring is invisible to
external consumers.

---

## Pattern 4: Transport Negotiation for Platform Portability

**Problem.** UDS-first transport works on Linux/macOS but not Windows or
containers without shared filesystem. Hardcoded transport selection limits
deployment flexibility.

**Solution.** `connect_transport(address)` parses protocol prefixes:
- `unix:/path/to/socket.sock` → UDS
- `tcp:127.0.0.1:9100` → TCP
- `/path/to/socket.sock` (starts with `/`) → implicit UDS
- `127.0.0.1:9100` (anything else) → implicit TCP

**Ecosystem alignment.** Mirrors primalSpring's transport negotiation. TCP is
the default for cross-platform portability; UDS is the performance path for
biomeOS-native deployments.

---

## Pattern 5: Graceful Degradation as Architecture (Not Error Handling)

**Problem.** Traditional error handling treats missing dependencies as
failures. In a composed system where primals may be unavailable, this blocks
gameplay.

**Solution.** Every primal interaction is wrapped in a degradation pipeline:
1. Check domain availability via `bridge.has(domain)`
2. Attempt the call
3. If the call fails, degrade to mechanical defaults
4. Never block gameplay for a missing primal

The 6-phase enrichment pipeline in `enrichment.rs` exemplifies this:
1. AI narration (Squirrel via `ai.suggest` / `ai.query`)
2. NPC dialogue (Squirrel via `ai.query` with NPC context)
3. Flow evaluation (local `science/` module — no IPC)
4. Scene push to UI (petalTongue)
5. Provenance vertex append (rhizoCrypt)
6. Session completion check (DAG close)

**Key insight.** "Degradation placeholder" strings are clearly labeled (e.g.
`[AI primal unavailable — narration placeholder for: {prompt}]`) so no path
pretends to be a real primal response. This is honesty, not failure.

---

## Pattern 6: Coverage as Quality Gate, Not Vanity Metric

**Problem.** Coverage numbers are meaningless if they measure test boilerplate
rather than behavior. Simply adding `assert!(true)` tests inflates numbers
without catching bugs.

**Solution.** Target specific uncovered *behavior paths*:
- Content validation edge cases (missing NPC, empty compound predicates)
- Topological sort edge cases (diamond dependencies, missing deps, cycles)
- Metadata ingestion edge cases (missing fields, bad TOML)
- Protocol handling (TCP listener parse errors, empty lines)

**Metric.** 84.42% → 90.84% lines, achieved by adding 149 targeted tests
that exercise previously untested code paths, not by adding trivial assertions.

**Constraint.** Rust 2024's `unsafe` classification of `std::env::set_var()`
means env-var-dependent code paths cannot be directly unit-tested under
`#![forbid(unsafe_code)]`. Design code so env-var paths have pure-function
alternatives.

---

## Pattern 7: `#[expect]` Over `#[allow]` (V5.1)

**Problem.** `#[allow(clippy::some_lint)]` silently suppresses warnings
forever — if a refactoring fixes the underlying issue, the dead suppression
persists as noise.

**Solution.** Migrate all lint suppressions to `#[expect(clippy::some_lint,
reason = "justification")]`. This:
- **Fires a warning** if the lint is no longer triggered (dead suppressions
  surface automatically)
- **Requires a reason string**, documenting why the suppression is justified
- Narrows scope precisely (e.g. `expect_used` only, not `unwrap_used`, when
  the test module uses `.expect()` but not `.unwrap()`)

**Key insight.** During migration, several `#[allow]` attributes were found
to be dead. For example, `handle_tools_list` had `#[allow(too_many_lines)]`
but the function was 5 lines — the lint never fired. Rather than converting
dead suppressions to `#[expect]` (which would trigger unfulfilled-expectation
warnings), remove them entirely.

**Better than suppressing.** In some cases, refactoring eliminates the lint
at the source. `handle_tcp_connection` had `#[allow(needless_pass_by_value)]`
because it took owned `TcpStream`. Refactoring the signature to accept
`&TcpStream` eliminates the lint without any suppression attribute.

---

## Pattern 8: Dynamic Port Allocation in Tests (V5.1)

**Problem.** Hardcoded ports in experiment binaries and integration tests
cause spurious failures during parallel test execution.

**Solution.** Bind to `127.0.0.1:0` and let the OS assign an ephemeral port:

```rust
fn allocate_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}
```

This pattern eliminates port collisions without configuration.

---

## Pattern 9: Signal Dispatch — Collapsing Multi-Call Sequences (V8)

**Problem.** Provenance requires four sequential IPC calls: `content.put` →
`dag.event.append` → `spine.seal` → `braid.create`. Each can fail
independently, creating a combinatorial error space. Network round-trips
accumulate latency.

**Solution.** biomeOS Wave 17 introduced atomic signals: `nest.store` and
`nest.commit`. A single `ctx.dispatch("nest.store", payload)` tells the
orchestration layer to execute the full 4-call sequence internally. The
caller receives one result.

**Implementation in Webb.** Bridge methods `nest_store()` and `nest_commit()`
check for Neural API availability first. If present, they dispatch the
atomic signal. If absent, they fall back to direct domain calls
(`dag.event.append` and `dag.session.complete` respectively). The enrichment
pipeline calls `bridge.nest_store()` — it neither knows nor cares which
path executed.

**Key insight.** Signal dispatch is a *transparent optimization*. Callers
don't change their error handling, retry logic, or degradation patterns.
The bridge absorbs the complexity.

**Ecosystem relevance.** Any garden or spring that coordinates multiple
primals for a single logical operation should adopt signal dispatch when
biomeOS is available. The fallback-to-direct pattern ensures standalone
mode continues to work.

---

## Pattern 10: Self-Announcement at Startup (V8)

**Problem.** Primals discover each other via filesystem socket probes. This
works for clients discovering servers, but servers have no way to broadcast
their availability or capability set proactively.

**Solution.** On serve startup, call `primal.announce` via the Neural API
with the full list of exposed methods and the socket path. biomeOS routes
this to Songbird for registration in the capability registry.

**Implementation in Webb.** `announce_to_biomeos()` is called between
bridge construction and IPC server start. It passes 24 method names and
the UDS socket path. If Neural API is unavailable, it degrades silently —
the existing filesystem probe discovery continues to work.

**Key insight.** Self-announcement turns composition from "pull-only" (who
is out there?) to "push-pull" (I'm here, and here's what I can do). This
is critical for dynamic composition where primals start and stop.

---

## Pattern 11: Canonical Schema Consumption — Wave 20 Envelope Normalization (V9)

**Problem.** The ecosystem has two generations of `capability.list` responses:
pre-Wave-20 primals return raw arrays, while Wave 20+ primals return the
canonical envelope `{ capabilities, count, primal }`. A consumer that calls
`capabilities.list` on arbitrary primals must handle both.

**Solution.** Add an envelope normalization layer (`unwrap_capabilities_envelope`)
between the raw IPC response and consumer code. If the response has the
canonical `capabilities` + `count` keys, pass it through. If it's a raw array,
wrap it in the canonical shape. Consumers always see a consistent envelope.

**Implementation in Webb.** `PrimalClient::capabilities()` calls three method
name variants (`capabilities.list`, `capability.list`, `primal.capabilities`),
then passes the result through `unwrap_capabilities_envelope()`. Webb's own
`handle_capabilities_list()` now emits the canonical shape with `count`.

**Key insight.** Normalization at the consumer boundary — not at the producer
or the wire — is the least disruptive way to handle schema evolution in a
heterogeneous ecosystem where primals ship at different Wave levels.

---

## Pattern 12: Stability Tier Awareness — Method Lifetime Contracts (V9)

**Problem.** Consumers hardcode method names in dispatch maps without knowing
whether those names are frozen or evolving. A renamed `evolving` method breaks
consumers silently.

**Solution.** Annotate method groups in `capability_registry.toml` with
stability tiers: `stable` (wire name frozen), `evolving` (may change with
deprecation cycle), `internal` (implementation detail).

**Implementation in Webb.** All sourDough, lifecycle, session, and domain
methods are annotated `stable`. MCP tools are `evolving` (the MCP spec is
still evolving). The annotation is documentation, not runtime enforcement —
consumers reference it when deciding which methods to hardcode.

**Key insight.** Stability tiers are the ecosystem's equivalent of semver for
individual methods. They enable informed dependency decisions without requiring
full versioned APIs.

---

## Pattern 13: Degradation Behavior Contracts — Written Failure Modes (V9)

**Problem.** Each primal domain degrades differently when unreachable, but the
behavior was implicit in code paths. Consumers and upstream teams had to read
Rust source to understand failure modes.

**Solution.** Document per-domain degradation in `docs/DEGRADATION_BEHAVIOR.md`
as a formal contract: domain, primal, unreachable behavior, consumer impact.
This aligns with the ecosystem invariant from Wave 20: "Science is never gated
behind primal availability."

**Implementation in Webb.** 9 domain degradation contracts, signal dispatch
fallback table, trio partial completion state table, standalone vs composition
mode documentation.

**Key insight.** Written degradation contracts are more valuable than the code
that implements them. They let upstream teams (springs, other gardens) design
for Webb's failure envelope without reading Rust source.

---

## Pattern 14: Trio Partial Completion Tracking (V9)

**Problem.** The provenance trio (rhizoCrypt DAG, loamSpine spine, sweetGrass
braid) is not atomic. In real deployments, some primals may be unreachable.
Without tracking which primals responded, consumers cannot distinguish
"full provenance" from "DAG only" from "standalone."

**Solution.** Add `primals_reached: Vec<String>` to `WorldState`. Populate it
during provenance operations: `["dag"]` for DAG only, `["dag", "spine", "braid"]`
for full trio. Empty means standalone.

**Implementation in Webb.** `record_provenance_vertex()` pushes `"dag"` to
`primals_reached` on successful `nest.store`. Future spine and braid
integration will add their entries. Session state serialization exposes this
to API consumers.

**Key insight.** Partial provenance is valid provenance. The consumer decides
whether partial is acceptable — not the producer. This is the trio integration
guide's core insight: "no rollback on partial."

---

## Remaining Open Gaps

| GAP | Summary | Owner | Status |
|-----|---------|-------|--------|
| GAP-002 | Visualization primal lacks CRPG dialogue tree scene type | petalTongue | Open |
| GAP-003 | AI primal NPC dialogue constraint enforcement | Squirrel | Open |
| GAP-004 | Provenance trio end-to-end (wiring complete, live validation pending) | rhizoCrypt / loamSpine / sweetGrass | Open |
| GAP-006 | Discovery primal capability-filtered queries (tier-5) | Songbird | Open |
| GAP-007 | Voice interjection preview without live AI | esotericWebb (self) + Squirrel | Open |
| GAP-008 | Creative content pack format for distribution | esotericWebb (self) | Open |
| GAP-009 | RulesetCert YAML authoring and per-plane validation | esotericWebb (self) | Open |
| GAP-010 | plasmidBin population and deployment automation | biomeOS / primalSpring | Open |
| GAP-016 | ~~ludoSpring UDS-only transport~~ | superseded (V6) | Absorbed |
| GAP-017 | biomeOS neural-api fails to start in benchScale | biomeOS | Open |
| GAP-018 | neuralAPI executors not exposed on JSON-RPC | biomeOS | Open |
| GAP-019 | beardog crypto domain not wired into Webb bridge | esotericWebb (self) | Open |
| GAP-020 | Deploy graph format divergence (TOML vs JSON) | primalSpring / wateringHole | Open |
| GAP-021 | Game science has no standalone primal | primalSpring / wateringHole | Open |
| GAP-024 | Signal dispatch not exercised E2E against live biomeOS | esotericWebb / biomeOS | Open |
| GAP-025 | `primal.announce` outbound wiring | esotericWebb (self) | **Resolved V8** |
| GAP-026 | `capabilities.list` canonical envelope | esotericWebb (self) | **Resolved V9** |
| GAP-027 | Stability tier annotations | esotericWebb (self) | **Resolved V9** |
| GAP-028 | Degradation behavior documentation | esotericWebb (self) | **Resolved V9** |
| GAP-029 | Trio partial completion tracking | esotericWebb (self) | **Resolved V9** |
| GAP-030 | Bridge canonical envelope parsing | esotericWebb (self) | **Resolved V9** |

---

## Architecture Summary (V9)

```
Springs (science + experiments)  →  produce  →  primals (genomeBin/ecoBin)
                                                       ↓
                                               plasmidBin/ (deployment)
                                                       ↓
                              Webb discovers + composes via JSON-RPC IPC
                                     (TCP default, UDS for biomeOS)
                                                       ↓
                              On startup: primal.announce → biomeOS (24 capabilities)
                                        ↓                    ↓
                              local science/         signal-first primal calls
                              (flow, engagement,     nest.store → (fallback: dag.*)
                               DDA — no IPC)         nest.commit → (fallback: dag.session.complete)
                                        ↓                    ↓
                              6-phase enrichment pipeline per action:
                                narrate → dialogue → flow → scene → nest.store → nest.commit
                                                       ↓
                              All phases degrade gracefully → gameplay never blocked
                              Signal dispatch collapses multi-call sequences when biomeOS present
                                                       ↓
                              primals_reached tracks trio partial completion per session
                              capabilities.list emits canonical Wave 20 envelope
                              capability_registry.toml annotated with stability tiers
```

43 Rust files, ~13.2k LOC, 357 tests, ~91% coverage, zero unsafe, zero C
dependencies, pure Rust edition 2024. No spring dependencies — self-composed
via primal composition only. 24 capabilities exposed, Wave 17 signal adoption,
Wave 20 canonical schema compliance, stability tier awareness.

---

## The Three-Generation Validation Story: Python → Rust → Primal Composition

The ecoPrimals ecosystem validates peer-reviewed science through three
generations of increasing abstraction. Each generation's output becomes
the next generation's validation target.

### Layer 1: Python Baselines (Springs)

Springs begin with canonical Python implementations of peer-reviewed science.
These scripts are reproducible, provenance-tracked, and produce golden JSON
targets:

```
# ludoSpring example: Fitts's law, flow theory, Perlin noise
python baselines/python/interaction_laws.py  → combined_baselines.json
python baselines/python/flow_engagement.py   → combined_baselines.json
python baselines/python/perlin_noise.py      → combined_baselines.json
```

Every baseline carries provenance: script name, git commit, date, exact
command. Tolerances are named, centralized, and justified from the source
paper's reported precision.

### Layer 2: Rust Validation (Springs → Primals)

Springs re-implement the same algorithms in pure Rust (via barraCuda for
numerical primitives). Validation binaries compare Rust output against
Python golden values within documented tolerances:

```
validate_interaction  — Fitts, Hick, Steering, GOMS laws
validate_procedural   — Perlin, fBm, BSP, L-system Fibonacci
validate_engagement   — composite metrics, Four Keys classification
```

Each binary exits 0 (pass) or 1 (fail). The `validate_all` meta-runner
aggregates results. This is the "hotSpring pattern" — hardcoded expected
values with explicit pass/fail. ludoSpring V43 achieved 790+ tests and
three-layer validation across all game science domains.

### Layer 3: Primal Composition Validation (Springs + Gardens)

The breakthrough: once Rust code is proven against Python, the same values
become golden targets for **IPC composition validation**. Springs generate
`composition_targets.json` — the same science, but now the expected outputs
for JSON-RPC calls through the primal stack:

```
validate_composition  — calls primals via IPC, compares against golden JSON
                        exits 0 (pass), 1 (fail), 2 (skip if server absent)
```

This proves that science works identically whether called as:
- A Python function (`interaction_laws.fitts_cost(...)`)
- A Rust library call (`ludospring_barracuda::interaction::fitts::cost(...)`)
- A primal IPC call (`{"method": "game.fitts_cost", "params": {...}}`)

### Layer 4: Pure Composition (Gardens)

Gardens like Esoteric Webb take this further. They **don't re-prove the
math at all**. The science was already validated three times over by the
springs. Gardens are JUST primal compositions:

```
esotericWebb
├── science/         ← absorbed pure-math (flow, DDA, engagement)
│                      no IPC, no validation ladder needed
├── ipc/bridge/      ← routes to primals by capability
│                      ai.query → Squirrel
│                      dag.* → rhizoCrypt
│                      viz.* → petalTongue
│                      storage.* → NestGate
└── deploy graphs    ← biomeOS compositions: phases, ordering, health
```

Gardens validate **composition correctness** (do primals respond? do
capabilities degrade gracefully?) not **numerical correctness** (that was
proven by the springs). This is the gen3→gen4 boundary.

### The Cycle

```
Python (peer-reviewed)
    ↓ golden targets
Rust (barraCuda primitives)
    ↓ golden targets
Primal IPC (composition validation)
    ↓ proven capabilities
Garden composition (product)
    ↓ gap discovery
Spring evolution (absorb gap → new primal capability)
    ↓ new golden targets
```

Every gap discovered by a garden feeds back through wateringHole handoffs
to the spring that produces the relevant primal. The spring evolves, the
primal absorbs, plasmidBin deploys, and the garden discovers the next gap.
This is NUCLEUS composition as a validation strategy — not just a deployment
model.
