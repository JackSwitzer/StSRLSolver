# Ultra-Granular Work Units: Defect Orbs

## Model-facing actions (no UI)
- [ ] Orb behavior should not require new UI; it should be reachable via card actions. (action: play_card{card_index})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Core orb system
- [ ] Orb slots: track slot count and max slots; initialize with base slots. (action: none{})
- [ ] Channel: add orb to next slot or evoke oldest if full. (action: none{})
- [ ] Evoke: trigger orb evoke effect and remove orb. (action: none{})
- [ ] Passive: trigger orb passive at end of turn (or per Java timing). (action: none{})
- [ ] Focus: apply Focus to passive/evoke values with correct sign. (action: none{})

## Orb types
- [ ] Lightning: passive damage to random enemy; evoke damage to random enemy. (action: none{})
- [ ] Frost: passive block; evoke block. (action: none{})
- [ ] Dark: passive charges; evoke damage scaling with charge. (action: none{})
- [ ] Plasma: passive energy; evoke energy. (action: none{})

## Action integration (model-facing)
- [ ] Expose channel/evoke effects via existing card actions (no UI). (action: none{})
- [ ] Ensure action list reflects target requirements when orbs need a target. (action: play_card{card_index,target_index})

## Tests
- [ ] Add unit tests for channel/evoke/passive timing for each orb type. (action: none{})
- [ ] Add Focus modifier tests (positive and negative). (action: none{})
