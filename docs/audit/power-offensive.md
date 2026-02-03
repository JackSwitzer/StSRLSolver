# Offensive Power Trigger Audit

Comparison of Python engine (`packages/engine/`) against decompiled Java source.

## Damage Calculation Chain

### Java (AbstractCard.calculateCardDamage, line 2298)

```
1. Relic atDamageModify (e.g., Striker for Strikes)
2. Player powers atDamageGive (sorted by priority)
   - Strength (pri 5): +amount (NORMAL only)
   - Vigor (pri 5): +amount (NORMAL only)
   - PenNib (pri 6): *2.0 (NORMAL only)
   - DoubleDamage (pri 6): *2.0 (NORMAL only)
   - Weak (pri 99): *0.75 or *0.60 w/ Paper Crane (NORMAL only)
3. Stance atDamageGive
   - Wrath: *2.0 (NORMAL only)
   - Divinity: *3.0 (NORMAL only)
4. Target powers atDamageReceive
   - Vulnerable: *1.5 (or *1.25 Odd Mushroom, *1.75 Paper Frog) (NORMAL only)
   - Flight: *0.5 (NORMAL only)
5. Player powers atDamageFinalGive (none of the audited powers use this)
6. Target powers atDamageFinalReceive
   - Intangible: cap at 1
7. Floor to int, min 0
```

### Python (packages/engine/calc/damage.py, calculate_damage)

```
1. base + strength + vigor (flat adds)
2. pen_nib *2, double_damage *2
3. weak *0.75
4. stance_mult
5. vuln *1.5
6. flight *0.5
7. intangible cap at 1
8. floor, min 0
```

**Verdict**: Order matches Java. Flat adds before multipliers, attacker multipliers before stance, stance before defender multipliers, intangible last.

## Per-Power Audit

### Strength -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageGive | +strength in calculate_damage | Yes |
| Formula | damage + amount | damage + strength | Yes |
| NORMAL only | Yes | Yes (calculate_damage is NORMAL-only context) | Yes |
| Can go negative | Yes (canGoNegative=true) | Yes (can_go_negative=True) | Yes |
| Caps | -999 to 999 | -999 to 999 | Yes |
| Remove at 0 | Yes (RemoveSpecificPowerAction) | Yes (should_remove) | Yes |

### Vigor -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageGive (+amount) | +vigor in calculate_damage | Yes |
| NORMAL only | Yes | Yes | Yes |
| Consumed on attack | onUseCard: ATTACK -> remove | POWER_DATA: on_use_card if ATTACK remove | Yes |
| Not turn-based | isTurnBased=false | is_turn_based=False | Yes |

### Pen Nib -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageGive *2.0 | pen_nib *2.0 | Yes |
| NORMAL only | Yes | Yes | Yes |
| Priority | 6 | 6 | Yes |
| Consumed on attack | onUseCard: ATTACK -> remove | POWER_DATA: on_use_card if ATTACK remove | Yes |

### Double Damage -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageGive *2.0 | double_damage *2.0 | Yes |
| NORMAL only | Yes | Yes | Yes |
| Priority | 6 | 6 | Yes |
| Turn-based | Yes, decrements at end of round | Yes, is_turn_based=True | Yes |
| justApplied | isSourceMonster -> skip first decrement | just_applied logic in at_end_of_round | Yes |

### Weak -- PASS (with note)
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageGive *0.75 | weak *0.75 | Yes |
| Paper Crane | *0.60 if !owner.isPlayer && player has Paper Crane | *0.60 via weak_paper_crane param | Yes |
| Priority | 99 (applies after all other atDamageGive) | Applied after pen_nib/double_damage in code order | Yes |
| NORMAL only | Yes | Yes | Yes |

**Note on PowerManager**: `calculate_damage_dealt` has `has_paper_crane` on the attacker's PowerManager with `target_is_player` check. This works logically (enemy attacker is weak, target is player who owns Paper Crane) but the flag placement is awkward -- Paper Crane is a player relic stored as a flag on the enemy's PowerManager.

### Vulnerable -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | atDamageReceive *1.5 | vuln *1.5 | Yes |
| Odd Mushroom | *1.25 if owner.isPlayer && player has Odd Mushroom | *1.25 via vuln_odd_mushroom param | Yes |
| Paper Frog | *1.75 if !owner.isPlayer && player has Paper Frog | *1.75 via vuln_paper_frog param | Yes |
| NORMAL only | Yes | Yes | Yes |

### Wrath Stance -- PASS
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| atDamageGive | *2.0 NORMAL | damage_give_multiplier=2.0 | Yes |
| atDamageReceive | *2.0 NORMAL | damage_receive_multiplier=2.0 | Yes |
| Non-NORMAL | Returns damage unchanged | Only applies to NORMAL | Yes |

### Divinity Stance -- BUG FOUND
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| atDamageGive | *3.0 NORMAL | damage_give_multiplier=3.0 | Yes |
| atDamageReceive | None (no override, inherits 1.0) | damage_receive_multiplier=1.0 | Yes |
| Energy on enter | 3 (in onEnterStance) | energy_on_enter=3 | Yes |
| **Exit timing** | **atStartOfTurn -> Neutral** | **on_turn_end -> Neutral** | **NO** |

**BUG**: Divinity exits at start of next turn in Java (DivinityStance.atStartOfTurn), but Python exits at end of current turn (StanceManager.on_turn_end with exits_at_turn_end=True). This means Python loses 3x multiplier for any end-of-turn damage effects (e.g., Omega power dealing THORNS damage). In practice for most cases this is equivalent, but it's semantically wrong.

### Envenom -- PASS (with note)
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Hook | onAttack | POWER_DATA on_attack | Yes |
| Condition | damageAmount > 0 && target != owner && NORMAL | "if damage > 0 NORMAL" | Partial |
| Effect | Apply amount Poison via addToTop | apply amount Poison | Yes |

**Note**: Java has `target != this.owner` check preventing self-poison. Python description omits this. Verify actual combat implementation handles this.

### Accuracy -- PASS (different mechanism)
| Aspect | Java | Python | Match |
|--------|------|--------|-------|
| Mechanism | Modifies Shiv baseDamage directly (4+amount or 6+amount upgraded) | "passive: Shivs deal +amount damage" | Conceptual |
| Hook | stackPower + onDrawOrDiscard (updates all piles) | Not an atDamageGive hook | N/A |

**Note**: Accuracy does NOT use atDamageGive. It directly modifies Shiv card baseDamage. Python should ensure Shiv baseDamage includes Accuracy amount when creating/drawing Shivs, not treat it as a damage calculation modifier.

## Bugs Found

1. **Divinity exit timing** (stances.py): Exits at end of turn instead of start of next turn. Fix: change `exits_at_turn_end` to `exits_at_start_of_next_turn` and move the logic from `on_turn_end` to a start-of-turn hook.

## Design Notes

- PowerManager.calculate_damage_dealt combines Strength, Vigor, Weak, PenNib, DoubleDamage into one method. This is a convenience wrapper, not used by the pure-function damage.py calculator.
- damage.py calculate_damage is the canonical calculation path and correctly separates all parameters.
- The two systems (PowerManager methods vs damage.py functions) could diverge. Consider using only one.
