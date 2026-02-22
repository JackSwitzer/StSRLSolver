# Powers Domain Audit

## Status
- Critical hook-order fixes landed (including `onAfterUseCard` / `onAfterCardPlayed` path updates).
- Inventory-level parity is still incomplete and must be closed systematically.

## Confirmed inventory snapshot
- Java classes (local decompile): 149 (excluding `AbstractPower`)
- Python power entries: 94 (`packages/engine/content/powers.py::POWER_DATA`)
- Normalized unmatched Java candidates: 69

## Confirmed open gaps
- [ ] `POW-001` map all unmatched Java classes to Python status (`exact|missing|alias-only`).
- [ ] `POW-002` implement remaining missing hook/timing behaviors for parity-critical classes.
- [ ] `POW-003` add integration tests for power interactions with relics/orbs/combat order.

## Java references
- `com/megacrit/cardcrawl/powers/*.java`
- `com/megacrit/cardcrawl/powers/watcher/*.java`

## Python touchpoints
- `packages/engine/content/powers.py`
- `packages/engine/registry/powers.py`
- `packages/engine/handlers/combat.py`

## Dependency note
- Some missing power behavior depends on orb infrastructure tracked under `ORB-001`.
