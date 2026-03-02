# Ultra-Granular Work Units: Powers

## Model-facing actions (no UI)
- [ ] Power triggers should not require UI; any selection must emit explicit action options. (action: none{})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## 2026-02-23 POW closure status
- [x] `POW-001` deterministic Java-vs-Python per-class manifest generated (`docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.json`).
- [x] `POW-001` inventory mapping closed: 149 Java classes map to Python (`exact=134`, `alias=15`, `missing=0`).
- [x] Canonicalization layer added (`normalize_power_id`, expanded alias map, inventory auto-merge from `power_inventory_autogen.py`).
- [x] Runtime dispatch coverage added for high-priority gaps: `atStartOfTurnPostDraw`, `onCardDraw`, `onApplyPower`, `onScry`, `onAttackedToChangeDamage`.
- [x] New audit tests added: `tests/test_audit_power_manifest.py`.
- [ ] `POW-002` remaining hook-order/behavior exactness for long-tail powers.
- [ ] `POW-003` cross-system integration lock (powers + relics + orbs + cards) beyond current coverage.
- [x] `POW-002B` lifecycle dispatch slice: wire `wasHPLost` and `onDeath` in both runtimes and lock with dispatch audit.
- [x] `POW-002C` attack-path dispatch slice: wire `onAttack` and `onAttacked` in both runtimes and include existing `onManualDiscard` runtime path in dispatch audit.
- [x] `POW-002D` damage-hook dispatch slice: wire `atDamageGive`, `atDamageReceive`, `atDamageFinalReceive` in both runtimes and lock dispatch parity.
- [x] `POW-003A` alias/lifecycle slice: close `IntangiblePlayer`, `DrawCardNextTurn`, `WaveOfTheHandPower`, and `ThornsPower` hook parity in registry behavior.
- [x] `POW-003B` defensive/offensive long-tail slice: close registry/runtime hook coverage for `Flight`, `Malleable`, `Invincible`, `Pen Nib`, `Equilibrium`, and `Echo Form` (Java refs: `FlightPower`, `MalleablePower`, `InvinciblePower`, `PenNibPower`, `EquilibriumPower`, `EchoPower`).

## System / Shared
- [x] Add `onAfterUseCard` + `onAfterCardPlayed` hooks in registry + combat flow (Java ordering: onPlayCard -> card effects -> onUseCard -> onAfterUseCard -> onAfterCardPlayed). (action: none{})
- [x] Slow - atDamageReceive, onAfterUseCard, atEndOfRound reset. (action: none{})
- [x] Lock-On - orb damage modifier, atEndOfRound decrement. (action: none{})
- [x] NoBlockPower - modifyBlockLast hook + atEndOfRound decrement + modifyBlockLast dispatch in combat engine. (action: none{})
- [x] Entangled - canPlayCard + remove at end of turn. (action: none{})
- [x] No Draw - skip draw + remove at end of turn. (action: none{})
- [ ] Draw Reduction - onInitialApplication/onRemove + atEndOfRound decrement. (action: none{})
- [ ] Draw - onInitialApplication/onRemove. (action: none{})
- [x] Artifact - consume on debuff block (specific trigger). (action: none{})
- [x] IntangiblePlayer - atEndOfRound decrement. (action: none{})
- [x] Double Damage - atDamageGive + atEndOfRound decrement. (action: none{})
- [x] Pen Nib - atDamageGive + onUseCard remove. (action: none{})
- [x] Blur - atEndOfRound decrement. (action: none{})
- [x] Barricade - block retention hook. (action: none{})
- [x] Repair - onVictory heal + onVictory power dispatch in combat engine. (action: none{})
- [x] Thorns - ensure onAttacked hook (not onAttack). (action: none{})

## Ironclad
- [x] Corruption - onCardDraw skills cost 0, onUseCard skills exhaust. (action: none{})

## Silent
- [x] Accuracy - passive Shiv base damage boost. (action: none{})
- [x] Thousand Cuts - move to onAfterCardPlayed hook. (action: none{})

## Defect
- [x] Bias - atStartOfTurn lose Focus timing. (action: none{})
- [x] Storm - onUseCard Power -> channel Lightning. (action: none{})
- [x] Static Discharge - onAttacked channel Lightning. (action: none{})
- [ ] Electro - passive Lightning hits all. (action: none{})
- [ ] Focus / Lock-On - orb modifiers (ties to system items). (action: none{})

## Watcher
- [x] Mantra - onStack enter Divinity at 10 (handled inline in Devotion + _add_mantra). (action: none{})
- [x] BlockReturnPower - onAttacked gain block when attacking marked enemy. (action: none{})
- [x] EstablishmentPower - atEndOfTurn reduce retained card costs (placeholder; needs per-card cost tracking). (action: none{})
- [x] WaveOfTheHandPower - atEndOfRound remove. (action: none{})
- [x] FreeAttackPower - onUseCard decrement on ATTACK. (action: none{})
- [x] CannotChangeStancePower - block stance change + remove at end of turn. (action: none{})

## Boss / Enemy
- [x] Beat of Death - onAfterUseCard damage. (action: none{})
- [x] Curiosity - onUseCard Power -> gain Strength. (action: none{})
- [x] Time Warp - onAfterUseCard counter + end turn + gain Strength. (action: none{})
- [x] Invincible - onAttackedToChangeDamage + atStartOfTurn reset. (action: none{})
- [x] Angry - onAttacked gain Strength. (action: none{})
- [x] GrowthPower / Ritual - atEndOfRound with skip-first logic. (action: none{})
- [x] Fading - duringTurn decrement / suicide. (action: none{})
- [x] Thievery - onAttack steal gold. (action: none{})
- [ ] Mode Shift / Split / Life Link - passive system hooks. (action: none{})

## Colorless / Special
- [x] Panache - atStartOfTurn reset counter. (action: none{})
- [x] Double Tap - onUseCard ATTACK replay + decrement. (action: none{})
- [x] Burst - onUseCard SKILL replay + decrement. (action: none{})
- [x] Echo Form - atStartOfTurn mark first card doubles; full replay queue parity remains under `POW-003` integration closure. (action: none{})
- [ ] Retain Cards - atEndOfTurn choose retain count. (action: select_cards{pile:hand,card_indices})
- [x] Equilibrium - retain hand + atEndOfRound decrement. (action: select_cards{pile:hand,card_indices})

## Tests
- [x] Add focused tests for each hook (registry integration + edge cases) -- tests/test_power_handlers_new.py (70 tests). (action: none{})
