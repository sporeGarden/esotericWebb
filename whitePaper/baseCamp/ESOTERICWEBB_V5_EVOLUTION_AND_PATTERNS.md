<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Esoteric Webb V5 — Evolution Patterns and Ecosystem Learnings

**Date:** March 25, 2026
**Author:** ecoPrimals / sporeGarden
**Foundation:** V1–V4 bootstrap through live primal composition; V5 deep debt resolution
**Coverage:** 90.84% lines (329 tests)

---

## What This Is

This document captures patterns, discoveries, and architectural learnings from
Esoteric Webb's evolution from V1 (bootstrap) through V5 (deep debt
resolution). Webb is the first gen4 consumer in the ecoPrimals ecosystem —
a CRPG substrate that composes primals via JSON-RPC IPC without importing
any spring or primal Rust crates.

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

**Problem.** `session.rs` grew to 1192 lines (violating 1000-line quality
gate). Naive splitting would scatter related logic across files without
improving cohesion.

**Solution.** Identify *semantic boundaries* within the module:
1. **Data structures** → `session/types.rs` (ActionRecord, ActionKind,
   AvailableAction, GameStateSnapshot, NarrationContext, PrimalEnrichment,
   VoiceEnrichment)
2. **Primal composition pipeline** → `session/enrichment.rs` (enrich_action,
   push_scene_to_ui, complete_provenance_if_ended, record_provenance_vertex)
3. **Core session logic** → `session/mod.rs` (act, state management,
   initialization, serialization)

**Key insight.** Make fields `pub(crate)` to allow submodules to operate on
shared state while keeping the public API unchanged. The refactoring is
invisible to external consumers.

**Metric.** 1192 lines → 891 + 178 + 15 (types) = 1084 total across three
files, each under 1000 lines.

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
biomeOS-native deployments. The `ESOTERICWEBB_TRANSPORT_PRIORITY` env var
allows runtime override.

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
- Launcher pure functions (topological sort cycles, diamonds, TOML parsing)
- Discovery metadata ingestion (missing fields, bad TOML, unknown domains)
- Client fallback chains (capabilities method negotiation, health liveness)
- Enrichment pipeline paths (standalone bridge, provenance with history)
- TCP listener protocol handling (valid JSON-RPC, parse errors, empty lines)

**Metric.** 84.42% → 90.84% lines, achieved by adding 149 targeted tests
(267 → 316 unit tests) that exercise previously untested code paths, not
by adding trivial assertions.

**Constraint.** `unsafe_code = "forbid"` in Rust 2024 edition means
`std::env::set_var()` and `std::env::remove_var()` are unavailable in tests.
This limits env-var-dependent test coverage. Design code so env-var paths
have pure-function alternatives that can be unit-tested independently.

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

## Architecture Summary (V5)

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

35 Rust files, ~12.5k LOC, 329 tests, 90.84% coverage, zero unsafe, zero C
dependencies, pure Rust edition 2024.
