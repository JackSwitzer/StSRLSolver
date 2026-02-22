# Ultra-Granular Work Units: Powers

## Model-facing actions (no UI)
- [ ] Power triggers should not require UI; any selection must emit explicit action options. (action: none{})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## System / Shared
- [ ] Add `onAfterUseCard` + `onAfterCardPlayed` hooks in registry + combat flow (Java ordering: onPlayCard -> card effects -> onUseCard -> onAfterUseCard -> onAfterCardPlayed). (action: none{})
- [ ] Slow - atDamageReceive, onAfterUseCard, atEndOfRound reset. (action: none{})
- [ ] Lock-On - orb damage modifier, atEndOfRound decrement. (action: none{})
- [ ] NoBlockPower - modifyBlockLast hook. (action: none{})
- [ ] Entangled - canPlayCard + remove at end of turn. (action: none{})
- [ ] No Draw - skip draw + remove at end of turn. (action: none{})
- [ ] Draw Reduction - onInitialApplication/onRemove + atEndOfRound decrement. (action: none{})
- [ ] Draw - onInitialApplication/onRemove. (action: none{})
- [ ] Artifact - consume on debuff block (specific trigger). (action: none{})
- [ ] IntangiblePlayer - atEndOfRound decrement. (action: none{})
- [ ] Double Damage - atDamageGive + atEndOfRound decrement. (action: none{})
- [ ] Pen Nib - atDamageGive + onUseCard remove. (action: none{})
- [ ] Blur - atEndOfRound decrement. (action: none{})
- [ ] Barricade - block retention hook. (action: none{})
- [ ] Repair - onVictory heal. (action: none{})
- [ ] Thorns - ensure onAttacked hook (not onAttack). (action: none{})

## Ironclad
- [ ] Corruption - onCardDraw skills cost 0, onUseCard skills exhaust. (action: none{})

## Silent
- [ ] Accuracy - passive Shiv base damage boost. (action: none{})
- [ ] Thousand Cuts - move to onAfterCardPlayed hook. (action: none{})

## Defect
- [ ] Bias - atStartOfTurn lose Focus timing. (action: none{})
- [ ] Storm - onUseCard Power -> channel Lightning. (action: none{})
- [ ] Static Discharge - onAttacked channel Lightning. (action: none{})
- [ ] Electro - passive Lightning hits all. (action: none{})
- [ ] Focus / Lock-On - orb modifiers (ties to system items). (action: none{})

## Watcher
- [ ] Mantra - onStack enter Divinity at 10. (action: none{})
- [ ] BlockReturnPower - onAttacked gain block when attacking marked enemy. (action: none{})
- [ ] EstablishmentPower - atEndOfTurn reduce retained card costs. (action: none{})
- [ ] WaveOfTheHandPower - atEndOfRound remove. (action: none{})
- [ ] FreeAttackPower - onUseCard decrement on ATTACK. (action: none{})
- [ ] CannotChangeStancePower - block stance change + remove at end of turn. (action: none{})

## Boss / Enemy
- [ ] Beat of Death - onAfterUseCard damage. (action: none{})
- [ ] Curiosity - onUseCard Power -> gain Strength. (action: none{})
- [ ] Time Warp - onAfterUseCard counter + end turn + gain Strength. (action: none{})
- [ ] Invincible - onAttackedToChangeDamage + atStartOfTurn reset. (action: none{})
- [ ] Angry - onAttacked gain Strength. (action: none{})
- [ ] GrowthPower / Ritual - atEndOfRound with skip-first logic. (action: none{})
- [ ] Fading - duringTurn decrement / suicide. (action: none{})
- [ ] Thievery - onAttack steal gold. (action: none{})
- [ ] Mode Shift / Split / Life Link - passive system hooks. (action: none{})

## Colorless / Special
- [ ] Panache - atStartOfTurn reset counter. (action: none{})
- [ ] Double Tap - onUseCard ATTACK replay + decrement. (action: none{})
- [ ] Burst - onUseCard SKILL replay + decrement. (action: none{})
- [ ] Echo Form - atStartOfTurn mark first card doubles. (action: none{})
- [ ] Retain Cards - atEndOfTurn choose retain count. (action: select_cards{pile:hand,card_indices})
- [ ] Equilibrium - retain hand + atEndOfRound decrement. (action: select_cards{pile:hand,card_indices})

## Tests
- [ ] Add focused tests for each hook (registry integration + edge cases). (action: none{})
