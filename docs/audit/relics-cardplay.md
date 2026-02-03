# Audit: Card-Play-Triggered Relics

Comparison of decompiled Java (`decompiled/java-src/com/megacrit/cardcrawl/relics/`) against Python engine (`packages/engine/handlers/combat.py`).

## Summary

| Relic | Card Filter | Java Trigger | Counter | Python Status | Bugs |
|-------|------------|-------------|---------|---------------|------|
| Shuriken | ATTACK | onUseCard | 3/turn, reset atTurnStart | Implemented | Counter not reset per turn |
| Kunai | ATTACK | onUseCard | 3/turn, reset atTurnStart | Implemented | Counter not reset per turn |
| OrnamentalFan | ATTACK | onUseCard | 3/turn, reset atTurnStart | Implemented | Counter not reset per turn |
| PenNib | ATTACK | onUseCard | 10/permanent, persists across combats | Implemented (in damage calc) | Counter logic split across two methods; works but fragile |
| LetterOpener | SKILL | onUseCard | 3/turn, reset atTurnStart | Implemented | Counter not reset per turn; ID mismatch; damage should be THORNS type |
| InkBottle | ANY | onUseCard | 10/permanent, persists across combats | Implemented | OK |
| Nunchaku | ATTACK | onUseCard | 10/permanent, persists across combats | Implemented | OK |
| Art of War | ATTACK | onUseCard (sets flag false) | boolean flag per turn | Implemented | ID mismatch: code uses "ArtOfWar", relic id is "Art of War" |
| Duality | ATTACK | onUseCard | No counter | MISSING | Not implemented in _trigger_on_play_relics |
| StoneCalendar | N/A | onPlayerEndTurn (turn 7) | turn counter | MISSING | Not a card-play trigger; not in _trigger_end_of_turn either |

## Detailed Findings

### BUG-1: Counter-per-turn relics not reset at turn start

**Affected**: Shuriken, Kunai, OrnamentalFan, LetterOpener

Java resets `this.counter = 0` in `atTurnStart()` for these relics. The Python `_start_player_turn()` method does not reset these relic counters. This means the counter accumulates across turns, allowing triggers with fewer plays per turn than intended.

**Fix**: In `_start_player_turn()`, reset counters for Shuriken, Kunai, OrnamentalFan, LetterOpener to 0.

### BUG-2: Relic ID mismatches in combat.py

Several `has_relic()` calls use camelCase IDs that don't match the relic registry IDs:

| Code uses | Registry ID | Match? |
|-----------|-------------|--------|
| `"ArtOfWar"` | `"Art of War"` | NO |
| `"LetterOpener"` | `"Letter Opener"` | NO |
| `"OrnamentalFan"` | `"Ornamental Fan"` | NO |
| `"MummifiedHand"` | `"Mummified Hand"` | NO |
| `"BirdFacedUrn"` | `"Bird Faced Urn"` | NO |
| `"Shuriken"` | `"Shuriken"` | YES |
| `"Kunai"` | `"Kunai"` | YES |
| `"Nunchaku"` | `"Nunchaku"` | YES |
| `"InkBottle"` | `"InkBottle"` | YES |
| `"Pen Nib"` | `"Pen Nib"` | YES |

These mismatches mean the relics silently never trigger in combat.

### BUG-3: Duality not implemented

Java: On every ATTACK play, grants +1 Dexterity and +1 LoseDexterity (temporary, lost at end of turn). The LoseDexterity power removes the Dexterity at end of turn, making it a per-turn buff.

Python `_trigger_on_play_relics` has no Duality/Yang handling at all.

### BUG-4: StoneCalendar not implemented

Java: Counter increments each turn in `atTurnStart()`. At turn 7, `onPlayerEndTurn()` deals 52 THORNS damage to all enemies (once, then grays out). Not a card-play relic.

Python `_trigger_end_of_turn()` has no StoneCalendar handling.

### BUG-5: LetterOpener damage type

Java uses `DamageInfo.DamageType.THORNS` and `DamageInfo.createDamageMatrix(5, true)` where `true` = isFixed (ignores strength/powers). Python calls `_deal_damage_to_enemy(enemy, 5)` which is HP damage after block, but the damage value is correct (fixed 5).

This is minor -- the THORNS type in Java means the damage ignores powers like Vulnerable on the target and the player's Strength. Since Python hardcodes 5, it's actually correct behavior, just not using the same abstraction.

### BUG-6: Art of War first-turn skip

Java tracks `firstTurn` boolean to skip giving energy on the very first turn of combat (since no previous turn existed). Python does not have this guard, but the counter defaults to 0 which means "no attacks played", so it would give energy on turn 1. In Java, `atTurnStart` checks `!this.firstTurn` before granting energy.

However, this bug is moot because BUG-2 means Art of War never triggers anyway (ID mismatch).

### PenNib counter correctness (NOT a bug, but fragile)

Java: Increments counter on each ATTACK via `onUseCard`. At counter==9, applies PenNibPower (next attack double damage). At counter==10, resets to 0.

Python: Counter is tracked in `_calculate_player_damage()` (increments and checks >= 9 to apply double damage, then resets at 10 in `_trigger_start_of_combat_relics`). This works but the counter logic being in the damage calc rather than the on-play hook is unusual.

Also: Java applies PenNibPower at counter 9 (which doubles the NEXT attack), not the current one. Python applies the double damage when counter >= 9 during the current damage calculation. This means Python applies the double damage one attack too early -- at attack #10 Java gives the power, then attack #11 gets the bonus. Python gives the bonus to attack #10 itself.

**This is a behavioral difference**: Java buffs the 11th attack; Python buffs the 10th attack.

## Relic Trigger Order

Java `onUseCard` is called during `UseCardAction.update()`, AFTER the card's effects have been applied. Python calls `_trigger_on_play_relics(card)` AFTER `_apply_card_effects()`, which matches Java's ordering.

However, InkBottle is triggered inline during `play_card()` BEFORE `_trigger_on_play_relics()`. In Java, InkBottle also uses `onUseCard`, so all card-play relics fire in the same hook. Python splits them, which could cause ordering differences.
