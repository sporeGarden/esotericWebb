<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# wateringHole — Esoteric Webb

Operational documentation, guides, and evolution handoffs for Webb.

## Structure

```
wateringHole/
├── README.md           ← you are here
├── handoffs/           ← evolution handoffs to springs (dated, kept as fossil record)
```

## How handoffs work

1. Webb exercises a primal composition and discovers a gap
2. The gap is logged in `EVOLUTION_GAPS.md` with evidence
3. When the gap is actionable, a handoff document is written in `handoffs/`
4. A corresponding handoff is filed at `ecoPrimals/infra/wateringHole/handoffs/`
5. The owning spring picks up the handoff, evolves, and redeploys
6. Webb absorbs the new capability via `plasmidBin/` discovery

## Handoff format

```markdown
# HANDOFF: <short description>

- **Date**: YYYY-MM-DD
- **Gap**: GAP-NNN (from EVOLUTION_GAPS.md)
- **Target spring**: <spring name>
- **Target primal**: <primal name>
- **Evidence**: <what Webb tried and what happened>
- **Requested capability**: <specific JSON-RPC method or behavior>
- **Priority**: critical / high / medium / low
```

## Active handoffs

_None. All handoffs archived — V22 is green on all dimensions._

## Archived handoffs (fossil record)

| Handoff | Direction | Summary |
|---------|-----------|---------|
| `ESOTERICWEBB_WAVE150O_RESTART_AAR_JUL20_2026.md` | Outbound | Service restart, scorecard corrections (0 prod unwraps, 0 >800L) |
| `ESOTERICWEBB_V22_SCENE_BINDING_AAR_JUL18_2026.md` | Outbound | V22 scene graph binding fix, `game_scene` format, fallback to `ui.render` |
| `ESOTERICWEBB_V21_LIVE_VISUAL_AAR_JUL18_2026.md` | Outbound | V21 live visual system, petalTongue `ui.render` composition, HTML frontend |
| `ESOTERICWEBB_DEPLOY_UNBLOCK_AAR_JUL18_2026.md` | Outbound | Deploy blocker resolution: binary in depot, forgejo synced, flockGate:8090 live |
| `ESOTERICWEBB_V16_LIVE_COMPOSITION_AAR_HANDOFF_JUL17_2026.md` | Outbound | V16 live primal composition on flockGate, discovery reverse-mapping |
| `ESOTERICWEBB_V15_DEEP_DEBT_EVOLUTION_HANDOFF_JUL17_2026.md` | Outbound | V15 domain wiring (crypto, mesh), mock cleanup, voice engine |
| `ESOTERICWEBB_V14_WAVE107_INTROSPECTION_HANDOFF_JUN10_2026.md` | Outbound | V14 method.describe, TransportEndpoint |
| `ESOTERICWEBB_V13_WAVE75_METRICS_HANDOFF_JUN03_2026.md` | Outbound | V13 session metrics, mesh push propagation |
| `ESOTERICWEBB_V12_WAVE74_ZERO_DEBT_HANDOFF_JUN03_2026.md` | Outbound | V12 zero debt, typed constructors, mesh registration |
| `ESOTERICWEBB_V11_WAVE67_POLISH_HANDOFF_JUN01_2026.md` | Outbound | V11 dead code removal, `#![forbid(unsafe_code)]` |
| `ESOTERICWEBB_V10_WAVE46_ABSORPTION_HANDOFF_MAY23_2026.md` | Outbound | V10 env_keys, deploy graph metadata, announce hints |
| `ESOTERICWEBB_V9_WAVE20_ABSORPTION_HANDOFF_MAY17_2026.md` | Outbound | V9 canonical schemas, stability tiers, degradation contracts |
| `ESOTERICWEBB_V8_SIGNAL_ADOPTION_HANDOFF_MAY16_2026.md` | Outbound | V8 signal dispatch, primal.announce |
| `ESOTERICWEBB_V7_COMPOSITION_PATTERNS_HANDOFF_APR17_2026.md` | Outbound | V7 composition patterns |
| `ESOTERICWEBB_V6_PRIMAL_CAPABILITY_GAPS_HANDOFF_MAR29_2026.md` | Outbound | V6 capability gaps matrix |
| `ESOTERICWEBB_V51_AUDIT_EVOLUTION_HANDOFF_MAR29_2026.md` | Outbound | V5.1 use-case gaps (GAP-016–020) |
| `ESOTERICWEBB_V4_LIVE_PRIMAL_COMPOSITION_HANDOFF_MAR24_2026.md` | Outbound | V4 bridge methods, provenance lifecycle |
| `ESOTERICWEBB_V4_ECOSYSTEM_REVIEW_ABSORPTION_HANDOFF_MAR24_2026.md` | Inbound | Absorption from 8 sibling springs |
| `ESOTERICWEBB_V3_ECOSYSTEM_ABSORPTION_HANDOFF_MAR24_2026.md` | Outbound | V3 IPC handler split, MCP, resilience |

Corresponding ecosystem handoffs live at:
`ecoPrimals/infra/wateringHole/handoffs/`

## Guides

_Guides will be added as the team grows. Topics planned:_

- Setting up a local primal stack from `plasmidBin/`
- Writing your first narrative content
- Running experiments and reading results
- Deploying Webb as a BYOB composition
