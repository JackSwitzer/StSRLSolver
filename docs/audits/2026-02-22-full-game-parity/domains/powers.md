# Powers Domain Audit

## Status
- Critical hook-order path fixes are landed in combat handling.
- Inventory-level parity is still the largest remaining behavior gap.

## Confirmed inventory snapshot
- Java power classes (local decompile, excluding `AbstractPower`): `149`
- Python power entries (`packages/engine/content/powers.py::POWER_DATA`): `94`
- Normalized unmatched Java candidates: `69`

## Confirmed implemented fixes
- `onAfterUseCard` / `onAfterCardPlayed` registration and trigger path updates.
- Hook ordering improvements in combat flow for parity-critical cases.

## Confirmed open gaps
- [ ] `POW-001` map unmatched Java power classes to explicit status (`exact`, `missing`, `alias-only`, `intentional defer`).
- [ ] `POW-002` close remaining hook/timing behavior mismatches class-by-class.
- [ ] `POW-003` add interaction tests with relics/orbs and turn-order semantics.

## Dependency note
- Some powers cannot be fully parity-closed until `ORB-001` is implemented.

## Java references
- `com/megacrit/cardcrawl/powers/*.java`
- `com/megacrit/cardcrawl/powers/watcher/*.java`

## Python touchpoints
- `packages/engine/content/powers.py`
- `packages/engine/registry/powers.py`
- `packages/engine/handlers/combat.py`
