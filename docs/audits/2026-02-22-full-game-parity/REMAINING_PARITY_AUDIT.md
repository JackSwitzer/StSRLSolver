# Remaining Parity Audit (Comprehensive)

Last updated: 2026-02-23
Baseline branch: `codex/cons-aud-001`

## Verified baseline
- Command: `uv run pytest tests/ -q`
- Result: `4708 passed, 5 skipped, 0 failed`
- Replay-artifact skips remain isolated to `tests/test_parity.py`.

## Script-generated inventory/mapping snapshot
Source artifact: `docs/audits/2026-02-22-full-game-parity/traceability/parity-diff.json`

| Domain | Java | Python | Exact | Alias | Missing | Status summary |
|---|---:|---:|---:|---:|---:|---|
| cards | 361 | 370 | 228 | 112 | 21 | still open; inventory/behavior closure required |
| relics | 181 | 181 | 75 | 106 | 0 | inventory parity closed |
| events | 52 | 51 | 40 | 11 | 1 | only unresolved Java class is `SpireHeart` |
| powers | 149 | 148 | 125 | 24 | 0 | Java inventory mapping closed |
| potions | unavailable locally | 42 | 0 | 0 | 0 | local decompile snapshot lacks `potions/` source |

### Card rows currently unresolved (generated)
- `Alchemize`, `Apparition`, `Defend_Blue`, `Defend_Green`, `Defend_Red`, `Defend_Watcher`, `Equilibrium`, `Fasting`, `Nightmare`, `Overclock`, `PressurePoints`, `Recursion`, `SimmeringFury`, `SneakyStrike`, `SteamBarrier`, `Strike_Blue`, `Strike_Green`, `Strike_Purple`, `Strike_Red`, `Tranquility`, `VoidCard`

## Power dispatch parity snapshot
Source artifact: `docs/audits/2026-02-22-full-game-parity/traceability/power-hook-coverage.json`

- Registry hooks: `25`
- Runtime-dispatched hooks: `14`
- Registered-but-undispatched hooks: `11`
- Undispatched hooks:
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

## Consolidation status
- Canonical repo lock documented in `REPO_CANONICAL.md`.
- Wrapper migration decisions documented in `traceability/repo-consolidation-manifest.md`.
- Curated deterministic training utilities migrated into `packages/training/` with tests.

## Remaining closure order (locked)
1. `CONS-002`: combat runtime unification (`CombatEngine` canonical, `CombatRunner` compatibility path only).
2. `POW-002`: close the 11 registered-but-undispatched power hooks.
3. `CRD-INV-003`: resolve 21 card rows from generated parity diff.
4. `CONS-001B`: finish RNG normalization in parity-critical runtime paths.
5. `AUD-002`: remove default-run skips to reach `0 skipped, 0 failed`.
6. `AUD-003`: RL launch sign-off with frozen contracts.
