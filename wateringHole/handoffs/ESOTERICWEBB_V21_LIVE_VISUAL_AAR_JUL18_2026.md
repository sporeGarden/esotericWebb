# esotericWebb V21 — Live Visual System AAR

**Date**: Jul 18, 2026
**Author**: esotericWebb (flockGate)
**Version**: V21

## Summary

Wired petalTongue for live visual rendering of game scenes. The composition
pipeline now pushes scene content via `ui.render` (confirmed working) rather
than `visualization.render.scene` (which requires a full 3D SceneGraph wire
format). Added an interactive HTML frontend at `webb.primals.eco/` — the game
is now browser-playable with no external dependencies.

## What Shipped

| Item | Status |
|------|--------|
| `push_scene_to_ui()` → `ui.render` | Done — `scene_pushed: true` confirmed |
| `session.poll_input` JSON-RPC method | Done |
| HTML frontend (`GET /`) | Done — self-contained, no build step |
| `GET /api/status` JSON endpoint | Preserved |
| GAP-002 updated | partial — workaround shipped |

## Findings for Upstream

### petalTongue: SceneGraph schema (for petalTongue team)

`visualization.render.scene` expects a **map-based SceneGraph** with:
- `scene.nodes` — map of node ID → node object
- Each node requires `transform` (position/rotation/scale)
- Edges field with `a` field (edge source)

This is a 3D visualization format, not suited for text/narrative composition.
Webb's CRPG scenes are `{node, description, npcs, turn, is_ending}`.

**Recommendation**: Either:
1. Define a `scene_type: "narrative"` variant that accepts text content, or
2. Document `ui.render` as the canonical method for text-mode composition

### petalTongue: `ui.render` is the working path

Confirmed: `{"type": "text", "content": "...", "metadata": {...}}` → `{"rendered": true, "modality": "visual", "window_id": "main"}`

Webb uses this for all scene pushes going forward.

### petalTongue: `interaction.poll` ready

Webb exposes `session.poll_input` which delegates to petalTongue's
`interaction.poll`. When petalTongue has a rendering surface (window/panel),
input events from that surface will flow back through this path.

### petalTongue capabilities confirmed (v1.6.6)

56 capabilities including: `ui.render`, `visualization.render.scene`,
`visualization.render.stream`, `interaction.subscribe`, `interaction.poll`,
`motor.*`, `audio.synthesize`, `modality.*`, and more.

## Metrics

- 453 tests passing
- 6/9 primals connected (composition mode)
- `scene_pushed: true` confirmed with live petalTongue
- HTML frontend served at `GET /`
- Zero external JS/CSS dependencies

## Remaining Upstream Decisions

1. **SceneGraph vs ui.render** — petalTongue team to clarify intended path
   for narrative/text composition
2. **Rendering surface** — petalTongue renders to "window_id: main" but there's
   no user-facing window on the server. For browser users, Webb's HTML frontend
   is self-sufficient. For native rendering, petalTongue would need a display.
3. **Input loop** — When petalTongue has a native rendering surface, it can
   push input events via `interaction.poll`. Webb already polls this endpoint.

## Files Changed

- `webb/src/session/enrichment.rs` — switch to `render_ui`, fix semantics
- `webb/src/ipc/bridge/domains.rs` — add `render_ui()` method
- `webb/src/ipc/petaltongue.rs` — add method constants
- `webb/src/ipc/handlers/session.rs` — add `handle_session_poll_input`
- `webb/src/ipc/handlers/mod.rs` — wire `session.poll_input`
- `webb/src/ipc/mod.rs` — add `METHOD_SESSION_POLL_INPUT`
- `webb/src/session/mod.rs` — add `poll_visualization_input()`
- `webb/src/ipc/listener.rs` — HTML frontend, path routing
- `webb/static/index.html` — self-contained game UI
- `EVOLUTION_GAPS.md` — GAP-002 update
- `CHANGELOG.md` — V21 entry
