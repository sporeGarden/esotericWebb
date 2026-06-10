<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HANDOFF: esotericWebb V14 — Method Introspection, TransportEndpoint, Wave 107 Absorption

- **Date**: 2026-06-10
- **From**: esotericWebb (sporeGarden/ironGate)
- **To**: primalSpring, biomeOS, songBird, ecosystem consumers
- **Version**: V14
- **Wave compliance**: 107

---

## What Landed

### `method.describe` — Runtime Method Introspection (barraCuda Pattern)

New `method.describe` IPC method — structured metadata for any exposed method:

```json
{
  "method": "session.act",
  "description": "Perform a player action — returns outcome and narration context",
  "domain": "session",
  "stability": "stable",
  "access": "public",
  "params": "{ kind: string, id: string }",
  "primal": "esotericwebb"
}
```

Unknown methods return `{ "method": "...", "found": false }` (graceful, no error).

- 26th capability (24 stable + 2 evolving)
- Follows barraCuda v0.4.0 pattern exactly
- Enables self-correcting distributed compositions

### TransportEndpoint — Ecosystem Wire Format

New structured transport type in `ipc/discovery.rs`:

```rust
enum TransportEndpoint {
    Uds { path: String },
    Tcp { host: String, port: u16 },
    MeshRelay { peer_id: String, relay: String },
}
```

- Serde-tagged: `{"transport":"uds","path":"..."}` matches songBird wire format
- `PrimalEndpoint::resolve_transport()` — best-effort (UDS > TCP)
- `PrimalEndpoint::available_transports()` — enumerate all
- Ready for `ipc.resolve` consumption (GAP-006 next step)

### Mesh Registration Updated

- `method.describe` added to stable tier in `route.register` payload
- 26 capabilities total (24 stable + 2 evolving)

---

## Posture

| Metric | Value |
|--------|-------|
| Tests | 453 (434 unit + 18 E2E + 1 validation) |
| Capabilities | 26 (24 stable + 2 evolving) |
| Clippy | Clean (-D warnings, pedantic + nursery) |
| Unsafe | `#![forbid(unsafe_code)]` |
| C deps | Zero (ecoBin compliant) |
| `Result<_, String>` | Zero in production |
| Files > 800L | Zero |
| Wave compliance | 107 |

---

## Ecosystem Absorption Notes

### From Wave 107

| Source | What We Absorbed | How |
|--------|-----------------|-----|
| barraCuda v0.4.0 | `method.describe` pattern | New `introspection.rs` handler module |
| songBird `ipc.resolve` | TransportEndpoint wire format | New enum in `discovery.rs` |
| southGate mesh validation | Confirmed wire format shapes | Serde deserialization tests |
| biomeOS auto-register | Awareness of auto-registration | Mesh payload includes full capability set |

### Not Yet Absorbed (Blocked Externally)

| Item | Blocked On |
|------|-----------|
| `ipc.resolve` live query (GAP-006) | songBird UDS on ironGate |
| Trio E2E (GAP-004) | Live rhizoCrypt binary |
| petalTongue CRPG scenes (GAP-002) | petalTongue VizRegistry |
| Squirrel NPC constraints (GAP-003) | Squirrel personality cert support |

---

## For Upstream Teams

### primalSpring
- Registry should note `method.describe` as 26th stable method
- Webb now has full introspection parity with barraCuda

### songBird
- Webb's `TransportEndpoint` type is ready to deserialize `ipc.resolve` responses
- When songBird is available on ironGate UDS, Webb can wire Tier 5 discovery

### biomeOS
- `route.register` payload now includes `method.describe` in stable tier
- 26 capabilities registered (was 25)
