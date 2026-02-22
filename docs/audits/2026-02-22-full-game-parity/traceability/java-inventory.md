# Java Inventory

Repository-local Java source root used for this audit:
- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`

This file tracks what Java contains and where parity closure still requires explicit mapping in this repo.

## Snapshot counts (2026-02-22)

| domain | java source scope | java count | python count | delta notes |
|---|---|---:|---:|---|
| relics | `relics/*.java` (excluding `AbstractRelic`, `Test*`) | 181 | 180 (`content/relics.py`) | normalized gap candidates remain |
| events | `events/**/*.java` (excluding framework classes) | 51 | 51 (event definitions in handler) | counts match; alias map still needed |
| powers | `powers/*.java` + `powers/watcher/*.java` (excluding `AbstractPower`) | 149 | 94 (`content/powers.py::POWER_DATA`) | large inventory gap remains |
| potions | `potions/*.java` | unavailable in local decompile snapshot | 42 (`content/potions.py`) | must use other Java refs until local class set is restored |

## Confirmed relic inventory gap candidates (normalized)
- `Abacus`
- `ChampionsBelt`
- `Courier`
- `Duality`
- `GoldPlatedCables`
- `NeowsLament`
- `PhilosopherStone`
- `SnakeRing`
- `SneckoSkull`
- `Toolbox`
- `Waffle`
- `WhiteBeast`
- `WingBoots`

Notes:
- Several are likely alias/canonical-ID mismatches (`Courier` vs `The Courier`, etc.).
- `Toolbox` is a confirmed open gap in Python content coverage.

## Confirmed event alias mismatches (counts match)
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

## Confirmed powers inventory gap
- 149 Java power classes vs 94 Python power entries.
- 69 normalized Java class names have no direct Python power-data mapping yet.
- This is tracked as `POW-001` in the manifest.

## Intake checklist
- [ ] Add explicit alias rows for relic/event naming mismatches.
- [ ] Convert powers inventory gap list into per-class manifest rows (`POW-001-*`).
- [ ] Restore or link local Java potion class inventory for fully auditable potion-class parity.
- [ ] Link each gap row to exact Java class and method where behavior is asserted.
