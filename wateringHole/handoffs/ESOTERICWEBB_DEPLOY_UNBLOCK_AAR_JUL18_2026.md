<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# AAR: esotericWebb Deploy Unblock — flockGate Live

- **Date**: 2026-07-18
- **From**: esotericWebb (flockGate team)
- **Wave**: 147i response
- **Audience**: cellMembrane ops, sporeGate ops, eastGate overwatch

## Blockers Resolved

| Blocker (from 147i) | Resolution |
|---------------------|-----------|
| Binary not in depot | **DONE** — `infra/plasmidBin/primals/esotericwebb` (3.3M, stripped x86_64 PIE). Manifest updated to V17. |
| Repo not on sporeGate | **ALREADY DONE** — Forgejo remote `git.primals.eco:2222/sporeGarden/esotericWebb` confirmed up to date (V17). |
| flockGate:8080 not responding | **CLARIFIED** — port 8080 is **nestGate**, not Webb. Webb was never on 8080. Now live on **8090**. |

## Current State (flockGate)

```
esotericWebb (release, V17)
  Listen:    0.0.0.0:8090  (TCP JSON-RPC, mesh-accessible)
  UDS:       /run/user/1000/biomeos/esotericwebb.sock
  Primals:   6/9 connected (squirrel, petaltongue, nestgate, loamspine, sweetgrass, beardog)
  Content:   The Weaver's Parlor (5 NPCs, 8 abilities, 11 scenes, 11 nodes)
  Health:    {"status":"healthy","version":"0.1.0"}
  Binary:    3.3M, zero C deps, 472 tests
```

Confirmed accessible from mesh: `10.13.37.6:8090` responds to `health.liveness`.

## What We Own (esotericWebb team)

- The code (V17, pushed to both GitHub and Forgejo)
- The binary (stripped release in plasmidBin)
- The content (`content/` directory in repo)
- The deploy fragment (`deploy/esotericwebb.toml`)

## What Upstream Owns (persistence, routing)

| Item | Owner | Detail |
|------|-------|--------|
| systemd service unit | cellMembrane | NUCLEUS pattern: `ExecStart=esotericwebb serve --content /path/to/content --listen 0.0.0.0:8090` |
| Caddy route `/webb/` | cellMembrane / golgiBody | Reverse proxy to `flockGate:8090` (or wherever Webb deploys permanently) |
| Auto-restart on crash | cellMembrane | `Restart=on-failure` in unit |
| Deploy to sporeGate | sporeGate ops | Either clone from Forgejo and build, or fetch binary from plasmidBin |
| HPC composition | cellMembrane | Webb discovers primals across gates via songBird mesh — no code changes needed, just primal availability |

## Deploy Command

```bash
# From repo checkout:
cargo build --release --bin esotericwebb
strip target/release/esotericwebb

# Run:
./esotericwebb serve --content content/ --listen 0.0.0.0:8090

# Or with explicit graph-driven primal launch:
./esotericwebb serve --content content/ --listen 0.0.0.0:8090 --launch --graph deploy/esotericwebb.toml
```

## Notes

- Webb is a **composition consumer** — it discovers primals at runtime, wherever they run. If primals run on ironGate, sporeGate, or northGate, Webb finds them via UDS (local) or songBird (mesh). No code changes needed for HPC topology.
- The binary is statically self-contained. Only runtime requirement is the `content/` directory.
- Port 8090 matches the footPrint pattern. Caddy can route `/webb/` the same way it routes `/footprint/`.
- songBird closed 3 items for us in 147f (PROXY_PATH, raw JSON-RPC, discovery schemas). We should re-test mesh discovery now that songBird has `/jsonrpc`.
