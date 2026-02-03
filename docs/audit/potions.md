# Potion Audit: Python Engine vs Java Source

## Audit Date: 2026-02-02

## Summary

| Category | Count |
|----------|-------|
| Total potions defined (content/potions.py) | 42 |
| Potions with effect logic (handlers/combat.py) | 7 |
| Potions with effect logic (combat_engine.py) | 10 |
| Potions with NO effect implementation | 25 |
| Value mismatches vs Java | 0 |
| Sacred Bark bugs | 1 (combat_engine.py ignores Sacred Bark entirely) |
| Missing potions in data | 0 |

## Data Layer (content/potions.py) - All Values Correct

All 42 potions are defined with correct base potency, rarity, target type, and Sacred Bark scaling flags. Values verified against Java bytecode:

| Potion | Python Potency | Java Potency | Match | Sacred Bark |
|--------|---------------|-------------|-------|-------------|
| Block Potion | 12 | 12 | OK | 2x -> 24 |
| Dexterity Potion | 2 | 2 | OK | 2x -> 4 |
| Energy Potion | 2 | 2 | OK | 2x -> 4 |
| Explosive Potion | 10 | 10 | OK | 2x -> 20 |
| Fire Potion | 20 | 20 | OK | 2x -> 40 |
| Strength Potion | 2 | 2 | OK | 2x -> 4 |
| Swift Potion | 3 | 3 | OK | 2x -> 6 |
| Weak Potion | 3 | 3 | OK | 2x -> 6 |
| Fear Potion | 3 | 3 | OK | 2x -> 6 |
| Attack Potion | 1 | 1 | OK | 2x (2 copies) |
| Skill Potion | 1 | 1 | OK | 2x (2 copies) |
| Power Potion | 1 | 1 | OK | 2x (2 copies) |
| Colorless Potion | 1 | 1 | OK | 2x (2 copies) |
| Speed Potion | 5 | 5 | OK | 2x -> 10 |
| Flex Potion (Steroid) | 5 | 5 | OK | 2x -> 10 |
| Blessing of the Forge | 0 | 0 | OK | No effect |
| Blood Potion | 20 | 20 | OK | 2x -> 40% |
| Poison Potion | 6 | 6 | OK | 2x -> 12 |
| Focus Potion | 2 | 2 | OK | 2x -> 4 |
| Bottled Miracle | 2 | 2 | OK | 2x -> 4 |
| Ancient Potion | 1 | 1 | OK | 2x -> 2 |
| Regen Potion | 5 | 5 | OK | 2x -> 10 |
| Gambler's Brew | 0 | 0 | OK | No effect |
| Liquid Bronze | 3 | 3 | OK | 2x -> 6 |
| Liquid Memories | 1 | 1 | OK | 2x -> 2 |
| Essence of Steel | 4 | 4 | OK | 2x -> 8 |
| Duplication Potion | 1 | 1 | OK | 2x -> 2 |
| Distilled Chaos | 3 | 3 | OK | 2x -> 6 |
| Elixir | 0 | 0 | OK | No effect |
| Cunning Potion | 3 | 3 | OK | 2x -> 6 |
| Potion of Capacity | 2 | 2 | OK | 2x -> 4 |
| Stance Potion | 0 | 0 | OK | No effect |
| Cultist Potion | 1 | 1 | OK | 2x -> 2 |
| Fruit Juice | 5 | 5 | OK | 2x -> 10 |
| Snecko Oil | 5 | 5 | OK | 2x -> 10 |
| Fairy in a Bottle | 30 | 30 | OK | 2x -> 60% |
| Smoke Bomb | 0 | 0 | OK | No effect |
| Entropic Brew | 3 | 3 | OK | No effect |
| Heart of Iron | 6 | 6 | OK | 2x -> 12 |
| Ghost In A Jar | 1 | 1 | OK | 2x -> 2 |
| Essence of Darkness | 1 | 1 | OK | 2x -> 2 |
| Ambrosia | 2 | 2 | OK | No effect |

## Effect Implementation Layer - Critical Gaps

### handlers/combat.py._apply_potion_effect (7 potions, all with Sacred Bark)

Implements: Block Potion, Fire Potion, Strength Potion, Dexterity Potion, Weak Potion, Fear Potion, Energy Potion

All 7 are correct with proper Sacred Bark doubling.

### combat_engine.py.use_potion (10 potions, NO Sacred Bark)

Implements: Block Potion, Strength Potion, Dexterity Potion, Fire Potion, Energy Potion, Swift Potion, Fear Potion, Weak Potion, Explosive Potion

**BUG**: This implementation hardcodes base values and does NOT check for Sacred Bark. If this code path is used when the player has Sacred Bark, all potions will have half their intended effect.

### Potions with NO effect implementation (25 potions)

These potions exist in the data layer but have no use() logic in either combat handler:

| Potion | Effect Needed |
|--------|--------------|
| Swift Potion | Draw cards (only in combat_engine.py) |
| Explosive Potion | AoE damage (only in combat_engine.py) |
| Attack Potion | Discovery (choose 1 of 3 attacks) |
| Skill Potion | Discovery (choose 1 of 3 skills) |
| Power Potion | Discovery (choose 1 of 3 powers) |
| Colorless Potion | Discovery (choose 1 of 3 colorless) |
| Speed Potion | Temp dexterity + LoseDexterityPower |
| Flex Potion | Temp strength + LoseStrengthPower |
| Blessing of the Forge | Upgrade all hand cards |
| Blood Potion | Heal % max HP |
| Poison Potion | Apply poison to target |
| Focus Potion | Gain Focus |
| Bottled Miracle | Add Miracle cards to hand |
| Ancient Potion | Gain Artifact |
| Regen Potion | Gain RegenPower |
| Gambler's Brew | Discard and redraw |
| Liquid Bronze | Gain ThornsPower |
| Liquid Memories | Return card from discard |
| Essence of Steel | Gain PlatedArmorPower |
| Duplication Potion | Gain DuplicationPower |
| Distilled Chaos | Play top N cards |
| Elixir | Exhaust chosen cards |
| Cunning Potion | Add upgraded Shivs |
| Potion of Capacity | Gain orb slots |
| Stance Potion | Choose Calm/Wrath |
| Cultist Potion | Gain RitualPower |
| Fruit Juice | Gain max HP (outside combat too) |
| Snecko Oil | Draw 5 + randomize costs |
| Fairy in a Bottle | Auto-revive on death |
| Smoke Bomb | Flee combat |
| Entropic Brew | Fill empty potion slots |
| Heart of Iron | Gain MetallicizePower |
| Ghost In A Jar | Gain IntangiblePower |
| Essence of Darkness | Channel Dark orbs |
| Ambrosia | Enter Divinity |

## Java Reference Notes

### FairyPotion.java
- `getPotency()`: returns `AbstractDungeon.player.hasRelic("Sacred Bark") ? 60 : 30`
- `onPlayerDeath()`: sets HP to `floor(maxHP * potency / 100.0f)`, discards self
- `canUse()`: returns false (auto-trigger only)

### BlockPotion.java
- `use()`: `addToBot(new GainBlockAction(p, p, potency))`
- Block gained this way IS affected by Frail in Java (GainBlockAction applies Frail)
- Python says "Block gained ignores Dexterity" -- this is correct (no dex), but it DOES NOT ignore Frail. The special_mechanics note is misleading; the block ignores Dexterity but Frail still applies via GainBlockAction.

### FirePotion.java
- `use()`: `addToBot(new DamageAction(target, new DamageInfo(source, potency, DamageType.THORNS)))`
- Uses THORNS damage type -- not affected by player Strength or Weak
- IS affected by target Vulnerable (THORNS still checks vuln)
- Python special_mechanics correctly notes THORNS type

### SwiftPotion.java
- `use()`: `addToBot(new DrawCardAction(p, potency))`
- Potency = 3, Sacred Bark = 6

### EntropicBrew.java
- `use()`: loops `p.potionSlots`, fills empty slots via `AbstractDungeon.returnRandomPotion()`
- Sozu check: if player has Sozu, flash relics and return
- Sacred Bark does NOT affect it (potency unused in logic)

### SmokeBomb.java
- `canUse()`: returns false if in boss room or enemy has BackAttack
- `use()`: `AbstractDungeon.getCurrRoom().smoked = true; AbstractRoom.waitTimer = 0; AbstractDungeon.player.hideHealthBar(); AbstractDungeon.player.isEscaping = true`

### DuplicationPotion.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new DuplicationPower(p, potency)))`
- Potency = 1, Sacred Bark = 2

### DistilledChaosPotion.java (note: Python uses DistilledChaos, Java uses DistilledChaosPotion)
- `use()`: for i in range(potency): `addToBot(new RandomCardFromDrawPileToHandAction())`... actually it's `addToBot(new PlayTopCardAction(AbstractDungeon.getRandomMonster(), false))`
- Potency = 3, Sacred Bark = 6

### Ambrosia.java
- `use()`: `addToBot(new ChangeStanceAction("Divinity"))`
- Potency technically 1, but unused. Sacred Bark has no effect.

### StancePotion.java
- `use()`: shows card reward screen with ChooseCalm and ChooseWrath option cards
- Sacred Bark has no effect

### FruitJuice.java
- `use()`: `AbstractDungeon.player.increaseMaxHp(potency, true)`
- Potency = 5, Sacred Bark = 10
- `canUse()` in AbstractPotion allows use outside combat for this potion

### SneckoOil.java
- `use()`: `addToBot(new DrawCardAction(potency)); addToBot(new RandomizeHandCostAction())`
- Potency = 5, Sacred Bark = 10

### GhostInAJar.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new IntangiblePlayerPower(p, potency)))`
- Potency = 1, Sacred Bark = 2

### HeartOfIron.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new MetallicizePower(p, potency)))`
- Potency = 6, Sacred Bark = 12

### EssenceOfSteel.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new PlatedArmorPower(p, potency)))`
- Potency = 4, Sacred Bark = 8

### CultistPotion.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new RitualPower(p, potency, true)))`
- Potency = 1, Sacred Bark = 2. The `true` means player-owned (no vuln to ArtifactPower removal).

### LiquidBronze.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new ThornsPower(p, potency)))`
- Potency = 3, Sacred Bark = 6

### LiquidMemories.java
- `use()`: `addToBot(new RecallAction(potency))`... actually `BetterDiscardPileToHandAction` with `potency` number of cards
- Potency = 1, Sacred Bark = 2

### RegenPotion.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new RegenPower(p, potency)))`
- Potency = 5, Sacred Bark = 10

### AncientPotion.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new ArtifactPower(p, potency)))`
- Potency = 1, Sacred Bark = 2

### BlessingOfTheForge.java
- `use()`: `addToBot(new ArmamentsAction(true))` -- upgrades ALL hand cards
- Sacred Bark has no effect (does not use potency)

### GamblersBrew.java
- `use()`: `addToBot(new GamblingChipAction(p, true))`
- Sacred Bark has no effect

### SpeedPotion.java
- `use()`: `addToBot(new ApplyPowerAction(p, p, new DexterityPower(p, potency))); addToBot(new ApplyPowerAction(p, p, new LoseDexterityPower(p, potency)))`
- Potency = 5, Sacred Bark = 10

### SteroidPotion.java (Flex Potion)
- `use()`: `addToBot(new ApplyPowerAction(p, p, new StrengthPower(p, potency))); addToBot(new ApplyPowerAction(p, p, new LoseStrengthPower(p, potency)))`
- Potency = 5, Sacred Bark = 10

### WeakenPotion.java (Weak Potion)
- ID is "Weak Potion" not "WeakenPotion"
- `use()`: `addToBot(new ApplyPowerAction(target, p, new WeakPower(target, potency, false)))`
- Potency = 3, Sacred Bark = 6

## Specific Issues Found

### Issue 1: combat_engine.py ignores Sacred Bark (MEDIUM)
The `combat_engine.py` use_potion method hardcodes base values and never checks for Sacred Bark. The `handlers/combat.py` version correctly handles it.

### Issue 2: Block Potion and Frail (LOW)
Python special_mechanics says "Block gained ignores Dexterity" which is correct, but the Java GainBlockAction DOES apply Frail reduction. Neither Python implementation applies Frail to Block Potion. This is a minor parity issue since Block Potion in Java goes through GainBlockAction which checks Frail.

### Issue 3: Fear Potion ID inconsistency (LOW)
Python uses `id="FearPotion"` but Java class is `FearPotion` with ID `"FearPotion"`. This is fine for the data layer but `handlers/combat.py` checks for `"Fear Potion"` (with space) which would NOT match the data definition's `id="FearPotion"`. There may be a mismatch between how potions are stored and looked up.

### Issue 4: Weak Potion ID (LOW)
Java class is `WeakenPotion` but the ID is `"Weak Potion"`. Python uses `id="Weak Potion"` which is correct.

### Issue 5: 25 potions have no effect implementation (HIGH)
Most potions cannot actually be used in combat. Only basic damage/block/buff/debuff potions work.

## Recommendations

1. **Consolidate** the two use_potion implementations into one that uses content/potions.py data
2. **Add Sacred Bark** checks to combat_engine.py (or deprecate it in favor of handlers/combat.py)
3. **Implement** the remaining 25 potion effects, prioritizing:
   - Fairy in a Bottle (death prevention)
   - Smoke Bomb (combat escape)
   - Fruit Juice (max HP, usable outside combat)
   - Speed/Flex Potions (temp buffs with end-of-turn loss)
   - Discovery potions (Attack/Skill/Power/Colorless)
4. **Fix** Block Potion to apply Frail if the player is Frail
5. **Standardize** potion ID format (spaces vs camelCase)
