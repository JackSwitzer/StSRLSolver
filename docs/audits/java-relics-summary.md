# Java Relic Implementation Audit

Generated: 2026-03-03

## Summary

| Metric | Count |
|--------|-------|
| **Total Java relics (excl. AbstractRelic, Tests, Deprecated)** | 180 |
| **Python IMPLEMENTED (trigger or passive)** | 129 |
| **Python MISSING** | 45 |
| **Python PARTIAL** | 6 |

Note: "IMPLEMENTED" means the relic has a trigger handler in `relics.py` and/or a passive entry in `relics_passive.py` covering its primary behavior. "PARTIAL" means some hooks are covered but important behavior is missing. "MISSING" means no Python implementation exists.

---

## Starter Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| BurningBlood | `Burning Blood` | `onVictory` | Heal 6 HP after combat. Ironclad starter. | IMPLEMENTED |
| SnakeRing | `Ring of the Snake` | `onEquip`, `onUnequip`, `atTurnStart` | Draw 2 extra cards turn 1. Silent starter. Upgrades to draw 1 extra per turn. | IMPLEMENTED |
| CrackedCore | `Cracked Core` | `atPreBattle` | Channel 1 Lightning at start of combat. Defect starter. | MISSING |
| PureWater | `PureWater` | `atBattleStartPreDraw` | Add 1 Miracle to hand at combat start. Watcher starter. | IMPLEMENTED |

---

## Common Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| Akabeko | `Akabeko` | `atBattleStart` | Gain 8 Vigor at combat start. | IMPLEMENTED |
| Anchor | `Anchor` | `atBattleStart`, `justEnteredRoom` | Gain 10 Block at combat start. | IMPLEMENTED |
| AncientTeaSet | `Ancient Tea Set` | `atTurnStart`, `atPreBattle`, `onEnterRestRoom`, `setCounter`, `canSpawn` | Gain 2 Energy on first turn if last room was rest site. Counter -2 flags activation. | IMPLEMENTED |
| ArtOfWar | `Art of War` | `atPreBattle`, `atTurnStart`, `onUseCard`, `onVictory` | Gain 1 Energy if no Attacks played previous turn. Tracks via boolean flags. | IMPLEMENTED |
| BagOfMarbles | `Bag of Marbles` | `atBattleStart` | Apply 1 Vulnerable to ALL enemies at combat start. | IMPLEMENTED |
| BagOfPreparation | `Bag of Preparation` | `atBattleStart` | Draw 2 extra cards at combat start. | IMPLEMENTED |
| BloodVial | `Blood Vial` | `atBattleStart` | Heal 2 HP at combat start. | IMPLEMENTED |
| Boot | `Boot` | `onAttackToChangeDamage` | If unblocked attack damage is >0 and <5, set to 5 (min 5 damage). | IMPLEMENTED |
| BronzeScales | `Bronze Scales` | `atBattleStart` | Gain 3 Thorns at combat start. | IMPLEMENTED |
| CentennialPuzzle | `Centennial Puzzle` | `atPreBattle`, `wasHPLost`, `onVictory` | Draw 3 cards first time you lose HP each combat. | IMPLEMENTED |
| CeramicFish | `CeramicFish` | `onObtainCard` | Gain 9 Gold when adding a card to deck. | IMPLEMENTED |
| Damaru | `Damaru` | `atTurnStart` | Gain 1 Mantra at start of each turn. Watcher only (via pool). | IMPLEMENTED |
| DataDisk | `DataDisk` | `atBattleStart` | Gain 1 Focus at combat start. Defect only (via pool). | IMPLEMENTED |
| DreamCatcher | `Dream Catcher` | `canSpawn` (passive) | Card reward when resting. No active hooks -- purely checked by rest site code. | IMPLEMENTED (passive) |
| HappyFlower | `Happy Flower` | `onEquip`, `atTurnStart` | Every 3 turns gain 1 Energy. Counter-based relic. | IMPLEMENTED |
| Lantern | `Lantern` | `atPreBattle`, `atTurnStart` | Gain 1 Energy on turn 1. First turn only flag. | IMPLEMENTED |
| MawBank | `MawBank` | `onEnterRoom`, `onSpendGold`, `canSpawn` | Gain 12 Gold entering non-shop rooms. Deactivates on gold spend at shop. | IMPLEMENTED |
| MealTicket | `MealTicket` | `onEnterRoom` (via passive) | Heal 15 HP when entering a shop. | IMPLEMENTED |
| Nunchaku | `Nunchaku` | `onUseCard` | Every 10th Attack played gains 1 Energy. Counter-based. | IMPLEMENTED |
| Omamori | `Omamori` | `setCounter`, `use` (passive) | Negate next 2 Curses. Counter decrements. No active combat hooks. | MISSING |
| Orichalcum | `Orichalcum` | `onPlayerEndTurn`, `atTurnStart`, `onPlayerGainedBlock`, `onVictory` | Gain 6 Block at end of turn if you have 0 Block. | IMPLEMENTED |
| PenNib | `Pen Nib` | `onUseCard`, `atBattleStart` | Every 10th Attack deals double damage. Counter persists across combats. | IMPLEMENTED |
| PotionBelt | `Potion Belt` | `onEquip` | Gain 2 extra potion slots on equip. No combat hooks. | MISSING |
| PreservedInsect | `PreservedInsect` | `atBattleStart` | Elites start with 25% less HP. | IMPLEMENTED |
| RedSkull | `Red Skull` | `atBattleStart`, `onBloodied`, `onNotBloodied`, `onVictory` | Gain 3 Strength while at 50% or less HP. Toggles on HP changes. | IMPLEMENTED |
| RegalPillow | `Regal Pillow` | `canSpawn` (passive) | Heal 15 additional HP when resting. Passive flag only. | IMPLEMENTED (passive) |
| SmilingMask | `Smiling Mask` | `onEnterRoom` (passive) | Card removal always costs 50 Gold. Passive flag. | IMPLEMENTED (passive) |
| SneckoSkull | `Snake Skull` | (passive) | +1 to all Poison applied. No hooks in Java -- checked in ApplyPower. | IMPLEMENTED |
| Strawberry | `Strawberry` | `onEquip` | Gain 7 Max HP on pickup. | IMPLEMENTED |
| TinyChest | `Tiny Chest` | `onEquip` | Every 4th ? room becomes a Treasure room. Counter-based. | MISSING |
| ToyOrnithopter | `Toy Ornithopter` | `onUsePotion` | Heal 5 HP when using a potion. | IMPLEMENTED |
| Vajra | `Vajra` | `atBattleStart` | Gain 1 Strength at combat start. | IMPLEMENTED |
| WarPaint | `War Paint` | `onEquip` | Upgrade 2 random Skills in deck on pickup. | IMPLEMENTED |
| Whetstone | `Whetstone` | `onEquip` | Upgrade 2 random Attacks in deck on pickup. | IMPLEMENTED |
| OddlySmoothStone | `Oddly Smooth Stone` | `atBattleStart` | Gain 1 Dexterity at combat start. | IMPLEMENTED |
| InkBottle | `InkBottle` | `onUseCard`, `atBattleStart` | Every 10th card played draws 1. Counter persists across combats. | IMPLEMENTED |

---

## Uncommon Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| BirdFacedUrn | `Bird Faced Urn` | `onUseCard` | Heal 2 HP when playing a Power card. | IMPLEMENTED |
| BlueCandle | `Blue Candle` | `onUseCard` | Curses can be played; playing exhausts them and costs 1 HP. | PARTIAL (passive flag exists, onUseCard TODO) |
| BottledFlame | `Bottled Flame` | `onEquip`, `onUnequip`, `atBattleStart`, `canSpawn` | Choose an Attack; it starts in hand each combat. | IMPLEMENTED |
| BottledLightning | `Bottled Lightning` | `onEquip`, `onUnequip`, `atBattleStart`, `canSpawn` | Choose a Skill; it starts in hand each combat. | IMPLEMENTED |
| BottledTornado | `Bottled Tornado` | `onEquip`, `onUnequip`, `atBattleStart`, `canSpawn` | Choose a Power; it starts in hand each combat. | IMPLEMENTED |
| Courier | `The Courier` | `onEnterRoom` (passive) | Shop always has a card removal + relic. Passive flag. | IMPLEMENTED (passive) |
| DarkstonePeriapt | `Darkstone Periapt` | `onObtainCard` | Gain 6 Max HP when obtaining a Curse. | IMPLEMENTED |
| DiscerningMonocle | `Discerning Monocle` | `onEnterRoom` (passive) | Shop items cost 25% less. Passive similar to Membership Card. | MISSING |
| Duality | `Yang` | `onUseCard` | Gain 1 temporary Dexterity when playing an Attack. Watcher only. | IMPLEMENTED |
| EternalFeather | `Eternal Feather` | `onEnterRoom` | Heal 3 HP per 5 cards in deck when entering rest site. | MISSING |
| FrozenEgg2 | `Frozen Egg 2` | `onEquip`, `onPreviewObtainCard`, `onObtainCard` | Auto-upgrade Power cards obtained. | IMPLEMENTED |
| GoldPlatedCables | `Cables` | `atTurnStart` (passive) | Trigger first Orb passive at turn start. Defect only. | IMPLEMENTED |
| GremlinHorn | `Gremlin Horn` | `onMonsterDeath` | Gain 1 Energy + draw 1 card when enemy dies. | IMPLEMENTED |
| HornCleat | `HornCleat` | `atBattleStart`, `atTurnStart`, `onVictory` | Gain 14 Block on turn 2. | IMPLEMENTED |
| Kunai | `Kunai` | `atTurnStart`, `onUseCard`, `onVictory` | Gain 1 Dexterity after playing 3 Attacks per turn. | IMPLEMENTED |
| LetterOpener | `Letter Opener` | `atTurnStart`, `onUseCard`, `onVictory` | Deal 5 damage to ALL enemies after playing 3 Skills per turn. | IMPLEMENTED |
| Matryoshka | `Matryoshka` | `onChestOpen`, `setCounter`, `canSpawn` | Next 2 non-boss chests give extra relics. Counter-based. | MISSING |
| MeatOnTheBone | `Meat on the Bone` | `onTrigger`, `onBloodied`, `onNotBloodied` | Heal 12 HP if at <=50% after combat. Called via onTrigger. | IMPLEMENTED |
| MedicalKit | `Medical Kit` | `onUseCard` | Status cards can be played and exhaust. | PARTIAL (passive flag exists, onUseCard TODO) |
| MercuryHourglass | `Mercury Hourglass` | `atTurnStart` | Deal 3 damage to ALL enemies at start of turn. | IMPLEMENTED |
| MoltenEgg2 | `Molten Egg 2` | `onEquip`, `onPreviewObtainCard`, `onObtainCard` | Auto-upgrade Attack cards obtained. | IMPLEMENTED |
| MummifiedHand | `Mummified Hand` | `onUseCard` | Playing a Power reduces a random card in hand cost by 1. | IMPLEMENTED |
| NinjaScroll | `Ninja Scroll` | `atBattleStartPreDraw` | Add 3 Shivs to hand at combat start. Silent only. | IMPLEMENTED |
| OrnamentalFan | `Ornamental Fan` | `atTurnStart`, `onUseCard`, `onVictory` | Gain 4 Block after playing 3 Attacks per turn. | IMPLEMENTED |
| PaperCrane | `Paper Crane` | (passive) | Weak enemies deal 40% damage instead of 25%. Passive modifier. | IMPLEMENTED (passive) |
| PaperFrog | `Paper Frog` | (passive) | Vulnerable enemies take 75% more damage instead of 50%. Passive. | IMPLEMENTED (passive) |
| Pantograph | `Pantograph` | `atBattleStart` | Heal 25 HP at start of boss combat. | IMPLEMENTED |
| Pear | `Pear` | `onEquip` | Gain 10 Max HP on pickup. | IMPLEMENTED |
| QuestionCard | `Question Card` | `canSpawn` (passive) | Card rewards have 1 extra choice. Passive flag. | MISSING |
| SelfFormingClay | `Self Forming Clay` | `wasHPLost` | Gain 3 Block next turn when losing HP. | IMPLEMENTED |
| Shuriken | `Shuriken` | `atTurnStart`, `onUseCard`, `onVictory` | Gain 1 Strength after playing 3 Attacks per turn. | IMPLEMENTED |
| SingingBowl | `Singing Bowl` | (passive) | Skip card reward to gain 2 Max HP. Passive check in reward screen. | MISSING |
| StrikeDummy | `StrikeDummy` | `atDamageModify` | Cards with "Strike" deal 3 extra damage. | IMPLEMENTED |
| Sundial | `Sundial` | `onEquip`, `onShuffle` | Every 3 deck shuffles gain 2 Energy. Counter-based. | IMPLEMENTED |
| SymbioticVirus | `Symbiotic Virus` | `atPreBattle` | Channel 1 Dark orb at combat start. Defect only. | IMPLEMENTED |
| TeardropLocket | `TeardropLocket` | `atBattleStart` | Start combat in Calm stance. Watcher only. | IMPLEMENTED |
| ToxicEgg2 | `Toxic Egg 2` | `onEquip`, `onPreviewObtainCard`, `onObtainCard` | Auto-upgrade Skill cards obtained. | IMPLEMENTED |
| WhiteBeast | `White Beast Statue` | (passive) | Potions drop more frequently. Passive modifier on potionRng. | MISSING |

---

## Rare Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| Calipers | `Calipers` | (passive) | At turn start, lose only 15 Block instead of all. No explicit hooks in Java -- handled by block loss code. | IMPLEMENTED |
| CaptainsWheel | `CaptainsWheel` | `atBattleStart`, `atTurnStart`, `onVictory` | Gain 18 Block at start of turn 3. | IMPLEMENTED |
| ChampionsBelt | `Champion Belt` | `onTrigger(target)` | When applying Vulnerable, also apply 1 Weak. | IMPLEMENTED |
| CharonsAshes | `Charon's Ashes` | `onExhaust` | Deal 3 damage to ALL enemies when a card is exhausted. | IMPLEMENTED |
| CloakClasp | `CloakClasp` | `onPlayerEndTurn` | Gain 1 Block per card in hand at end of turn. | IMPLEMENTED |
| DeadBranch | `Dead Branch` | `onExhaust` | Add a random non-curse, non-status card to hand on exhaust. | IMPLEMENTED |
| DuVuDoll | `Du-Vu Doll` | `onMasterDeckChange`, `onEquip`, `atBattleStart` | Gain 1 Strength per Curse in deck at combat start. | IMPLEMENTED |
| EmotionChip | `Emotion Chip` | `atTurnStart`, `wasHPLost`, `onVictory` | If HP lost last turn, trigger all orb passives at turn start. Defect only. | IMPLEMENTED |
| FossilizedHelix | `FossilizedHelix` | `atBattleStart` | Gain 1 Buffer at combat start. | IMPLEMENTED |
| GamblingChip | `Gambling Chip` | `atBattleStartPreDraw`, `atTurnStartPostDraw` | Discard hand and redraw turn 1. | IMPLEMENTED |
| Ginger | `Ginger` | (passive) | Cannot be Weakened. Passive flag. | IMPLEMENTED (passive) |
| Girya | `Girya` | `atBattleStart`, `addCampfireOption`, `canSpawn` | Lift at rest: +1 Strength (up to 3x). Gain Strength at combat start. | PARTIAL (combat hook works; campfire lift not wired) |
| GoldenEye | `GoldenEye` | (passive) | +2 to all Scry amounts. Watcher only. Passive flag. | IMPLEMENTED (passive) |
| IceCream | `Ice Cream` | (passive) | Unspent Energy is conserved between turns. No hooks -- passive flag. | IMPLEMENTED (passive) |
| IncenseBurner | `Incense Burner` | `onEquip`, `atTurnStart` | Every 6 turns gain 1 Intangible. Counter-based. | IMPLEMENTED |
| LizardTail | `Lizard Tail` | `onTrigger` | When you would die, heal to 50% Max HP. Once per combat. | IMPLEMENTED |
| MagicFlower | `Magic Flower` | `onPlayerHeal` | Healing is 50% more effective. Returns modified heal amount. | IMPLEMENTED |
| Mango | `Mango` | `onEquip` | Gain 14 Max HP on pickup. | IMPLEMENTED |
| OldCoin | `Old Coin` | `onEquip` | Gain 300 Gold on pickup. | MISSING |
| PeacePipe | `Peace Pipe` | `addCampfireOption`, `canSpawn` | Toke at rest: remove a card. No combat hooks. | IMPLEMENTED (passive) |
| Pocketwatch | `Pocketwatch` | `atBattleStart`, `atTurnStartPostDraw`, `onPlayCard`, `onVictory` | Draw 3 next turn if played <=3 cards. Counter-based. | IMPLEMENTED |
| PrayerWheel | `Prayer Wheel` | `canSpawn` (passive) | Get 2 card rewards instead of 1. Passive. | MISSING |
| Shovel | `Shovel` | `addCampfireOption`, `canSpawn` | Dig at rest: get relic. No combat hooks. | IMPLEMENTED (passive) |
| StoneCalendar | `StoneCalendar` | `atBattleStart`, `atTurnStart`, `onPlayerEndTurn`, `onVictory` | Deal 52 damage to ALL enemies at end of turn 7. Counter tracks turns. | IMPLEMENTED |
| StrangeSpoon | `Strange Spoon` | (checked on exhaust) | 50% chance exhausted card goes to discard instead. | IMPLEMENTED |
| TheSpecimen | `The Specimen` | `onMonsterDeath` | Transfer dead enemy's Poison to random living enemy. Silent only. | IMPLEMENTED |
| ThreadAndNeedle | `Thread and Needle` | `atBattleStart` | Gain 4 Plated Armor at combat start. | IMPLEMENTED |
| Tingsha | `Tingsha` | `onManualDiscard` | Deal 3 damage to random enemy when manually discarding. | IMPLEMENTED |
| Torii | `Torii` | `onAttacked` | If taking 2-5 unblocked damage, reduce to 1. | IMPLEMENTED |
| ToughBandages | `Tough Bandages` | `onManualDiscard` | Gain 3 Block when manually discarding. | IMPLEMENTED |
| TungstenRod | `TungstenRod` | `onLoseHpLast` | Reduce all HP loss by 1. | IMPLEMENTED |
| Turnip | `Turnip` | (passive) | Cannot be Frailed. Passive flag. | IMPLEMENTED (passive) |
| UnceasingTop | `Unceasing Top` | `atPreBattle`, `atTurnStart`, `onRefreshHand` | Draw 1 card whenever hand is empty. | IMPLEMENTED |
| WingBoots | `WingedGreaves` | `setCounter`, `canSpawn` (passive) | Fly 3 times per act (ignore paths). Counter-based. Passive. | MISSING |

---

## Boss Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| Astrolabe | `Astrolabe` | `onEquip` | Choose 3 cards to transform and upgrade on pickup. | MISSING |
| BlackBlood | `Black Blood` | `onVictory` | Heal 12 HP after combat. Ironclad upgrade of Burning Blood. | IMPLEMENTED |
| BlackStar | `Black Star` | `onEnterRoom`, `onVictory` | Elites drop 2 relics instead of 1. Passive check. | IMPLEMENTED (passive) |
| BustedCrown | `Busted Crown` | `onEquip`, `onUnequip` | Gain 1 Energy; 2 fewer card choices at reward. Passive + energy. | PARTIAL (passive missing, energy equip missing) |
| CallingBell | `Calling Bell` | `onEquip` | Get 1 relic of each tier + 1 Curse on pickup. | MISSING |
| CoffeeDripper | `Coffee Dripper` | `onEquip`, `onUnequip` | Gain 1 Energy; cannot rest. Passive flag. | IMPLEMENTED (passive) |
| CursedKey | `Cursed Key` | `onChestOpen`, `onEquip`, `onUnequip` | Gain 1 Energy; gain Curse when opening chests. | PARTIAL (passive flag exists; onChestOpen MISSING) |
| Ectoplasm | `Ectoplasm` | `onEquip`, `onUnequip` | Gain 1 Energy; cannot gain Gold. Passive flag. | IMPLEMENTED (passive) |
| EmptyCage | `Empty Cage` | `onEquip` | Remove 2 cards from deck on pickup. | MISSING |
| FrozenCore | `FrozenCore` | `onPlayerEndTurn` | Channel 1 Frost if orb slot empty at end of turn. Defect upgrade of Cracked Core. | IMPLEMENTED |
| FusionHammer | `Fusion Hammer` | `onEquip`, `onUnequip` | Gain 1 Energy; cannot upgrade at rest. Passive flag. | IMPLEMENTED (passive) |
| HolyWater | `HolyWater` | `atBattleStartPreDraw`, `canSpawn` | Add 3 Miracles to hand at combat start. Watcher upgrade of PureWater. | IMPLEMENTED |
| HoveringKite | `HoveringKite` | `atTurnStart`, `onManualDiscard` | First card discarded each turn gives 1 Energy. Silent upgrade. | IMPLEMENTED |
| Inserter | `Inserter` | `onEquip`, `atTurnStart` | Every 2 turns gain 1 Orb Slot. Defect upgrade. | IMPLEMENTED |
| MarkOfPain | `Mark of Pain` | `atBattleStart`, `onEquip`, `onUnequip` | Gain 1 Energy; shuffle 2 Wounds into draw pile at combat start. | IMPLEMENTED |
| NuclearBattery | `Nuclear Battery` | `atPreBattle` | Channel 1 Plasma at combat start. Defect upgrade. | IMPLEMENTED |
| PandorasBox | `Pandora's Box` | `onEquip` | Transform all Strikes and Defends on pickup. | MISSING |
| PhilosopherStone | `Philosopher's Stone` | `atBattleStart`, `onSpawnMonster`, `onEquip`, `onUnequip` | Gain 1 Energy; ALL enemies gain 1 Strength at combat start. | IMPLEMENTED |
| RingOfTheSerpent | `Ring of the Serpent` | `onEquip`, `onUnequip`, `atTurnStart` | Draw 1 extra card per turn. Silent upgrade of Ring of the Snake. | MISSING |
| RunicCube | `Runic Cube` | `wasHPLost` | Draw 1 card whenever you lose HP. Ironclad only. | IMPLEMENTED |
| RunicDome | `Runic Dome` | `onEquip`, `onUnequip` | Gain 1 Energy; cannot see enemy intent. Passive. | MISSING |
| RunicPyramid | `Runic Pyramid` | (passive) | Do not discard hand at end of turn. Passive flag only. | IMPLEMENTED (passive) |
| SacredBark | `SacredBark` | `onEquip` | Double potion effectiveness. Passive flag. | IMPLEMENTED (passive) |
| SlaversCollar | `SlaversCollar` | `beforeEnergyPrep`, `onVictory` | Gain 1 Energy in Elite/Boss fights. Adds/removes energyMaster. | IMPLEMENTED |
| SneckoEye | `Snecko Eye` | `onEquip`, `onUnequip`, `atPreBattle` | Draw 2 extra cards; randomize card costs (Confused). | IMPLEMENTED |
| Sozu | `Sozu` | `onEquip`, `onUnequip` | Gain 1 Energy; cannot obtain potions. Passive flag. | IMPLEMENTED (passive) |
| TinyHouse | `Tiny House` | `onEquip` | Gain 50 Gold, 5 Max HP, potion, card, upgrade a card. | MISSING |
| VelvetChoker | `Velvet Choker` | `onEquip`, `onUnequip`, `atBattleStart`, `atTurnStart`, `onPlayCard`, `onVictory` | Gain 1 Energy; can only play 6 cards per turn. | IMPLEMENTED |
| VioletLotus | `VioletLotus` | `onChangeStance` | Gain 1 extra Energy when exiting Calm stance. Watcher upgrade. | IMPLEMENTED |
| WristBlade | `WristBlade` | `atDamageModify` | 0-cost Attacks deal 4 extra damage. Silent upgrade. | IMPLEMENTED |

---

## Shop Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| Abacus | `TheAbacus` | `onShuffle` | Gain 6 Block when shuffling draw pile. | IMPLEMENTED |
| Brimstone | `Brimstone` | `atTurnStart` | Gain 2 Strength each turn; ALL enemies gain 1 Strength. Ironclad only. | IMPLEMENTED |
| Cauldron | `Cauldron` | `onEquip` | Choose 5 potions to obtain on pickup. | MISSING |
| ChemicalX | `Chemical X` | (passive) | X-cost cards get +2 effect. Passive -- checked in card play code. | MISSING |
| ClockworkSouvenir | `ClockworkSouvenir` | `atBattleStart` | Gain 1 Artifact at combat start. | IMPLEMENTED |
| DollysMirror | `DollysMirror` | `onEquip` | Duplicate a card in deck on pickup. | MISSING |
| FrozenEye | `Frozen Eye` | (passive) | Can see draw pile order. Passive -- UI only. | MISSING |
| HandDrill | `HandDrill` | `onBlockBroken` | Apply 2 Vulnerable when enemy block broken. | IMPLEMENTED |
| MedicalKit | `Medical Kit` | `onUseCard` | Status cards can be played and exhaust. | PARTIAL (passive flag, onUseCard TODO) |
| Melange | `Melange` | `onShuffle` | Scry 3 whenever you shuffle. | IMPLEMENTED |
| MembershipCard | `Membership Card` | `onEnterRoom` (passive) | 50% shop discount. Passive flag. | IMPLEMENTED (passive) |
| OrangePellets | `OrangePellets` | `atTurnStart`, `onUseCard` | Remove all debuffs after playing Attack+Skill+Power same turn. | IMPLEMENTED |
| Orrery | `Orrery` | `onEquip` | Choose 5 cards to add to deck on pickup. Counter = 5 uses. | MISSING |
| PrismaticShard | `PrismaticShard` | `onEquip` | Card rewards can contain any class's cards. | MISSING |
| Sling | `Sling` | `atBattleStart` | Gain 2 Strength in Elite combats. | IMPLEMENTED |
| StrangeSpoon | `Strange Spoon` | (checked on exhaust) | 50% chance exhausted card goes to discard. | IMPLEMENTED |
| TwistedFunnel | `TwistedFunnel` | `atBattleStart` | Apply 4 Poison to ALL enemies at combat start. Silent only. | IMPLEMENTED |
| Waffle | `Lee's Waffle` | `onEquip` | Gain 7 Max HP and heal to full on pickup. | MISSING |

---

## Special/Event Relics

| Java Class | ID | Hooks | Description | Python Status |
|---|---|---|---|---|
| BloodyIdol | `Bloody Idol` | `onGainGold` | Heal 5 HP whenever you gain Gold. | IMPLEMENTED |
| Circlet | `Circlet` | `onEquip`, `onUnequip` | Placeholder relic when all relics exhausted. No game effect. | MISSING |
| CultistMask | `CultistMask` | `atBattleStart` | Gain 1 Ritual power at combat start. | IMPLEMENTED |
| Enchiridion | `Enchiridion` | `atPreBattle` | Add random 0-cost Power to hand at combat start. | IMPLEMENTED |
| FaceOfCleric | `FaceOfCleric` | `onVictory` | Gain 1 Max HP after combat. | IMPLEMENTED |
| GoldenIdol | `Golden Idol` | (passive) | Gain 25% more Gold. Passive flag. | IMPLEMENTED (passive) |
| GremlinMask | `GremlinMask` | `atBattleStart` | Apply 1 Weak to self at combat start. | IMPLEMENTED |
| MarkOfTheBloom | `Mark of the Bloom` | `onPlayerHeal` | Cannot heal. Returns 0 from heal hook. | IMPLEMENTED (passive) |
| MutagenicStrength | `MutagenicStrength` | `atBattleStart` | Gain 3 Strength and 3 LoseStrength at combat start. | IMPLEMENTED |
| Necronomicon | `Necronomicon` | `onEquip`, `onUnequip`, `onUseCard`, `atTurnStart` | First 2+ cost Attack each turn plays twice. | IMPLEMENTED |
| NeowsLament | `NeowsBlessing` | `atBattleStart`, `setCounter` | First 3 enemies have 1 HP. Counter decrements. | MISSING |
| NilrysCodex | `Nilry's Codex` | `onPlayerEndTurn` | End of turn: choose 1 of 3 random cards to add. | IMPLEMENTED |
| NlothsGift | `Nloth's Gift` | (passive) | N'loth takes a relic from you. Event reward. | MISSING |
| NlothsMask | `NlothsMask` | `onChestOpenAfter` | After opening a chest, get back the relic N'loth took. | MISSING |
| OddMushroom | `Odd Mushroom` | (passive) | Vulnerable only increases damage taken by 25% instead of 50%. | IMPLEMENTED (passive) |
| RedCirclet | `Red Circlet` | (passive) | Obtained when relic pools exhausted. No game effect. | MISSING |
| RedMask | `Red Mask` | `atBattleStart` | Apply 1 Weak to ALL enemies at combat start. | IMPLEMENTED |
| SpiritPoop | `Spirit Poop` | (passive) | No game effect. Just a joke relic. | MISSING |
| SsserpentHead | `SsserpentHead` | `onEnterRoom` | Gain 50 Gold when entering ? rooms. | IMPLEMENTED |
| WarpedTongs | `WarpedTongs` | `atTurnStartPostDraw` | Upgrade a random card in hand each turn (temporary). | IMPLEMENTED |

---

## Passive-Only Relics (no Java hooks, checked elsewhere)

These relics have no overridden hooks in their Java files. Their behavior is checked directly in game code (e.g., `player.hasRelic("Ice Cream")` in the energy system).

| Java Class | ID | Rarity | Description | Python Status |
|---|---|---|---|---|
| Calipers | `Calipers` | Rare | Lose only 15 Block at turn start | IMPLEMENTED |
| ChemicalX | `Chemical X` | Shop | X-cost cards get +2 | MISSING |
| FrozenEye | `Frozen Eye` | Shop | See draw pile order | MISSING |
| GoldenEye | `GoldenEye` | Rare | +2 to all Scry | IMPLEMENTED (passive) |
| GoldenIdol | `Golden Idol` | Special | +25% Gold gain | IMPLEMENTED (passive) |
| IceCream | `Ice Cream` | Rare | Energy persists between turns | IMPLEMENTED (passive) |
| OddMushroom | `Odd Mushroom` | Special | Vulnerable = 25% instead of 50% | IMPLEMENTED (passive) |
| PaperCrane | `Paper Crane` | Uncommon | Weak enemies deal 40% | IMPLEMENTED (passive) |
| PaperFrog | `Paper Frog` | Uncommon | Vulnerable = 75% instead of 50% | IMPLEMENTED (passive) |
| RunicPyramid | `Runic Pyramid` | Boss | No discard at turn end | IMPLEMENTED (passive) |

---

## Test/Deprecated Relics (excluded from counts)

| Java Class | ID | Rarity | Notes |
|---|---|---|---|
| Test1 | `Test 1` | Uncommon | Test relic -- onUsePotion |
| Test3 | `Test 3` | Rare | Test relic -- onEquip (remove Strike) |
| Test4 | `Test 4` | Rare | Test relic -- atBattleStart |
| Test5 | `Test 5` | Common | Test relic -- onEquip (rare card reward) |
| Test6 | `Test 6` | Uncommon | Test relic -- onPlayerEndTurn (damage) |

---

## Character Restriction Notes

Character restrictions are NOT enforced by the relic files themselves. They are controlled by the relic pool system in `AbstractDungeon`. However, certain relics can only appear for specific characters based on `canSpawn()` checks:

| Relic | Restriction |
|---|---|
| Burning Blood | Ironclad starter |
| Black Blood | Requires `Burning Blood` |
| Ring of the Snake | Silent starter |
| Ring of the Serpent | Silent (requires Ring of the Snake) |
| Cracked Core | Defect starter |
| Nuclear Battery / Frozen Core / Inserter | Defect (via pool) |
| Pure Water | Watcher starter |
| Holy Water | Watcher (requires PureWater) |
| Hovering Kite | Silent (via pool) |
| WristBlade | Silent (via pool) |
| Mark of Pain | Ironclad (via pool) |
| Runic Cube | Ironclad (via pool) |
| Violet Lotus | Watcher (via pool) |
| Damaru / Teardrop Locket / Duality | Watcher (via pool) |
| Data Disk / Gold-Plated Cables / Symbiotic Virus / Emotion Chip | Defect (via pool) |
| Ninja Scroll | Silent (via pool) |
| Twisted Funnel / The Specimen / Snecko Skull | Silent (via pool) |
| Brimstone / Slaver's Collar | Ironclad (via pool) |

---

## Missing Implementation Priority (for Watcher RL)

### HIGH Priority (affects Watcher gameplay)
| Relic | Why |
|---|---|
| Omamori | Negate curses from events -- major for pathing |
| Question Card | Extra card choices -- affects card reward decisions |
| Astrolabe | Boss relic option -- transform + upgrade |
| Calling Bell | Boss relic option -- 3 relics + curse |
| Empty Cage | Boss relic option -- remove 2 cards |
| Pandora's Box | Boss relic option -- transform all basics |
| Tiny House | Boss relic option -- 5 small bonuses |
| Busted Crown | Boss energy relic -- 2 fewer choices (passive incomplete) |
| Neow's Lament | 3 free combat wins -- major Act 1 |
| Old Coin | 300 Gold on pickup -- impacts shop decisions |
| Prayer Wheel | 2 card rewards -- affects deck building |
| Chemical X | +2 to X-cost cards -- affects card valuation |

### MEDIUM Priority
| Relic | Why |
|---|---|
| Cauldron | Shop potion selection |
| Dolly's Mirror | Duplicate card on pickup |
| Orrery | Add 5 cards on pickup |
| Waffle | +7 Max HP, full heal |
| Wing Boots | Path flexibility |
| Singing Bowl | Skip card = +2 Max HP |
| White Beast Statue | More potion drops |
| Eternal Feather | Heal at rest sites |
| Ring of the Serpent | Silent boss relic |
| Runic Dome | Boss energy relic |
| Matryoshka | Extra chest relics |
| Cursed Key (chest hook) | Curse on chest open |

### LOW Priority (non-Watcher or edge cases)
| Relic | Why |
|---|---|
| Circlet | No effect placeholder |
| Red Circlet | No effect placeholder |
| Spirit Poop | No effect |
| N'loth's Gift/Mask | Event-specific |
| Frozen Eye | UI-only |
| Prismatic Shard | Cross-class cards |
| Cracked Core | Defect starter |
| Discerning Monocle | Shop discount variant |

---

## Hook Coverage Summary

| Hook | Java Relics Using It | Python Covered |
|---|---|---|
| `atBattleStart` | 33 | 31 |
| `atTurnStart` | 22 | 22 |
| `onUseCard` / `onPlayCard` | 18 | 17 |
| `onEquip` | 26 | 9 |
| `onPlayerEndTurn` | 7 | 7 |
| `wasHPLost` | 4 | 4 |
| `onVictory` | 15 | 8 |
| `onExhaust` | 3 | 3 |
| `onManualDiscard` | 3 | 3 |
| `onMonsterDeath` | 2 | 2 |
| `onShuffle` | 3 | 3 |
| `onObtainCard` | 5 | 5 |
| `onGainGold` | 1 | 1 |
| `onEnterRoom` | 7 | 4 |
| `onChestOpen` | 2 | 0 |
| `onBlockBroken` | 1 | 1 |
| `onChangeStance` | 1 | 1 |
| `atBattleStartPreDraw` | 5 | 5 |
| `atTurnStartPostDraw` | 3 | 3 |
| `onTrigger` | 3 | 2 |
| `addCampfireOption` | 3 | 0 (passive) |
| `onPlayerHeal` | 2 | 2 |
| `onUsePotion` | 1 | 1 |
| `onAttacked` | 1 | 1 |
| `onAttackToChangeDamage` | 1 | 1 |
| `atDamageModify` | 2 | 2 |
| `onLoseHpLast` | 1 | 1 |
| `onRefreshHand` | 1 | 1 |
| `onBloodied` / `onNotBloodied` | 3 | 2 |
| `onSpawnMonster` | 1 | 1 |
| `onChestOpenAfter` | 1 | 0 |
| `onSpendGold` | 1 | 0 |
| `onMasterDeckChange` | 1 | 1 |
| `beforeEnergyPrep` | 1 | 1 (via atTurnStart) |

---

## Key Implementation Gaps

### 1. onEquip (26 relics)
Most `onEquip` hooks handle one-time pickup effects (transform cards, gain gold, add potions, etc.). Only 9 are implemented in Python. The missing ones are primarily boss/shop relics that need the run-level GameRunner context rather than combat context.

### 2. onChestOpen (2 relics: CursedKey, Matryoshka)
Neither chest-open trigger is implemented. These fire when the player opens a treasure chest.

### 3. addCampfireOption (3 relics: Girya, PeacePipe, Shovel)
Campfire/rest site options are handled as passive flags but the actual campfire mechanics (lift, toke, dig) need game loop integration.

### 4. Passive-only relics without explicit triggers
Several relics (Chemical X, Frozen Eye, Prayer Wheel, Question Card, Singing Bowl, White Beast Statue) are checked inline in Java game code via `player.hasRelic()`. These need equivalent checks in the Python engine at the appropriate code points.
