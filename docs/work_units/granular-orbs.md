# Ultra-Granular Work Units: Defect Orbs

## Java parity references
- `characters/AbstractPlayer.java::channelOrb`, `evokeOrb`, `increaseMaxOrbSlots`, `decreaseMaxOrbSlots`, `applyStartOfTurnOrbs`
- `actions/defect/ImpulseAction.java`
- `actions/defect/TriggerEndOfTurnOrbsAction.java`
- `powers/LoopPower.java`
- `relics/CrackedCore.java`
- `relics/NuclearBattery.java`
- `relics/SymbioticVirus.java`
- `relics/GoldPlatedCables.java`
- `relics/FrozenCore.java`
- `relics/EmotionChip.java`
- `relics/Inserter.java`

## Model-facing actions (no UI)
- [x] Orb behavior should not require new UI; it should be reachable via card actions. (action: play_card{card_index})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Core orb system
- [x] Orb slots: track slot count and max slots; initialize with base slots, keep state and manager synchronized. (action: none{})
- [x] Channel: add orb to next slot or evoke oldest if full. (action: none{})
- [x] Evoke: trigger orb evoke effect and remove orb. (action: none{})
- [x] Passive timing: trigger start/end orb semantics in Java-consistent order for start-of-turn and Impulse/Emotion Chip paths. (action: none{})
- [x] Focus: apply Focus to passive/evoke values with correct sign and ensure manager focus sync after status changes. (action: none{})

## Orb types
- [x] Lightning: passive damage to random enemy; evoke damage to random enemy. (action: none{})
- [x] Frost: passive block; evoke block. (action: none{})
- [x] Dark: passive charges; evoke damage scaling with charge. (action: none{})
- [x] Plasma: passive energy; evoke energy. (action: none{})

## Orb-linked relic closure
- [x] Cracked Core: channels Lightning at battle start. (action: none{})
- [x] Nuclear Battery: channels Plasma at battle start. (action: none{})
- [x] Symbiotic Virus: channels Dark at battle start. (action: none{})
- [x] Gold-Plated Cables (`Cables`): extra first-orb trigger uses real orb runtime (no placeholder checks). (action: none{})
- [x] Frozen Core: channel Frost based on empty-slot condition using real orb manager state. (action: none{})
- [x] Emotion Chip: HP-loss pulse triggers Impulse-equivalent orb start/end behavior next turn. (action: none{})
- [x] Inserter: every 2 turns increases orb slots through runtime slot system (no placeholder path). (action: none{})

## Action integration (model-facing)
- [x] Expose channel/evoke effects via existing card actions (no UI). (action: none{})
- [x] Ensure action list reflects target requirements when orbs need a target. (action: play_card{card_index,target_index})

## RNG stream ownership
- [x] `channel_random_orb` uses owned combat RNG streams (no direct Python `random`). (action: none{})
- [x] Orb random target selection uses owned combat RNG streams (no direct Python `random`). (action: none{})

## Tests
- [x] Add unit tests for channel/evoke/passive timing for each orb type. (action: none{})
- [x] Add Focus modifier tests (positive and negative). (action: none{})
- [x] Convert placeholder relic orb tests to strict assertions (`Cables`, `Frozen Core`, `Emotion Chip`, `Nuclear Battery`, `Symbiotic Virus`, `Inserter`). (action: none{})
- [x] Add deterministic RNG tests for random orb channeling and random lightning targeting. (action: none{})

## ORB-001 acceptance checks
- [x] No placeholder orb branches remain in active orb-linked relic handlers.
- [x] Start-of-turn orb passive coverage is explicit and deterministic.
- [x] All ORB-001 targeted tests and full suite are green.

## Closure note (2026-02-23)
- Targeted verification: `uv run pytest tests/test_orb_runtime_orb001.py tests/test_relic_triggers_combat.py tests/test_defect_cards.py -q` -> `163 passed`.
- Full-suite verification: `uv run pytest tests/ -q` -> `4676 passed, 5 skipped, 0 failed`.
