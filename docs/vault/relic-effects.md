# Slay the Spire - Complete Relic Effects Reference

Extracted from decompiled source code. Includes mechanical effects, trigger conditions, and counter mechanics.

## Table of Contents
- [Watcher-Specific Relics](#watcher-specific-relics)
- [Starter Relics](#starter-relics)
- [Damage Calculation Relics](#damage-calculation-relics)
- [Energy Relics](#energy-relics)
- [Card Draw Relics](#card-draw-relics)
- [Common Relics](#common-relics)
- [Uncommon Relics](#uncommon-relics)
- [Rare Relics](#rare-relics)
- [Boss Relics](#boss-relics)
- [Shop Relics](#shop-relics)
- [Event/Special Relics](#eventspecial-relics)

---

## Watcher-Specific Relics

### Pure Water
- **ID:** `PureWater`
- **Tier:** Starter (Watcher)
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Add 1 Miracle card to hand at combat start
- **Code:** `new MakeTempCardInHandAction(new Miracle(), 1, false)`

### Holy Water (Upgrade of Pure Water)
- **ID:** `HolyWater`
- **Tier:** Boss (upgrades Pure Water)
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Add 3 Miracles to hand at combat start
- **Code:** `new MakeTempCardInHandAction(new Miracle(), 3, false)`

### Violet Lotus
- **ID:** `Violet Lotus`
- **Tier:** Boss (Watcher exclusive)
- **Trigger:** `onChangeStance()` when exiting Calm
- **Effect:** Gain 1 additional Energy when exiting Calm stance (total +3 instead of +2)
- **Code:** `new GainEnergyAction(1)` on Calm exit

### Teardrop Locket
- **ID:** `TeardropLocket`
- **Tier:** Boss (Watcher exclusive)
- **Trigger:** `atBattleStart()`
- **Effect:** Start each combat in Calm stance
- **Code:** `new ChangeStanceAction(new CalmStance())`

### Damaru
- **ID:** `Damaru`
- **Tier:** Uncommon (Watcher exclusive)
- **Trigger:** `atTurnStart()`
- **Effect:** Gain 1 Mantra at the start of each turn
- **Code:** `new ApplyPowerAction(player, player, new MantraPower(player, 1), 1)`

### Duality
- **ID:** `Duality`
- **Tier:** Uncommon (Watcher exclusive)
- **Trigger:** `onPlayCard()` when playing an Attack
- **Effect:** Gain 1 Dexterity for turn when playing an Attack
- **Code:** `new ApplyPowerAction(player, player, new DexterityPower(player, 1), 1)` (not permanent, uses LoseDexterityPower to remove at turn end)

### Cloak Clasp
- **ID:** `CloakClasp`
- **Tier:** Uncommon (Watcher exclusive)
- **Trigger:** `onPlayerEndTurn()`
- **Effect:** Gain 1 Block for each card in hand at end of turn
- **Code:** `new GainBlockAction(player, player, AbstractDungeon.player.hand.size())`

### Golden Eye
- **ID:** `GoldenEye`
- **Tier:** Rare (Watcher exclusive)
- **Trigger:** `onShuffle()`
- **Effect:** Scry 2 whenever you shuffle your draw pile
- **Counter:** Uses counter to track (sets to 2 on shuffle)
- **Code:** `new ScryAction(2)`

---

## Starter Relics

### Burning Blood (Ironclad)
- **ID:** `Burning Blood`
- **Tier:** Starter
- **Trigger:** `onVictory()`
- **Effect:** Heal 6 HP at end of combat
- **Code:** `player.heal(6)`

### Black Blood (Upgrade of Burning Blood)
- **ID:** `Black Blood`
- **Tier:** Boss (upgrades Burning Blood)
- **Trigger:** `onVictory()`
- **Effect:** Heal 12 HP at end of combat
- **Code:** `player.heal(12)`

### Cracked Core (Defect)
- **ID:** `Cracked Core`
- **Tier:** Starter
- **Trigger:** `atPreBattle()`
- **Effect:** Channel 1 Lightning orb at combat start
- **Code:** `player.channelOrb(new Lightning())`

### Frozen Core (Upgrade of Cracked Core)
- **ID:** `FrozenCore`
- **Tier:** Boss (upgrades Cracked Core)
- **Trigger:** `onPlayerEndTurn()` when empty orb slot exists
- **Effect:** If you end turn with empty orb slot, channel 1 Frost
- **Code:** `new ChannelAction(new Frost())`

### Ring of the Snake (Silent)
- **ID:** `Ring of the Snake`
- **Tier:** Starter
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Draw 2 extra cards at start of combat
- **Code:** `new DrawCardAction(player, 2)`

### Ring of the Serpent (Upgrade of Ring of the Snake)
- **ID:** `Ring of the Serpent`
- **Tier:** Boss (upgrades Ring of the Snake)
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Draw 1 additional card at start of each combat (total 3 extra)
- **Code:** `new DrawCardAction(player, 1)` (on top of normal draw)

---

## Damage Calculation Relics

### Vajra
- **ID:** `Vajra`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 1 Strength at combat start
- **Code:** `new ApplyPowerAction(player, player, new StrengthPower(player, 1), 1)`
- **Damage Impact:** +1 damage per Attack card hit

### Pen Nib
- **ID:** `Pen Nib`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Attack cards
- **Counter:** Tracks attacks played (0-9)
- **Effect:** Every 10th Attack deals double damage
- **Code:** Counter increments on Attack. At counter=10, `counter=0` and next attack doubled
- **Damage Impact:** 2x damage multiplier on 10th Attack

### Akabeko
- **ID:** `Akabeko`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** First Attack each combat deals +8 damage (Vigor buff)
- **Code:** `new ApplyPowerAction(player, player, new VigorPower(player, 8), 8)`
- **Damage Impact:** +8 damage on first Attack only

### Strike Dummy
- **ID:** `StrikeDummy`
- **Tier:** Uncommon
- **Trigger:** `atDamageModify()` on Strike cards
- **Effect:** Cards with "Strike" in name deal +3 damage
- **Code:** Returns `damage + 3.0f` if card name contains "Strike"
- **Damage Impact:** +3 damage per Strike card

### Wrist Blade
- **ID:** `WristBlade`
- **Tier:** Boss (Silent exclusive)
- **Trigger:** `atDamageModify()` for 0-cost attacks
- **Effect:** Attacks that cost 0 deal +4 damage
- **Code:** Returns `damage + 4.0f` if `c.costForTurn == 0` or `c.freeToPlayOnce && c.cost != -1`
- **Damage Impact:** +4 damage for 0-cost Attacks

### Kunai
- **ID:** `Kunai`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Attack cards
- **Counter:** Tracks attacks per turn (0-2)
- **Effect:** Every 3rd Attack played per turn grants +1 Dexterity
- **Code:** At counter=3, `new ApplyPowerAction(player, player, new DexterityPower(player, 1), 1)`

### Shuriken
- **ID:** `Shuriken`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Attack cards
- **Counter:** Tracks attacks per turn (0-2)
- **Effect:** Every 3rd Attack played per turn grants +1 Strength
- **Code:** At counter=3, `new ApplyPowerAction(player, player, new StrengthPower(player, 1), 1)`
- **Damage Impact:** Cumulative +1 damage per hit

### Ornamental Fan
- **ID:** `Ornamental Fan`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Attack cards
- **Counter:** Tracks attacks per turn (0-2)
- **Effect:** Every 3rd Attack played per turn grants 4 Block
- **Code:** At counter=3, `new GainBlockAction(player, player, 4)`

### Necronomicon
- **ID:** `Necronomicon`
- **Tier:** Rare
- **Trigger:** `onUseCard()` for Attack cards costing 2+
- **Effect:** First Attack each turn that costs 2+ is played twice (once per turn)
- **Code:** `new PlayTopCardAction(action.target, false)` with `triggered = true` flag
- **Damage Impact:** 2x damage for first 2+ cost Attack per turn

### Chemical X
- **ID:** `Chemical X`
- **Tier:** Boss (Silent exclusive)
- **Effect:** X-cost cards get +2 to X value
- **Implementation:** Checked in X-cost card code, not in relic (passive effect)
- **Damage Impact:** +2X damage on X-cost damage cards

### Paper Frog
- **ID:** `Paper Frog`
- **Tier:** Uncommon
- **Effect:** Vulnerable now increases damage by 75% instead of 50%
- **Code:** `VULN_EFFECTIVENESS = 1.75f`
- **Damage Impact:** Vulnerable = 1.75x damage (instead of 1.5x)

### Paper Crane
- **ID:** `Paper Crane`
- **Tier:** Uncommon
- **Effect:** Weak now reduces damage by 40% instead of 25%
- **Code:** `WEAK_EFFECTIVENESS = 0.6f` (enemies deal 60% damage when Weak)
- **Damage Impact:** Reduces incoming damage by additional 15%

### Boot
- **ID:** `The Boot`
- **Tier:** Common
- **Effect:** Attacks deal minimum 5 damage
- **Code:** `if (damage > 0 && damage < 5) return 5`
- **Damage Impact:** Floor of 5 damage on non-zero attacks

### Bronze Scales
- **ID:** `Bronze Scales`
- **Tier:** Common
- **Trigger:** `onAttacked()` (passive thorns)
- **Effect:** Whenever you take damage from attack, deal 3 damage back
- **Code:** `new ApplyPowerAction(player, player, new ThornsPower(player, 3), 3)` at battle start
- **Damage Impact:** 3 Thorns damage per hit received

---

## Energy Relics

### Lantern
- **ID:** `Lantern`
- **Tier:** Common
- **Trigger:** `atTurnStart()` on first turn only
- **Effect:** Gain 1 Energy on first turn of combat
- **Code:** `new GainEnergyAction(1)` with `firstTurn` flag
- **Energy Impact:** +1 Energy turn 1

### Art of War
- **ID:** `Art of War`
- **Tier:** Uncommon
- **Trigger:** `onPlayerEndTurn()` when no Attack played
- **Effect:** If you play no Attacks during turn, gain 1 extra Energy next turn
- **Code:** Applies `EnergizedBluePower` for next turn
- **Energy Impact:** +1 Energy next turn (conditional)

### Happy Flower
- **ID:** `Happy Flower`
- **Tier:** Common
- **Trigger:** `atTurnStart()`
- **Counter:** 0-2, resets to 0 when reaching 3
- **Effect:** Every 3 turns, gain 1 Energy
- **Code:** At counter=3, `new GainEnergyAction(1)`
- **Energy Impact:** +1 Energy every 3rd turn

### Nunchaku
- **ID:** `Nunchaku`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Attack cards
- **Counter:** 0-9, resets on trigger
- **Effect:** Every 10th Attack played grants 1 Energy
- **Code:** At counter=10, `new GainEnergyAction(1)`
- **Energy Impact:** +1 Energy per 10 Attacks

### Sundial
- **ID:** `Sundial`
- **Tier:** Uncommon
- **Trigger:** `onShuffle()`
- **Counter:** 0-2, resets on trigger
- **Effect:** Every 3rd shuffle, gain 2 Energy
- **Code:** At counter=3, `new GainEnergyAction(2)`
- **Energy Impact:** +2 Energy per 3 shuffles

### Gremlin Horn
- **ID:** `Gremlin Horn`
- **Tier:** Uncommon
- **Trigger:** `onMonsterDeath()`
- **Effect:** When enemy dies, gain 1 Energy and draw 1 card
- **Code:** `new GainEnergyAction(1)` + `new DrawCardAction(player, 1)`
- **Energy Impact:** +1 Energy per enemy kill

### Ancient Tea Set
- **ID:** `Ancient Tea Set`
- **Tier:** Common
- **Trigger:** `atTurnStart()` first turn after resting
- **Counter:** -2 when rested, -1 after triggered
- **Effect:** After resting, gain 2 Energy on first turn of next combat
- **Code:** `new GainEnergyAction(2)` if `counter == -2`
- **Energy Impact:** +2 Energy (conditional on rest)

### Ice Cream
- **ID:** `Ice Cream`
- **Tier:** Rare
- **Effect:** Energy no longer depletes at end of turn (passive - checked in energy system)
- **Energy Impact:** Energy carries over between turns

### Hovering Kite
- **ID:** `HoveringKite`
- **Tier:** Boss (Silent exclusive)
- **Trigger:** `onManualDiscard()` (first per turn)
- **Effect:** First card discarded each turn grants 1 Energy
- **Code:** `new GainEnergyAction(1)` with `triggeredThisTurn` flag
- **Energy Impact:** +1 Energy per turn (requires discard)

### Boss Energy Relics (Common Pattern)
All grant +1 max Energy with downside:
- **Cursed Key** (`Cursed Key`): Gain curse on chest open
- **Ectoplasm** (`Ectoplasm`): Cannot gain Gold
- **Busted Crown** (`Busted Crown`): 2 fewer card rewards
- **Coffee Dripper** (`CoffeeDripper`): Cannot rest at campfires
- **Fusion Hammer** (`FusionHammer`): Cannot upgrade cards
- **Runic Dome** (`Runic Dome`): Cannot see enemy intents
- **Philosopher's Stone** (`Philosopher's Stone`): Enemies gain 1 Strength
- **Velvet Choker** (`Velvet Choker`): Cannot play more than 6 cards/turn
- **Slaver's Collar** (`SlaversCollar`): +1 Energy in Elite/Boss fights only
- **Sozu** (`Sozu`): Cannot obtain potions
- **Mark of Pain** (`Mark of Pain`): Shuffle 2 Wounds into draw at combat start

---

## Card Draw Relics

### Gambling Chip
- **ID:** `GamblingChip`
- **Tier:** Boss
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** At combat start, discard any cards and draw that many
- **Code:** Opens grid select screen, allows selecting cards to discard, draws equal amount
- **Draw Impact:** Variable - replaces unwanted starting cards

### Bag of Preparation
- **ID:** `Bag of Preparation`
- **Tier:** Common
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Draw 2 additional cards at start of combat
- **Code:** `new DrawCardAction(player, 2)`
- **Draw Impact:** +2 cards turn 1

### Ink Bottle
- **ID:** `Ink Bottle`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()`
- **Counter:** 0-9, resets on trigger
- **Effect:** Every 10th card played, draw 1 card
- **Code:** At counter=10, `new DrawCardAction(player, 1)`
- **Draw Impact:** +1 card per 10 plays

### Unceasing Top
- **ID:** `Unceasing Top`
- **Tier:** Rare
- **Trigger:** `onRefreshHand()` when hand empty
- **Effect:** Whenever hand is empty, draw 1 card
- **Code:** Checks `player.hand.isEmpty()` and `!actionManager.actions.isEmpty()`, draws if true
- **Draw Impact:** Continuous draw while hand empty

### Runic Cube
- **ID:** `Runic Cube`
- **Tier:** Boss
- **Trigger:** `wasHPLost()`
- **Effect:** Whenever you lose HP, draw 1 card
- **Code:** `new DrawCardAction(player, 1)` on HP loss > 0
- **Draw Impact:** +1 card per HP loss instance

### Centennial Puzzle
- **ID:** `Centennial Puzzle`
- **Tier:** Common
- **Trigger:** `wasHPLost()` (first time per combat)
- **Effect:** First time you lose HP each combat, draw 3 cards
- **Code:** `new DrawCardAction(player, 3)` with `usedThisCombat` flag
- **Draw Impact:** +3 cards once per combat

### Pocketwatch
- **ID:** `Pocketwatch`
- **Tier:** Rare
- **Trigger:** `atTurnStartPostDraw()` if played <=3 cards last turn
- **Counter:** Cards played (resets each turn)
- **Effect:** If you played 3 or fewer cards last turn, draw 3
- **Code:** `new DrawCardAction(player, 3)` if `counter <= 3`
- **Draw Impact:** +3 cards (conditional on low activity)

### Dead Branch
- **ID:** `Dead Branch`
- **Tier:** Rare
- **Trigger:** `onExhaust()`
- **Effect:** Whenever you exhaust a card, add random card to hand
- **Code:** `new MakeTempCardInHandAction(returnTrulyRandomCard(), 1)`
- **Draw Impact:** +1 random card per exhaust

---

## Common Relics

### Anchor
- **ID:** `Anchor`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 10 Block at combat start
- **Code:** `new GainBlockAction(player, player, 10)`

### Bag of Marbles
- **ID:** `Bag of Marbles`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Apply 1 Vulnerable to ALL enemies at combat start
- **Code:** `new ApplyPowerAction(m, player, new VulnerablePower(m, 1, false), 1)` for each monster

### Blood Vial
- **ID:** `Blood Vial`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Heal 2 HP at combat start
- **Code:** `new HealAction(player, player, 2, 0.0f)`

### Orichalcum
- **ID:** `Orichalcum`
- **Tier:** Common
- **Trigger:** `onPlayerEndTurn()` when Block = 0
- **Effect:** If you end turn with 0 Block, gain 6 Block
- **Code:** `new GainBlockAction(player, player, 6)` if `player.currentBlock == 0`

### Oddly Smooth Stone
- **ID:** `Oddly Smooth Stone`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 1 Dexterity at combat start
- **Code:** `new ApplyPowerAction(player, player, new DexterityPower(player, 1), 1)`

### Red Skull
- **ID:** `Red Skull`
- **Tier:** Common
- **Trigger:** `onBloodied()` / `onNotBloodied()`
- **Effect:** While at 50% or less HP, gain 3 Strength (removed when healed above)
- **Code:** `new ApplyPowerAction(player, player, new StrengthPower(player, 3), 3)` on bloodied
- **Dynamic:** +3/-3 Strength as you cross 50% HP threshold

### Toy Ornithopter
- **ID:** `Toy Ornithopter`
- **Tier:** Common
- **Trigger:** `onUsePotion()`
- **Effect:** Heal 5 HP whenever you use a potion
- **Code:** `new HealAction(player, player, 5)`

### Data Disk (Defect)
- **ID:** `DataDisk`
- **Tier:** Common
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 1 Focus at combat start
- **Code:** `new ApplyPowerAction(player, player, new FocusPower(player, 1), 1)`

### Preserved Insect
- **ID:** `PreservedInsect`
- **Tier:** Common
- **Trigger:** `atBattleStart()` in Elite fights
- **Effect:** Elites start with 25% less HP
- **Code:** Sets `m.currentHealth = (int)((float)m.maxHealth * 0.75f)`

### Potion Belt
- **ID:** `Potion Belt`
- **Tier:** Common
- **Trigger:** `onEquip()`
- **Effect:** +2 potion slots (permanent)
- **Code:** `player.potionSlots += 2`

### Omamori
- **ID:** `Omamori`
- **Tier:** Common
- **Counter:** Starts at 2, decrements when curse would be gained
- **Effect:** Negate next 2 curses you would gain
- **Code:** Counter-based, `use()` decrements and blocks curse

### Dream Catcher
- **ID:** `Dream Catcher`
- **Tier:** Common
- **Effect:** Whenever you rest, add a card to your deck (opens card select)
- **Implementation:** Checked in rest site code

### Regal Pillow
- **ID:** `Regal Pillow`
- **Tier:** Common
- **Effect:** Heal an additional 15 HP when you rest
- **Code:** Passive - checked in rest heal calculation
- **Heal Impact:** +15 HP per rest

### Snecko Skull (Silent)
- **ID:** `Snake Skull`
- **Tier:** Common
- **Effect:** Whenever you apply Poison, apply 1 additional Poison
- **Code:** `EFFECT = 1` (checked in poison application)

---

## Uncommon Relics

### Bottled Flame/Lightning/Tornado
- **ID:** `Bottled Flame`, `Bottled Lightning`, `Bottled Tornado`
- **Tier:** Uncommon
- **Trigger:** `onEquip()` + `atBattleStart()`
- **Effect:** Choose Attack/Skill/Power to always start in hand
- **Code:** Sets `card.inBottleFlame/Lightning/Tornado = true`, card system puts in opening hand

### Meat on the Bone
- **ID:** `Meat on the Bone`
- **Tier:** Uncommon
- **Trigger:** `onTrigger()` at end of combat when at 50% or less HP
- **Effect:** If at 50% or less HP at end of combat, heal 12
- **Code:** `player.heal(12)` if `currentHealth <= maxHealth / 2`

### Self-Forming Clay
- **ID:** `Self Forming Clay`
- **Tier:** Uncommon
- **Trigger:** `wasHPLost()`
- **Effect:** Whenever you lose HP, gain 3 Block next turn
- **Code:** `new ApplyPowerAction(player, player, new NextTurnBlockPower(player, 3), 3)`

### Frozen Egg
- **ID:** `Frozen Egg 2`
- **Tier:** Uncommon
- **Trigger:** `onObtainCard()` for Power cards
- **Effect:** Power cards you obtain are automatically upgraded
- **Code:** `c.upgrade()` if `c.type == CardType.POWER && c.canUpgrade()`

### Molten Egg
- **ID:** `Molten Egg 2`
- **Tier:** Uncommon
- **Trigger:** `onObtainCard()` for Attack cards
- **Effect:** Attack cards you obtain are automatically upgraded
- **Code:** `c.upgrade()` if `c.type == CardType.ATTACK && c.canUpgrade()`

### Toxic Egg
- **ID:** `Toxic Egg 2`
- **Tier:** Uncommon
- **Trigger:** `onObtainCard()` for Skill cards
- **Effect:** Skill cards you obtain are automatically upgraded
- **Code:** `c.upgrade()` if `c.type == CardType.SKILL && c.canUpgrade()`

### Blue Candle
- **ID:** `Blue Candle`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Curse cards
- **Effect:** Curse cards can be played. Playing them exhausts and deals 1 HP loss
- **Code:** Makes curses playable, `new LoseHPAction(player, player, 1)` + `card.exhaust = true`

### Mercury Hourglass
- **ID:** `Mercury Hourglass`
- **Tier:** Uncommon
- **Trigger:** `atTurnStart()`
- **Effect:** At the start of your turn, deal 3 damage to ALL enemies
- **Code:** `new DamageAllEnemiesAction(null, createDamageMatrix(3, true), THORNS, BLUNT_LIGHT)`

### Letter Opener
- **ID:** `Letter Opener`
- **Tier:** Uncommon
- **Trigger:** `onUseCard()` for Skill cards
- **Counter:** 0-2, resets on trigger
- **Effect:** Every 3rd Skill played, deal 5 damage to ALL enemies
- **Code:** At counter=3, `new DamageAllEnemiesAction(null, createDamageMatrix(5, true), THORNS, SLASH_HEAVY)`

### Ninja Scroll (Silent)
- **ID:** `Ninja Scroll`
- **Tier:** Uncommon
- **Trigger:** `atBattleStartPreDraw()`
- **Effect:** Start combat with 3 Shivs in hand
- **Code:** `new MakeTempCardInHandAction(new Shiv(), 3, false)`

### Horn Cleat
- **ID:** `HornCleat`
- **Tier:** Uncommon
- **Trigger:** `atTurnStart()` on turn 2
- **Counter:** Tracks turns
- **Effect:** At the start of turn 2, gain 14 Block
- **Code:** At counter=2, `new GainBlockAction(player, player, 14)` then grays out

### Symbiotic Virus (Defect)
- **ID:** `Symbiotic Virus`
- **Tier:** Uncommon
- **Trigger:** `atPreBattle()`
- **Effect:** Channel 1 Dark orb at combat start
- **Code:** `player.channelOrb(new Dark())`

### Gold-Plated Cables (Defect)
- **ID:** `Cables`
- **Tier:** Uncommon
- **Effect:** Rightmost orb triggers its passive an additional time
- **Code:** Passive - checked in orb trigger code

### Pantograph
- **ID:** `Pantograph`
- **Tier:** Uncommon
- **Trigger:** `atBattleStart()` when fighting Boss
- **Effect:** Heal 25 HP at the start of Boss fights
- **Code:** `new HealAction(player, player, 25, 0.0f)` if monster type is BOSS

### Darkstone Periapt
- **ID:** `Darkstone Periapt`
- **Tier:** Uncommon
- **Trigger:** `onObtainCard()` for Curse cards
- **Effect:** Whenever you gain a Curse, gain 6 Max HP
- **Code:** `player.increaseMaxHp(6, true)` if `card.color == CardColor.CURSE`

---

## Rare Relics

### Torii
- **ID:** `Torii`
- **Tier:** Rare
- **Effect:** When you would receive 5 or less damage, reduce to 1
- **Code:** Passive - checked in damage calculation
- **Damage Reduction:** Attacks dealing 2-5 reduced to 1

### Tungsten Rod
- **ID:** `Tungsten Rod`
- **Tier:** Rare
- **Effect:** When you would lose HP, lose 1 less
- **Code:** Passive - checked in HP loss calculation
- **Damage Reduction:** -1 to all HP loss

### Calipers
- **ID:** `Calipers`
- **Tier:** Rare
- **Effect:** Block decays by 15 instead of all at turn start
- **Code:** `BLOCK_LOSS = 15` - checked in block decay system

### Girya
- **ID:** `Girya`
- **Tier:** Rare
- **Trigger:** `atBattleStart()` + campfire option
- **Counter:** 0-3 (strength gained from lifting)
- **Effect:** Can Lift at campfire (max 3 times), gain that much Strength at combat start
- **Code:** `new ApplyPowerAction(player, player, new StrengthPower(player, counter), counter)`

### Peace Pipe
- **ID:** `Peace Pipe`
- **Tier:** Rare
- **Effect:** Adds "Toke" option at campfires - remove a card
- **Code:** Adds `TokeOption` to campfire options

### Shovel
- **ID:** `Shovel`
- **Tier:** Rare
- **Effect:** Adds "Dig" option at campfires - get a relic
- **Code:** Adds `DigOption` to campfire options

### Lizard Tail
- **ID:** `Lizard Tail`
- **Tier:** Rare
- **Trigger:** `onTrigger()` when would die
- **Counter:** -1 (unused), -2 (used up)
- **Effect:** Once per run, revive at 50% HP when you would die
- **Code:** `player.heal(maxHealth / 2, true)` then `setCounter(-2)`

### Incense Burner
- **ID:** `Incense Burner`
- **Tier:** Rare
- **Trigger:** `atTurnStart()`
- **Counter:** 0-5, resets on trigger
- **Effect:** Every 6 turns, gain 1 Intangible
- **Code:** At counter=6, `new ApplyPowerAction(player, null, new IntangiblePlayerPower(player, 1), 1)`

### Thread and Needle
- **ID:** `Thread and Needle`
- **Tier:** Rare
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 4 Plated Armor at combat start
- **Code:** `new ApplyPowerAction(player, player, new PlatedArmorPower(player, 4), 4)`

### Fossilized Helix
- **ID:** `FossilizedHelix`
- **Tier:** Rare
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 1 Buffer at combat start (prevent first instance of damage)
- **Code:** `new ApplyPowerAction(player, player, new BufferPower(player, 1), 1)`

### Captain's Wheel
- **ID:** `CaptainsWheel`
- **Tier:** Rare
- **Trigger:** `atTurnStart()` on turn 3
- **Counter:** Tracks turns
- **Effect:** At the start of turn 3, gain 18 Block
- **Code:** At counter=3, `new GainBlockAction(player, player, 18)` then grays out

### Stone Calendar
- **ID:** `StoneCalendar`
- **Tier:** Rare
- **Trigger:** `onPlayerEndTurn()` on turn 7
- **Counter:** Tracks turns
- **Effect:** At the end of turn 7, deal 52 damage to ALL enemies
- **Code:** At counter=7, `new DamageAllEnemiesAction(null, createDamageMatrix(52, true), THORNS, BLUNT_HEAVY)`

### Old Coin
- **ID:** `Old Coin`
- **Tier:** Rare
- **Trigger:** `onEquip()`
- **Effect:** Gain 300 Gold on pickup
- **Code:** `player.gainGold(300)`

### Champion's Belt
- **ID:** `Champion Belt`
- **Tier:** Rare
- **Trigger:** `onTrigger()` when applying Vulnerable
- **Effect:** Whenever you apply Vulnerable, also apply 1 Weak
- **Code:** `new ApplyPowerAction(target, player, new WeakPower(target, 1, false), 1)`

### Charon's Ashes
- **ID:** `Charon's Ashes`
- **Tier:** Rare
- **Trigger:** `onExhaust()`
- **Effect:** Whenever you exhaust a card, deal 3 damage to ALL enemies
- **Code:** `new DamageAllEnemiesAction(null, createDamageMatrix(3, true), THORNS, FIRE)`

### Tingsha (Silent)
- **ID:** `Tingsha`
- **Tier:** Rare
- **Trigger:** `onManualDiscard()`
- **Effect:** Whenever you discard a card, deal 3 damage to random enemy
- **Code:** `new DamageRandomEnemyAction(new DamageInfo(player, 3, THORNS), FIRE)`

### Tough Bandages (Silent)
- **ID:** `Tough Bandages`
- **Tier:** Rare
- **Trigger:** `onManualDiscard()`
- **Effect:** Whenever you discard a card, gain 3 Block
- **Code:** `new GainBlockAction(player, player, 3, true)`

### The Specimen (Silent)
- **ID:** `The Specimen`
- **Tier:** Rare
- **Trigger:** `onMonsterDeath()` when poisoned enemy dies
- **Effect:** When poisoned enemy dies, transfer poison to random enemy
- **Code:** `new ApplyPowerToRandomEnemyAction(player, new PoisonPower(null, player, amount), amount)`

### Emotion Chip (Defect)
- **ID:** `Emotion Chip`
- **Tier:** Rare
- **Trigger:** `atTurnStart()` after taking damage last turn
- **Effect:** If you took damage last turn, trigger passive of all orbs at turn start
- **Code:** `new ImpulseAction()` (triggers all orb passives)

### Mummified Hand
- **ID:** `Mummified Hand`
- **Tier:** Rare
- **Trigger:** `onUseCard()` for Power cards
- **Effect:** Whenever you play a Power, a random card in hand costs 0 this turn
- **Code:** `new MummifiedHandAction()` - sets random card cost to 0

---

## Boss Relics

### Snecko Eye
- **ID:** `Snecko Eye`
- **Tier:** Boss
- **Trigger:** `atBattleStart()` + continuous effect
- **Effect:** Draw 2 extra cards each turn. All cards costs are randomized (0-3)
- **Code:** Applies `SneckoPower` + `new DrawCardAction(player, 2)` at battle start

### Runic Pyramid
- **ID:** `Runic Pyramid`
- **Tier:** Boss
- **Effect:** You no longer discard cards at end of turn (passive)
- **Code:** Checked in discard phase - skipped when relic present

### Calling Bell
- **ID:** `Calling Bell`
- **Tier:** Boss
- **Trigger:** `onEquip()`
- **Effect:** Obtain 3 relics (Common, Uncommon, Rare) but gain Curse of the Bell
- **Code:** Opens reward screen with 3 relics after adding Curse of the Bell

### Sacred Bark
- **ID:** `SacredBark`
- **Tier:** Boss
- **Effect:** Double the effectiveness of all potions
- **Code:** Potions check for this relic and multiply effects by 2

### Nuclear Battery (Defect)
- **ID:** `Nuclear Battery`
- **Tier:** Boss
- **Trigger:** `atPreBattle()`
- **Effect:** Channel 1 Plasma orb at combat start
- **Code:** `player.channelOrb(new Plasma())`

### Inserter (Defect)
- **ID:** `Inserter`
- **Tier:** Boss
- **Trigger:** `atTurnStart()`
- **Counter:** 0-1, resets on trigger
- **Effect:** Every other turn, gain 1 orb slot
- **Code:** At counter=2, `new IncreaseMaxOrbAction(1)`

---

## Shop Relics

### Prismatic Shard
- **ID:** `PrismaticShard`
- **Tier:** Shop
- **Trigger:** `onEquip()`
- **Effect:** Card rewards can contain cards from any color. Non-Defects get 1 orb slot.
- **Code:** Sets `masterMaxOrbs = 1` if not Defect

### Sling
- **ID:** `Sling`
- **Tier:** Shop
- **Trigger:** `atBattleStart()` in Elite fights
- **Effect:** At the start of Elite combats, gain 2 Strength
- **Code:** `new ApplyPowerAction(player, player, new StrengthPower(player, 2), 2)` if `eliteTrigger`

### Hand Drill
- **ID:** `HandDrill`
- **Tier:** Shop
- **Trigger:** `onBlockBroken()`
- **Effect:** When you break enemy Block, apply 2 Vulnerable
- **Code:** `new ApplyPowerAction(m, player, new VulnerablePower(m, 2, false), 2)`

### Medical Kit
- **ID:** `Medical Kit`
- **Tier:** Shop
- **Trigger:** `onUseCard()` for Status cards
- **Effect:** Status cards can be played and exhaust when played
- **Code:** `card.exhaust = true; action.exhaustCard = true` for Status cards

### Orange Pellets
- **ID:** `OrangePellets`
- **Tier:** Shop
- **Trigger:** `onUseCard()` when all 3 types played
- **Effect:** When you play Attack, Skill, and Power in same turn, remove all debuffs
- **Code:** Tracks `SKILL/POWER/ATTACK` flags, `new RemoveDebuffsAction(player)` when all true

### Brimstone (Ironclad)
- **ID:** `Brimstone`
- **Tier:** Shop
- **Trigger:** `atTurnStart()`
- **Effect:** At the start of each turn, ALL enemies gain 1 Strength, you gain 2 Strength
- **Code:** Applies +1 Strength to all monsters, +2 to player

### Runic Capacitor (Defect)
- **ID:** `Runic Capacitor`
- **Tier:** Shop
- **Trigger:** `atTurnStart()` first turn only
- **Effect:** Gain 3 orb slots at the start of each combat
- **Code:** `new IncreaseMaxOrbAction(3)` with `firstTurn` flag

### Twisted Funnel (Silent)
- **ID:** `TwistedFunnel`
- **Tier:** Shop
- **Trigger:** `atBattleStart()`
- **Effect:** Apply 4 Poison to ALL enemies at combat start
- **Code:** `new ApplyPowerAction(m, player, new PoisonPower(m, player, 4), 4)` for each monster

### The Abacus
- **ID:** `TheAbacus`
- **Tier:** Shop
- **Trigger:** `onShuffle()`
- **Effect:** Gain 6 Block whenever you shuffle your draw pile
- **Code:** `new GainBlockAction(player, player, 6)`

---

## Event/Special Relics

### Neow's Lament
- **ID:** `NeowsBlessing`
- **Tier:** Special (Neow gift)
- **Trigger:** `atBattleStart()` (first 3 combats)
- **Counter:** Starts at 3, decrements per use
- **Effect:** First 3 combats, enemies start with 1 HP
- **Code:** Sets all `m.currentHealth = 1` for counter > 0

### Mutagenic Strength
- **ID:** `MutagenicStrength`
- **Tier:** Special (Event)
- **Trigger:** `atBattleStart()`
- **Effect:** Gain 3 Strength at combat start, lose 3 Strength at end of turn
- **Code:** Applies `StrengthPower(+3)` and `LoseStrengthPower(3)`

### Nilry's Codex
- **ID:** `Nilry's Codex`
- **Tier:** Special (Event)
- **Trigger:** `onPlayerEndTurn()`
- **Effect:** At the end of each turn, you may add a card from 1 of 3 random choices to hand
- **Code:** `new CodexAction()` opens selection screen

### Enchiridion
- **ID:** `Enchiridion`
- **Tier:** Special (Event)
- **Trigger:** `atPreBattle()`
- **Effect:** At combat start, add random Power card to hand, it costs 0 this turn
- **Code:** `new MakeTempCardInHandAction(c)` where c is random Power with cost set to 0

### Warped Tongs
- **ID:** `WarpedTongs`
- **Tier:** Special (Event)
- **Trigger:** `atTurnStartPostDraw()`
- **Effect:** At the start of each turn, upgrade a random card in hand for the combat
- **Code:** `new UpgradeRandomCardAction()`

---

## Key Relic Interactions for Watcher

### Damage Multipliers Stack
Order: Base Damage -> Strength -> Wrath (2x) -> Pen Nib (2x) -> Vulnerable (1.5x or 1.75x with Paper Frog)

### Energy Economy
- **Violet Lotus + Calm cycling**: +3 Energy per Calm exit instead of +2
- **Ice Cream**: Enables banking Energy for big Wrath turns
- **Nunchaku**: Attack-heavy decks generate extra Energy

### Stance Synergies
- **Teardrop Locket**: Start in Calm for guaranteed +2 Energy turn 1
- **Damaru**: Passive Mantra for easier Divinity access
- **Duality**: Watcher Attacks grant defensive Dexterity

### Card Draw Synergies
- **Unceasing Top + Rushdown**: Empty hand in Wrath draws continuously
- **Pocketwatch**: Watcher can often play 3 or fewer cards intentionally

---

## Implementation Notes for Damage Calculator

### Relic Check Order for Damage
```
1. atDamageModify() - Vajra adds to base, Strike Dummy adds +3, Wrist Blade adds +4
2. Pen Nib check - 2x if counter at 10
3. Stance multipliers - Wrath 2x, Divinity 3x
4. Vulnerable check - 1.5x (or 1.75x with Paper Frog)
5. Boot minimum - floor to 5 if 1-4 damage
```

### Key Counter-Based Relics
| Relic | Counter Max | Effect |
|-------|-------------|--------|
| Pen Nib | 10 | 2x damage |
| Nunchaku | 10 | +1 Energy |
| Ink Bottle | 10 | Draw 1 |
| Kunai | 3 | +1 Dex |
| Shuriken | 3 | +1 Str |
| Fan | 3 | +4 Block |
| Letter Opener | 3 | 5 AoE |
| Sundial | 3 | +2 Energy |
| Happy Flower | 3 | +1 Energy |
| Incense Burner | 6 | Intangible |
| Stone Calendar | 7 | 52 AoE |

### Conditional Battle Start Effects
Many relics only trigger once per combat or under specific conditions:
- `firstTurn` flag: Lantern, Ancient Tea Set
- `usedThisCombat` flag: Centennial Puzzle, Necronomicon
- `grayscale` visual: Horn Cleat, Captain's Wheel, Fossilized Helix (indicates used)
