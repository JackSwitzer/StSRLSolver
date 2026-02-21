# Core TODO: RL-Blocking Parity

Last updated: 2026-02-21

## Current baseline
- Full tests: `4602 passed, 0 skipped, 0 failed`
- Command: `uv run pytest tests/ -ra`
- Primary residual risk is now concentrated in events/relic edge parity and remaining potion execution-path de-duplication.

## Comprehensive remaining TODO (Java parity + agent completeness)
1. Potion parity (highest risk)
- [x] Finish Java-exact behavior for Discovery/Liquid Memories/Distilled Chaos/Entropic Brew/Gambler's Brew/Elixir/Stance Potion/Snecko Oil/Smoke Bomb.
- [ ] Add RNG-stream assertions (`cardRng`, `cardRandomRng`, `potionRng`) for all selection and random-output potions.
- [ ] Ensure every selection potion fully round-trips through action dict API (missing params => candidate actions).
2. Event parity and action-surface closure
- [ ] Ensure all event choice generators are registered and action-emitted.
- [ ] Close Java mismatches for Dead Adventurer, Falling, Knowing Skull, Gremlin wheel/match games, Note for Yourself.
- [ ] Add deterministic multi-step event tests (choice availability, follow-up phase transitions, rewards/costs).
3. Relic parity remaining
- [ ] Replace deterministic auto-picks for on-acquire relic choices with explicit agent selection actions (Astrolabe, Empty Cage, Orrery, bottled assignment).
- [ ] Verify boss/chest/reward ordering edge cases against Java with regression tests.
- [ ] Keep alias normalization consistent across run/combat/handlers.
4. Power and hook-order parity
- [ ] Re-audit power hook ordering beyond current `onAfter*` coverage.
- [ ] Expand cross-power interaction tests for timing-sensitive powers (including turn-bound debuffs/buffs).
5. API and documentation hardening
- [ ] Keep `docs/work_units/granular-*.md` synced per merged behavior batch.
- [ ] Keep `docs/audits/*` updated with exact Java references, RNG notes, and test deltas.
- [ ] Add/maintain domain-level parity snapshots and rerun after each major merge batch.

## P0 (must complete before serious RL training)
- [x] Implement model-selectable potion flows (`Discovery`, `LiquidMemories`, `GamblersBrew`, `Elixir`, `StancePotion`) with explicit action candidates.
  - Details: [`domains/potions.md`](./domains/potions.md)
- [x] Implement true `DistilledChaos` play-top-cards behavior (not draw fallback) including targeting and trigger order.
  - Details: [`domains/potions.md`](./domains/potions.md)
- [ ] Complete relic pickup selection workflows (Astrolabe/Empty Cage/Orrery/bottled relics) with deterministic selection APIs.
  - Details: [`domains/relics.md`](./domains/relics.md)
- [x] Unskip and make green critical relic suites (`test_relic_pickup.py`, `test_relic_bottled.py`, `test_relic_acquisition.py`, `test_relic_rest_site.py`).
  - Details: [`domains/relics.md`](./domains/relics.md)

## P1 (important for long-horizon policy quality)
- [x] Align Entropic Brew class pool parity and RNG-stream advancement with Java across all call paths.
  - Details: [`domains/potions.md`](./domains/potions.md)
- [ ] Re-audit event logic against Java with selection/action-level parity checks (not only handler existence).
  - Details: [`domains/events.md`](./domains/events.md)
- [x] Add regression tests for Smoke Bomb restrictions/reward suppression and escape reward policy.
  - Details: [`domains/potions.md`](./domains/potions.md)

## P2 (cleanup and maintenance)
- [ ] Remove remaining duplicate potion semantics between `packages/engine/combat_engine.py` and `packages/engine/registry/potions.py` (single authoritative path).
- [ ] Normalize relic ID aliases (`MawBank` vs `Maw Bank`, etc.) across run/combat contexts.
- [ ] Keep granular checklists in `docs/work_units/` synced with implemented status.

## Already completed in this pass
- [x] Fixed failing Thousand Cuts timing test to use `onAfterCardPlayed`.
- [x] Added `onAfterUseCard` and `onAfterCardPlayed` execution in `packages/engine/handlers/combat.py`.
- [x] Added Ectoplasm blocked-gold tracking (`RunState.gold_blocked`) and enabled corresponding test.
- [x] Improved Smoke Bomb handling path (boss validation + escape flow support in game loop).
- [x] Added combat metadata/RNG attachment from `GameRunner` into combat state for potion/relic parity hooks.
- [x] Added alias-safe relic matching in `RunState` and duplicate relic fallback to `Circlet`.
- [x] Added Mark of Bloom no-heal handling in rest flow.
- [x] Added action-surface selection context for potion flows (`select_cards` / `select_stance`) in `GameRunner.take_action_dict`.
- [x] Added action-api tests for potion selection roundtrips.
- [x] Replaced mock/placeholder relic trigger suites with engine-backed tests and reached `0` skips in full test suite.
- [x] Centralized Egg relic behavior for card acquisition in `RunState.add_card` and added generated-path coverage (`tests/test_relic_eggs.py`).

## Execution order
1. Potions
2. Relics
3. Events re-audit
4. Full unskip campaign + full-test gate
