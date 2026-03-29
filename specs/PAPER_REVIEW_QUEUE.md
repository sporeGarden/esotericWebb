<!--
SPDX-License-Identifier: AGPL-3.0-or-later
Documentation and creative text in this file: CC-BY-SA-4.0
-->

# Paper Review Queue

Esoteric Webb is a **tool spring** (gen4 consumer), not a science spring.
It does not produce paper-parity experiment suites against published
baselines. Instead, it exercises the primal stack via BYOB composition and
feeds discovered gaps back into the science springs that own the primals.

## Applicable papers (informing design, not requiring parity)

| Paper / reference | Domain | How it informs Webb | Owning spring |
|-------------------|--------|---------------------|---------------|
| ZA/UM internal design (Disco Elysium postmortems) | Narrative design | Bounded-infinite architecture, skill-as-voice, failure-as-content | esotericWebb (self) |
| Cliche Studio (Esoteric Ebb) | Game mechanics | Multi-plane play, transparent dice, RulesetCert per plane | esotericWebb (self) |
| RPGPT internal voices spec | AI + game science | `VoiceId` system, personality constraints, voice temperature | ludoSpring |
| Provenance DAG lifecycle | Data provenance | `dag.session.create` → `dag.event.append` → `dag.merkle.root` | rhizoCrypt |
| Creative attribution chains | Attribution | `braid.create`, `attribution.chain`, PROV-O export | sweetGrass |
| NPC personality certificates | Lineage | `certificate.mint` for trust-gated knowledge bounds | loamSpine |

## Validation approach

Webb validates via **structural experiments** (numbered exp001–exp005+), not
numerical baseline comparison. Each experiment exercises a specific
composition pattern with explicit pass/fail and exit codes.

See `experiments/README.md` for the full experiment matrix and
`EVOLUTION_GAPS.md` for discovered gaps feeding back to springs.
