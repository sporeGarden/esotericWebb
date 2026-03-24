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

## Guides

_Guides will be added as the team grows. Topics planned:_

- Setting up a local primal stack from `plasmidBin/`
- Writing your first narrative content
- Running experiments and reading results
- Deploying Webb as a BYOB composition
