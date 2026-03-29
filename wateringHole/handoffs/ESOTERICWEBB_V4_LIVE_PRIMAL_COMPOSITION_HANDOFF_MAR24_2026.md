> **ARCHIVE NOTE (V6)**: This handoff describes V4 architecture including
> `game.*` domain IPC via ludoSpring and `ai.chat`/`ai.inference`. V6
> removed all spring dependencies. Retained as evolution fossil record.

# ESOTERICWEBB V4 — Live Primal Composition Handoff

**Date**: 2026-03-24
**Version**: V4
**Author**: esotericWebb development

## Summary

V4 wires live primal composition into the game loop. Every `GameSession::act()`
now runs the full ecosystem pipeline: AI narration, NPC dialogue, scene
rendering, game science evaluation, and provenance lifecycle. All calls
degrade gracefully — standalone mode works unchanged.

## Changes

### Critical fix: session.start bridge preservation

- `handle_session_start` now extracts the `PrimalBridge` from the previous
  session via `take_bridge()` before creating a new one
- This fixes a bug where IPC `session.start` dropped all primal composition
  capabilities by creating `GameSession::new()` (bridge-less)

### Full PrimalBridge method coverage

11 new methods on `PrimalBridge`, all with graceful degradation:

| Method | Domain | Delegates via |
|--------|--------|---------------|
| `engagement()` | game | resilient_call → ludoSpring |
| `npc_dialogue()` | game | resilient_call → ludoSpring → Squirrel |
| `narrate_action()` | game | resilient_call → ludoSpring → Squirrel |
| `voice_check()` | game | resilient_call → ludoSpring |
| `game_push_scene()` | game | resilient_call → ludoSpring → petalTongue |
| `game_begin_session()` | game | resilient_call → ludoSpring |
| `game_complete_session()` | game | resilient_call → ludoSpring |
| `dag_session_complete()` | dag | resilient_call → rhizoCrypt |
| `dag_query_vertices()` | dag | resilient_call → rhizoCrypt |
| `mint_certificate()` | lineage | resilient_call → loamSpine |
| `poll_input()` | visualization | resilient_call → petalTongue |

### AI narration pipeline in act()

The `enrich_action()` method implements a three-tier narration strategy:

1. **ludoSpring narrate_action** — game-science-enriched narration via
   `game.narrate_action` (ludoSpring adds game context, delegates to Squirrel)
2. **Direct Squirrel** — if game science unavailable, direct `ai.chat`
   for plain AI narration
3. **Mechanical text** — if no AI primal connected, returns director outcome

For talk actions, `game.npc_dialogue` returns dialogue text and voice
interjections (VoiceNote → VoiceEnrichment).

### Scene rendering via petalTongue

`push_scene_to_ui()` sends scene state (node, description, NPCs, turn,
is_ending) to petalTongue via `bridge.render_scene()` after each action.

### Game science flow evaluation

Per-action `bridge.evaluate_flow()` runs when the game science primal is
connected. Results (flow_score, in_flow) included in `PrimalEnrichment`.

### Provenance lifecycle

- **Session start**: `initialize_provenance()` calls `dag.session.create`,
  stores returned session_id in `WorldState.session_id`
- **Per action**: existing `dag.event.append` now uses real session_id
- **Session end**: `complete_provenance_if_ended()` calls `dag.session.complete`
  when director reaches an ending node

### New types

- `PrimalEnrichment` — serializable struct in NarrationContext carrying all
  composition results (ai_narration, npc_dialogue, voice_notes, flow_score,
  in_flow, scene_pushed)
- `VoiceEnrichment` — voice interjection with voice_id and text
- `GameSession::take_bridge()` — extracts bridge for session replacement

## Quality gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets` | pass (0 errors) |
| `cargo test --workspace` | 160 passed, 0 failed |
| `cargo doc --workspace --no-deps` | pass |
| `cargo deny check` | pass |

## Files modified

- `webb/src/session.rs` — PrimalEnrichment, VoiceEnrichment types;
  take_bridge, initialize_provenance, enrich_action, push_scene_to_ui,
  complete_provenance_if_ended methods; enrichment field in NarrationContext
- `webb/src/ipc/bridge.rs` — 11 new bridge methods with degradation + tests
- `webb/src/ipc/ludospring.rs` — 7 new typed client methods (npc_dialogue,
  narrate_action, voice_check, push_scene, begin_session, complete_session)
- `webb/src/ipc/mod.rs` — METHOD_DAG_SESSION_COMPLETE, METHOD_DAG_QUERY_VERTICES,
  METHOD_CERT_MINT constants
- `webb/src/ipc/handlers/session.rs` — bridge preservation + provenance init
- `webb/src/ipc/handlers/mcp.rs` — test lint fix

## Consumed primals (V4 surface)

| Domain | Primal | Methods wired |
|--------|--------|---------------|
| ai | Squirrel | ai.chat, ai.summarize |
| game | ludoSpring | evaluate_flow, engagement, difficulty_adjustment, npc_dialogue, narrate_action, voice_check, push_scene, begin_session, complete_session |
| visualization | petalTongue | render.scene, interaction.poll |
| dag | rhizoCrypt | session.create, event.append, frontier.get, merkle.root, session.complete, query.vertices |
| lineage | loamSpine | certificate.mint |
| compute | toadStool | compute.dispatch.submit |
| storage | nestGate | storage.store, storage.retrieve |

## Ecosystem feedback

- GAP-001 (IPC degradation stubs) → **resolved** — all domains wired
- GAP-004 (provenance trio not end-to-end) → **wiring complete** — awaiting
  live integration test against rhizoCrypt from plasmidBin
