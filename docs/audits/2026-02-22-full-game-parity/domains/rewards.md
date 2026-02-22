# Rewards Domain Audit

## Status
- Reward generation is centralized in `RewardHandler`.
- Runner reward action emission/execution now routes through `RewardHandler`.
- Proceed gating parity is now locked with explicit unresolved-mandatory tests.
- Remaining reward gap is modifier interaction closure.

## Confirmed open gaps
- [x] `RWD-001` `GameRunner._get_reward_actions` uses canonical `RewardHandler.get_available_actions`.
- [x] `RWD-002` `GameRunner._handle_reward_action` uses canonical `RewardHandler.handle_action` result semantics for claim/skip actions.
- [x] `RWD-003` proceed gating and gold-claim invariants now have explicit parity lock tests.
- [ ] `RWD-004` reward modifier interactions (Question Card / Prayer Wheel / Busted Crown / Black Star / key flow) need closure.

## RWD-001 / RWD-002 implementation result
- `GameRunner._get_reward_actions` now maps from `RewardHandler.get_available_actions(...)` via action adapters.
- `GameRunner._handle_reward_action` now maps to `RewardHandler.handle_action(...)` and propagates invalid-claim failures (`success=False`) through runner action execution.
- `RewardHandler.handle_action` now accepts optional `selection_card_indices` so runner relic-selection flows remain wired for selection-required relic rewards.
- `take_action_dict` now surfaces handler-provided action errors when action execution fails, instead of generic invalid-action text.

## Python touchpoints
- `packages/engine/game.py`
- `packages/engine/handlers/reward_handler.py`
- `tests/test_agent_api.py`
- `tests/test_rewards.py`

## Tests added in this slice
- `tests/test_agent_api.py::TestActionExecution::test_reward_action_surface_matches_reward_handler`
- `tests/test_agent_api.py::TestActionExecution::test_claim_gold_returns_error_when_already_claimed`
- `tests/test_agent_api.py::TestActionExecution::test_proceed_from_rewards_fails_with_unresolved_card_reward`
- `tests/test_agent_api.py::TestActionExecution::test_proceed_from_rewards_fails_with_unresolved_relic_reward`
- Full suite after change: `4656 passed, 5 skipped, 0 failed`.

## Next commit order
1. `RWD-004`
