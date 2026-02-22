# Python Coverage Inventory

This file records implemented coverage in this repo and remaining parity-critical gaps.

## Snapshot (2026-02-22)

| domain | implementing files | status | concrete evidence |
|---|---|---|---|
| potions | `packages/engine/registry/potions.py`, `packages/engine/game.py` | strong | high-priority potion parity slice landed (selection flows + RNG stream tests) |
| relics | `packages/engine/state/run.py`, `packages/engine/registry/relics.py`, `packages/engine/game.py`, `packages/engine/content/relics.py`, `packages/engine/state/combat.py` | partial | `REL-003/004/005/006/007` closed; orb-linked relic behavior still open under `ORB-001` |
| events | `packages/engine/handlers/event_handler.py`, `packages/engine/game.py` | strong | definitions/handlers/choice generators are complete (`51/51/51`), plus explicit event selection action flow |
| powers | `packages/engine/content/powers.py`, `packages/engine/registry/powers.py`, `packages/engine/handlers/combat.py` | partial | hook-order fixes landed, but inventory and long-tail behavior remain incomplete (`POW-*`) |
| rewards/shops/rest/map | `packages/engine/handlers/reward_handler.py`, `shop_handler.py`, `rooms.py`, `game.py` | strong | `RWD-001..004` closed (canonical reward emission/execution, proceed gating, indexed secondary relic claims) |
| cards | `packages/engine/effects/cards.py`, `packages/engine/effects/defect_cards.py`, `content/cards.py` | partial | non-Defect manifest published (`domains/cards-manifest-non-defect.md`); full class-by-class parity closure remains in `CRD-*` tracks |
| orbs | `packages/engine/effects/orbs.py`, `packages/engine/registry/relics.py` | open | orb parity remains a blocker; placeholder TODO logic remains in orb-linked relic hooks |

## Confirmed implementation facts
- Event infrastructure completeness:
  - `ACT1_EVENTS + ACT2_EVENTS + ACT3_EVENTS + SHRINE_EVENTS + SPECIAL_ONE_TIME_EVENTS` = `51`
  - `EVENT_CHOICE_GENERATORS` = `51`
  - `EVENT_HANDLERS` = `51`
- Reward handler canonicalization:
  - runner reward actions map to and execute through `RewardHandler`.
- Indexed relic claims in rewards:
  - `claim_relic{relic_reward_index:0|1}` supported and tested.

## Test-surface quality notes
- Current suite baseline:
  - `4663 passed, 5 skipped, 0 failed`
- Executed skips in normal run are artifact-dependent replay skips from `tests/test_parity.py`.
- Additional contingency `pytest.skip(...)` callsites remain in some tests and should be hardened for strict CI.

## Coverage checklist
- [ ] Close `POW-001` inventory mapping and add per-class parity assertions.
- [ ] Close `ORB-001` and remove orb placeholder branches in relic/power hooks.
- [ ] Complete `CRD-*` long-tail behavior closure.
- [ ] Remove gameplay-critical direct Python `random` usage in engine logic and enforce owned RNG streams (23 callsites remain after `CONS-001`).
