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
- Completed runtime dispatch coverage for all registered power hooks (`25/25`).
- Added alias/lifecycle registry closures:
  - `DrawCardNextTurn` post-draw hook alias.
  - `IntangiblePlayer` final-damage + end-of-round handling.
  - `WaveOfTheHandPower` end-of-round expiration.
  - `Thorns` `onAttacked` handling (with attacker block interaction).
- Added long-tail hook/runtime closures (`POW-003B`):
  - `Flight`: `atStartOfTurn`, `atDamageFinalReceive`, `onAttacked`.
  - `Malleable`: `onAttacked`, owner-specific reset (`atEndOfTurn` enemy, `atEndOfRound` player).
  - `Invincible`: `onAttackedToChangeDamage` cap tracking + `atStartOfTurn` reset.
  - `Pen Nib`: `atDamageGive` (priority-aligned) + `onUseCard` removal on attacks.
  - `Equilibrium`: retain-hand state integration + `atEndOfRound` decrement/removal.
  - `Echo Form`: `atStartOfTurn` per-turn reset + `onUseCard` replay marker emission.
- Added manifest/hook audit test:
  - `tests/test_audit_power_manifest.py`

## Dispatch audit snapshot (2026-02-23)
- Registry hook types (`@power_trigger`): `25`
- Runtime-dispatched hook types (`execute_power_triggers` callsites across both combat runtimes + effect runtime): `25`
- Registered but not runtime-dispatched hook types: `0`

## Open gaps
- [ ] `POW-002` complete hook-order/semantics parity for remaining long-tail powers (dispatch coverage is now complete; behavior/order parity remains).
- [ ] `POW-003` broaden integration tests for powers + relics + orbs + card-flow edge cases.
- [ ] `POW-003C` close remaining card-play queue parity for replay-style powers (`Echo Form`, `Burst`, `Double Tap`, `Amplify`) under integration tests.

## Remaining registry behavior gaps (from manifest diff)
- Classes with at least one Java-overridden hook not represented in current registry handlers: `61`
- Largest remaining hook families by count:
  - `atEndOfTurn`: `16`
  - `atEndOfRound`: `13`
  - `atStartOfTurn`: `11`
  - `onUseCard`: `9`
  - `onAttacked`: `6`
- Additional remaining hook families:
  - `onEnergyRecharge`: `2`
  - `onChangeStance`: `2`
  - `onAfterUseCard`: `2`
  - `onDeath`: `2`
  - `onCardDraw`: `1`
  - `atDamageGive`: `1`
  - `atDamageFinalReceive`: `1`
  - `onApplyPower`: `1`
- High-priority classes with multi-hook deltas:
  - `AmplifyPower`: `atEndOfTurn`, `onUseCard`
  - `AttackBurnPower`: `atEndOfRound`, `onUseCard`
  - `DoubleDamagePower`: `atDamageGive`, `atEndOfRound`
  - `ReboundPower`: `atEndOfTurn`, `onAfterUseCard`

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
