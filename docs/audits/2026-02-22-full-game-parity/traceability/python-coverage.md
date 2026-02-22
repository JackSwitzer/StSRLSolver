# Python Coverage Inventory

This file records what is implemented in this repo and where parity-critical behavior is still incomplete.

## Snapshot (2026-02-22)

| domain | implementing files | status | concrete evidence |
|---|---|---|---|
| potions | `packages/engine/registry/potions.py`, `packages/engine/game.py` | strong | discovery/liquid/stance/smoke paths covered; baseline tests green |
| relics | `packages/engine/state/run.py`, `packages/engine/registry/relics.py`, `packages/engine/game.py` | partial | Orrery + bottled + Dolly selection-surface fixed; alias/ID cleanup remains |
| events | `packages/engine/handlers/event_handler.py`, `packages/engine/game.py` | partial | definitions/handlers/generators are complete (51/51/51), but action-surface card selection is incomplete at runner boundary |
| powers | `packages/engine/content/powers.py`, `packages/engine/registry/powers.py`, `packages/engine/handlers/combat.py` | partial | hook fixes landed, but inventory coverage remains incomplete |
| rewards/shops/rest/map | `packages/engine/handlers/reward_handler.py`, `shop_handler.py`, `rooms.py`, `game.py` | partial | selection interception now covers Orrery + bottled + Dolly; event-path normalization remains |
| orbs | `packages/engine/registry/relics.py` + combat state | open | placeholder TODO logic remains for orb-linked relics |

## Confirmed implementation facts
- Event infrastructure completeness:
  - `ACT1_EVENTS + ACT2_EVENTS + ACT3_EVENTS + SHRINE_EVENTS + SPECIAL_ONE_TIME_EVENTS` = 51
  - `EVENT_CHOICE_GENERATORS` = 51
  - `EVENT_HANDLERS` = 51
- `GameRunner._handle_event_action` currently executes handlers with `card_idx=None`.
- Orb-related relic behavior in `packages/engine/registry/relics.py` still contains placeholder TODO paths.

## Test-surface quality notes
- Suite result is green (`4619 passed, 5 skipped`), but some audit tests still document known bugs instead of enforcing parity behavior.
- Examples:
  - `tests/test_audit_relics_combat.py` contains bug-documentation assertions.
  - `tests/test_audit_damage.py` notes known Torii ordering bug.

## Coverage checklist
- [ ] Replace bug-documentation assertions with parity assertions once fixes land.
- [ ] Add deterministic ID/validation tests for expanded relic-selection action surface (`REL-005`).
- [ ] Add action-surface tests for event card-required choices.
- [ ] Add per-class powers manifest rows with executable parity tests.
