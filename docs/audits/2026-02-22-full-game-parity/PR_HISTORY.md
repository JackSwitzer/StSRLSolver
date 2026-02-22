# PR History (Merged to `main`)

Last updated: 2026-02-22  
Merged branch lineage covered here: `codex/parity-core-loop` -> `main`

## Merged parity PRs

| PR | Merged (UTC) | Feature IDs | Scope | Primary files | Primary tests |
|---|---|---|---|---|---|
| [#14](https://github.com/JackSwitzer/StSRLSolver/pull/14) | 2026-02-22T17:55:59Z | `REL-006` | Canonical relic alias resolver + `Toolbox` parity | `packages/engine/content/relics.py`, `packages/engine/state/run.py` | `tests/test_relic_aliases.py` |
| [#15](https://github.com/JackSwitzer/StSRLSolver/pull/15) | 2026-02-22T18:06:57Z | `REL-007` | Relic ordering/context regressions (combat/victory/start hooks) | `packages/engine/game.py`, `packages/engine/registry/relics.py`, `packages/engine/state/combat.py` | `tests/test_relic_ordering_rel007.py`, `tests/test_audit_relics_combat.py` |
| [#16](https://github.com/JackSwitzer/StSRLSolver/pull/16) | 2026-02-22T18:20:26Z | `EVT-001`, `EVT-002` | Explicit event selection actions + selected-card passthrough | `packages/engine/game.py`, `packages/engine/handlers/event_handler.py` | `tests/test_agent_api.py` event selection tests |
| [#17](https://github.com/JackSwitzer/StSRLSolver/pull/17) | 2026-02-22T18:23:23Z | `EVT-003` | Deterministic multi-phase event action-surface coverage | `packages/engine/game.py`, `tests/test_agent_api.py` | `tests/test_agent_api.py` Golden Idol multi-phase tests |
| [#18](https://github.com/JackSwitzer/StSRLSolver/pull/18) | 2026-02-22T18:27:01Z | `EVT-004` | Java event alias normalization closure | `packages/engine/handlers/event_handler.py` | `tests/test_audit_events.py` alias normalization tests |
| [#19](https://github.com/JackSwitzer/StSRLSolver/pull/19) | 2026-02-22T18:38:53Z | `RWD-001`, `RWD-002` | Canonical reward action emission/execution path | `packages/engine/game.py`, `packages/engine/handlers/reward_handler.py` | `tests/test_agent_api.py` reward action mapping tests |
| [#20](https://github.com/JackSwitzer/StSRLSolver/pull/20) | 2026-02-22T18:42:38Z | `RWD-003` | Proceed gating for unresolved mandatory rewards | `packages/engine/game.py` | `tests/test_agent_api.py` proceed gating tests |
| [#21](https://github.com/JackSwitzer/StSRLSolver/pull/21) | 2026-02-22T19:07:48Z | `RWD-004` | Indexed Black Star relic claims + secondary relic gating | `packages/engine/game.py`, `packages/engine/handlers/reward_handler.py` | `tests/test_agent_api.py` second-relic indexed tests |

## Cumulative outcome through #21
- Region closure:
  - `R1` relic action-surface and ordering fixes: complete.
  - `R2` event action-surface completeness and alias normalization: complete.
  - `R3` reward action canonicalization and proceed gating: complete.
- Baseline after merge:
  - `uv run pytest tests/ -q`
  - `4659 passed, 5 skipped, 0 failed`.

## Work not in this lineage
- [#8](https://github.com/JackSwitzer/StSRLSolver/pull/8) is open on `consolidation/clean-base-2026-02-03` and is not part of the `codex/parity-core-loop` merge chain above.
