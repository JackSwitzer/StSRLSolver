# Verified Seed Data (Ground Truth)

This document contains verified game data for specific seeds, used as ground truth for RNG prediction testing.

## Seed: TEST123

**Seed Value:** 52248462423
**Character:** Watcher
**Ascension:** 0 (assumed)

### Neow Options
| Option | Type | Value |
|--------|------|-------|
| 1 | Basic | RANDOM_COLORLESS |
| 2 | Small | HUNDRED_GOLD |
| 3 | Drawback | PERCENT_DAMAGE |
| 3 | Reward | TRANSFORM_TWO_CARDS |
| 4 | Boss Swap | Coffee Dripper |

### Floor 1
- **Enemy:** Small Slimes (Acid Slime S + Spike Slime M)
  - Acid Slime S: 8 HP, 3 damage attack
  - Spike Slime M: 31 HP, inflicts Frail
- **Rewards:** 11 gold
- **Card Rewards:** Talk to the Hand, Third Eye, Empty Body
- **Potion:** Fairy in a Bottle

### Floor 2
- **Enemy:** Jaw Worm
  - HP: 41
  - First turn: 11 damage attack
  - Second turn: Block and buff intent
- **Rewards:** Blessing of the Forge (upgrade)
- **Card Rewards:** Sands of Time, Simmering Fury, Tranquility
- **Gold Total:** 124 (after floor 2)

### Floor 3
- **Enemy:** Cultist
  - HP: 49
- **Rewards:** 19 gold
- **Card Rewards:** Meditate, Pressure Points, Signature Move

### First ? Room (Floor 3 area)
- **Event:** Wing Statue (Golden Wing)
- **Options:**
  - Pray: Remove card for 7 HP
  - Destroy: Gain 50-80 Gold
  - Leave

---

## Seed: 1ABCD

**Seed Value:** 1943283
**Character:** Watcher
**Ascension:** 0

### Floor 1
- **Enemy:** Jaw Worm
  - HP: 40
  - First turn: 11 damage attack
- **Card Rewards:** Like Water, Bowling Bash, Deceive Reality

### Floor 2
- **Enemy:** Cultist
  - HP: 51
- **Card Rewards:** Sash Whip, Evaluate, Worship

### Floor 3
- **Enemy:** Small Slimes

---

## Seed: GA (Calling Bell Test)

**Seed Value:** 570
**Character:** Watcher
**Ascension:** 0

### Neow Boss Swap
- **Boss Relic:** Calling Bell (VERIFIED)
- **Calling Bell Relics:** Lantern (C), Eternal Feather (U), Shovel (R) (VERIFIED)
- **Prediction was wrong:** Predicted Vajra, Yang, Ginger

### Floor 1
- **Enemy:** Cultist 52 HP (VERIFIED - matches prediction)
- **Gold:** 17
- **Card Rewards:** Conclude, Empty Fist, Flurry of Blows
  - Note: Prediction had these for Floor 2, off by 1

### Floor 2
- **Enemy:** 2 Louse (11 HP attacking 7, 14 HP buffing) (VERIFIED)
- **Gold:** 20
- **Card Rewards:** Halt, Wallop, Consecrate
  - Note: Prediction had these for Floor 3, off by 1

### Investigation Notes
- Encounters: CORRECT
- Card rewards: Off by 1 floor (something consuming cardRng before floor 1)
- Boss swap prediction: CORRECT (Calling Bell at position 0)
- Calling Bell relics: WRONG - predicted indices 0,0,0 but actual were at indices 9, 37, 17

### ROOT CAUSE ANALYSIS (Jan 16 2026)

**Issue 1: Calling Bell Relic Predictions Wrong**

**Root Cause: Locked relics are EXCLUDED from pools before shuffling**

From `RelicLibrary.java` line 626:
```java
if (r.getValue().tier != tier || UnlockTracker.isRelicLocked(r.getKey()) && !Settings.treatEverythingAsUnlocked()) continue;
pool.add(r.getKey());
```

This means:
- If any relics are locked (not yet unlocked through gameplay), they are **completely excluded** from the relic pool
- The pool is then shuffled with `Collections.shuffle(pool, new java.util.Random(relicRng.randomLong()))`
- Our prediction assumes ALL relics are available, but the actual game has some locked
- Result: Different pool composition → different shuffle result → wrong relic at index 0

**Why Boss Pool Was Correct:** All boss relics are likely unlocked (fewer total, unlocked earlier through gameplay).

**Solution:** Unlock all content in the game using one of these methods:

**Method 1: Run the unlock script (Recommended)**
```bash
uv run python core/utils/generate_full_unlocks.py --backup --apply
```
This backs up your current unlocks and writes a complete unlock file.

**Method 2: Manual unlock via BaseMod console**
1. Enable developer console in BaseMod settings
2. Use `relic [ID]` commands to obtain locked relics (this marks them as unlocked)

**Method 3: Edit STSUnlocks file directly**
Location: `~/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/preferences/STSUnlocks`
Add all relics with value "2" for unlocked.

**Issue 2: Card Rewards Shifted by 1 Floor**

**Finding: NeowEvent uses SEPARATE RNG stream**

From `NeowEvent.java` line 356:
```java
rng = new Random(Settings.seed);  // Fresh RNG, NOT AbstractDungeon.cardRng
```

- `NeowEvent.rng` is completely separate from `AbstractDungeon.cardRng`
- The 3-call shift is NOT from Neow option generation consuming cardRng directly
- Root cause still under investigation - likely somewhere during dungeon initialization or first floor setup

**Detailed Analysis:**

NeowReward uses `NeowEvent.rng` (seeded with `Settings.seed`), NOT `AbstractDungeon.cardRng`.
Combat rewards use `AbstractDungeon.cardRng.random(99)` for rarity rolls.

The 3-call shift suggests something calls `getRewardCards()` during Exordium initialization
BEFORE the first actual combat, consuming 3 `cardRng.random(99)` calls.

**Possible causes:**
1. Neow THREE_CARDS option pre-generates cards during option creation (even if not selected)
2. Some initialization code pre-generates first floor rewards
3. A reward preview mechanism that advances cardRng

**Solution Implemented:**
The `SeedPredictor` class now has a `card_rng_floor_offset` parameter.

### Neow Option cardRng Consumption (Jan 16 2026)

| Neow Option | cardRng Consumed | Offset | Notes |
|-------------|------------------|--------|-------|
| UPGRADE_CARD | None | 0 | ✓ Verified (seed B) |
| HUNDRED_GOLD | None | 0 | ✓ Verified (seed A) |
| TEN_PERCENT_HP_BONUS | None | 0 | Expected |
| RANDOM_COMMON_RELIC | None | 0 | Expected |
| THREE_ENEMY_KILL | None | 0 | ✓ Verified (seed N) |
| Boss Swap (most relics) | None | 0 | ✓ Verified (seed B - Astrolabe) |
| Boss Swap (Calling Bell) | ~3 calls (1 floor shift) | 1 | ✓ Verified (seed GA) |
| THREE_CARDS | None (uses NeowEvent.rng) | 0 | ✓ Verified (seed G) |
| THREE_RARE_CARDS | Variable | ??? | May use cardRng for card selection |
| RANDOM_COLORLESS | 3+ calls | Variable | ✓ Verified consumes cardRng (seed F) - see analysis |
| RANDOM_COLORLESS_2 | 3+ calls | Variable | ✓ Verified consumes cardRng (seed C) |
| ONE_RANDOM_RARE_CARD | None (uses NeowEvent.rng) | 0 | ✓ Verified (seed D) |
| TRANSFORM_CARD | None (uses NeowEvent.rng) | 0 | ✓ Verified (seed GA) |
| REMOVE_CARD | None | 0 | ✓ Verified (seed H) |
| PERCENT_DAMAGE drawback | None | 0 | ✓ Verified (seed I) |
| CURSE drawback | 1 call | Variable | Curse selection consumes cardRng |

### Code Evidence

**Colorless cards consume cardRng (3+ calls per selection):**
```java
// NeowReward.getColorlessRewardCards() lines 294-315
// For each of 3 cards:
//   1. rollRarity() uses NeowEvent.rng (safe)
//   2. getColorlessCardFromPool(rarity) uses cardRng (consumes!)
//   3. Duplicate check loop may consume additional cardRng calls

// AbstractDungeon.getColorlessCardFromPool() -> CardGroup.getRandomCard()
public AbstractCard getRandomCard(boolean useRng, AbstractCard.CardRarity rarity) {
    // ... filter by rarity ...
    if (useRng) {
        return tmp.get(AbstractDungeon.cardRng.random(tmp.size() - 1));  // Line 504
    }
}
```
Minimum 3 calls (one per card), but duplicate checks can consume more.

**Neow rare cards use NeowEvent.rng (NOT cardRng):**
```java
// NeowReward.getCard()
AbstractDungeon.rareCardPool.getRandomCard(NeowEvent.rng)  // Uses separate RNG
```

**Transform uses NeowEvent.rng:**
```java
// NeowReward line 143
AbstractDungeon.transformCard(..., NeowEvent.rng)
```

**CURSE drawback consumes cardRng:**
```java
// CardLibrary.getCurse() line 1014
return cards.get(tmp.get(AbstractDungeon.cardRng.random(0, tmp.size() - 1)));
```
The CURSE drawback calls `cardRng.random(0, 9)` to select from 10 available curses (excluding AscendersBane, Necronomicurse, CurseOfTheBell, Pride). This single call shifts the entire RNG stream, causing variable apparent offsets depending on how the RNG values align with subsequent card generation.

### Verified Test Results

| Seed | Neow Choice | Floor 1 Cards | Offset | Notes |
|------|-------------|---------------|--------|-------|
| TEST123 | Unknown | Talk to the Hand, Third Eye, Empty Body | 0 | |
| GA | Calling Bell | Conclude, Empty Fist, Flurry of Blows | 1 | |
| GA | Transform Card | Mental Fortress, Cut Through Fate, Empty Body | 0 | |
| B | Astrolabe | Follow-Up, Crescendo, Pressure Points | 0 | |
| B | Upgrade Card | Follow-Up, Crescendo, Pressure Points | 0 | |
| B | CURSE+THREE_RARE | Empty Fist, Conclude, Fear No Evil | Variable | |
| C | CURSE+COLORLESS_2 | Follow-Up, Pray, Evaluate | Variable | |
| A | HUNDRED_GOLD | Pray, Weave, Foreign Influence | 0 | ✓ All 3 floors match |
| H | REMOVE_CARD | Bowling Bash, Wallop, Collect | 0 | ✓ Verified |
| I | PERCENT_DAMAGE -> 20% HP | Tantrum, Pray, Evaluate | 0 | ✓ All 3 floors match |
| G | THREE_CARDS | Empty Body, Third Eye, Sash Whip | 0 | ✓ THREE_CARDS uses NeowEvent.rng |
| P | CURSE -> ONE_RARE_RELIC | Worship, Empty Body, Third Eye | Variable | Curse=Decay, F2/F3 match offset=0 |
| R | CURSE -> ONE_RARE_RELIC | Protect, Deceive Reality, Empty Body | Variable | Curse=Parasite, no simple offset match |
| D | ONE_RANDOM_RARE_CARD | Inner Peace, Perseverance, Tranquility | 0 | ✓ Uses NeowEvent.rng |
| F | RANDOM_COLORLESS | Like Water, Pressure Points, Prostrate | Variable | ✓ Consumes 3+ cardRng calls |
| N | THREE_ENEMY_KILL | Sanctity, Meditate, Talk to the Hand | 0 | ✓ Verified |

### Practical Usage

```python
# Simple Neow options (upgrade, gold, HP, non-card relics)
predictor = SeedPredictor("SEED", card_rng_floor_offset=0)

# Calling Bell boss swap specifically
predictor = SeedPredictor("SEED", card_rng_floor_offset=1)

# Card-granting Neow options - prediction unreliable without simulating exact consumption
```

**Code locations:**
- `AbstractDungeon.getColorlessCardFromPool()` - uses cardRng
- `NeowReward.getCard()` - uses NeowEvent.rng (safe)
- `NeowReward.getColorlessRewardCards()` - calls getColorlessCardFromPool (uses cardRng)

---

---

## Edge Case Test Seeds (Boss Relics)

These seeds test specific boss relic predictions:

| Seed | Boss Swap | Test Purpose |
|------|-----------|--------------|
| `GA` | Calling Bell | Tests full relic init (grants 3 random relics + curse) |
| `B` | Astrolabe | First in shared BOSS list - early HashMap iteration |
| `GC` | VioletLotus | Watcher-specific boss relic |
| `Y` | HolyWater | Watcher-specific with canSpawn() check |
| `RELIC` | Tiny House | Late in list |
| `L` | Velvet Choker | Late in list |

**Verification Status:** GA verified (relics wrong, encounters correct)

---

## Notes

### Shop and Event cardRng Consumption

**CRITICAL:** Shops consume `cardRng` when generating their card inventory!

| Room Type | Consumes cardRng | Notes |
|-----------|------------------|-------|
| **Shop** | ✅ YES | `Merchant.java:54-80` - rolls for each card |
| Shrines | ❌ NO | Uses `miscRng` for transforms |
| Events (transform) | ❌ NO | Uses `miscRng` for transforms |
| Treasure | ❌ NO | Uses `treasureRng` |
| Combat | ✅ YES | Card rewards |

**Implication:** Card reward predictions are only accurate if you track the path taken. Visiting a shop between combats will shift all subsequent card predictions.

```java
// Merchant.java lines 54-78
// Each shop card uses cardRng:
AbstractCard c = AbstractDungeon.getCardFromPool(
    AbstractDungeon.rollRarity(),  // Uses cardRng
    AbstractCard.CardType.ATTACK, true
).makeCopy();
```

### RNG Streams
- `cardRng`: Card rewards, shop cards (persistent across floors)
- `monsterRng`: Encounter selection
- `monsterHpRng`: Enemy HP (reseeded per floor: `seed + floorNum`)
- `relicRng`: Relic pool shuffles and relic drops
- `eventRng`: Event selection in ? rooms

### Boss Relic Pool Order
The boss relic pool is populated by iterating the full sharedRelics HashMap (138 entries)
and class-specific HashMap, filtering to BOSS tier. The iteration order depends on ALL
entries in the HashMap due to Java HashMap's bucket-based iteration.
