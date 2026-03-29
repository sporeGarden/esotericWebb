<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Esoteric Webb V5.1 — Evolution Patterns and Ecosystem Learnings

**Date:** March 29, 2026
**Author:** ecoPrimals / sporeGarden
**Foundation:** V1–V4 bootstrap through live primal composition; V5 deep debt resolution; V5.1 audit evolution
**Coverage:** 90.84% lines (335 tests)

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
1. AI narration (ludoSpring → Squirrel, fallback to direct Squirrel)
2. NPC dialogue (talk actions only)
3. Flow evaluation (game science)
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

## Remaining Open Gaps

| GAP | Summary | Owner |
|-----|---------|-------|
| GAP-002 | Visualization primal lacks CRPG dialogue tree scene type | petalTongue |
| GAP-003 | AI primal NPC dialogue constraint enforcement | Squirrel |
| GAP-004 | Provenance trio end-to-end (wiring complete, live validation pending) | rhizoCrypt / loamSpine / sweetGrass |
| GAP-006 | Discovery primal capability-filtered queries (tier-5) | Songbird |
| GAP-007 | Voice interjection preview without live AI | esotericWebb (self) + Squirrel |
| GAP-008 | Creative content pack format for distribution | esotericWebb (self) |
| GAP-009 | RulesetCert YAML authoring and per-plane validation | esotericWebb (self) + ludoSpring |
| GAP-010 | plasmidBin population and deployment automation | biomeOS / primalSpring |

---

## Architecture Summary (V5.1)

```
Springs (science + experiments)  →  produce  →  primals (genomeBin/ecoBin)
                                                       ↓
                                               plasmidBin/ (deployment)
                                                       ↓
                              Webb discovers + composes via JSON-RPC IPC
                                     (TCP default, UDS for biomeOS)
                                                       ↓
                              6-phase enrichment pipeline per action:
                                narrate → dialogue → flow → scene → DAG → close
                                                       ↓
                              All phases degrade gracefully → gameplay never blocked
```

37 Rust files, ~12.5k LOC, 335 tests, 90.84% coverage, zero unsafe, zero C
dependencies, pure Rust edition 2024.
