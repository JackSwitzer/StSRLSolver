# Relics Audit

## Summary
Relic parity coverage is now anchored to real engine behavior for pickup/rest/chest/out-of-combat flows. Placeholder assertions and mock handlers were replaced with handler-backed tests; remaining skips are external parity-artifact skips in `tests/test_parity.py`.

## Status vs composer 1.5 gap list
- Pickup effects (`Astrolabe`, `Calling Bell`, `Empty Cage`, `Tiny House`, etc.): `PARTIAL`
  - Runtime behavior implemented and test-covered; explicit agent card-selection APIs remain pending for `Empty Cage`, `Orrery`, and bottled assignment.
- Bottled relic `atBattleStart` trigger: `DONE`
- War Paint/Whetstone `miscRng`: `DONE`
- Chest counters (`Tiny Chest`, `Matryoshka`, `Cursed Key`): `DONE`
- N'loth's Hungry Face chest removal behavior: `DONE`
- Maw Bank room-entry gain + spend deactivation: `DONE`
- Ectoplasm energy/gold handling: `DONE` for blocked-gold tracking and interactions covered by tests.
- onObtainCard relic effects in run-state path (Ceramic Fish, Egg relics, Darkstone): `DONE`

## Code areas
- `packages/engine/state/run.py`
- `packages/engine/game.py`
- `packages/engine/handlers/rooms.py`
- `packages/engine/handlers/shop_handler.py`
- `tests/test_relic_*`
- `docs/work_units/granular-relics.md`

## What changed in this pass
- Replaced `tests/test_relic_rest_site.py` with direct `RestHandler` behavior tests.
- Replaced `tests/test_relic_acquisition.py` with direct `TreasureHandler`/`RewardHandler` behavior tests.
- Replaced `tests/test_relic_triggers_outofcombat.py` with cross-system integration tests (GameRunner + reward/shop/run-state).
- Centralized `RunState.add_card` on-obtain-card policy for Egg upgrades plus resource side effects (Ceramic Fish, Darkstone Periapt).
- Added `tests/test_relic_eggs.py` to verify Egg behavior across shop purchases, combat rewards, timing, and multi-egg stacking.
- Extended egg coverage to off-class card acquisition (`Toxic Egg 2` upgrades non-Watcher skills after pickup).
- Added room-entry triggers in `GameRunner` for Maw Bank and Ssserpent Head.
- Corrected chest handling for N'loth's Hungry Face and Cursed Key ordering in `TreasureHandler.open_chest`.
- Wired Smiling Mask into shop purge-cost generation path.
- Added explicit Astrolabe boss-relic selection flow in action API:
  - `pick_boss_relic` with `Astrolabe` now returns `requires_selection` + `select_cards` candidates.
  - Selected indices are applied through `RunState.add_relic(..., selection_card_indices=...)`.
  - Added tests in `tests/test_agent_api.py` for Astrolabe selection-required and roundtrip application.

## Remaining implementation tasks
- Implement explicit agent-facing card-selection flows for remaining on-acquire relic choices (Empty Cage/Orrery/bottled assignment), replacing deterministic auto-picks.
- Re-audit Java parity details for on-acquire selection ordering once action-surface card selection is complete.
