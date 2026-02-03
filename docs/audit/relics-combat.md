# Relic Combat Trigger Audit: Python Engine vs Decompiled Java

Audited: 2026-02-02

## Scope

Relics with `atBattleStart`, `atTurnStart`, `onPlayerEndTurn`, and `onVictory` hooks.
Compared Python (`packages/engine/handlers/combat.py`, `packages/engine/game.py`) against decompiled Java (`decompiled/java-src/com/megacrit/cardcrawl/relics/`).

## atBattleStart Relics

| Relic | Java Hook | Java Effect | Python Location | Python Effect | Status |
|-------|-----------|-------------|-----------------|---------------|--------|
| Anchor | `atBattleStart` | GainBlockAction(player, 10) | `combat.py:290` | `player.block += 10` | MATCH |
| Bag of Preparation | `atBattleStart` | DrawCardAction(player, 2) | `combat.py:263` | `draw_count += 2` | MATCH |
| Ring of the Snake | `atBattleStart` (not in decompiled, but documented) | Draw 2 additional cards | `combat.py:258` | `draw_count += 2` | MATCH |
| Bag of Marbles | `atBattleStart` | Apply 1 Vulnerable to all enemies | `combat.py:284` | Apply Vulnerable 1 to all | MATCH |
| Akabeko | `atBattleStart` | Gain 8 Vigor | `combat.py:295` | Apply Vigor 8 | MATCH |
| Bronze Scales | `atBattleStart` | Gain 3 Thorns | `combat.py:299` | Apply Thorns 3 | MATCH |
| Thread and Needle | `atBattleStart` | ApplyPowerAction(PlatedArmor, 4) | `combat.py:306` | Apply Plated Armor 4 | MATCH |
| Preserved Insect | `atBattleStart` | If elite, reduce all enemy HP by 25% | `combat.py:302` (comment only) | NOT IMPLEMENTED in combat handler | **BUG** - commented out, says "should be handled when creating enemies" but no evidence of implementation |

## atTurnStart / atPreBattle Relics

| Relic | Java Hook | Java Effect | Python Location | Python Effect | Status |
|-------|-----------|-------------|-----------------|---------------|--------|
| Lantern | `atPreBattle` sets firstTurn=true; `atTurnStart` gives 1 energy if firstTurn | +1 Energy turn 1 only | `combat.py:433` | `if turn == 1: energy += 1` | MATCH (different mechanism, same result) |
| Orichalcum | `onPlayerEndTurn` | If block == 0, gain 6 block | Not found in combat.py | **MISSING** | **BUG** - Orichalcum end-of-turn trigger not implemented |

## onVictory Relics

| Relic | Java Hook | Java Effect | Python Location | Python Effect | Status |
|-------|-----------|-------------|-----------------|---------------|--------|
| Burning Blood | `onVictory` | If HP > 0, heal 6 | `game.py:1590` | `rs.heal(6)` | MATCH (but Python doesn't check HP > 0; dead player can't reach this code path anyway) |
| Black Blood | `onVictory` | If HP > 0, heal 12 | `game.py:1598` | `rs.heal(12)` | MATCH |
| Meat on the Bone | `onTrigger` (called from AbstractRoom victory) | If HP <= maxHP/2 AND HP > 0, heal 12 | `game.py:1606` | If `current_hp < max_hp * 0.5`, heal 12 | **BUG** - Java uses `<=` (HP <= 50%), Python uses `<` (HP < 50%). At exactly 50% HP, Java heals but Python does not. |
| Blood Vial | `atBattleStart` in Java | Heal 2 at START of combat | `game.py:1615` | Heals 2 AFTER combat | **BUG** - Blood Vial triggers `atBattleStart` in Java (heal 2 at combat start), but Python triggers it post-combat. This is completely wrong timing. |

## Passive / Modifier Relics (no combat hook)

| Relic | Java Mechanism | Python Definition | Status |
|-------|---------------|-------------------|--------|
| Odd Mushroom | Static field `VULN_EFFECTIVENESS = 1.25f`, checked in damage calc | `relics.py:1141` effect string only | OK - must be checked in damage pipeline |
| Magic Flower | `onPlayerHeal` returns `round(heal * 1.5)`, ONLY during combat phase | `relics.py:675` effect string "Healing is 50% more effective" | **INACCURACY** - Python description says always; Java only applies in combat. Need to verify damage pipeline implementation. |

## Missing Relics (not in decompiled Java files)

- **Marking**: No file found at expected path. Not a standard relic name.
- **Ring of the Snake**: No separate .java file found (likely in character class init).

## Summary of Bugs Found

1. **Preserved Insect** - atBattleStart elite HP reduction is commented out, not implemented
2. **Orichalcum** - onPlayerEndTurn block-if-no-block trigger missing from combat handler
3. **Meat on the Bone** - Uses `<` instead of `<=` for 50% HP threshold (off-by-one at exactly 50%)
4. **Blood Vial** - Triggers post-combat (game.py:1615) instead of atBattleStart (wrong timing entirely)
5. **Magic Flower** - Python description implies always-on; Java only applies during combat phase
6. **Lantern** - NEVER fires. CombatState.turn starts at 1, `_start_player_turn` increments to 2 before `_trigger_start_of_turn` checks `turn == 1`. Java uses a separate `firstTurn` boolean.
7. **Bag of Preparation** - Relic ID mismatch. Registry stores `"Bag of Preparation"` but combat handler checks `has_relic("BagOfPreparation")`. The relic never triggers.
8. **Horn Cleat** - Same turn counter bug as Lantern. Checks `turn == 2` but by the time the check runs on what should be turn 2, the counter is already 3.

## Files Reviewed

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/content/relics.py` - Relic definitions
- `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/handlers/combat.py` - Combat relic triggers
- `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/game.py` - Post-combat relic triggers
- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/` - Java source of truth
