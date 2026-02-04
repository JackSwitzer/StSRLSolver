# Event handler completeness + ID normalization

## Scope summary
- Bring `packages/engine/handlers/event_handler.py` to full event coverage (handlers + choice generators for every event in ACT/SHRINE/SPECIAL pools).
- Normalize event IDs between `packages/engine/content/events.py` and the new handler (canonical IDs + aliases for legacy/display names).
- Align pool membership (act vs shrine vs special) so selection and logging are consistent.

## Missing handlers/choice generators and ID mismatches
- Handlers missing: `GremlinMatchGame`, `GremlinWheelGame` (definitions exist, no handlers/registry entries), `NoteForYourself` (present in `content/events.py`, missing from handler definitions/registry).
- Choice generators missing (defaulting to `[Leave]`): Act 1 `DeadAdventurer`, `Mushrooms`, `ShiningLight`, `Sssserpent`, `WingStatue`; Act 2 `Addict`, `Augmenter`, `BackToBasics`, `Beggar`, `CursedTome`, `ForgottenAltar`, `Ghosts`, `Nest`, `Vampires`; Act 3 `Falling`, `MoaiHead`, `MysteriousSphere`, `SecretPortal`, `SensoryStone`, `TombOfLordRedMask`, `WindingHalls`; Shrines `GremlinMatchGame`, `GremlinWheelGame`; Special `AccursedBlacksmith`, `BonfireElementals`, `Designer`, `FaceTrader`, `FountainOfCleansing`, `TheJoust`, `TheLab`, `Nloth`, `WeMeetAgain`, `WomanInBlue` (plus `NoteForYourself` once added).
- ID mismatches / pool mismatches: systematic formatting mismatch (content uses display IDs with spaces; handler uses camelcase, e.g., `Big Fish`->`BigFish`, `The Cleric`->`TheCleric`, `World of Goop`->`WorldOfGoop`, `The Moai Head`->`MoaiHead`, `Tomb of Lord Red Mask`->`TombOfLordRedMask`, `Upgrade Shrine`->`UpgradeShrine`, `The Woman in Blue`->`WomanInBlue`, `The Joust`->`TheJoust`, `Lab`->`TheLab`, `N'loth`->`Nloth`); name divergences (`Golden Wing` vs `WingStatue`, `Liars Game` vs `Sssserpent`, `Drug Dealer` vs `Augmenter`, `Match and Keep!` vs `GremlinMatchGame`, `Wheel of Change` vs `GremlinWheelGame`, `Transmorgrifier` vs `Transmogrifier`); pool classification mismatches (`Knowing Skull` Act 2 vs special one-time, `SecretPortal` Act 3 vs special one-time, `NoteForYourself` content-only).

## Suggested task batches with acceptance criteria
1. Canonical IDs + alias map. Define canonical event IDs (likely Java IDs) and an alias map for content/display IDs; update event selection/logging to resolve aliases consistently across handler and content. Acceptance: every event ID in `content/events.py` resolves to a canonical ID; no duplicate IDs in pools.
2. Add missing event definitions/handlers. Add `NoteForYourself` to handler pools + implement handler/registry entry; implement handlers for `GremlinMatchGame` and `GremlinWheelGame`. Acceptance: `EVENT_HANDLERS` covers every `EventDefinition`; no missing handler warnings.
3. Choice generators (Act 1 + Act 2). Implement `_get_*_choices` for all missing Act 1/2 events with proper requirements. Acceptance: `EVENT_CHOICE_GENERATORS` includes all Act 1/2 events; no fallback to default `[Leave]` for those events.
4. Choice generators (Act 3 + Shrines + Special). Implement choice generators for remaining Act 3, shrine, and special events, including multi-phase options. Acceptance: every event in ACT3/SHRINE/SPECIAL has an explicit choice generator.
5. Pool consistency pass. Align act/shrine/special pools between `content/events.py` and handler definitions. Acceptance: `KnowingSkull`/`SecretPortal` placement consistent across both systems; pool counts match expected totals.

## Files to touch
- `packages/engine/handlers/event_handler.py`
- `packages/engine/content/events.py`
- `packages/engine/handlers/rooms.py` (if aliasing legacy event selection)
- `packages/engine/game.py` (if logging/ID normalization changes)
- `tests/test_events.py` (only if ID normalization requires updates)
