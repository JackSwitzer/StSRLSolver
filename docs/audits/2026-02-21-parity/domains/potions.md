# Potions Audit

## Status
- `P0 potion parity blockers addressed in runtime path`
- Selection-required potions are action-surface complete through `take_action_dict`.
- Full suite after this pass: `4602 passed, 0 skipped, 0 failed`.

## Implemented in this pass
- `DistilledChaos` now plays top cards (3/6 with Sacred Bark) instead of draw fallback.
  - Runtime path: `packages/engine/combat_engine.py` (`use_potion` + top-deck autoplay helpers).
  - Registry path also executes top-card autoplay via CombatEngine helper.
  - Java references:
    - `decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java`
    - `decompiled/java-src/com/megacrit/cardcrawl/actions/utility/NewQueueCardAction.java`
- `LiquidMemories` now uses `ctx.potency` directly (no Sacred Bark special-case branch), and action API cleanly errors on empty discard.
  - Java reference: potion uses selection flow with Sacred Bark increasing card count.
- `EntropicBrew` now fills true empty slots from class-specific PotionHelper-order pools, using `potion_rng`, and respects Sozu.
  - Java reference:
    - `decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java`
- `SneckoOil` randomization now aligns better with `RandomizeHandCostAction` behavior:
  - uses `card_random_rng` stream
  - skips negative-cost cards
  - keeps no-op when rolled cost equals current base cost
  - Java reference:
    - `decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RandomizeHandCostAction.java`
- `SmokeBomb` validation now blocks when boss/cannot-escape/BackAttack is present in runtime path.
  - Java reference: `SmokeBomb.canUse` constraints (boss + BackAttack).
- `PotionTargetType` now drives runtime potion target emission (`CombatEngine._get_potion_target`), removing hardcoded target lists.
- `onUsePotion` relic hooks now fire across selection potion paths (`LiquidMemories`, `GamblersBrew`, `ElixirPotion`, `StancePotion`, discovery family) in `GameRunner`.

## RNG stream notes
- Discovery offer sampling: `card_rng` (existing behavior retained).
- Distilled Chaos target selection: `card_random_rng` preferred.
- Snecko Oil hand cost randomization: `card_random_rng` preferred.
- Entropic Brew slot fill: `potion_rng`.

## Tests added/updated
- Updated potion behavior expectations:
  - `tests/test_potion_effects_full.py` (`DistilledChaos`, `EntropicBrew`, fixture correctness for enemy creation).
- Added runtime parity checks:
  - `tests/test_potion_sacred_bark.py` (`DistilledChaos` autoplay semantics, Sacred Bark 6-card path, Smoke Bomb BackAttack restriction).
- Added action API parity checks:
  - `tests/test_agent_api.py` (`LiquidMemories` empty-discard error, `onUsePotion` trigger via Toy Ornithopter for selection flow).

## Remaining potion-domain items
- Normalize/centralize duplicate potion execution between registry and `CombatEngine` (single authoritative path).
- Expand dedicated stream-advance assertions for every RNG-sensitive potion outcome (beyond behavior-level assertions).
