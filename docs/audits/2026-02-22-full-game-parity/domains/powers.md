# Powers Domain Audit

## Status
- `POW-001` inventory closure is implemented and test-locked.
- `POW-002` hook/timing closure is partially implemented (high-priority runtime dispatch paths added).
- `POW-003` integration expansion remains open.

## Inventory snapshot (2026-02-23)
- Java power classes (local decompile, excluding `AbstractPower` and deprecated): `149`
- Python canonical power entries (`packages/engine/content/powers.py::POWER_DATA`): `148`
- Canonical mapping coverage (`traceability/power-manifest.json`):
  - `exact`: `134`
  - `alias`: `15`
  - `missing`: `0`
  - note: inventory closure is by canonical mapping, not strict 1:1 entry count (some Java classes intentionally resolve through canonical aliases).

## Manifest + generation
- Manifest JSON: `docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.json`
- Manifest summary: `docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.md`
- Generator: `scripts/generate_power_manifest.py`

## Implemented in this closure pass
- Canonical ID layer:
  - `normalize_power_id(...)`
  - expanded `POWER_ID_ALIASES`
  - Java inventory auto-supplement merge (`packages/engine/content/power_inventory_autogen.py`)
- Runtime hook canonicalization in `execute_power_triggers(...)` for alias/class-name-safe status matching.
- Added runtime dispatch coverage for previously missing high-priority hooks:
  - `atStartOfTurnPostDraw`
  - `onCardDraw`
  - `onApplyPower`
  - `onScry`
  - `onAttackedToChangeDamage`
- Added manifest/hook audit test:
  - `tests/test_audit_power_manifest.py`

## Dispatch audit snapshot (2026-02-23)
- Registry hook types (`@power_trigger`): `25`
- Runtime-dispatched hook types (`execute_power_triggers` callsites across both combat runtimes): `17`
- Registered but not runtime-dispatched hook types: `8`
  - `atDamageFinalReceive`
  - `atDamageGive`
  - `atDamageReceive`
  - `onAttack`
  - `onAttacked`
  - `onDeath`
  - `onManualDiscard`
  - `wasHPLost`

## Open gaps
- [ ] `POW-002` complete hook-order/semantics parity for remaining long-tail powers and the 8 undispatched hook families.
- [ ] `POW-003` broaden integration tests for powers + relics + orbs + card-flow edge cases.

## Java references
- `com/megacrit/cardcrawl/powers/*.java`
- `com/megacrit/cardcrawl/powers/watcher/*.java`

## Python touchpoints
- `packages/engine/content/powers.py`
- `packages/engine/content/power_inventory_autogen.py`
- `packages/engine/registry/powers.py`
- `packages/engine/registry/__init__.py`
- `packages/engine/combat_engine.py`
- `packages/engine/handlers/combat.py`
