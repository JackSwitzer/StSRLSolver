# Rewards / Shops / Rest / Map Domain Audit

## Status
- Reward action emission and execution are now canonicalized through `RewardHandler` (`RWD-001` and `RWD-002`).
- Mandatory reward proceed gating is locked (`RWD-003`).
- Indexed secondary relic claim/gating for Black Star is closed (`RWD-004`).
- Reward-domain queue is complete for this campaign slice.

## Queue status
- [x] `RWD-001` canonical reward action emission path
- [x] `RWD-002` canonical reward action execution path
- [x] `RWD-003` proceed gating parity
- [x] `RWD-004` modifier/indexed-relic parity

## What is explicitly covered
- Reward actions are emitted from `RewardHandler.get_available_actions(...)`.
- Reward actions are executed via `RewardHandler.handle_action(...)`.
- `proceed_from_rewards` is blocked until all mandatory rewards are resolved.
- `claim_relic{relic_reward_index}` supports both primary and secondary relic rewards.
- Selection-required relic effects in reward and shop phases expose explicit follow-up `select_cards` actions.

## Remaining cross-domain risks (not in RWD queue)
- Combat reward generation still uses placeholder enemy kill count:
  - `packages/engine/game.py:3285`
- Shop/rest/map interactions remain functionally covered but still depend on open powers/cards/orbs parity work for full Java closure.

## Python touchpoints
- `packages/engine/handlers/reward_handler.py`
- `packages/engine/handlers/shop_handler.py`
- `packages/engine/handlers/rooms.py`
- `packages/engine/game.py`

## Test evidence
- Reward/agent API parity tests in `tests/test_agent_api.py` (including second-relic indexed flow).
- Reward unit/integration coverage in `tests/test_rewards.py` and relic acquisition suites.
