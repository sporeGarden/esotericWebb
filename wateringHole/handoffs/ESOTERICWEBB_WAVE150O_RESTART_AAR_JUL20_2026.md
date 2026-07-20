# esotericWebb Wave 150o — Restart + Scorecard AAR

**Date**: Jul 20, 2026
**Author**: esotericWebb (flockGate)
**Wave**: 150o

## Summary

Restarted esotericWebb on flockGate (was 502). Investigated scorecard
discrepancies from the fresh ecosystem audit.

## P1: Service Restart

`webb.primals.eco` was returning 502 — the esotericWebb process had died
(likely from the server being backgrounded without `nohup`). Restarted with
`nohup` for persistence. Returning 200 on `0.0.0.0:8090`.

**Recommendation**: `loginctl enable-linger flockgate` + systemd user unit
for esotericWebb so it survives logout and reboots.

## Scorecard Corrections

### >800L files: 0 (not 2)

The scorecard shows `client.rs (855L), discovery.rs (813L)` — this is stale
data from before the refactor commit (`98496ac`). Current line counts:

- `client.rs`: 772 lines (HTTP transport extracted to `client_http.rs`)
- `discovery.rs`: 753 lines (path helpers extracted to `discovery_paths.rs`)

Both under 800. Fix landed in Wave 150i, pushed to Forgejo.

### Production `.unwrap()`: 0 (not 406)

The ecosystem grep methodology (`grep -rn '.unwrap()' --include='*.rs'`
excluding test files) doesn't handle Rust's inline `mod tests {}` blocks.
All 406 matches are inside `#[cfg(test)]` modules or dedicated `_tests.rs`
files. Every test module has `#[expect(clippy::unwrap_used)]`.

**This false-positive likely inflates counts across the entire ecosystem.**
Recommend the audit script parse `#[cfg(test)]` module boundaries rather
than relying on filename patterns.

### `unsafe`: 0

`#![forbid(unsafe_code)]` is set at the workspace root (`webb/src/lib.rs`).
The scorecard shows 1, which may be counting the `forbid` attribute itself
or a dependency.

## Metrics

- 453 tests passing, 0 clippy, 0 fmt drift
- 0 production unwraps, 0 unsafe, 0 files >800L, 0 TODO/FIXME
- 6/9 primals connected, V22 live at `webb.primals.eco`
