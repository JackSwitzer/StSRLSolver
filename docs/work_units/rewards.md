# Reward Actions & Skip Flow - Work Units

## Scope summary
- Finish reward action processing so RL can reliably claim/skip/proceed.
- Ensure rewards follow Java ordering (gold → potion → cards → relic/keys).
- Provide explicit skip actions (cards/potions/emerald key) and a proceed action.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing/partial behaviors
- Reward action dataclasses exist, but full reward phase behavior is still partial in `GameRunner`.
- Skip actions and proceed logic need consistent, tested handling.
- Boss relic choice resolution should gate proceed until resolved.

## Task batches (unit-sized)
1) **Core reward action resolution**
- Wire `RewardHandler.get_available_actions()` and `execute_action()` into `GameRunner` reward phase paths.
- Ensure gold is auto-claimed but still present as an action for logging.
Acceptance: each action updates run state + reward state correctly and returns a success result.

2) **Skip actions for RL**
- Guarantee `SkipCardAction`, `SkipPotionAction`, `SkipEmeraldKeyAction`, `ProceedFromRewardsAction` are emitted when valid.
- Prevent dead-ends where only invalid actions are offered.
Acceptance: RL can always skip optional rewards and proceed when mandatory rewards are resolved.

3) **Boss relic choices**
- Allow choose or explicit skip of boss relics, then remove other options.
- Gate `ProceedFromRewardsAction` until boss relic is chosen or skipped.
Acceptance: boss relic flow is model-traversable (choose or skip, no double-picks, no proceed before resolution).

4) **Tests**
- Add/update targeted tests for claim/skip/proceed actions.
- Cover boss relic pick/skip and emerald key skip.
Acceptance: tests assert reward state transitions and run state changes for each action.

## Files to touch
- `packages/engine/handlers/reward_handler.py`
- `packages/engine/game.py`
- `packages/engine/state/run.py`
- `tests/test_handlers.py`
- `tests/test_rewards.py` (or add a focused new file)
