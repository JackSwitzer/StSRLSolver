# Unit Chunks: Remaining Parity Closure

Last updated: 2026-02-24
Canonical repo: `/Users/jackswitzer/Desktop/SlayTheSpireRL`
Spec index: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

This file is the canonical unit-level execution map for remaining parity work.
Each row is one feature-sized commit target.

## Operating constraints
- One feature ID per commit.
- Sequence per feature: docs -> tests -> code -> tracker update.
- Merge gate per feature: targeted tests green + full suite green (`uv run pytest tests/ -q`).
- Java behavior is source of truth where conflicts exist.

## Feature chunks

| feature_id | lane | domain | priority | dependencies | acceptance tests | java refs | rng notes | status |
|---|---|---|---|---|---|---|---|---|
| `DOC-TODO-001` | A | docs/process | P0 | none | docs lint/readability pass | n/a | n/a | completed |
| `DOC-ACTION-001` | A | action layer | P0 | `DOC-TODO-001` | `tests/test_agent_api.py` | action API shape in runtime | deterministic action IDs only | completed |
| `DOC-WFLOW-001` | A | execution process | P1 | `DOC-TODO-001` | n/a (docs-only) | n/a | n/a | completed |
| `DOC-SPEC-CRD-001` | A | card behavior spec | P0 | `DOC-TODO-001` | n/a (docs-only) | `cards/*` Java refs index | n/a | completed |
| `DOC-SPEC-POW-001` | A | power behavior/order spec | P0 | `DOC-TODO-001` | n/a (docs-only) | `powers/*` Java refs index | n/a | completed |
| `DOC-SPEC-RNG-001` | D | rng migration spec | P0 | `RNG-SPEC-001` | n/a (docs-only) | runtime callsite inventory | stream mapping locked | completed |
| `DOC-SPEC-RL-001` | D | RL dashboard/search spec | P1 | `DOC-ACTION-001` | n/a (docs-only) | n/a | n/a | completed |
| `DOC-SPEC-INDEX-001` | A | spec index cleanup | P0 | `DOC-SPEC-CRD-001`, `DOC-SPEC-POW-001`, `DOC-SPEC-RNG-001`, `DOC-SPEC-RL-001` | n/a (docs-only) | n/a | n/a | completed |
| `RNG-SPEC-001` | D | rng spec | P0 | `DOC-TODO-001` | `tests/test_rng_audit.py` (reference expectations) | Java Random/stream ownership semantics | stream ownership matrix | completed |
| `AUD-GEN-001` | A | java inventory | P0 | `DOC-TODO-001` | `tests/test_audit_inventory_manifest.py` | decompile root + class fallback for potions | no stream mutation (audit-only) | completed |
| `AUD-GEN-002` | A | manifests | P0 | `AUD-GEN-001` | `tests/test_audit_inventory_manifest.py` | generator script rows mapped to Java root | deterministic sorted output | completed |
| `AUD-GEN-003` | A | manifest anomalies | P1 | `AUD-GEN-002` | `tests/test_audit_inventory_manifest.py` | `events/beyond/SpireHeart.java`, starter card classes | n/a | completed |
| `CRD-INV-003A` | B | card inventory | P0 | `AUD-GEN-002` | `tests/test_card_id_aliases_audit.py` | card class IDs under `cards/*` | n/a | completed |
| `CRD-INV-003B` | B | card aliases | P0 | `CRD-INV-003A` | `tests/test_card_id_aliases_audit.py`, manifest diff | Java class-name vs cardID mismatches | n/a | completed |
| `POW-002A` | C | dispatch coverage | P0 | `AUD-GEN-002` | `tests/test_audit_power_dispatch.py` | `powers/*.java` hook overrides | deterministic dispatch map | completed |
| `POW-002B` | C | hook ordering | P0 | `POW-002A` | `tests/test_power_registry_integration.py` | Java turn/card trigger sequence | no random in dispatch | pending |
| `POW-003A` | C | power behaviors | P1 | `POW-002B` | `tests/test_powers.py`, `tests/test_power_edge_cases.py` | per-power class behavior | no direct `random.*` | pending |
| `POW-003B` | C | cross-system powers | P1 | `POW-003A` | `tests/test_effects_and_combat.py`, orb/relic integration | power+card+orb interactions | deterministic replay required | pending |
| `CRD-IC-001` | B | Ironclad behaviors | P1 | `CRD-INV-003B` | `tests/test_ironclad_cards.py` + parity suites | `cards/red/*.java` | touch only owned streams | pending |
| `CRD-SI-001` | B | Silent behaviors | P1 | `CRD-INV-003B` | `tests/test_silent_cards.py` + parity suites | `cards/green/*.java` | touch only owned streams | pending |
| `CRD-WA-001` | B | Watcher behaviors | P1 | `CRD-INV-003B` | `tests/test_cards.py`, `tests/test_watcher_card_effects.py` | `cards/purple/*.java` | touch only owned streams | pending |
| `CRD-SH-002` | B | shared cards/status | P1 | `CRD-INV-003B` | `tests/test_status_curse.py`, card parity suites | `cards/colorless|curses|status/*.java` | touch only owned streams | pending |
| `CRD-DE-001` | B | Defect behaviors | P1 | `POW-003A` | `tests/test_defect_cards.py` + integration | `cards/blue/*.java` | orb/power stream consistency | pending |
| `RNG-MOD-001` | D | rng module | P0 | `RNG-SPEC-001` | `tests/test_rng.py`, `tests/test_rng_parity.py` | Java Random wrapper parity | no direct `random.*` in touched paths | pending |
| `RNG-MOD-002` | D | rng migration | P0 | `RNG-MOD-001` | `tests/test_rng_audit.py`, touched domain tests | touched runtime modules only | replace `random.*` with stream access | pending |
| `RNG-TEST-001` | D | determinism | P0 | `RNG-MOD-002` | replay/determinism suites | deterministic seed+action trajectories | assert stable outcomes | pending |
| `RL-ACT-001` | D | action mask | P1 | `DOC-ACTION-001` | `tests/test_agent_api.py`, `tests/test_agent_readiness.py` | n/a | stable ordered legal list | pending |
| `RL-OBS-001` | D | observation profiles | P1 | `RL-ACT-001` | `tests/test_agent_readiness.py` | n/a | human-visible default, debug optional | pending |
| `RL-DASH-001` | D | dashboard | P2 | `RL-ACT-001`, `RL-OBS-001` | dashboard smoke tests | n/a | read-only on episode logs | pending |
| `RL-SEARCH-001` | D | planner architecture | P2 | `RL-ACT-001` | planner unit tests (new) | n/a | macros external to env API | pending |

## Immediate execution queue
1. `POW-002B`
2. `POW-003A`
3. `CRD-IC-001`
4. `CRD-SI-001`
5. `RNG-MOD-001`
