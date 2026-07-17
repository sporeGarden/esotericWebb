<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Degradation Behavior — Esoteric Webb

Per-primal degradation contracts. Gameplay is never blocked by an
unavailable primal (ecosystem invariant).

## Invariant

> **Gameplay is never gated behind primal availability.**
> All IPC calls return `Result`. No method panics on unreachable primals.
> The enrichment pipeline completes in all degradation states.

## Per-Domain Degradation

| Domain | Primal | Unreachable Behavior | Consumer Impact |
|--------|--------|----------------------|-----------------|
| ai | Squirrel | `ai_narrate()` returns labeled placeholder: `[AI primal unavailable — narration placeholder for: {prompt}]` | Narration is mechanical, not generative. Playable. |
| ai | Squirrel | `ai_summarize()` returns truncated context: `[degraded: summary unavailable] ...` | Context summaries are raw truncations. |
| visualization | petalTongue | `render_scene()` returns `Ok(())` silently | No visual rendering. Text-mode output only. |
| dag | rhizoCrypt | `dag_session_create()` returns `None`; `dag_event_append()` returns `Err` | No provenance session. `session_id` stays empty. `primals_reached` stays empty. |
| lineage | loamSpine | `certificate.mint` returns `Err` | NPC personality certs unavailable. Trust system works locally. |
| compute | toadStool | `compute.dispatch.submit` returns `Err` | GPU compute unavailable. Local science (flow, DDA) still runs. |
| storage | nestGate | `storage.store` returns `Err` | Key-value persistence unavailable. Session state is in-memory only. |
| provenance | sweetGrass | `braid.create` / `braid.query` returns `None` | Creative attribution unavailable. DAG provenance may still record (partial). |
| crypto | bearDog | `crypto_sign` returns `None`; `crypto_verify` returns `false` | Provenance vertices unsigned. Content integrity trust-on-first-use. |
| mesh | songBird | `mesh_topology` / `mesh_health` returns `None` | Topology data unavailable. Webb operates with local discovery only. |
| orchestration | biomeOS | Neural API socket not found | Signal dispatch (`nest.store`, `nest.commit`) falls back to direct domain calls. `primal.announce` is a no-op. |

## Signal Dispatch Degradation

| Signal | biomeOS Available | biomeOS Unavailable |
|--------|:-----------------:|:-------------------:|
| `nest.store` | Atomic 4-call collapse via orchestration | Falls back to `dag.event.append` |
| `nest.commit` | Atomic session finalization | Falls back to `dag.session.complete` |
| `primal.announce` | Broadcasts 24 capabilities to registry | Silent no-op; filesystem probe discovery continues |

## Trio Partial Completion States

Per `infra/wateringHole/PROVENANCE_TRIO_INTEGRATION_GUIDE.md`:

| State | DAG | Spine | Braid | Valid? | Webb Behavior |
|-------|:---:|:-----:|:-----:|:------:|---------------|
| Full | YES | YES | YES | YES | Complete provenance chain. `primals_reached: [dag, spine, braid]` |
| DAG + spine | YES | YES | no | YES | Ledger entry without attribution. `primals_reached: [dag, spine]` |
| DAG only | YES | no | no | YES | Session recorded, unbacked. `primals_reached: [dag]` |
| None | no | no | no | YES | Standalone mode. `primals_reached: []` |

**Rule**: No rollback on partial. Webb accepts whatever trio data is available
and reports `primals_reached` in session state. Consumers (AI agents,
petalTongue renderers) decide whether partial provenance is acceptable.

## Standalone Mode

When zero primals are connected, Webb operates in standalone mode:
- All gameplay mechanics work (narrative, state, abilities, trust, conditions)
- Flow evaluation runs locally (`science/flow` — no IPC)
- DDA runs locally (`science/dda` — no IPC)
- All enrichment phases return defaults or empty values
- Provenance is not recorded
- Scene rendering is text-mode only

This is the development and testing mode. No configuration needed.

## Composition Mode

When one or more primals are connected:
- Bridge methods attempt IPC before falling back
- Each enrichment phase degrades independently
- Partial composition is normal and expected
- `bridge.statuses()` reports per-domain health
