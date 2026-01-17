# Shop Mechanics

Extracted from decompiled source: ShopScreen.java, StorePotion.java, StoreRelic.java, Merchant.java

## Shop Layout

The shop contains:
- **5 colored cards** (top row): 2 Attacks, 2 Skills, 1 Power
- **2 colorless cards** (bottom row): 1 Uncommon, 1 Rare
- **3 relics**: 2 rolled tier, 1 guaranteed SHOP tier
- **3 potions**: Random from pool
- **Card removal service**: Available once per shop

---

## Card Pricing

### Base Prices by Rarity
```java
// AbstractCard.getPrice(CardRarity rarity)
COMMON:   50 gold
UNCOMMON: 75 gold
RARE:     150 gold
BASIC:    9999 (error case)
SPECIAL:  9999 (error case)
```

### Price Variation Formula
```java
// Colored cards:
price = basePrice * merchantRng.random(0.9f, 1.1f)

// Colorless cards get 20% markup:
price = basePrice * merchantRng.random(0.9f, 1.1f) * 1.2f
```

**Effective colorless prices:**
- Uncommon colorless: 75 * 0.9-1.1 * 1.2 = 81-99 gold (avg 90)
- Rare colorless: 150 * 0.9-1.1 * 1.2 = 162-198 gold (avg 180)

### On-Sale Discount
```java
// One random colored card (index 0-4) is selected for 50% off
AbstractCard saleCard = coloredCards.get(merchantRng.random(0, 4));
saleCard.price /= 2;
```

---

## Card Pool Generation

### Shop Card Selection (Merchant.java)
```java
// 2 Attack cards (different from each other, not colorless)
cards1.add(getCardFromPool(rollRarity(), ATTACK, true));  // Rolls rarity using shop's enhanced chances
cards1.add(getCardFromPool(rollRarity(), ATTACK, true));  // Must differ from previous

// 2 Skill cards (different from each other, not colorless)
cards1.add(getCardFromPool(rollRarity(), SKILL, true));
cards1.add(getCardFromPool(rollRarity(), SKILL, true));  // Must differ from previous

// 1 Power card (not colorless)
cards1.add(getCardFromPool(rollRarity(), POWER, true));

// 2 Colorless cards (fixed rarity)
cards2.add(getColorlessCardFromPool(UNCOMMON));
cards2.add(getColorlessCardFromPool(RARE));
```

### Shop Rarity Chances (ShopRoom.java)
```java
// ShopRoom overrides base rarity chances:
baseRareCardChance = 9;     // vs 3 for normal rooms
baseUncommonCardChance = 37; // same as normal

// Roll interpretation for shop cards:
roll < 9:           RARE
roll < 9+37 (46):   UNCOMMON
roll >= 46:         COMMON
```

The shop uses `getCardRarity(roll, false)` which means rarity alternation (card blizzard system) is NOT applied to shop card generation.

---

## Relic Pricing

### Base Prices by Tier
```java
// AbstractRelic.getPrice()
STARTER:    300 gold
COMMON:     150 gold
UNCOMMON:   250 gold
RARE:       300 gold
SHOP:       150 gold
SPECIAL:    400 gold
BOSS:       999 gold
DEPRECATED: -1
```

### Price Variation
```java
// Applied in ShopScreen.initRelics():
price = basePrice * merchantRng.random(0.95f, 1.05f)
```

### Shop Relic Selection
```java
// 3 relics total:
for (int i = 0; i < 3; ++i) {
    if (i != 2) {
        relic = returnRandomRelicEnd(rollRelicTier());  // Slots 0,1: rolled tier
    } else {
        relic = returnRandomRelicEnd(SHOP);  // Slot 2: guaranteed SHOP tier
    }
}
```

### Shop Relic Tier Roll
```java
// ShopScreen.rollRelicTier()
int roll = merchantRng.random(99);
if (roll < 48):      return COMMON;    // 48%
if (roll < 82):      return UNCOMMON;  // 34%
return RARE;                           // 18%
```

---

## Potion Pricing

### Base Prices by Rarity (from AbstractPotion.getPrice)
```java
// Potion prices are determined by potion rarity:
COMMON:   50 gold
UNCOMMON: 75 gold
RARE:     100 gold
```

### Price Variation
```java
// Applied in ShopScreen.initPotions():
price = basePrice * merchantRng.random(0.95f, 1.05f)
```

### Potion Rarity Roll
```java
// From AbstractDungeon.returnRandomPotion():
int roll = potionRng.random(0, 99);
if (roll < 65):      return COMMON;    // 65%
if (roll < 65+25):   return UNCOMMON;  // 25%
return RARE;                           // 10%
```

---

## Card Removal Service

### Base Cost and Scaling
```java
public static int purgeCost = 75;       // Base cost
private static final int PURGE_COST_RAMP = 25;  // Cost increase per removal

// After each removal:
purgeCost += 25;  // Increases permanently for the run
```

**Removal cost progression:**
- 1st removal: 75 gold
- 2nd removal: 100 gold
- 3rd removal: 125 gold
- nth removal: 75 + (n-1)*25 gold

### Smiling Mask Effect
```java
// With Smiling Mask relic:
actualPurgeCost = 50;  // Fixed at 50, ignores scaling
```

---

## Discount Relics

### The Courier (20% discount + restocking)
```java
// Applies 0.8x multiplier to all prices
applyDiscount(0.8f, true);  // true = affects purge cost too

// After purchase, item restocks with new item
// Replaces sold item with new random item of same category
```

### Membership Card (50% discount)
```java
// Applies 0.5x multiplier to all prices
applyDiscount(0.5f, true);

// If bought during shop visit, immediately applies to remaining items
if (relic.relicId.equals("Membership Card")) {
    shopScreen.applyDiscount(0.5f, true);
}
```

### Combined Discount
```java
// Both relics stack multiplicatively:
// Courier (0.8) + Membership (0.5) = 0.4x final price

// For card removal with both:
actualPurgeCost = MathUtils.round(purgeCost * 0.8f * 0.5f);
```

---

## Ascension 16+ Effect

```java
// At A16+, all shop prices increased by 10%
if (AbstractDungeon.ascensionLevel >= 16) {
    applyDiscount(1.1f, false);  // 1.1x multiplier, doesn't affect purge
}
```

Note: The 1.1x multiplier is applied BEFORE discount relics, so discounts still work normally.

---

## RNG Seeds

The shop uses `merchantRng` for:
- Card price variation
- Relic price variation
- Potion price variation
- On-sale card selection
- Relic tier rolling
- Card rarity rolling for shop cards

This is a separate RNG stream from combat, allowing deterministic shop generation based on seed.

---

## Summary Table

| Item Type | Base Price | Variation | Notes |
|-----------|------------|-----------|-------|
| Common Card | 50 | +/-10% | |
| Uncommon Card | 75 | +/-10% | |
| Rare Card | 150 | +/-10% | |
| Colorless Uncommon | 90 | +/-10% | 1.2x markup |
| Colorless Rare | 180 | +/-10% | 1.2x markup |
| Common Relic | 150 | +/-5% | |
| Uncommon Relic | 250 | +/-5% | |
| Rare Relic | 300 | +/-5% | |
| Shop Relic | 150 | +/-5% | Guaranteed slot |
| Common Potion | 50 | +/-5% | |
| Uncommon Potion | 75 | +/-5% | |
| Rare Potion | 100 | +/-5% | |
| Card Removal | 75+25n | None | n = # previous removals |
