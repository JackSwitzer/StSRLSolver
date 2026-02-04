# Watcher Card Effects Work Unit

## Scope summary
- Watcher-only card effects parity in the engine, focused on the single missing effect key.
- Align card definition and effect registry naming for Inner Peace, preserving canonical IDs used by the engine.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing effects
- Inner Peace (`InnerPeace`): effect key `if_calm_draw_else_calm` is missing (implementation exists as `if_calm_draw_3_else_calm`).

## Suggested tasks (small batches)
- Task 1: Add the `if_calm_draw_else_calm` effect key to the existing implementation and make it the canonical key used by `InnerPeace`. Acceptance: executing `if_calm_draw_else_calm` draws 3 (4 upgraded) in Calm, otherwise enters Calm; no missing-effect lookups.
- Task 2: Align tests to the canonical key and behavior. Acceptance: `test_inner_peace_dual_effect` and effect tests assert draw/stance behavior using the canonical key.

## Files to touch
- `packages/engine/content/cards.py`
- `packages/engine/effects/cards.py`
- `tests/test_cards.py`
- `tests/test_watcher_card_effects.py`
- `tests/test_effects_and_combat.py`
