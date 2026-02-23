# Orbs / System Support Domain Audit

## Status
- `ORB-001` foundational closure is implemented and test-locked.
- Core orb runtime is now wired into combat turn-start flow (`packages/engine/combat_engine.py`, `packages/engine/handlers/combat.py`).
- Orb-linked relic hooks now execute against real orb manager runtime state (no placeholder `state.orbs` branches in active handlers).

## Confirmed gaps
- [x] Orb slot/channel/evoke/passive timing infrastructure parity in live combat turn flow.
- [ ] Focus and Lock-On interactions where applicable.
- [x] Remove parity-critical placeholder behavior from relic/power hooks (`Cables`, `Frozen Core`, `Emotion Chip`, `Inserter`, battle-start channel relics).
- [x] Replace residual direct Python `random` usage in orb runtime with owned RNG streams as part of `ORB-001`.
- [x] Ensure orb-linked relic handlers operate on the actual orb manager state instead of missing `state.orbs` placeholders.

## Feature IDs
- `ORB-001` foundational orb parity implementation
- `POW-003` integration coverage consuming orb system behavior

## Java references locked for ORB-001
- `characters/AbstractPlayer.java::channelOrb`
- `characters/AbstractPlayer.java::evokeOrb`
- `characters/AbstractPlayer.java::increaseMaxOrbSlots`
- `characters/AbstractPlayer.java::decreaseMaxOrbSlots`
- `characters/AbstractPlayer.java::applyStartOfTurnOrbs`
- `actions/defect/ImpulseAction.java`
- `powers/LoopPower.java`
- `relics/CrackedCore.java`
- `relics/NuclearBattery.java`
- `relics/SymbioticVirus.java`
- `relics/GoldPlatedCables.java`
- `relics/FrozenCore.java`
- `relics/EmotionChip.java`
- `relics/Inserter.java`

## ORB-001 acceptance
- [x] Start-of-turn orb passives are executed in deterministic combat runtime paths.
- [x] Orb slot changes from cards/potions/relics stay synchronized with orb manager state.
- [x] Orb-linked relic behavior is test-locked with strict assertions (not placeholder comments).
- [x] Targeted orb/relic suites pass and full suite remains green.

## Verification snapshot
- Targeted: `uv run pytest tests/test_orb_runtime_orb001.py tests/test_relic_triggers_combat.py tests/test_defect_cards.py -q` -> `163 passed`.
- Full suite: `uv run pytest tests/ -q` -> `4676 passed, 5 skipped, 0 failed`.
