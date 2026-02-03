# Stance Mechanics Audit: Python Engine vs Decompiled Java

## Files Compared

**Java (decompiled):**
- `decompiled/java-src/com/megacrit/cardcrawl/stances/AbstractStance.java`
- `decompiled/java-src/com/megacrit/cardcrawl/stances/WrathStance.java`
- `decompiled/java-src/com/megacrit/cardcrawl/stances/CalmStance.java`
- `decompiled/java-src/com/megacrit/cardcrawl/stances/DivinityStance.java`
- `decompiled/java-src/com/megacrit/cardcrawl/stances/NeutralStance.java`
- `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/RushdownPower.java`
- `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MentalFortressPower.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlurryOfBlows.java`
- `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ChangeStanceAction.java`

**Python:**
- `packages/engine/content/stances.py` (StanceManager, StanceID, STANCES dict)
- `packages/engine/combat_engine.py` (`_change_stance`, `end_turn`, enemy damage loop)
- `packages/engine/calc/damage.py` (damage multipliers)

## Findings

### CORRECT: Wrath 2x Damage Dealt (NORMAL only)
- **Java**: `WrathStance.atDamageGive()` returns `damage * 2.0f` for `DamageType.NORMAL`
- **Python**: `combat_engine.py` uses `WRATH_MULT = 2.0` in damage calc, `stances.py` has `damage_give_multiplier=2.0`

### CORRECT: Wrath 2x Damage Received (NORMAL only)
- **Java**: `WrathStance.atDamageReceive()` returns `damage * 2.0f` for `DamageType.NORMAL`
- **Python**: `combat_engine.py` line ~548: `stance_mult = 2.0` when in Wrath during enemy attack resolution

### CORRECT: Calm Exit +2 Energy
- **Java**: `CalmStance.onExitStance()` adds `GainEnergyAction(2)`
- **Python**: `_change_stance()` adds 2 energy (or 3 with Violet Lotus)

### CORRECT: Divinity 3x Damage Dealt
- **Java**: `DivinityStance.atDamageGive()` returns `damage * 3.0f` for `DamageType.NORMAL`
- **Python**: `DIVINITY_MULT = 3.0` in damage calc

### CORRECT: Divinity +3 Energy on Enter
- **Java**: `DivinityStance.onEnterStance()` adds `GainEnergyAction(3)`
- **Python**: `_change_stance()` adds 3 energy when entering Divinity

### CORRECT: Divinity No Extra Incoming Damage
- **Java**: `DivinityStance` has no `atDamageReceive` override (inherits 1x from AbstractStance)
- **Python**: `damage_receive_multiplier=1.0` in STANCES dict

### CORRECT: Mental Fortress Block on Any Stance Change
- **Java**: `MentalFortressPower.onChangeStance()` grants block if `!oldStance.ID.equals(newStance.ID)`
- **Python**: `_change_stance()` adds block if MentalFortress status > 0 (already guarded by same-stance check)

### CORRECT: Rushdown Draw on Entering Wrath Only
- **Java**: `RushdownPower.onChangeStance()` draws if `newStance.ID.equals("Wrath")` and old != new
- **Python**: `_change_stance()` draws if `new_stance == StanceID.WRATH` and rushdown > 0

### CORRECT: Flurry of Blows Discard-to-Hand on Stance Change
- **Java**: `ChangeStanceAction` iterates `discardPile.group` calling `triggerExhaustedCardsOnStanceChange`, FlurryOfBlows adds `DiscardToHandAction`
- **Python**: `_change_stance()` moves FlurryOfBlows cards from discard_pile to hand

### CORRECT: Mantra Threshold and Overflow
- **Java**: Devotion adds mantra; at 10+, enters Divinity, excess carries over
- **Python**: `_add_mantra()` checks >= 10, subtracts 10, calls `_change_stance(DIVINITY)`

### BUG: Divinity Auto-Exit Timing
- **Java**: `DivinityStance.atStartOfTurn()` calls `ChangeStanceAction("Neutral")` -- exits at START of next player turn
- **Python**: `end_turn()` exits Divinity at END of current player turn

**Impact**: In Java, Divinity persists through the entire enemy turn. This means:
1. Any reactive damage (Thorns, etc.) during the enemy turn still gets 3x from Divinity
2. The stance change to Neutral triggers at start of next turn, which fires Mental Fortress block and Flurry of Blows at that timing
3. The energy from Calm exit (if somehow entering Calm from Divinity at start of turn) would happen at turn start

**Severity**: Medium. Affects edge cases with reactive damage and power/card trigger timing. The early exit means the player gets Mental Fortress block at end of turn instead of start of next turn (block would be cleared by then in the current implementation anyway), but loses 3x on any thorns-like effects during enemy turn.

## Summary

| Mechanic | Status | Notes |
|----------|--------|-------|
| Wrath 2x dealt | CORRECT | NORMAL type only |
| Wrath 2x received | CORRECT | NORMAL type only |
| Calm +2 energy exit | CORRECT | +3 with Violet Lotus |
| Divinity 3x dealt | CORRECT | NORMAL type only |
| Divinity +3 energy enter | CORRECT | |
| Divinity no extra received | CORRECT | |
| Divinity auto-exit timing | **BUG** | Should be start of next turn, not end of current turn |
| Mental Fortress | CORRECT | Block on any change |
| Rushdown | CORRECT | Draw on Wrath entry only |
| Flurry of Blows | CORRECT | Discard to hand |
| Mantra overflow | CORRECT | Excess carries over |
| Same-stance no-op | CORRECT | Both skip if already in stance |
