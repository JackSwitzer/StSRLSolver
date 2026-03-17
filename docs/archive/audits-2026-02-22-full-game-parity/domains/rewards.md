# Rewards Domain Audit

## Status
- Reward generation is centralized in `RewardHandler`.
- Runner reward action emission/execution routes through `RewardHandler`.
- Proceed gating parity is locked with explicit unresolved-mandatory tests.
- Black Star second-relic indexing/gating parity is now explicitly covered.
- Reward-domain queue (`RWD-001`..`RWD-004`) is complete.

## Queue status
- [x] `RWD-001` `GameRunner._get_reward_actions` uses canonical `RewardHandler.get_available_actions`.
- [x] `RWD-002` `GameRunner._handle_reward_action` uses canonical `RewardHandler.handle_action` result semantics for claim/skip actions.
- [x] `RWD-003` proceed gating and gold-claim invariants have explicit parity lock tests.
- [x] `RWD-004` indexed relic-claim support for Black Star second relic rewards.

## RWD-004 implementation result
- `ClaimRelicAction` now carries `reward_index` and reward action emission includes both relic entries when present:
  - `claim_relic{relic_reward_index:0}` for `relic`
  - `claim_relic{relic_reward_index:1}` for `second_relic`
- Relic claim execution now resolves by reward index and returns `relic_reward_index` in result metadata.
- Mandatory proceed gating now treats `second_relic` as mandatory (both handler and runner checks).
- Selection-required relic handling for `claim_relic` now resolves the selected reward index (primary or second) before building `select_cards` follow-up actions.
- `get_available_action_dicts()` now annotates `claim_relic` with `requires=["card_indices"]` per indexed relic reward when that relic requires card selection.

## Tests added in this slice
- `tests/test_agent_api.py::TestActionExecution::test_reward_actions_include_second_relic_claim_index`
- `tests/test_agent_api.py::TestActionExecution::test_claim_second_relic_by_index`
- `tests/test_agent_api.py::TestActionExecution::test_proceed_blocked_until_second_relic_claimed`
- Full suite after change: `4659 passed, 5 skipped, 0 failed`.

## Java references
- `com.megacrit.cardcrawl.relics.BlackStar`
- `com.megacrit.cardcrawl.rewards.RewardItem`
- `com.megacrit.cardcrawl.screens.CombatRewardScreen`

## RNG notes
- No new RNG stream introduced for indexed relic-claim handling.
- Existing relic pickup side-effect routing remains unchanged (`misc_rng`, `card_rng`, `relic_rng`, `potion_rng`).

## Python touchpoints
- `packages/engine/handlers/reward_handler.py`
- `packages/engine/game.py`
- `tests/test_agent_api.py`
