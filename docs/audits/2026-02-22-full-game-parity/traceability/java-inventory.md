# Java Inventory

Repository-local Java source root used for this audit:
- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`

This file tracks what Java contains and where parity closure still requires explicit mapping in this repo.

## Snapshot counts (2026-02-22)

| domain | java source scope | java count | python count | delta notes |
|---|---|---:|---:|---|
| cards (core) | `cards/{red,green,purple,colorless,curses,status,blue}/*.java` (excluding `deprecated`, `optionCards`, `tempCards`) | 361 | 360 raw-key overlap (`361/361` via lookup alias resolution) | `Discipline` and `Impulse` are implemented; remaining raw-key variance is Java `Gash` resolved by alias to Python `Claw` |
| relics | `relics/*.java` (excluding `AbstractRelic`, `Test*`) | 181 | 181 (`content/relics.py`) | inventory count parity reached; alias forms resolved via canonical lookup |
| events | `events/**/*.java` (excluding framework classes) | 51 | 51 (event definitions in handler) | counts match; Java/display alias normalization locked in handler |
| powers | `powers/*.java` + `powers/watcher/*.java` (excluding `AbstractPower`) | 149 | 94 (`content/powers.py::POWER_DATA`) | large inventory gap remains |
| potions | `potions/*.java` | unavailable in local decompile snapshot | 42 (`content/potions.py`) | must use other Java refs until local class set is restored |

## Relic inventory parity status
- `Toolbox` coverage is now closed.
- Java/class-name alias forms (`Abacus`, `Courier`, `Waffle`, `WhiteBeast`, `WingBoots`, etc.) are resolved by `content/relics.py::resolve_relic_id`.
- Remaining relic work is behavior-level (`REL-007`/`ORB-001`), not inventory count mismatch.

## Event alias mapping coverage (closed)
Java stems not directly equal to Python event IDs include:
- `Cleric` vs `TheCleric`
- `DrugDealer` vs `Augmenter`
- `FountainOfCurseRemoval` vs `FountainOfCleansing`
- `GoldShrine` vs `GoldenShrine`
- `GoldenWing` vs `WingStatue`
- `GoopPuddle` vs `WorldOfGoop`
- `Lab` vs `TheLab`
- `PurificationShrine` vs `Purifier`
- `TombRedMask` vs `TombOfLordRedMask`
- `Bonfire` vs `BonfireElementals`
All of these aliases are now normalized by `handlers/event_handler.py::EVENT_ID_ALIASES` and locked by `tests/test_audit_events.py::TestEventAliasNormalization`.

## Confirmed powers inventory gap
- 149 Java power classes vs 94 Python power entries.
- 69 normalized Java class names have no direct Python power-data mapping yet.
- This is tracked as `POW-001` in the manifest.

## Intake checklist
- [x] Add explicit alias rows for relic/event naming mismatches.
- [x] Convert card inventory deltas (`Discipline`, `Gash`, `Impulse`) into explicit disposition rows (`alias`, `legacy-decompile`, `implement`) in card manifests.
- [ ] Convert powers inventory gap list into per-class manifest rows (`POW-001-*`).
- [ ] Restore or link local Java potion class inventory for fully auditable potion-class parity.
- [ ] Link each gap row to exact Java class and method where behavior is asserted.
