# Powers Audit

## Summary
The specific high-priority power gaps from the prior audit are now aligned in both registry and combat execution paths.

## Status vs composer 1.5 gap list
- Add `onAfterUseCard` and `onAfterCardPlayed` hooks + trigger ordering: `DONE`
- Move Thousand Cuts to `onAfterCardPlayed`: `DONE`
- Bias timing to `atStartOfTurn`: `DONE`

## Code areas
- `packages/engine/registry/powers.py`
- `packages/engine/combat_engine.py`
- `packages/engine/handlers/combat.py`
- `tests/test_power_edge_cases.py`

## What changed in this pass
- Added missing post-card hooks in `handlers/combat.py` (was only firing `onUseCard`).
- Updated Thousand Cuts failing test to assert `onAfterCardPlayed` timing.

## Residual risks
- Broader power parity list in `docs/work_units/granular-powers.md` still contains many unchecked items outside this focused pass.
