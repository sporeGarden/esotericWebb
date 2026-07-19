# esotericWebb V22 — Scene Graph Binding Fix AAR

**Date**: Jul 18, 2026
**Author**: esotericWebb (flockGate)
**Version**: V22

## Summary

Fixed P1 bug identified in Wave 150h: `ui.render` usage was incorrect per the
ecosystem binding standard. Switched `push_scene_to_ui()` to attempt
`visualization.render.scene` first with a `game_scene` SceneGraph, falling
back to `ui.render` when the scene graph format is rejected.

## What Was Fixed

The blurb identified: "esotericWebb `ui.render` → `visualization.render`
Current `ui.render` usage is incorrect — switch to `game_scene` binding (P1 bug)"

**Root cause**: V21 used `ui.render` exclusively because the deployed
petalTongue (v1.6.6) rejected our scene payloads. However, the correct
architectural pattern is to attempt the canonical `visualization.render.scene`
first and only fall back when rejected.

**Fix**: `push_scene_to_ui()` now:
1. Builds a SceneGraph with `game_scene` typed nodes (Transform3D at z=0)
2. Attempts `visualization.render.scene`
3. On rejection, falls back to `ui.render` with text content
4. This is forward-compatible: when petalTongue v1.7+ deploys (with optional
   Transform3D), step 2 will succeed and step 3 won't execute

## Findings for Upstream

### petalTongue: Current v1.6.6 still requires Transform3D

The deployed petalTongue binary (v1.6.6, uptime ~57h) still requires `transform`
and has the "missing field `a`" edge deserialization issue. The Wave 150h
scene unification (optional Transform3D, flatten_3d, 14 tests) has been shipped
in code but the binary on flockGate has not been updated.

**Action**: Deploy petalTongue v1.7+ to flockGate to activate the scene graph
path. Webb will auto-switch without any code changes.

### SceneGraph format used by Webb

```json
{
  "nodes": {
    "entrance": {
      "id": "entrance",
      "label": "entrance",
      "type": "game_scene",
      "description": "A dusty antique shop...",
      "transform": { "position": [0,0,0], "scale": [1,1,1] },
      "metadata": { "turn": 0, "is_ending": false }
    },
    "npc_vale": {
      "id": "npc_vale",
      "label": "vale",
      "type": "game_npc",
      "transform": { "position": [2,0,0], "scale": [1,1,1] }
    }
  },
  "edges": [
    { "a": "entrance", "b": "npc_vale", "label": "present_in" }
  ]
}
```

This uses:
- `game_scene` node type for the current room
- `game_npc` node type for present NPCs
- Transform3D at z=0 (orthographic/2D-as-3D-slice)
- Edges for spatial relationships

Once petalTongue accepts this, the full scene graph pipeline is active.

## Metrics

- 453 tests passing
- 0 clippy warnings
- `scene_pushed: true` via fallback (ui.render)
- Forward-compatible for petalTongue v1.7+
