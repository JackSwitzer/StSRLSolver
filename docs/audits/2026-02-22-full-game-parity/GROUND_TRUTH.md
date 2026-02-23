# Ground Truth: Java Parity + Agent Contract

Last updated: 2026-02-23
Working branch: `codex/cons-002b`

## Baseline
- Command: `uv run pytest tests/ -q`
- Result: `4715 passed, 0 skipped, 0 failed`
- No skips executed in the current baseline run.

## Canonical sources
- Java reference root:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`
- Canonical Python repo:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL`
- Script-generated parity artifacts:
  - `docs/audits/2026-02-22-full-game-parity/traceability/java-inventory.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/python-inventory.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/parity-diff.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/power-hook-coverage.json`

## Inventory and mapping snapshot (generated)

| Domain | Java count | Python count | Exact | Alias | Missing | Notes |
|---|---:|---:|---:|---:|---:|---|
| cards | 361 | 370 | 228 | 112 | 21 | behavior/inventory gaps still present |
| relics | 181 | 181 | 75 | 106 | 0 | inventory parity closed |
| events | 52 | 51 | 40 | 11 | 1 | only unmatched Java class: `SpireHeart` |
| powers | 149 | 148 | 125 | 24 | 0 | full Java class mapping achieved |
| potions | unavailable locally | 42 | 0 | 0 | 0 | local decompile snapshot has no `potions/` dir |

## Power hook dispatch snapshot (generated)
- Registry hooks: `25`
- Runtime-dispatched hooks: `14`
- Registered-but-undispatched hooks: `11`
- Current undispatched set:
  - `atDamageFinalReceive`
  - `atDamageGive`
  - `atDamageReceive`
  - `modifyBlock`
  - `onAttack`
  - `onAttacked`
  - `onAttackedToChangeDamage`
  - `onCardDraw`
  - `onManualDiscard`
  - `onScry`
  - `wasHPLost`

## Agent contract snapshot

### Stable public API
- `GameRunner.get_available_action_dicts()`
- `GameRunner.take_action_dict()`
- `GameRunner.get_observation()`

### Observation contract versions (non-breaking)
- `observation_schema_version` is now emitted at observation root.
- `action_schema_version` is now emitted at observation root.

### Current explicit action-surface phases
- `neow`, `map`, `combat`, `reward`, `boss_reward`, `event`, `shop`, `rest`, `treasure`.
- Follow-up selection actions:
  - `select_cards`
  - `select_stance`

## Consolidation state
- Canonical repo lock file added: `REPO_CANONICAL.md`.
- Training-wrapper migration manifest added:
  - `docs/audits/2026-02-22-full-game-parity/traceability/repo-consolidation-manifest.md`
- Curated training utilities migrated into:
  - `packages/training/`
- Desktop one-folder realignment verified:
  - `docs/audits/2026-02-22-full-game-parity/traceability/desktop-realignment-2026-02-23.md`
- Combat runtime unification phase A completed:
  - `CombatRunner` now wraps `CombatEngine` (compatibility facade)
  - Compatibility lock tests: `tests/test_combat_runner_compat.py`
- Combat runtime unification phase B completed:
  - removed duplicated legacy CombatRunner runtime implementation block
  - `packages/engine/handlers/combat.py` now retains shim + encounter helpers only

## Priority remaining blockers
1. Card inventory/behavior closure for the 21 Java-side card rows marked missing.
2. Power runtime hook dispatch closure (`11` registered hooks not yet dispatched).
3. Potion inventory audit completion once local Java potion sources are restored.
4. Sustain `0 skipped` baseline in default CI while preserving parity replay checks.
