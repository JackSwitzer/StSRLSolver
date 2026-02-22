# Python Coverage Inventory

Track implementing files and current parity status by domain.

| domain | key python files | status | notes |
|---|---|---|---|
| potions | `packages/engine/registry/potions.py`, `packages/engine/combat_engine.py`, `packages/engine/game.py` | strong | maintain via RNG + action-surface regression tests |
| relics | `packages/engine/state/run.py`, `packages/engine/registry/relics.py`, `packages/engine/game.py` | partial | selection auto-picks and alias gaps remain |
| events | `packages/engine/handlers/event_handler.py`, `packages/engine/game.py` | partial | card-selection action plumbing incomplete |
| powers | `packages/engine/registry/powers.py`, `packages/engine/handlers/combat.py` | partial | broad residual list beyond focused fixes |
| cards | `packages/engine/content/cards.py`, `packages/engine/effects/*.py` | open | class backlog remains |
| rewards/shops/rest/map | `packages/engine/handlers/reward_handler.py`, `shop_handler.py`, `rooms.py`, `game.py` | partial | single-path action normalization pending |
| orbs | multiple | open | parity-critical TODO placeholders remain |

## Coverage checklist
- [ ] Confirm one-to-one mapping from manifest rows to Python symbols.
- [ ] Mark each row with `exact|approximate|missing|action-surface-missing`.
- [ ] Record RNG stream usage and tests for each parity-critical row.
