<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# wateringHole — Esoteric Webb

Operational documentation, guides, and evolution handoffs for Webb.

## Structure

```
wateringHole/
├── README.md           ← you are here
├── handoffs/           ← active evolution handoffs to springs
│   └── archive/        ← completed handoffs (dated, never deleted)
```

## How handoffs work

1. Webb exercises a primal composition and discovers a gap
2. The gap is logged in `EVOLUTION_GAPS.md` with evidence
3. When the gap is actionable, a handoff document is written in `handoffs/`
4. The owning spring picks up the handoff, evolves, and redeploys
5. The handoff moves to `handoffs/archive/` with a completion date
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

| Handoff | Direction | Summary |
|---------|-----------|---------|
| `ESOTERICWEBB_V3_ECOSYSTEM_ABSORPTION_HANDOFF_MAR24_2026.md` | Outbound | V3 IPC handler split, MCP, resilience patterns |
| `ESOTERICWEBB_V4_LIVE_PRIMAL_COMPOSITION_HANDOFF_MAR24_2026.md` | Outbound | V4 bridge methods, composition pipeline, provenance lifecycle |
| `ESOTERICWEBB_V4_ECOSYSTEM_REVIEW_ABSORPTION_HANDOFF_MAR24_2026.md` | Inbound | Absorption opportunities from 8 sibling springs |

A corresponding ecosystem handoff lives at:
`ecoPrimals/wateringHole/handoffs/ESOTERICWEBB_V4_GEN4_FIRST_CONSUMER_HANDOFF_MAR24_2026.md`

## Guides

_Guides will be added as the team grows. Topics planned:_

- Setting up a local primal stack from `plasmidBin/`
- Writing your first narrative content
- Running experiments and reading results
- Deploying Webb as a BYOB composition
