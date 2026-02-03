# Event Outcomes Audit: Python Engine vs Decompiled Java

Audit date: 2026-02-02
Scope: Act 1 (Exordium), Act 2 (City), and Shrine events

## Summary

| Severity | Count | Description |
|----------|-------|-------------|
| CRITICAL | 2 | Wrong event mechanics (Beggar, Nest structure) |
| HIGH | 4 | Wrong rounding/math (Big Fish, Golden Idol, Shining Light, Forgotten Altar) |
| MEDIUM | 2 | Missing A15 modifiers (Nest gold), missing stop option (Cursed Tome) |
| LOW | 1 | Knowing Skull escalation model incomplete |

## Critical Issues

### 1. Beggar (Act 2) - WRONG MECHANICS
- **Python**: Option 0 = "Pay 75 gold, obtain random relic". Option 1 = "Gain ~75 gold and Doubt curse".
- **Java**: Option 0 = Pay 75 gold, **remove a card** (card purge, not relic). Option 1 = Leave. There is NO steal/curse option.
- **Fix**: Beggar should be: Option 0 = pay 75g + remove card, Option 1 = leave. The Python event is entirely fabricated for option 1.

### 2. Nest (Act 2) - WRONG STRUCTURE
- **Python**: Option 0 = "Take: Obtain 99 gold and Ritual Dagger card". Option 1 = "Leave".
- **Java**: Option 0 = Steal gold only (99g, or 50g on A15+). Option 1 = Join cult: take 6 HP damage + obtain Ritual Dagger. These are separate options.
- **Fix**: Split into two distinct options. Add A15 gold modifier (50 vs 99).

## High Severity Issues

### 3. Big Fish (Act 1) - WRONG HEAL CALCULATION
- **Python**: `value_percent=0.33` -> `int(maxHP * 0.33)`
- **Java**: `maxHealth / 3` (integer division)
- **Difference**: For maxHP=72: Python=23, Java=24. For maxHP=100: Python=33, Java=33.
- **Fix**: Use `maxHP // 3` instead of `int(maxHP * 0.33)`.

### 4. Golden Idol Escape (Act 1) - WRONG ROUNDING
- **Damage option**: Python uses `ceil(abs(maxHP * 0.25))`. Java uses `(int)(maxHP * 0.25f)` (truncation).
- **Max HP loss option**: Python uses `ceil(abs(maxHP * 0.08))`. Java uses `(int)(maxHP * 0.08f)` (truncation), with min=1.
- **A15 variants**: Same pattern - Java truncates, Python ceils.
- **Fix**: Use `int()` truncation, not `math.ceil()`. Add min=1 guard for max HP loss.

### 5. Shining Light (Act 1) - WRONG ROUNDING
- **Python**: Uses `ceil()` via `calculate_outcome_value` for negative percents.
- **Java**: Uses `MathUtils.round()` (standard rounding, not ceil).
- **Example**: maxHP=72, A0: Python=`ceil(72*0.2)=15`, Java=`round(72*0.2)=14`.
- **Fix**: Use `round()` instead of `ceil()`.

### 6. Forgotten Altar (Act 2) - WRONG ROUNDING
- **Python**: Sacrifice damage uses `ceil()` for percentage.
- **Java**: Uses `MathUtils.round()`.
- **Fix**: Use `round()`.

## Medium Severity Issues

### 7. Nest (Act 2) - MISSING A15 MODIFIER
- **Python**: Gold is always 99.
- **Java**: Gold is 99 normally, 50 on A15+.
- **Fix**: Add ascension check.

### 8. Cursed Tome (Act 2) - MISSING STOP OPTION
- **Python**: Models only "read all pages" (16/21 damage) or "leave".
- **Java**: After page 3, player can choose to stop (takes additional 3 damage = total 1+2+3+3=9) or continue (takes finalDmg = 10/15, total = 1+2+3+10=16 or 1+2+3+15=21).
- **Fix**: Add stop option after page 3.

## Low Severity Issues

### 9. Knowing Skull (Act 2) - INCOMPLETE ESCALATION MODEL
- **Python**: Says "Pay 6 HP (escalates)" for each option.
- **Java**: Each option (potion, gold, card) has its own cost counter starting at 6, incrementing by 1 per use of THAT option. Leave cost is always 6 (never escalates).
- **Fix**: Model per-option cost escalation. Document that leave cost is fixed.

## Verified Correct

| Event | Notes |
|-------|-------|
| Cleric | Heal = `int(maxHP * 0.25)`, purify cost 50/75 on A15+. Correct. |
| Goop Puddle | 75 gold + 11 damage, or lose 20-50/35-75 gold. Correct. |
| Sssserpent | 175/150 gold + Doubt curse. Correct. |
| Scrap Ooze | 3/5 base damage, +1 per attempt, 25% +10% relic chance. Correct. |
| Living Wall | Remove/transform/upgrade. Correct. |
| Dead Adventurer | Structure and mechanics match. Correct. |
| Addict | 85 gold for relic, or steal (relic + Shame curse). Correct. |
| Ghosts | `ceil(maxHP * 0.5)` max HP loss, 5/3 Apparitions. Correct. |
| Vampires | `ceil(maxHP * 0.3)` max HP loss, remove strikes + 5 bites. Correct. |
| Gold Shrine | 100/50 gold pray, 275 gold + Regret desecrate. Correct. |
| Mushrooms | Fight or heal 25% + Parasite. Correct (uses truncation like Java). |

## Rounding Method Summary

Different Java events use different rounding for HP percentage calculations:

| Method | Events |
|--------|--------|
| `(int)(maxHP * pct)` (truncation) | Big Fish, Cleric, Golden Idol, Mushrooms |
| `MathUtils.round()` | Shining Light, Forgotten Altar |
| `MathUtils.ceil()` | Ghosts, Vampires |
| `maxHP / N` (integer division) | Big Fish (heal = maxHP/3) |

The Python `calculate_outcome_value` function uses `ceil()` for all negative percentages, which is incorrect for events that use truncation or rounding.
