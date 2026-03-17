# Remaining Work Index (Spec Canonical)

Last updated: 2026-02-24
Canonical repo: `/Users/jackswitzer/Desktop/SlayTheSpireRL`

This index is the canonical entry point for all remaining parity and RL work.

## Open feature inventory

| feature_id | area | priority | depends_on | spec |
|---|---|---|---|---|
| `POW-002B` | power hook order | P1 | `POW-002A` | `POWER_BEHAVIOR_ORDER_SPEC.md` |
| `POW-003A` | power behavior closure | P1 | `POW-002B` | `POWER_BEHAVIOR_ORDER_SPEC.md` |
| `POW-003B` | power integration tests | P1 | `POW-003A` | `POWER_BEHAVIOR_ORDER_SPEC.md` |
| `CRD-IC-001` | Ironclad cards | P1 | `CRD-INV-003B` | `CARD_BEHAVIOR_PARITY_SPEC.md` |
| `CRD-SI-001` | Silent cards | P1 | `CRD-INV-003B` | `CARD_BEHAVIOR_PARITY_SPEC.md` |
| `CRD-WA-001` | Watcher cards | P1 | `CRD-INV-003B` | `CARD_BEHAVIOR_PARITY_SPEC.md` |
| `CRD-SH-002` | shared cards/status | P1 | `CRD-INV-003B` | `CARD_BEHAVIOR_PARITY_SPEC.md` |
| `CRD-DE-001` | Defect cards | P1 | `POW-003A` | `CARD_BEHAVIOR_PARITY_SPEC.md` |
| `RNG-MOD-001` | RNG stream plumbing | P1 | `RNG-SPEC-001` | `RNG_RUNTIME_MIGRATION_SPEC.md` |
| `RNG-MOD-002` | RNG callsite migration | P1 | `RNG-MOD-001` | `RNG_RUNTIME_MIGRATION_SPEC.md` |
| `RNG-TEST-001` | RNG determinism lock | P1 | `RNG-MOD-002` | `RNG_RUNTIME_MIGRATION_SPEC.md` |
| `RL-ACT-001` | action mask contract | P2 | `DOC-ACTION-001` | `RL_RUNBOARD_AND_SEARCH_SPEC.md` |
| `RL-OBS-001` | observation profiles | P2 | `RL-ACT-001` | `RL_RUNBOARD_AND_SEARCH_SPEC.md` |
| `RL-DASH-001` | runboard dashboard | P2 | `RL-ACT-001`, `RL-OBS-001` | `RL_RUNBOARD_AND_SEARCH_SPEC.md` |
| `RL-SEARCH-001` | planner/search layer | P2 | `RL-ACT-001` | `RL_RUNBOARD_AND_SEARCH_SPEC.md` |
| `AUD-001` | final behavior re-audit | P2 | parity features above | `CORE_TODO.md` |
| `AUD-003` | RL launch sign-off | P2 | `AUD-001`, RL features above | `rl/rl-readiness.md` |

## Locked execution order
1. `POW-002B`
2. `POW-003A`
3. `CRD-IC-001`
4. `CRD-SI-001`
5. `RNG-MOD-001`
6. `RNG-MOD-002`
7. `RNG-TEST-001`
8. `CRD-WA-001`
9. `CRD-SH-002`
10. `CRD-DE-001`
11. `POW-003B`
12. `RL-ACT-001`
13. `RL-OBS-001`
14. `RL-DASH-001`
15. `RL-SEARCH-001`
16. `AUD-001`
17. `AUD-003`

## Commit and PR operating rules
1. One `feature_id` per commit.
2. Per-feature loop: `docs -> tests -> code -> tracker update -> commit`.
3. Gate every commit with targeted tests and full `uv run pytest tests/ -q`.
4. Update all trackers: `TODO.md`, `CORE_TODO.md`, `UNIT_CHUNKS.md`.

## Fast links
- Card spec: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/CARD_BEHAVIOR_PARITY_SPEC.md`
- Power spec: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/POWER_BEHAVIOR_ORDER_SPEC.md`
- RNG spec: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/RNG_RUNTIME_MIGRATION_SPEC.md`
- RL spec: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/RL_RUNBOARD_AND_SEARCH_SPEC.md`
- Unit chunks: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/traceability/UNIT_CHUNKS.md`
- Core TODO: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/CORE_TODO.md`
