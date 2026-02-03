# Card Reward Mechanics

Extracted from decompiled source: AbstractDungeon.java, RewardItem.java, AbstractRoom.java

## Overview

Card rewards are generated when:
- Defeating non-boss combats
- Certain events offer card rewards
- Boss fights (use same system)

---

## Number of Cards Shown

### Base: 3 cards

```java
int numCards = 3;
for (AbstractRelic r : AbstractDungeon.player.relics) {
    numCards = r.changeNumberOfCardsInReward(numCards);
}
```

### Modifiers

| Relic | Effect | Result |
|-------|--------|--------|
| Question Card | +1 card | 4 cards |
| Busted Crown | -2 cards | 1 card |
| Both | Net -1 | 2 cards |
| Binary (daily mod) | -1 card | Stacks |

---

## Rarity Roll System

### Base Rarity Thresholds (AbstractRoom.java)
```java
baseRareCardChance = 3;      // 3% base rare chance
baseUncommonCardChance = 37; // 37% uncommon chance
// Remaining = 60% common

// Roll interpretation:
roll < rareCardChance:                         RARE
roll < rareCardChance + uncommonCardChance:    UNCOMMON
else:                                          COMMON
```

### Roll Calculation
```java
// AbstractDungeon.rollRarity()
int roll = cardRng.random(99);   // 0-99
roll += cardBlizzRandomizer;      // Modified by card blizzard system
return getCurrRoom().getCardRarity(roll);
```

---

## Card Blizzard System (Pity Timer)

Ensures players eventually get rare/uncommon cards after streaks of commons.

### Variables
```java
cardBlizzStartOffset = 5;    // Starting bonus
cardBlizzRandomizer = 5;     // Current bonus (modified per roll)
cardBlizzGrowth = 1;         // How much bonus increases per common
cardBlizzMaxOffset = -40;    // Maximum bonus (makes rare guaranteed)
```

### How It Works
```java
// After each card is rolled:
switch (rarity) {
    case RARE:
        cardBlizzRandomizer = cardBlizzStartOffset;  // Reset to +5
        break;
    case UNCOMMON:
        // No change
        break;
    case COMMON:
        cardBlizzRandomizer -= cardBlizzGrowth;  // Decrease by 1
        if (cardBlizzRandomizer > cardBlizzMaxOffset) {
            cardBlizzRandomizer = cardBlizzMaxOffset;  // Cap at -40
        }
        break;
}
```

### Effect
- Start: roll + 5 (slightly higher rare/uncommon chance)
- Each common: -1 to modifier
- After 45 commons: roll + (-40), guaranteeing at least uncommon
- Getting a rare: resets modifier to +5

---

## Room-Specific Rarity Modifiers

### Elite Rooms (MonsterRoomElite.java)
```java
// Elite Swarm daily mod:
if (ModHelper.isModEnabled("Elite Swarm")) {
    return CardRarity.RARE;  // Always rare from elites
}
// Otherwise uses normal rarity system
```

### Shop Rooms (ShopRoom.java)
```java
// Enhanced rare chance for shop cards:
baseRareCardChance = 9;     // 9% (vs 3% normal)
baseUncommonCardChance = 37; // 37% (same)
```

**Important**: Shop cards use `getCardRarity(roll, false)` which means:
1. The card blizzard (pity timer) system is NOT applied to shop card generation
2. Each shop card roll is independent - no rare card streaks protection
3. See `docs/vault/shop-mechanics.md` for full shop generation details

---

## Relic Effects on Rarity

### Rarity Chance Modifiers
```java
// During rarity determination:
for (AbstractRelic r : AbstractDungeon.player.relics) {
    rareCardChance = r.changeRareCardRewardChance(rareCardChance);
}
for (AbstractRelic r : AbstractDungeon.player.relics) {
    uncommonCardChance = r.changeUncommonCardRewardChance(uncommonCardChance);
}
```

Notable relics:
- **Nloth's Gift**: Can affect rarity chances
- **Prayer Wheel**: Adds second card reward (separate roll)

---

## Upgrade Chance

### Standard Upgrade Chance
```java
// After generating card list:
for (AbstractCard c : retVal2) {
    // Rare cards are NEVER auto-upgraded
    if (c.rarity != CardRarity.RARE
        && cardRng.randomBoolean(cardUpgradedChance)
        && c.canUpgrade()) {
        c.upgrade();
    }
}
```

### Upgrade Chance Value
- `cardUpgradedChance` - Set per act/dungeon
- Typically around 0.0 to 0.25 depending on act

---

## Card Pool Selection

### Standard Rewards
```java
// AbstractDungeon.getRewardCards()
for (int i = 0; i < numCards; ++i) {
    CardRarity rarity = rollRarity();
    AbstractCard card = null;

    // Get card from appropriate pool
    card = player.hasRelic("PrismaticShard")
        ? CardLibrary.getAnyColorCard(rarity)  // Any color
        : AbstractDungeon.getCard(rarity);     // Player's color

    // Ensure no duplicates in same reward
    while (retVal.contains(card)) {
        card = getCard(rarity);  // Re-roll if duplicate
    }
    retVal.add(card);
}
```

### Card Pools
```java
// AbstractDungeon.getCard(rarity)
switch (rarity) {
    case RARE:     return rareCardPool.getRandomCard(true);
    case UNCOMMON: return uncommonCardPool.getRandomCard(true);
    case COMMON:   return commonCardPool.getRandomCard(true);
    case CURSE:    return curseCardPool.getRandomCard(true);
}
```

---

## Colorless Card Rewards

### Generation (AbstractDungeon.getColorlessRewardCards)
```java
// Colorless rewards only have UNCOMMON or RARE
for (int i = 0; i < numCards; ++i) {
    CardRarity rarity = rollRareOrUncommon(colorlessRareChance);
    AbstractCard card = getColorlessCardFromPool(rarity);

    // Ensure no duplicates
    while (retVal.contains(card)) {
        card = getColorlessCardFromPool(rarity);
    }
    retVal.add(card);
}
```

### Colorless Rarity Roll
```java
// rollRareOrUncommon(float rareChance)
if (cardRng.randomBoolean(rareChance)) {
    return CardRarity.RARE;
}
return CardRarity.UNCOMMON;
```

`colorlessRareChance` is typically ~0.33 (33% rare, 67% uncommon)

---

## Singing Bowl Relic

### Effect
When present, adds a "Skip for +2 Max HP" option to card rewards.

```java
// SingingBowl.java
public static final int HP_AMT = 2;

// When skip is selected, player gains 2 max HP
// (Implementation in CardRewardScreen, not in relic file)
```

### Spawn Condition
```java
public boolean canSpawn() {
    return Settings.isEndless || AbstractDungeon.floorNum <= 48;
}
// Only appears in Act 1-3, not Act 4
```

---

## Question Card Relic

### Effect
```java
// QuestionCard.java
private static final int CARDS_ADDED = 1;

@Override
public int changeNumberOfCardsInReward(int numberOfCards) {
    return numberOfCards + 1;
}
```

### Spawn Condition
```java
public boolean canSpawn() {
    return Settings.isEndless || AbstractDungeon.floorNum <= 48;
}
// Only appears in Act 1-3, not Act 4
```

---

## Busted Crown Relic

### Effect
```java
// BustedCrown.java
private static final int CARDS_SUBTRACTED = 2;

@Override
public int changeNumberOfCardsInReward(int numberOfCards) {
    return numberOfCards - 2;
}

// Also grants +1 energy
@Override
public void onEquip() {
    ++AbstractDungeon.player.energy.energyMaster;
}
```

---

## Reward Item Types

```java
public enum RewardType {
    CARD,
    GOLD,
    RELIC,
    POTION,
    STOLEN_GOLD,
    EMERALD_KEY,
    SAPPHIRE_KEY
}
```

Card rewards specifically:
```java
// RewardItem constructor for cards
public RewardItem() {
    this.type = RewardType.CARD;
    this.isBoss = AbstractDungeon.getCurrRoom() instanceof MonsterRoomBoss;
    this.cards = AbstractDungeon.getRewardCards();
    this.text = TEXT[2];  // "Card"
}
```

---

## Summary: Rarity Probabilities

### Normal Combat (with card blizzard at start +5)
| Rarity | Base % | After +5 Modifier |
|--------|--------|-------------------|
| Rare | 3% | ~8% |
| Uncommon | 37% | ~37% |
| Common | 60% | ~55% |

### After Long Common Streak (modifier at -40)
| Rarity | Roll Range | Chance |
|--------|------------|--------|
| Rare | roll < -37 | ~3% |
| Uncommon | roll < -3 | ~97% |
| Common | Never | 0% |

### Shop Cards
| Rarity | Chance |
|--------|--------|
| Rare | 9% |
| Uncommon | 37% |
| Common | 54% |

### Colorless Rewards
| Rarity | Chance |
|--------|--------|
| Rare | ~33% (colorlessRareChance) |
| Uncommon | ~67% |

---

## RNG Seeds

Card rewards use `cardRng` for:
- Rarity rolling
- Card selection from pool
- Upgrade chance
- Duplicate avoidance re-rolls

This ensures deterministic card rewards based on seed and decision order.

---

## Implementation

Our implementation is in `packages/engine/generation/rewards.py`. Key classes and functions:

- `CardBlizzardState`: Tracks the pity timer modifier
- `roll_card_rarity()`: Performs rarity roll with blizzard system
- `generate_card_rewards()`: Full card reward generation
- `ShopGenerator`: Generates shop inventory (separate from combat rewards)

### Rarity Threshold Constants

From our implementation matching decompiled source:

```python
CARD_RARITY_THRESHOLDS = {
    "normal": {"rare": 3, "uncommon": 37},
    "elite": {"rare": 10, "uncommon": 40},
    "shop": {"rare": 3, "uncommon": 37},  # Same as normal, but no blizzard
}

CARD_BLIZZ_START_OFFSET = 5   # Initial +5 to roll
CARD_BLIZZ_GROWTH = 1         # -1 per common card
CARD_BLIZZ_MAX_OFFSET = -40   # Maximum penalty (guarantees uncommon+)
```

### Key Difference: Elite vs Normal Rewards

Elites have higher rare chance (10% vs 3%) and uncommon threshold (40 vs 37), making them significantly better for rare cards.
