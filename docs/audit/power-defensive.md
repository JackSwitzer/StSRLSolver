# Defensive Power Audit: Python Engine vs Decompiled Java

Audited: 2026-02-02

## Powers Audited

| Power | Java Class | Python ID | Verdict |
|-------|-----------|-----------|---------|
| Dexterity | DexterityPower | "Dexterity" | BUG - not removed at 0 stacks |
| Frail | FrailPower | "Frail" | MATCH |
| Buffer | BufferPower | "Buffer" | BUG - triggers after block instead of before |
| IntangiblePlayer | IntangiblePlayerPower | "Intangible" | BUG - wrong ID, wrong decrement hook |
| Intangible (monster) | IntangiblePower | "Intangible" | PARTIAL - justApplied not enforced in combat engine |
| Metallicize | MetallicizePower | "Metallicize" | MATCH |
| Plated Armor | PlatedArmorPower | "Plated Armor" | BUG - missing damage type check in wasHPLost |
| Barricade | BarricadePower | "Barricade" | MATCH |
| Artifact | ArtifactPower | "Artifact" | MATCH |
| Blur | BlurPower | "Blur" | MATCH (timing cosmetically different but functionally equivalent) |

## Detailed Findings

### 1. Dexterity - MATCH

**Java** (`DexterityPower.java`):
- `modifyBlock(float blockAmount)`: adds `this.amount` to blockAmount, floors to 0 if negative
- `canGoNegative = true`, capped at +/-999
- Removed when stacked to exactly 0

**Python** (`powers.py` + `damage.py`):
- `calculate_block()`: `block += dexterity`, then floors to 0 via `max(0, int(block))`
- `can_go_negative: True`, capped at +/-999
- Removed when stacked to 0 via `should_remove()`

**Minor bug**: Java's `stackPower` explicitly removes Dexterity when amount reaches exactly 0 via `RemoveSpecificPowerAction`. Python's `should_remove()` returns `False` for `can_go_negative` powers at 0, so Dexterity persists at 0 stacks. Same issue affects Strength.

### 2. Frail - MATCH

**Java** (`FrailPower.java`):
- `modifyBlock(float blockAmount)`: returns `blockAmount * 0.75f`
- `priority = 10` (runs after Dexterity's default priority 5)
- Decrements at `atEndOfRound()`, with `justApplied` skip for monster-sourced
- `isTurnBased = true`, `type = DEBUFF`

**Python** (`damage.py`):
- `calculate_block()`: applies Dex first, then `block *= FRAIL_MULT` (0.75). Correct order.
- Decremented in `at_end_of_round()` with `just_applied` skip

No discrepancy.

### 3. Buffer - BUG

**Java** (`BufferPower.java`):
```java
public int onAttackedToChangeDamage(DamageInfo info, int damageAmount) {
    if (damageAmount > 0) {
        addToTop(new ReducePowerAction(this.owner, this.owner, this.ID, 1));
    }
    return 0;  // Always returns 0, setting damage to 0
}
```

The hook `onAttackedToChangeDamage` is called BEFORE block subtraction in Java's damage pipeline. Buffer prevents ALL damage (the full hit), so block is NOT consumed.

**Python** (`combat_engine.py` lines 569-583):
```python
# Apply block
blocked = min(self.state.player.block, damage)
hp_damage = damage - blocked
self.state.player.block -= blocked

# Buffer: prevent HP loss
if hp_damage > 0:
    buffer = self.state.player.statuses.get("Buffer", 0)
    ...
    hp_damage = 0
```

Python applies block FIRST, then checks Buffer on remaining `hp_damage`. This means:
- Block is consumed even when Buffer would prevent the hit
- Buffer only triggers when damage exceeds block (should trigger on any damage > 0 regardless of block)

**Fix needed**: Move Buffer check before block application. If Buffer triggers, set damage to 0 and skip block consumption.

### 4. IntangiblePlayer - BUG (dual ID issue)

**Java**: Two separate classes:
- `IntangiblePlayerPower` (ID: `"IntangiblePlayer"`) - decrements at `atEndOfRound()` (after all turns)
- `IntangiblePower` (ID: `"Intangible"`) - decrements at `atEndOfTurn()` with `justApplied` skip (monster version)

**Python**: Single entry in POWER_DATA with ID `"Intangible"`, documented as decrementing at `atEndOfTurn`.

The combat engine only checks `statuses.get("Intangible", 0)`. If the player gets Intangible via Wraith Form or Apparition, it should use ID `"IntangiblePlayer"` and decrement at end of round (not end of turn). Currently the engine would not recognize `"IntangiblePlayer"` at all.

**Fix needed**: Either use a single ID and handle timing based on owner type, or support both IDs in the combat engine damage pipeline.

### 5. Plated Armor - BUG (missing damage type check)

**Java** (`PlatedArmorPower.java`):
```java
public void wasHPLost(DamageInfo info, int damageAmount) {
    if (info.owner != null && info.owner != this.owner
        && info.type != DamageInfo.DamageType.HP_LOSS
        && info.type != DamageInfo.DamageType.THORNS
        && damageAmount > 0) {
        // decrement
    }
}
```

Plated Armor only decrements from NORMAL damage dealt by another creature. Self-damage, HP_LOSS (poison), and THORNS damage do NOT reduce it.

**Python** (`combat_engine.py` lines 592-600):
```python
if hp_damage > 0:
    plated = self.state.player.statuses.get("Plated Armor", 0)
    if plated > 0:
        plated -= 1
```

Python decrements on ANY `hp_damage > 0` without checking damage type or source. This means:
- Poison damage (HP_LOSS) would incorrectly decrement Plated Armor
- Self-damage from cards like Offering would incorrectly decrement it
- Thorns damage would incorrectly decrement it

**Fix needed**: Add damage type and source checks to the Plated Armor decrement logic.

### 6. Metallicize - MATCH

**Java** (`MetallicizePower.java`):
- `atEndOfTurnPreEndTurnCards(boolean isPlayer)`: gain `this.amount` block
- No conditions on isPlayer (both player and monster can use it)

**Python** (`combat_engine.py` lines 394-397 and 465-468):
- Player: `self.state.player.block += metallicize` at end of turn
- Enemy: `enemy.block += metallicize` at start of enemy turn

Both correct. The timing difference (Python does enemy Metallicize at start of enemy turn) is functionally equivalent since block is checked during the enemy's turn.

### 7. Barricade - MATCH

**Java** (`BarricadePower.java`):
- `amount = -1` (no stacking)
- No hooks -- block retention is handled by the game loop checking for Barricade presence

**Python**:
- `_has_barricade()` checks for Barricade relic, power, or relic counter
- Block is not removed at start of turn when `_has_barricade()` returns True

Correct.

### 8. Blur - MATCH (cosmetic timing difference)

**Java** (`BlurPower.java`):
- `isTurnBased = true`
- Decrements at `atEndOfRound()` (after all turns in a round)
- Block retention is handled by game loop checking for Blur presence

**Python** (`combat_engine.py` lines 288-297):
- Block retention checked at start of player turn
- Blur decremented at the same time

The timing is different (Java: end of round; Python: start of next turn) but the functional outcome is identical -- block is retained for one round per stack.

### 9. Artifact - MATCH

**Java** (`ArtifactPower.java`):
- `onSpecificTrigger()`: reduces amount by 1, removes at 0
- Called externally by debuff application logic when a debuff is blocked

**Python** (`combat_engine.py` line 1731-1739 and `powers.py` PowerManager.add_power):
- Checked inline when applying debuffs (Weak, Vulnerable, Frail)
- Decrements and blocks the debuff

Correct behavior. Both implementations consume one Artifact charge per blocked debuff.

## Summary of Bugs

1. **Buffer**: Triggers after block instead of before. Block is incorrectly consumed when Buffer should prevent the entire hit.
2. **IntangiblePlayer**: Player intangible uses wrong ID ("Intangible" instead of "IntangiblePlayer") and wrong decrement timing (should be atEndOfRound, not atEndOfTurn).
3. **Plated Armor**: Missing damage type and source checks in wasHPLost. Self-damage, HP_LOSS, and THORNS incorrectly decrement it.
4. **Dexterity/Strength**: Not removed when stacked to exactly 0. Java explicitly removes via `RemoveSpecificPowerAction` in `stackPower()`. Minor -- 0 stacks has no gameplay effect but leaves stale power data.
