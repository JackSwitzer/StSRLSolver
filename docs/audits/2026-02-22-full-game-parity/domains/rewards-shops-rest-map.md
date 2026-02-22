# Rewards / Shops / Rest / Map Domain Audit

## Status
- Core systems are implemented and broadly tested.
- Remaining work is normalization of action execution paths so selection-required relic effects are surfaced uniformly.

## Confirmed open gaps
- [ ] `RWD-001` reward action emission should be the canonical source of reward-phase action dicts.
- [ ] `RWD-002` reward/shop execution should route through one selection-aware path.
- [ ] `RWD-003` proceed gating must remain exact for all mandatory/optional reward combinations.
- [ ] `RWD-004` cross-relic modifier parity needs stronger interaction locks.

## Python touchpoints
- `packages/engine/handlers/reward_handler.py`
- `packages/engine/handlers/shop_handler.py`
- `packages/engine/handlers/rooms.py`
- `packages/engine/game.py` (`_handle_reward_action`, `_handle_shop_action`)

## Notes
- Current behavior works for many flows, but relic acquisition in reward/shop can bypass pending-selection interception for choice-based relic effects.
