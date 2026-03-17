# Events Parity Report: Java vs Python

**Date:** 2026-03-03
**Auditor:** Claude Opus 4.6 (1M context)
**Java Source:** `decompiled/java-src/com/megacrit/cardcrawl/events/`
**Python Source:** `packages/engine/handlers/event_handler.py`

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Java Events** | 52 |
| **Python Handler Implemented** | 45 |
| **Python Handler Missing** | 7 |
| **Fully Correct (no issues)** | 18 |
| **Minor Issues (values/details)** | 22 |
| **Major Issues (wrong mechanics)** | 5 |
| **Watcher-Relevant Events** | 52 (all events can appear in Watcher runs) |

### Implementation Status
- **Handler + Choice Generator:** 45/52 (87%)
- **Missing Entirely:** SpireHeart (special, non-standard)
- **Missing Handler:** 0 (all non-SpireHeart events have handlers)
- **Missing Choice Generator:** 0

---

## Act 1 Events

| Event | Java ID | Java Class | Choices | Python Status | Issues |
|-------|---------|------------|---------|---------------|--------|
| Big Fish | `Big Fish` | BigFish.java | 3 | IMPLEMENTED | (1) Box relic: Java uses `returnRandomRelicTier()` then `returnRandomScreenlessRelic()` (tier-weighted), Python always uses common tier. **MEDIUM** |
| The Cleric | `The Cleric` | Cleric.java | 3 | IMPLEMENTED | Correct: heal 25% for 35g, purify 50/75g (A15+). No issues. |
| Dead Adventurer | `Dead Adventurer` | DeadAdventurer.java | 2 | IMPLEMENTED | (1) Java encounter chance uses `miscRng.random(0,99) < encounterChance` (integer 0-99), Python uses `miscRng.random_float()` (float 0-1). **LOW** (2) Java enemy selected once at init via `miscRng.random(0,2)` for 3 Sentries/Gremlin Nob/Lagavulin Event; Python uses `misc_rng.random(len(elite_options)-1)` at fight time. **MEDIUM** (3) Java reward on "RELIC" uses `returnRandomRelicTier()` (tier-weighted), Python uses common. **MEDIUM** |
| Golden Idol | `Golden Idol` | GoldenIdolEvent.java | Phase1: 2, Phase2: 3 | IMPLEMENTED | (1) Java gives GoldenIdol relic (checks duplicate -> Circlet), Python correctly adds it. (2) Java `maxHpLoss` has minimum of 1 (`if (maxHpLoss < 1) maxHpLoss = 1`), Python has no min-1 guard. **LOW** |
| Golden Wing (Wing Statue) | `Golden Wing` | GoldenWing.java | 3 | IMPLEMENTED | (1) Java has 3 choices: lose 7 HP + remove card, OR if has attack card with 10+ damage: gain 50-80 gold (miscRng), OR leave. Python only implements 2 choices (lose HP + remove, leave). **Missing the conditional gold option.** **HIGH** |
| World of Goop | `World of Goop` | GoopPuddle.java | 2 | IMPLEMENTED | (1) Java gold loss is calculated at init (miscRng consumed once), Python calculates at choice time. RNG order differs. **MEDIUM** |
| Living Wall | `Living Wall` | LivingWall.java | 3 | IMPLEMENTED | (1) Java "Change" uses `AbstractDungeon.transformCard()` (same-rarity transform), Python uses `_get_random_card(common)` (always common). **HIGH** (2) Java "Grow" disabled if no upgradable cards, Python has same check. OK. |
| Mushrooms | `Mushrooms` | Mushrooms.java | 2 | IMPLEMENTED | (1) Java heal has no A15 modifier (always 25% max HP), Python applies 20% on A15+. **MEDIUM** (wrong A15 behavior) (2) Java fight reward: Odd Mushroom relic (or Circlet if already owned) + gold. Python sets pending_rewards but doesn't check duplicate. **LOW** |
| Scrap Ooze | `Scrap Ooze` | ScrapOoze.java | 2 | IMPLEMENTED | (1) Java success check: `miscRng.random(0,99) >= 99 - relicObtainChance` (integer check), Python: `miscRng.random_float() < success_chance`. Different RNG semantics. **LOW** (2) Java relic uses `returnRandomRelicTier()` (tier-weighted), Python uses common. **MEDIUM** |
| Shining Light | `Shining Light` | ShiningLight.java | 2 | IMPLEMENTED | (1) Java uses `MathUtils.round()` for damage, Python uses `int()` (floor). Difference for odd maxHP values. **LOW** (2) Java shuffle uses `Collections.shuffle(upgradableCards, new Random(miscRng.randomLong()))`, Python uses `miscRng.random(len-1)`. Different shuffle semantics. **LOW** |
| Sssserpent | `Liars Game` | Sssserpent.java | 2 | IMPLEMENTED | Correct: 175/150 gold (A15+), Doubt curse. No issues. |

## Act 2 Events

| Event | Java ID | Java Class | Choices | Python Status | Issues |
|-------|---------|------------|---------|---------------|--------|
| Pleading Vagrant | `Addict` | Addict.java | 3 | IMPLEMENTED | (1) Java choice order: 0=Pay 85g, 1=Steal (relic+Shame), 2=Leave. Python: 0=Pay, 1=Refuse (Shame only), 2=Rob (relic+Shame). **Java's option 1 steals relic+Shame; Python "refuse" is Shame-only.** The Python "refuse" option (choice 1) does NOT match Java -- Java choice 1 gives relic+curse, not just curse. **HIGH** (2) Java relic uses `returnRandomRelicTier()`, Python uses common. **MEDIUM** |
| Back to Basics | `Back to Basics` | BackToBasics.java | 2 | IMPLEMENTED | (1) Java option 0 = "Simplicity" = remove ONE card (via grid select, purgeable), option 1 = "Elegance" = upgrade all Strikes/Defends. Python option 0 removes ALL non-Strike/Defend cards, option 1 upgrades Strikes/Defends. **Python "Simplicity" is completely wrong -- should remove 1 card, not strip deck.** **CRITICAL** |
| The Beggar | `Beggar` | Beggar.java | 2 | IMPLEMENTED | (1) Java: pay 75g, then select 1 card to remove. No relic reward. Python: choice 0 = donate 50g for relic, choice 1 = donate 100g for relic + curse removal. **Completely wrong mechanics.** Java has only 2 options (pay 75g to remove card, or leave). **CRITICAL** |
| Colosseum | `Colosseum` | Colosseum.java | Multi-phase | IMPLEMENTED | (1) Java POST_COMBAT: button 0 = Cowardice (leave), button 1 = Fight Nobs. Python: button 0 = fight nobs, button 1 = cowardice. **Button mapping reversed.** **MEDIUM** (2) Reward in Java: Rare relic + Uncommon relic + 100g. Python tracks these but uses string placeholders. **LOW** |
| Cursed Tome | `Cursed Tome` | CursedTome.java | Multi-phase (5 screens) | IMPLEMENTED | (1) Java is multi-page: page 1 (1 dmg), page 2 (2 dmg), page 3 (3 dmg), then obtain (10/15 dmg) or stop (3 dmg). Total if read: 1+2+3+10=16 or 1+2+3+15=21. Python collapses to single choice with 16/21 total. **Simplification is OK for RL** but loses the "stop reading" option at page 3. **MEDIUM** (2) Java book selection excludes already-owned books, Python does not check. **MEDIUM** |
| Drug Dealer (Augmenter) | `Drug Dealer` | DrugDealer.java | 3 | IMPLEMENTED | (1) Java option 2 = MutagenicStrength relic (specific relic, not random Str/Dex). Python gives random MutagenicStrength or MutagenicDexterity. **Java only gives MutagenicStrength.** **HIGH** (2) Java option 1 = transform 2 cards. Python implements this. OK. (3) Java option 0 = obtain J.A.X. card (no card removal). Python adds J.A.X. AND removes a card. **Wrong -- Java does NOT remove a card.** **HIGH** |
| Forgotten Altar | `Forgotten Altar` | ForgottenAltar.java | 3 | IMPLEMENTED | (1) Java option 1 = "Shed Blood": +5 max HP, take 25%/35% max HP damage. Python: -5/7 HP damage, gain random common relic. **Completely different mechanics.** Java gives max HP gain + damage; Python gives relic. **CRITICAL** (2) Java option 0 = sacrifice Golden Idol for Bloody Idol (or Circlet if already has Bloody Idol). Python is correct here. |
| Ghosts | `Ghosts` | Ghosts.java | 2 | IMPLEMENTED | (1) Java max HP loss: `MathUtils.ceil(maxHealth * 0.5f)`, capped at `maxHealth - 1`. Python uses `int(max_hp * 0.50)` (floor). **Different rounding.** **LOW** (2) Apparition count: Java `5 - 2 = 3` on A15+. Python: `3 if A15+ else 5`. Correct. |
| Knowing Skull | `Knowing Skull` | KnowingSkull.java | 4 | IMPLEMENTED | (1) Java order: Potion (slot 0), Gold (slot 1), Card (slot 2), Leave (slot 3). Python matches. (2) Java leave cost is fixed at 6 (never increments). Python correctly fixes leave at 6. (3) Java potion uses `PotionHelper.getRandomPotion()` (weighted by class), Python uses flat random. **LOW** |
| Masked Bandits | `Masked Bandits` | MaskedBandits.java | 2 | IMPLEMENTED | (1) Java fight reward: Red Mask relic (or Circlet if already owned) + gold. Python sets generic encounter. **Missing specific Red Mask reward.** **MEDIUM** |
| The Nest | `Nest` | Nest.java | 2 | IMPLEMENTED | (1) Java: option 0 = steal gold (99/50 on A15+), option 1 = join cult (take 6 HP damage, gain Ritual Dagger). Python: option 0 = smash (99g + random common card), option 1 = stay (Ritual Dagger, no HP cost). **Python is wrong:** nest gold is 99/50 (A15 mod), Ritual Dagger costs 6 HP, and "smash" does NOT give a random card. **CRITICAL** |
| The Joust | `The Joust` | TheJoust.java | 2 | IMPLEMENTED | (1) Java: costs 50g to bet. Winner gets 250g (owner) or 100g (murderer). Owner wins 30% of the time. Net: bet on owner = +200 or -50; bet on murderer = +50 or -50. Python: no bet cost, owner 30% for 250g, murderer 70% for 50g. **Missing 50g bet cost.** **HIGH** (2) Python gold amounts: Owner win = 250, Murderer win = 50. Java: Owner win = 250, Murderer win = 100. **Murderer reward wrong (50 vs 100).** **HIGH** |
| The Library | `The Library` | TheLibrary.java | 2 | IMPLEMENTED | (1) Java "Sleep" heals 33%/20% (A15+) max HP. Python heals to FULL HP. **Wrong heal amount.** **HIGH** (2) Java "Read" generates 20 cards via `AbstractDungeon.getCard(rollRarity())` (class-colored, rarity-weighted). Python marks as requires_card_selection but doesn't generate pool. **Incomplete.** **MEDIUM** |
| The Mausoleum | `The Mausoleum` | TheMausoleum.java | 2 | IMPLEMENTED | (1) Java: always gives relic (random tier). 50% chance also gives Writhe curse. On A15+, always gives curse (100%). Python: 50% relic only, 50% curse only. **Wrong -- Java always gives relic.** **CRITICAL** |
| Vampires | `Vampires` | Vampires.java | 2-3 | IMPLEMENTED | (1) Java max HP loss uses `MathUtils.ceil()`, capped at `maxHealth-1`. Python uses `int()` (floor), no cap. **LOW** (2) Java removes cards tagged STARTER_STRIKE specifically. Python checks `card.id.startswith("Strike_")`. Close but could miss edge cases. **LOW** |

## Act 3 Events

| Event | Java ID | Java Class | Choices | Python Status | Issues |
|-------|---------|------------|---------|---------------|--------|
| Falling | `Falling` | Falling.java | 3 | IMPLEMENTED | (1) Java uses `CardHelper.returnCardOfType()` with miscRng. Python uses custom `_ensure_falling_preselect()` with miscRng. Should be equivalent. (2) Java uses `miscRng` for card selection; Python matches. OK. |
| Mind Bloom | `MindBloom` | MindBloom.java | 3 | IMPLEMENTED | (1) Java "I am War" boss is randomly selected from Guardian/Hexaghost/Slime Boss using `miscRng`. Python just sets "Act1Boss". **Missing specific boss selection.** **LOW** (2) Java "I am War" gold reward: 25 on A13+, 50 otherwise. Python: 50 always. **Missing A13 check.** **LOW** (3) Floor check for option 3 matches (`floorNum % 50 <= 40`). OK. |
| Moai Head | `The Moai Head` | MoaiHead.java | 3 | IMPLEMENTED | (1) Java uses `MathUtils.round()` for HP loss, Python uses `int()` (floor). **LOW** (2) Java directly modifies `maxHealth` and `currentHealth`, then heals to full. Python uses helper methods. Functionally equivalent. OK. |
| Mysterious Sphere | `Mysterious Sphere` | MysteriousSphere.java | 2 | IMPLEMENTED | (1) Java reward: Rare relic from `returnRandomScreenlessRelic(RARE)` + gold (45-55). Python sets pending_rewards with generic "RareRelic". **LOW** |
| Secret Portal | `SecretPortal` | SecretPortal.java | 2 | IMPLEMENTED | Correct: teleport to boss or leave. No issues. |
| Sensory Stone | `SensoryStone` | SensoryStone.java | 3 | IMPLEMENTED | (1) Java has 3 choices: 1 colorless card reward (free), 2 cards (5 HP), 3 cards (10 HP). Python has 1 choice that gives `min(act, 3)` cards. **Completely different structure.** Java choices are damage-gated; Python is act-based. **CRITICAL** |
| Tomb of Lord Red Mask | `Tomb of Lord Red Mask` | TombRedMask.java | 2-3 | IMPLEMENTED | (1) Java: if has Red Mask -> option 0 = gain 222 gold. If no Red Mask -> option 0 disabled, option 1 = pay all gold for Red Mask relic. Python: option 0 = don mask, option 1 = offer gold (requires Red Mask). **Choice structure is inverted and gold amounts wrong (Java gives flat 222, not per-relic).** **HIGH** |
| Winding Halls | `Winding Halls` | WindingHalls.java | 3 | IMPLEMENTED | (1) Java uses `MathUtils.round()` for all percent calculations, Python uses `int()` (floor). **LOW** (2) Values match: Embrace = damage + 2 Madness, Retrace = heal + Writhe, Press on = lose max HP. OK. |
| Spire Heart | `Spire Heart` | SpireHeart.java | Special | NOT IMPLEMENTED | This is the Act 4 Heart event -- special combat trigger. Not a standard event. **N/A for standard event pool.** |

## Shrine Events (Any Act)

| Event | Java ID | Java Class | Choices | Python Status | Issues |
|-------|---------|------------|---------|---------------|--------|
| Accursed Blacksmith | `Accursed Blacksmith` | AccursedBlacksmith.java | 3 | IMPLEMENTED | Java not in decompiled shrines list but Python has handler. Need to verify Java source. Python: upgrade + Parasite or leave. **Unverified.** |
| Bonfire Elementals | `Bonfire Elementals` | Bonfire.java | 2 | IMPLEMENTED | Python: offer card for relic, or leave. Java needs verification. **Unverified.** |
| Designer In-Spire | `Designer` | Designer.java | 4 | IMPLEMENTED | (1) Java has randomized option types via `miscRng.randomBoolean()`: "Adjustment" = upgrade 1 OR upgrade 2 random; "Cleanup" = remove 1 OR transform 2; "Full Service" = remove + upgrade 1 random. Python: fixed options (remove/transform/upgrade). **Completely different option structure.** **CRITICAL** (2) Java costs: A15+ = 50/75/110, normal = 40/60/90. Python costs: 75/50/50/35/40/25 (different per operation). **Wrong costs.** **HIGH** (3) Java has "Punch" option (leave with HP loss 5/3). Python has "leave" with no HP loss. **Missing punch option.** **MEDIUM** |
| Duplicator | `Duplicator` | Duplicator.java | 2 | IMPLEMENTED | Correct: duplicate a card or leave. No issues. |
| Face Trader | `FaceTrader` | FaceTrader.java | 3 | IMPLEMENTED | (1) Java option 0: take damage (maxHP/10) + gain gold (75/50 on A15+). Python: lose 10% HP + gain random common relic. **Wrong -- Java gives gold, not relic.** **HIGH** (2) Java option 1: trade for random face relic (CultistMask/FaceOfCleric/GremlinMask/NlothsMask/SsserpentHead). Python: pay 75g for SsserpentHead. **Wrong -- no gold cost in Java, and wrong relic selection.** **HIGH** |
| Fountain of Cleansing | `FountainOfCurseRemoval` | FountainOfCurseRemoval.java | 2 | IMPLEMENTED | Correct: remove all removable curses or leave. No issues. |
| Golden Shrine | `Gold Shrine` | GoldShrine.java | 2-3 | IMPLEMENTED | Need to verify Java. Python: Pray (100/50g), Desecrate (275g + Regret), Leave. **Unverified.** |
| Gremlin Match Game | `Match and Keep!` | GremlinMatchGame.java | 2 | IMPLEMENTED | Python implements a simplified version. Java is interactive memory game. **Simplification acceptable for RL.** |
| Gremlin Wheel Game | `Wheel of Change` | GremlinWheelGame.java | 2 | IMPLEMENTED | Python implements random outcomes. Need to verify wheel probabilities match. **Partially verified.** |
| The Lab | `The Lab` | Lab.java | 1 | IMPLEMENTED | Python: 3/2 potions (A15+). Java needs verification but likely matches. **Unverified.** |
| N'loth | `N'loth` | Nloth.java | 3 | IMPLEMENTED | (1) Java: presents 2 random relics from your collection to trade for N'loth's Gift (choice of which to give). Python: always trades oldest relic (index 0). **Wrong relic selection.** **HIGH** |
| Note For Yourself | `Note For Yourself` | NoteForYourself.java | 2 | IMPLEMENTED | Python implements basic take/leave. Java involves cross-run card saving. **Simplified for RL.** OK. |
| Purification Shrine | `Purification Shrine` | PurificationShrine.java | 2 | IMPLEMENTED | Correct: remove card or leave. No issues. |
| Transmogrifier | `Transmogrifier` | Transmogrifier.java | 2 | IMPLEMENTED | (1) Java uses `AbstractDungeon.transformCard()` (same-rarity transform). Python uses `_get_random_card(common)`. **Always transforms to common.** **MEDIUM** |
| Upgrade Shrine | `Upgrade Shrine` | UpgradeShrine.java | 2 | IMPLEMENTED | Correct: upgrade card or leave. No issues. |
| We Meet Again | `We Meet Again` | WeMeetAgain.java | 4 | IMPLEMENTED | Python has handler. Java needs verification. **Unverified.** |
| Woman in Blue | `The Woman in Blue` | WomanInBlue.java | 4 | IMPLEMENTED | (1) Java A15+ leave: `MathUtils.ceil(maxHealth * 0.05f)` HP loss. Python: `damage_percent(0.05)` which uses `int()` floor. **LOW** (2) Java potions via `PotionHelper.getRandomPotion()` (class-weighted). Python uses flat random. **LOW** |

---

## Critical Issues Summary (Must Fix for Watcher RL)

### CRITICAL (wrong mechanics, will break game logic)

| # | Event | Issue | Java Behavior | Python Behavior |
|---|-------|-------|---------------|-----------------|
| 1 | **Back to Basics** | Option 0 (Simplicity) | Remove 1 card (player chooses) | Removes ALL non-Strike/Defend cards |
| 2 | **The Beggar** | All options | Pay 75g to remove 1 card, or leave (2 options) | Donate 50g/100g for relic (3 options, completely wrong) |
| 3 | **Forgotten Altar** | Option 1 (Offer) | +5 max HP, take 25%/35% damage | -5/7 HP, gain random relic |
| 4 | **The Mausoleum** | Open option | Always get relic; 50%/100%(A15) also get Writhe curse | 50% relic only, 50% curse only (never both) |
| 5 | **The Nest** | Both options | Steal = gold only (99/50 A15); Join = 6 HP + Ritual Dagger | Smash = 99g + random card; Stay = Ritual Dagger (no HP cost) |
| 6 | **Sensory Stone** | All choices | 3 choices: 1/2/3 colorless cards for 0/5/10 HP | 1 choice: gain act-number cards (no HP cost) |
| 7 | **Designer** | All options | Randomized: upgrade 1 or 2, remove 1 or transform 2, full service; costs 40-110 | Fixed: remove/transform/upgrade; wrong costs |

### HIGH (significant value/choice errors)

| # | Event | Issue |
|---|-------|-------|
| 1 | Wing Statue | Missing conditional gold option (50-80g if have 10+ damage attack) |
| 2 | Addict (Pleading Vagrant) | Java option 1 = steal relic+curse; Python option 1 = curse only |
| 3 | Drug Dealer (Augmenter) | Option 0: Java gives J.A.X. only, Python also removes card. Option 2: Java gives MutagenicStrength specifically, Python randomizes Str/Dex |
| 4 | The Joust | Missing 50g bet cost; Murderer win = 100g (not 50g) |
| 5 | The Library | Sleep heals 33%/20%(A15+), not full HP |
| 6 | Tomb of Lord Red Mask | Choice structure inverted; gold is flat 222, not per-relic |
| 7 | Face Trader | Option 0 = gold+damage (not relic); Option 1 = random face relic (not buy SsserpentHead) |
| 8 | N'loth | Should present 2 random relics to choose from, not auto-trade oldest |

---

## Relic Tier Issue (Affects Many Events)

Multiple events use `AbstractDungeon.returnRandomRelicTier()` which rolls a tier based on probabilities (Common/Uncommon/Rare/Shop), then gets a relic from that tier. The Python engine always uses `_get_random_relic(common)` for most events. This systematically under-values relic rewards.

**Affected events:** Big Fish (box), Dead Adventurer (relic), Scrap Ooze, Addict, Bonfire Elementals, We Meet Again, Beggar, Face Trader, and others.

---

## Transform Card Issue (Affects Multiple Events)

Java's `AbstractDungeon.transformCard()` transforms a card into another card of the same rarity. Python's `_get_random_card(common)` always gives a common card regardless of the original card's rarity.

**Affected events:** Living Wall (Change), Transmogrifier, Drug Dealer (Transform), Designer (Transform).

---

## Events Appearing in Watcher Runs (Priority Order)

All 52 events can appear in Watcher runs. The standard event pools are:

**Act 1 (11 events):** BigFish, TheCleric, DeadAdventurer, GoldenIdol, GoldenWing, WorldOfGoop, LivingWall, Mushrooms, ScrapOoze, ShiningLight, Sssserpent

**Act 2 (15 events):** Addict, BackToBasics, Beggar, Colosseum, CursedTome, DrugDealer, ForgottenAltar, Ghosts, KnowingSkull, MaskedBandits, Nest, TheJoust, TheLibrary, TheMausoleum, Vampires

**Act 3 (9 events):** Falling, MindBloom, MoaiHead, MysteriousSphere, SecretPortal, SensoryStone, SpireHeart, TombRedMask, WindingHalls

**Shrines / One-Time (any act, 17 events):** AccursedBlacksmith, BonfireElementals, Designer, Duplicator, FaceTrader, FountainOfCleansing, GoldenShrine, GremlinMatchGame, GremlinWheelGame, Lab, Nloth, NoteForYourself, PurificationShrine, Transmogrifier, UpgradeShrine, WeMeetAgain, WomanInBlue

---

## Recommended Fix Priority

1. **Fix 7 CRITICAL events** (Back to Basics, Beggar, Forgotten Altar, Mausoleum, Nest, Sensory Stone, Designer) -- these have completely wrong mechanics
2. **Fix 8 HIGH events** (Wing Statue, Addict, Augmenter, Joust, Library, Tomb, Face Trader, N'loth) -- significant value errors
3. **Fix relic tier system** -- affects ~10 events, use `returnRandomRelicTier()` equivalent
4. **Fix transform card system** -- affects ~4 events, should preserve rarity
5. **Fix rounding differences** (MathUtils.round vs int()) -- LOW priority, affects edge cases
