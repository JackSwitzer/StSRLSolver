# Slay the Spire Event Mechanics

Comprehensive documentation of all event mechanics extracted from decompiled source code.

---

## Table of Contents
1. [Neow Bonus Options](#neow-bonus-options)
2. [Exordium Events (Act 1)](#exordium-events-act-1)
3. [City Events (Act 2)](#city-events-act-2)
4. [Beyond Events (Act 3)](#beyond-events-act-3)
5. [Shrine Events (Any Act)](#shrine-events-any-act)
6. [Act 4 Events](#act-4-events)
7. [Notes on RNG](#notes-on-rng)

---

## Neow Bonus Options

**Source:** `NeowReward.java`

Neow offers rewards based on categories. Each option is randomly selected from available options in that category.

### Category 0 - Basic Rewards (First Blessing)
| Reward Type | Effect |
|-------------|--------|
| THREE_CARDS | Choose 1 of 3 class cards |
| ONE_RANDOM_RARE_CARD | Obtain 1 random rare card |
| REMOVE_CARD | Remove 1 card from deck |
| UPGRADE_CARD | Upgrade 1 card |
| TRANSFORM_CARD | Transform 1 card |
| RANDOM_COLORLESS | Choose 1 of 3 colorless cards |

### Category 1 - Small Rewards (Mini Bonus)
| Reward Type | Effect |
|-------------|--------|
| THREE_SMALL_POTIONS | Obtain 3 random potions |
| RANDOM_COMMON_RELIC | Obtain 1 random common relic |
| TEN_PERCENT_HP_BONUS | Gain 10% max HP |
| THREE_ENEMY_KILL | Obtain Neow's Lament (first 3 enemies have 1 HP) |
| HUNDRED_GOLD | Gain 100 gold |

### Category 2 - Major Rewards with Drawbacks
**Possible Drawbacks:**
| Drawback | Effect |
|----------|--------|
| TEN_PERCENT_HP_LOSS | Lose 10% max HP |
| NO_GOLD | Lose all gold |
| CURSE | Obtain a random curse |
| PERCENT_DAMAGE | Take damage equal to 30% current HP |

**Possible Rewards:**
| Reward Type | Effect | Notes |
|-------------|--------|-------|
| RANDOM_COLORLESS_2 | Choose 1 of 3 rare colorless cards | |
| REMOVE_TWO | Remove 2 cards | Not available with CURSE drawback |
| ONE_RARE_RELIC | Obtain 1 random rare relic | |
| THREE_RARE_CARDS | Choose 1 of 3 rare cards | |
| TWO_FIFTY_GOLD | Gain 250 gold | Not available with NO_GOLD drawback |
| TRANSFORM_TWO_CARDS | Transform 2 cards | |
| TWENTY_PERCENT_HP_BONUS | Gain 20% max HP | Not available with TEN_PERCENT_HP_LOSS drawback |

### Category 3 - Boss Swap
| Reward Type | Effect |
|-------------|--------|
| BOSS_RELIC | Lose starting relic, gain random boss relic |

---

## Exordium Events (Act 1)

### Big Fish
**ID:** `Big Fish`

| Option | Effect |
|--------|--------|
| Banana | Heal 33% max HP |
| Donut | Gain 5 max HP |
| Box | Obtain random relic (any tier) + Regret curse |

---

### The Cleric
**ID:** `The Cleric`

| Option | Cost | Effect | A15+ |
|--------|------|--------|------|
| Heal | 35 gold | Heal 25% max HP | Same |
| Purify | 50 gold | Remove 1 card | 75 gold |
| Leave | - | Nothing | - |

---

### Dead Adventurer
**ID:** `Dead Adventurer`

**Mechanics:**
- Rewards pool: GOLD (30g), NOTHING, RELIC (random tier) - shuffled randomly
- Encounter chance starts at 25% (A15+: 35%)
- Each search increases chance by 25%
- Max 3 searches before all rewards found

| Option | Effect |
|--------|--------|
| Search | Random reward from pool OR trigger elite fight |
| Leave | Exit event |

**Possible Enemies:**
- 3 Sentries
- Gremlin Nob
- Lagavulin (awakened)

**Combat Rewards:** 25-35 gold + any remaining rewards from pool

---

### Golden Idol
**ID:** `Golden Idol`

**Phase 1:**
| Option | Effect |
|--------|--------|
| Take Idol | Obtain Golden Idol relic, triggers boulder trap |
| Leave | Nothing |

**Phase 2 (after taking idol):**
| Option | Effect | A15+ |
|--------|--------|------|
| Outrun | Obtain Injury curse | Same |
| Push through | Take 25% max HP damage (HP loss) | 35% max HP |
| Hide | Lose 8% max HP permanently (min 1) | 10% max HP |

**Note:** If player already has Golden Idol, obtains Circlet instead.

---

### Golden Wing
**ID:** `Golden Wing`

| Option | Condition | Effect |
|--------|-----------|--------|
| Offer blood | Always | Take 7 damage (HP loss), then remove 1 card |
| Pray | Have card with 10+ damage | Gain 50-80 gold (RNG) |
| Leave | Always | Nothing |

**Note:** The "Pray" option requires having an Attack card with 10+ base damage in deck.

---

### World of Goop
**ID:** `World of Goop`

| Option | Effect | A15+ |
|--------|--------|------|
| Gather gold | Take 11 damage (HP loss), gain 75 gold | Same |
| Leave | Lose 20-50 gold (RNG, capped at current gold) | Lose 35-75 gold |

---

### Living Wall
**ID:** `Living Wall`

| Option | Effect |
|--------|--------|
| Forget | Remove 1 card |
| Change | Transform 1 card |
| Grow | Upgrade 1 card (requires upgradable card) |

---

### Mushrooms
**ID:** `Mushrooms`

| Option | Effect |
|--------|--------|
| Fight | Fight Fungi Beasts, gain 20-30 gold + Odd Mushroom relic |
| Eat | Heal 25% max HP, obtain Parasite curse |

---

### Scrap Ooze
**ID:** `Scrap Ooze`

**Mechanics:**
- Starting chance: 25%
- Each failed attempt: +10% chance, +1 damage
- Starting damage: 3 (A15+: 5)

| Option | Effect |
|--------|--------|
| Reach | Take damage, chance to obtain random relic |
| Leave | Exit with damage taken |

---

### Shining Light
**ID:** `Shining Light`

| Option | Effect | A15+ |
|--------|--------|------|
| Enter | Take 20% max HP damage, upgrade 2 random cards | 30% max HP |
| Leave | Nothing | - |

---

### Liars Game (Sssserpent)
**ID:** `Liars Game`

| Option | Effect | A15+ |
|--------|--------|------|
| Agree | Gain 175 gold + Doubt curse | 150 gold |
| Disagree | Nothing | - |

---

## City Events (Act 2)

### Addict (Beginner's Trap)
**ID:** `Addict`

| Option | Effect |
|--------|--------|
| Pay 85 gold | Obtain random relic |
| Steal | Obtain random relic + Shame curse |
| Leave | Nothing |

---

### Back to Basics
**ID:** `Back to Basics`

| Option | Effect |
|--------|--------|
| Elegance | Remove 1 card |
| Simplicity | Upgrade all Strikes and Defends |

---

### Beggar
**ID:** `Beggar`

| Option | Effect |
|--------|--------|
| Pay 75 gold | Remove 1 card |
| Leave | Nothing |

---

### Colosseum
**ID:** `Colosseum`

**Phase 1:** Fight Colosseum Slavers (no rewards)

**Phase 2:**
| Option | Effect |
|--------|--------|
| Fight | Fight Colosseum Nobs, rewards: Rare relic, Uncommon relic, 100 gold |
| Flee | Exit event |

---

### Cursed Tome
**ID:** `Cursed Tome`

**Reading sequence (cumulative damage):**
1. Open book: 1 HP loss
2. Turn page: 2 HP loss
3. Continue: 3 HP loss
4. Finish reading: 10 HP loss (A15+: 15 HP loss)

| Option | Effect |
|--------|--------|
| Read | Progress through pages, obtain book relic |
| Stop | Take 3 damage and leave |
| Leave | Nothing |

**Book relics (random selection):** Necronomicon, Enchiridion, Nilry's Codex

---

### Drug Dealer (Augmenter)
**ID:** `Drug Dealer`

| Option | Effect |
|--------|--------|
| Ingest | Obtain J.A.X. card |
| Test Subject | Transform 2 cards |
| Inject | Obtain Mutagenic Strength relic |

---

### Forgotten Altar
**ID:** `Forgotten Altar`

| Option | Condition | Effect | A15+ |
|--------|-----------|--------|------|
| Sacrifice | Have Golden Idol | Golden Idol -> Bloody Idol | Same |
| Shed Blood | Always | Gain 5 max HP, take 25% max HP damage | 35% max HP |
| Smash | Always | Obtain Decay curse | Same |

---

### Ghosts (Council of Ghosts)
**ID:** `Ghosts`

| Option | Effect | A15+ |
|--------|--------|------|
| Accept | Lose 50% max HP (min 1), gain 5 Apparition cards | 3 Apparition cards |
| Refuse | Nothing | - |

---

### Knowing Skull
**ID:** `Knowing Skull`

**Mechanics:** All costs start at 6 HP, increase by 1 after each use

| Option | HP Cost | Effect |
|--------|---------|--------|
| Potion | 6+ | Obtain random potion |
| Gold | 6+ | Gain 90 gold |
| Card | 6+ | Obtain random uncommon colorless card |
| Leave | 6 | Exit event |

---

### Masked Bandits
**ID:** `Masked Bandits`

| Option | Effect |
|--------|--------|
| Pay | Lose all gold |
| Fight | Fight Bandits, gain 25-35 gold + Red Mask relic |

---

### Nest
**ID:** `Nest`

| Option | Effect | A15+ |
|--------|--------|------|
| Steal | Gain 99 gold | 50 gold |
| Join | Take 6 damage, obtain Ritual Dagger card | Same |

---

### The Joust
**ID:** `The Joust`

**Mechanics:** Owner wins with 30% probability

| Option | Cost | Win Result | Lose Result |
|--------|------|------------|-------------|
| Bet on Murderer | 50 gold | Gain 100 gold | Lose bet |
| Bet on Owner | 50 gold | Gain 250 gold | Lose bet |

---

### The Library
**ID:** `The Library`

| Option | Effect | A15+ |
|--------|--------|------|
| Read | Choose 1 of 20 random cards (any rarity) | Same |
| Sleep | Heal 33% max HP | 20% max HP |

---

### The Mausoleum
**ID:** `The Mausoleum`

| Option | Effect | A15+ |
|--------|--------|------|
| Open | Obtain random relic, 50% chance Writhe curse | 100% curse |
| Leave | Nothing | - |

---

### Vampires
**ID:** `Vampires`

| Option | Condition | Effect |
|--------|-----------|--------|
| Accept | Always | Lose 30% max HP, replace all Strikes with 5 Bite cards |
| Offer Vial | Have Blood Vial | Lose Blood Vial, replace all Strikes with 5 Bite cards |
| Refuse | Always | Nothing |

---

## Beyond Events (Act 3)

### Falling
**ID:** `Falling`

**Mechanics:** Must remove one card type (Skill, Power, or Attack). Card to remove is pre-selected randomly.

| Option | Condition | Effect |
|--------|-----------|--------|
| Land on feet | Has Skills | Remove pre-selected Skill card |
| Brace for impact | Has Powers | Remove pre-selected Power card |
| Dive | Has Attacks | Remove pre-selected Attack card |
| Continue | No removable cards | Nothing |

**Note:** Options are disabled if no cards of that type exist. Card selection uses `miscRng`.

---

### Mind Bloom
**ID:** `MindBloom`

**Mechanics:** Third option changes based on floor number (within act, mod 50)

| Option | Condition | Effect |
|--------|-----------|--------|
| Fight | Always | Fight random Act 1 boss (Guardian/Hexaghost/Slime Boss), gain Rare relic + 50 gold (A13+: 25 gold) |
| Upgrade | Always | Upgrade ALL cards, obtain Mark of the Bloom (prevents healing) |
| Gold | floor % 50 <= 40 | Gain 999 gold + 2 Normality curses |
| Heal | floor % 50 > 40 | Full heal + Doubt curse |

---

### Moai Head
**ID:** `The Moai Head`

| Option | Condition | Effect | A15+ |
|--------|-----------|--------|------|
| Pray | Always | Lose 12.5% max HP, heal to full | 18% max HP |
| Trade | Have Golden Idol | Lose Golden Idol, gain 333 gold | Same |
| Leave | Always | Nothing | - |

---

### Mysterious Sphere
**ID:** `Mysterious Sphere`

| Option | Effect |
|--------|--------|
| Open | Fight 2 Orb Walkers, gain 45-55 gold + Rare relic |
| Leave | Nothing |

---

### Secret Portal
**ID:** `SecretPortal`

| Option | Effect |
|--------|--------|
| Enter | Skip directly to Act 3 boss |
| Leave | Nothing |

---

### Sensory Stone
**ID:** `SensoryStone`

| Option | HP Cost | Effect |
|--------|---------|--------|
| 1 Memory | 0 | Choose 1 of 3 colorless cards (1 reward) |
| 2 Memories | 5 | Choose 1 of 3 colorless cards (2 rewards) |
| 3 Memories | 10 | Choose 1 of 3 colorless cards (3 rewards) |

---

### Tomb of Lord Red Mask
**ID:** `Tomb of Lord Red Mask`

| Option | Condition | Effect |
|--------|-----------|--------|
| Don Mask | Have Red Mask | Gain 222 gold |
| Pay respects | No Red Mask | Lose all gold, obtain Red Mask |
| Leave | Always | Nothing |

---

### Winding Halls
**ID:** `Winding Halls`

| Option | Effect | A15+ |
|--------|--------|------|
| Embrace | Take 12.5% max HP damage, gain 2 Madness cards | 18% max HP |
| Retrace | Heal 25% max HP, gain Writhe curse | 20% max HP |
| Press on | Lose 5% max HP permanently | Same |

---

## Shrine Events (Any Act)

### Accursed Blacksmith
**ID:** `Accursed Blacksmith`

| Option | Effect |
|--------|--------|
| Forge | Upgrade 1 card |
| Rummage | Obtain Warped Tongs relic + Pain curse |
| Leave | Nothing |

---

### Bonfire Elementals
**ID:** `Bonfire Elementals`

**Mechanics:** Reward depends on card rarity offered

| Card Rarity | Reward |
|-------------|--------|
| Curse | Spirit Poop relic |
| Basic | Nothing |
| Common/Special | Heal 5 HP |
| Uncommon | Full heal |
| Rare | +10 max HP + full heal |

---

### Designer (The Designer)
**ID:** `Designer`

**Costs:** (A15+: 50/75/110 gold, otherwise 40/60/90 gold)

| Option | Cost | Effect (RNG-based) | A15+ |
|--------|------|-------------------|------|
| Adjustments | 40/50 gold | Upgrade 1 card OR upgrade 2 random cards | Same |
| Clean Up | 60/75 gold | Remove 1 card OR transform 2 cards | Same |
| Full Service | 90/110 gold | Remove 1 card + upgrade 1 random card | Same |
| Punch | - | Take 3 damage | 5 damage |

---

### Duplicator
**ID:** `Duplicator`

| Option | Effect |
|--------|--------|
| Use | Duplicate 1 card from deck |
| Leave | Nothing |

---

### Face Trader
**ID:** `FaceTrader`

| Option | Effect | A15+ |
|--------|--------|------|
| Touch | Gain 75 gold, take 10% max HP damage | 50 gold |
| Trade | Obtain random mask relic | Same |
| Leave | Nothing | - |

**Mask relics:** Cultist Mask, Face of Cleric, Gremlin Mask, N'loth's Mask, Ssserpent Head

---

### Fountain of Cleansing
**ID:** `Fountain of Cleansing`

| Option | Effect |
|--------|--------|
| Drink | Remove ALL curses (except AscendersBane, CurseOfTheBell, Necronomicurse) |
| Leave | Nothing |

---

### Golden Shrine
**ID:** `Golden Shrine`

| Option | Effect | A15+ |
|--------|--------|------|
| Pray | Gain 100 gold | 50 gold |
| Desecrate | Gain 275 gold + Regret curse | Same |
| Leave | Nothing | - |

---

### Match and Keep! (Gremlin Match Game)
**ID:** `Match and Keep!`

**Mechanics:**
- 12 cards (6 pairs) face-down
- 5 attempts to match pairs
- Matched cards are obtained

**Card Pool:**
- 1 Rare
- 1 Uncommon
- 1 Common
- 1 Colorless Uncommon (A15+: replaced with Curse)
- 1 Curse
- 1 Starter card

---

### Wheel of Change (Gremlin Wheel Game)
**ID:** `Wheel of Change`

**Outcomes (1/6 each):**
| Result | Effect |
|--------|--------|
| Gold | Gain 100/200/300 gold (Act 1/2/3) |
| Relic | Obtain random relic |
| Heal | Full heal |
| Curse | Obtain Decay curse |
| Remove | Remove 1 card |
| Damage | Take 10% max HP damage (A15+: 15%) |

---

### Lab
**ID:** `Lab`

| Option | Effect | A15+ |
|--------|--------|------|
| Gather | Obtain 3 random potions | 2 potions |

---

### N'loth
**ID:** `N'loth`

**Mechanics:** Offers 2 random relics from your collection

| Option | Effect |
|--------|--------|
| Trade Relic 1 | Lose offered relic, gain N'loth's Gift |
| Trade Relic 2 | Lose offered relic, gain N'loth's Gift |
| Leave | Nothing |

---

### Note For Yourself
**ID:** `NoteForYourself`

**Mechanics:** Uses persistent storage across runs

| Option | Effect |
|--------|--------|
| Take | Obtain card from previous run, select card to store for future |
| Leave | Nothing |

---

### Purifier (Purification Shrine)
**ID:** `Purifier`

| Option | Effect |
|--------|--------|
| Pray | Remove 1 card |
| Leave | Nothing |

---

### Transmogrifier
**ID:** `Transmorgrifier`

| Option | Effect |
|--------|--------|
| Pray | Transform 1 card |
| Leave | Nothing |

---

### Upgrade Shrine
**ID:** `Upgrade Shrine`

| Option | Effect |
|--------|--------|
| Pray | Upgrade 1 card |
| Leave | Nothing |

---

### We Meet Again
**ID:** `WeMeetAgain`

| Option | Condition | Effect |
|--------|-----------|--------|
| Give Potion | Have potion | Lose 1 potion, obtain random relic |
| Give Gold | Have 50+ gold | Lose 50-150 gold, obtain random relic |
| Give Card | Have non-basic, non-curse | Remove 1 card, obtain random relic |
| Attack | Always | Nothing (A15+ cosmetic shake) |

---

### The Woman in Blue
**ID:** `The Woman in Blue`

| Option | Cost | Effect |
|--------|------|--------|
| Buy 1 | 20 gold | Obtain 1 random potion |
| Buy 2 | 30 gold | Obtain 2 random potions |
| Buy 3 | 40 gold | Obtain 3 random potions |
| Leave | - | Nothing (A15+: Take 5% max HP damage) |

---

## Act 4 Events

### Spire Heart
**ID:** `Spire Heart`

**Mechanics:** This is a scripted story event, not a choice event.

| Phase | Effect |
|-------|--------|
| Initial | View score/damage dealt |
| Continue | If all 3 keys collected: Proceed to Heart fight; Otherwise: "Victory" screen (run ends) |

**Note:** This event calculates your run score and displays global damage statistics. Having all three keys (Ruby, Emerald, Sapphire) unlocks the true final boss.

---

## Notes on RNG

### Seed-Dependent RNG
Most event outcomes use `AbstractDungeon.miscRng` which is seeded. Predictable outcomes include:
- Dead Adventurer reward order and enemy selection
- Scrap Ooze relic chance
- The Joust winner
- Gremlin Wheel result
- Random relic selections
- Card transformations

### Ascension Modifiers
Many events have A15+ variants that:
- Increase damage taken
- Reduce gold rewards
- Reduce healing
- Increase costs
- Remove beneficial options or add penalties

---

## Event Conditions

Events can have special conditions to appear:
- **Fountain of Cleansing**: Only appears if player has curses
- **Forgotten Altar**: Golden Idol option only if player has Golden Idol
- **Vampires**: Blood Vial option only if player has Blood Vial
- **Tomb of Lord Red Mask**: Different options based on Red Mask ownership
- **Note For Yourself**: Uses cross-run persistent storage
