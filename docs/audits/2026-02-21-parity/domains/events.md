# Events Audit

## Summary
Event parity for the previously identified high-priority gaps is in good shape. Handler registration coverage and key logic fixes are present and associated audit tests pass.

## Status vs composer 1.5 gap list
- Register `_get_*_choices` in `EVENT_CHOICE_GENERATORS`: `DONE`
- Add handlers for `GremlinMatchGame`, `GremlinWheelGame`, `NoteForYourself`: `DONE`
- Fix `DeadAdventurer`, `Falling`, `KnowingSkull` logic: `DONE` (current tests green)

## Code areas
- `packages/engine/handlers/event_handler.py`
- `tests/test_audit_events.py`
- `docs/work_units/granular-events.md`

## Residual risks
- Some event flows are still auto-resolved where Java expects richer card/choice interaction.
- RL action-surface parity is still uneven for choice-heavy rooms.

## Next checks
- Re-run Java-to-Python diff specifically for events after any future handler edits.
- Add action-level parity tests (not only handler/choice existence checks).
