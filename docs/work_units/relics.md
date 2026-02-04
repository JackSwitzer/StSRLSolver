# Relic Triggers & Watcher-Critical Relics - Work Units

## Scope summary
- Implement missing relic triggers and on-acquisition effects needed for Watcher RL.
- Focus on relics that impact energy, rewards, or action availability.
- Defer Defect orb-related relics unless explicitly needed for Prismatic Shard.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Watcher-critical gaps (priority)
- **Violet Lotus**: gain +1 energy when exiting Calm.
- **Singing Bowl**: card reward skip → +2 max HP.
- **Question Card**: +1 card reward choice.
- **Prayer Wheel**: extra card reward.
- **Busted Crown**: -2 card choices in rewards.
- **White Beast Statue**: guaranteed potion drop.
- **Snecko Eye**: randomized draw costs each turn.
- **Ice Cream**: unused energy carries over.

## Task batches (unit-sized)
1) **Watcher energy/reward relics**
- Implement Violet Lotus energy on Calm exit.
- Implement reward-modifying relics (Singing Bowl, Question Card, Prayer Wheel, Busted Crown).
Acceptance: reward generation and card choice counts reflect relic modifiers; Violet Lotus triggers on stance exit.

2) **Potion/reward relics**
- Implement White Beast Statue, Toy Ornithopter (if not already), Sozu interactions.
Acceptance: potion drop logic and potion-use healing mirror Java.

3) **Combat trigger relics**
- Implement Snecko Eye, Ice Cream, Incense Burner, Pen Nib, etc.
Acceptance: combat hooks fire at correct timings; effects persist across turns as expected.

4) **On-acquire/on-remove relics**
- Implement transform/swap relics: Astrolabe, Empty Cage, Pandora’s Box, Calling Bell, Tiny House.
Acceptance: acquisition effects modify deck/relics and require card selection when appropriate.

5) **Tests**
- Add targeted tests by relic category (rest-site, reward-modifying, combat triggers).
Acceptance: tests validate timing and state changes for each relic.

## Files to touch
- `packages/engine/registry/relics.py`
- `packages/engine/registry/relics_passive.py`
- `packages/engine/handlers/reward_handler.py`
- `packages/engine/handlers/combat.py`
- `packages/engine/state/combat.py`
- `tests/test_relic_*`
