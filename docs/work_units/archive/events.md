# ARCHIVED (use granular work units)

This legacy work unit is archived. Use `docs/work_units/granular-events.md`.

# Event handler completeness + ID normalization

## Scope summary
- Bring `packages/engine/handlers/event_handler.py` to full event coverage (handlers + choice generators for every event in ACT/SHRINE/SPECIAL pools).
- Normalize event IDs between `packages/engine/content/events.py` and the new handler (alias map now present; extend coverage/tests and align pools).
- Align pool membership (act vs shrine vs special) so selection and logging are consistent.
- Model-facing actions only (no UI); see `docs/work_units/granular-actions.md`.

## Missing handlers/choice generators and ID mismatches
- Handlers missing: `GremlinMatchGame`, `GremlinWheelGame` (definitions exist, no handlers/registry entries), `NoteForYourself` (present in `content/events.py`, missing from handler definitions/registry).
- Choice generators missing (defaulting to `[Leave]`): Act 1 `DeadAdventurer`, `Mushrooms`, `ShiningLight`, `Sssserpent`, `WingStatue`; Act 2 `Addict`, `Augmenter`, `BackToBasics`, `Beggar`, `CursedTome`, `ForgottenAltar`, `Ghosts`, `Nest`, `Vampires`; Act 3 `Falling`, `MoaiHead`, `MysteriousSphere`, `SecretPortal`, `SensoryStone`, `TombOfLordRedMask`, `WindingHalls`; Shrines `GremlinMatchGame`, `GremlinWheelGame`; Special `AccursedBlacksmith`, `BonfireElementals`, `Designer`, `FaceTrader`, `FountainOfCleansing`, `TheJoust`, `TheLab`, `Nloth`, `WeMeetAgain`, `WomanInBlue` (plus `NoteForYourself` once added).
- ID mismatches / pool mismatches: alias mapping now handles most formatting, but pool classification mismatches remain (`Knowing Skull` Act 2 vs special one-time, `SecretPortal` Act 3 vs special one-time, `NoteForYourself` content-only) and coverage gaps should be verified.

## Suggested task batches with acceptance criteria
1. Canonical IDs + alias map (in progress). Alias map exists in `event_handler.py`; extend coverage and add tests to ensure every event ID in `content/events.py` resolves to a canonical ID with no duplicates in pools.
2. Add missing event definitions/handlers. Add `NoteForYourself` to handler pools + implement handler/registry entry; implement handlers for `GremlinMatchGame` and `GremlinWheelGame`. Acceptance: `EVENT_HANDLERS` covers every `EventDefinition`; no missing handler warnings.
3. Choice generators (Act 1 + Act 2). Implement `_get_*_choices` for all missing Act 1/2 events with proper requirements. Acceptance: `EVENT_CHOICE_GENERATORS` includes all Act 1/2 events; no fallback to default `[Leave]` for those events.
4. Choice generators (Act 3 + Shrines + Special). Implement choice generators for remaining Act 3, shrine, and special events, including multi-phase options. Acceptance: every event in ACT3/SHRINE/SPECIAL has an explicit choice generator.
5. Pool consistency pass. Align act/shrine/special pools between `content/events.py` and handler definitions. Acceptance: `KnowingSkull`/`SecretPortal` placement consistent across both systems; pool counts match expected totals.

## Files to touch
- `packages/engine/handlers/event_handler.py`
- `packages/engine/content/events.py`
- `packages/engine/handlers/rooms.py` (if aliasing event selection)
- `packages/engine/game.py` (if logging/ID normalization changes)
- `tests/test_events.py` (only if ID normalization requires updates)
