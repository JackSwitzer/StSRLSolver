# Block Pipeline Audit

Audit of Python engine block calculation vs decompiled Java source.

## Sources

- **Java**: `decompiled/java-src/com/megacrit/cardcrawl/`
  - `cards/AbstractCard.java:2276` - `applyPowersToBlock()`
  - `powers/DexterityPower.java:83` - `modifyBlock()`
  - `powers/FrailPower.java:55` - `modifyBlock()`
  - `powers/BarricadePower.java` - Passive (no hook, checked in GameActionManager)
  - `powers/BlurPower.java:36` - `atEndOfRound()` decrements
  - `powers/MetallicizePower.java:39` - `atEndOfTurnPreEndTurnCards()`
  - `powers/PlatedArmorPower.java:69` - `atEndOfTurnPreEndTurnCards()`, `wasHPLost()` decrements
  - `powers/AfterImagePower.java:36` - `onUseCard()` grants block
  - `actions/GameActionManager.java:342-347` - Turn-start block removal
- **Python**: `packages/engine/`
  - `calc/damage.py:158` - `calculate_block()`
  - `combat_engine.py:1516` - `_calculate_block_gained()`
  - `combat_engine.py:288-302` - Turn-start block removal
  - `handlers/combat.py:397-409` - Turn-start block removal (alternate handler)

## Block Calculation (Card-based)

### Java (`AbstractCard.applyPowersToBlock`)
```java
float tmp = this.baseBlock;
for (AbstractPower p : player.powers) tmp = p.modifyBlock(tmp, this);  // Dex, Frail
for (AbstractPower p : player.powers) tmp = p.modifyBlockLast(tmp);    // NoBlock
if (tmp < 0.0f) tmp = 0.0f;
this.block = MathUtils.floor(tmp);
```

### Python (`calc/damage.py:calculate_block`)
```python
block = float(base)
block += dexterity           # Dex additive
if frail: block *= 0.75      # Frail multiplicative
if no_block: block = 0.0     # NoBlock
return max(0, int(block))    # Floor + clamp
```

### Verdict: CORRECT
- Dexterity applied additively before Frail -- matches Java DexterityPower.modifyBlock priority
- Frail 0.75 multiplier -- matches Java FrailPower.modifyBlock
- NoBlock applied last -- matches Java modifyBlockLast
- Floor + min 0 -- matches Java MathUtils.floor + clamp
- `int()` truncation vs `MathUtils.floor` is equivalent for non-negative results (clamped)

Note: Java DexterityPower.modifyBlock clamps to 0 internally, Python does not. Functionally equivalent because negative values * 0.75 remain negative and get clamped to 0 at the end.

## Turn-Start Block Removal

### Java (`GameActionManager.java:342-347`)
```java
if (!player.hasPower("Barricade") && !player.hasPower("Blur")) {
    if (!player.hasRelic("Calipers")) {
        player.loseBlock();          // Lose ALL block
    } else {
        player.loseBlock(15);        // Lose only 15
    }
}
```

### Python `combat_engine.py` (lines 288-302): CORRECT
```python
if not self._has_barricade():      # Barricade check
    blur = player.statuses.get("Blur", 0)
    if blur > 0:                   # Blur retains, decrements
        blur -= 1; ...
    elif self._has_calipers():     # Calipers: lose 15
        player.block = max(0, player.block - 15)
    else:
        player.block = 0           # Full loss
```
Correctly implements the priority chain: Barricade > Blur > Calipers > full loss.

### Python `handlers/combat.py` (lines 397-409): 2 BUGS

**Bug 1**: Blur does not decrement (line 403 is `pass` with no decrement logic).
```python
if blur > 0:
    pass  # Keep block based on blur stacks  <-- NO DECREMENT
```

**Bug 2**: Calipers check runs unconditionally after block removal, even when Barricade is present.
```python
# Lines 408-409 are OUTSIDE the Barricade/Blur conditional
if self.state.has_relic("Calipers"):
    self.state.player.block = max(0, self.state.player.block - 15)
```
This means if player has Barricade + Calipers, they still lose 15 block (Java retains all).

## End-of-Turn Block Powers

### Metallicize
- **Java**: `atEndOfTurnPreEndTurnCards` -> `GainBlockAction(owner, owner, amount)`
- **Python**: `player.block += metallicize` (both engines)
- **Verdict**: CORRECT. Raw block gain bypasses Dex/Frail (matches Java GainBlockAction).

### Plated Armor
- **Java**: `atEndOfTurnPreEndTurnCards` -> `GainBlockAction(owner, owner, amount)`. `wasHPLost` decrements by 1 on unblocked damage (not HP_LOSS, not THORNS).
- **Python**: `player.block += plated` (both engines). HP loss decrements in combat_engine.py:592-600.
- **Verdict**: CORRECT for block gain. HP loss decrement implemented.

### After Image
- **Java**: `onUseCard` -> `GainBlockAction(player, player, amount)` for every card played
- **Python**: Not found as an explicit onUseCard hook in either combat engine. Likely handled via effect system or not yet implemented.
- **Verdict**: NEEDS VERIFICATION. Could not confirm After Image triggers on every card play.

### Blur
- **Java**: No block-related hook. Prevents block loss at turn start (checked in GameActionManager). Decrements via `atEndOfRound`.
- **Python combat_engine.py**: Correctly retains block and decrements at turn start.
- **Python handlers/combat.py**: BUG -- does not decrement.

### Barricade
- **Java**: Passive power. Checked in GameActionManager to skip block removal.
- **Python**: Both engines check Barricade status. CORRECT.

## Summary of Issues

| Issue | Severity | Location | Description |
|-------|----------|----------|-------------|
| Blur no decrement | HIGH | `handlers/combat.py:403` | Blur stacks never decrease, infinite block retention |
| Calipers scope | HIGH | `handlers/combat.py:408` | Calipers deducts 15 even with Barricade active |
| After Image | MEDIUM | Both engines | Could not confirm onUseCard block gain is implemented |

## Verified Correct

- Dexterity additive application
- Frail 0.75 multiplier (floor rounding)
- NoBlock power
- Barricade full retention
- Metallicize end-of-turn raw block
- Plated Armor end-of-turn raw block + HP loss decrement
- Block absorbs damage before HP (both engines)
- combat_engine.py block removal priority chain
