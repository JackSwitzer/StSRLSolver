# Implementation Spec: Python Slay the Spire Clone

This spec summarizes what is implemented vs missing for a full Python clone (all characters, cards, relics, events, potions, etc.). It is based on current engine content (`packages/engine/content`), registries/handlers, and tests in this repo.

## High-Level Status

- **Character support**: Run factories exist for Watcher/Ironclad/Silent/Defect (starting decks/relics + ascension HP). Most non‑Watcher card/power/relic effects remain incomplete.
- **Parity status (from `CLAUDE.md`)**:
  - RNG system: 100%
  - Damage/block calc: 100%
  - Enemies: 100% (all 66 verified)
  - Stances: 100% (Watcher)
  - Cards: 97% (Watcher-focused)
  - Powers: 84%
  - Events: 95%
  - Potions: data 100% / effects 40%
  - Relics: active 65% / passive 15%
- **Tests**: 4100+ pytest tests; coverage ~68% (`uv run pytest tests/ --cov=packages/engine`).

## Useful Code Map (Where To Look)

- **Game loop**: `packages/engine/game.py` (GameRunner) and `packages/engine/combat_engine.py`
- **State**: `packages/engine/state/` (RNG, run/combat state)
- **Damage & combat math**: `packages/engine/calc/`
- **Content definitions**: `packages/engine/content/` (cards, relics, potions, enemies, events, powers, stances)
- **Effect system**: `packages/engine/effects/` (registry + executor)
- **Triggers & registries**: `packages/engine/registry/` (relic/power/potion triggers, passive relic flags)
- **Generation**: `packages/engine/generation/` (map, encounters, rewards, shop, treasure)
- **Handlers**: `packages/engine/handlers/` (event, reward, shop, rooms, combat)
- **Parity tooling**: `packages/parity/` (seed catalog, comparators, trackers)

## Cards (Per-Entity Status)

Status model used here:
- **Supported** = all effect names resolve via effect registry or executor (not full parity validation).
- **Missing** = one or more effect names have no handler registered.

**Legacy IDs**: Canonical data still uses Java IDs, but modern names are supported via alias mapping (Rushdown → `Adaptation`, Foresight → `Wireheading`, Wraith Form → `Wraith Form v2`).
**Note**: Card lists below primarily use internal IDs; the work-unit docs use modern display names where possible.

Legacy ID alias map (modern → canonical):
- `Rushdown` → `Adaptation`
- `Foresight` → `Wireheading`
- `Wraith Form` → `Wraith Form v2`

Totals by group:
- Watcher: 84 (Supported 83, Missing 1)
- Ironclad: 75 (Supported 14, Missing 61)
- Silent: 76 (Supported 16, Missing 60)
- Defect: 75 (Supported 8, Missing 67)
- Colorless: 39 (Supported 39)
- Curse: 14 (Supported 14)
- Status: 5 (Supported 5)

### Watcher
- Missing: `InnerPeace` (missing `if_calm_draw_else_calm`)
- Supported: `Alpha`, `BattleHymn`, `Beta`, `Blasphemy`, `BowlingBash`, `Brilliance`, `CarveReality`, `ClearTheMind`, `Collect`, `Conclude`, `ConjureBlade`, `Consecrate`, `Crescendo`, `CrushJoints`, `CutThroughFate`, `DeceiveReality`, `Defend_P`, `DeusExMachina`, `DevaForm`, `Devotion`, `EmptyBody`, `EmptyFist`, `EmptyMind`, `Eruption`, `Establishment`, `Evaluate`, `Expunger`, `Fasting2`, `FearNoEvil`, `FlurryOfBlows`, `FlyingSleeves`, `FollowUp`, `ForeignInfluence`, `Halt`, `Indignation`, `Insight`, `Judgement`, `JustLucky`, `LessonLearned`, `LikeWater`, `MasterReality`, `Meditate`, `MentalFortress`, `Miracle`, `Nirvana`, `Omega`, `Omniscience`, `PathToVictory`, `Perseverance`, `Pray`, `Prostrate`, `Protect`, `Ragnarok`, `ReachHeaven`, `Rushdown`, `Safety`, `Sanctity`, `SandsOfTime`, `SashWhip`, `Scrawl`, `SignatureMove`, `Smite`, `SpiritShield`, `Strike_P`, `Study`, `Swivel`, `TalkToTheHand`, `Tantrum`, `ThirdEye`, `ThroughViolence`, `Unraveling`, `Vault`, `Vengeance`, `Vigilance`, `Wallop`, `WaveOfTheHand`, `Weave`, `WheelKick`, `WindmillStrike`, `Foresight`, `Wish`, `Worship`, `WreathOfFlame`

### Ironclad
- Missing: `Anger` (add_copy_to_discard), `Armaments` (upgrade_card_in_hand), `Barricade` (block_not_lost), `Battle Trance` (draw_then_no_draw), `Berserk` (gain_vulnerable_gain_energy_per_turn), `Blood for Blood` (cost_reduces_when_damaged), `Bloodletting` (lose_hp_gain_energy), `Body Slam` (damage_equals_block), `Brutality` (start_turn_lose_hp_draw), `Burning Pact` (exhaust_to_draw), `Clash` (only_attacks_in_hand), `Combust` (end_turn_damage_all_lose_hp), `Corruption` (skills_cost_0_exhaust), `Dark Embrace` (draw_on_exhaust), `Demon Form` (gain_strength_each_turn), `Disarm` (reduce_enemy_strength), `Double Tap` (play_attacks_twice), `Dropkick` (if_vulnerable_draw_and_energy), `Dual Wield` (copy_attack_or_power), `Entrench` (double_block), `Evolve` (draw_on_status), `Exhume` (return_exhausted_card_to_hand), `Feed` (if_fatal_gain_max_hp), `Feel No Pain` (block_on_exhaust), `Fiend Fire` (exhaust_hand_damage_per_card), `Fire Breathing` (damage_on_status_curse), `Flame Barrier` (when_attacked_deal_damage), `Flex` (gain_temp_strength), `Havoc` (play_top_card), `Headbutt` (put_card_from_discard_on_draw), `Heavy Blade` (strength_multiplier), `Hemokinesis` (lose_hp), `Immolate` (add_burn_to_discard), `Infernal Blade` (add_random_attack_cost_0), `Inflame` (gain_strength), `Intimidate` (apply_weak_all), `Juggernaut` (damage_random_on_block), `Limit Break` (double_strength), `Metallicize` (end_turn_gain_block), `Offering` (lose_hp_gain_energy_draw), `Perfected Strike` (damage_per_strike), `Power Through` (add_wounds_to_hand), `Rage` (gain_block_per_attack), `Rampage` (increase_damage_on_use), `Reaper` (damage_all_heal_unblocked), `Reckless Charge` (shuffle_dazed_into_draw), `Rupture` (gain_strength_on_hp_loss), `Searing Blow` (can_upgrade_unlimited), `Second Wind` (exhaust_non_attacks_gain_block), `Seeing Red` (gain_2_energy), `Sentinel` (gain_energy_on_exhaust_2_3), `Sever Soul` (exhaust_all_non_attacks), `Shockwave` (apply_weak_and_vulnerable_all), `Spot Weakness` (gain_strength_if_enemy_attacking), `Sword Boomerang` (random_enemy_x_times), `Thunderclap` (apply_vulnerable_1_all), `True Grit` (exhaust_random_card), `Uppercut` (apply_weak_and_vulnerable), `Warcry` (draw_then_put_on_draw), `Whirlwind` (damage_all_x_times), `Wild Strike` (shuffle_wound_into_draw)
- Supported: `Bash`, `Bludgeon`, `Carnage`, `Cleave`, `Clothesline`, `Defend_R`, `Ghostly Armor`, `Impervious`, `Iron Wave`, `Pommel Strike`, `Pummel`, `Shrug It Off`, `Strike_R`, `Twin Strike`

### Silent
- Missing: `A Thousand Cuts` (deal_damage_per_card_played), `Accuracy` (shivs_deal_more_damage), `Acrobatics` (draw_x, discard_1), `After Image` (gain_1_block_per_card_played), `All Out Attack` (discard_random_1), `Bane` (double_damage_if_poisoned), `Blade Dance` (add_shivs_to_hand), `Blur` (block_not_removed_next_turn), `Bouncing Flask` (apply_poison_random_3_times), `Bullet Time` (no_draw_this_turn, cards_cost_0_this_turn), `Burst` (double_next_skills), `Calculated Gamble` (discard_hand_draw_same), `Caltrops` (gain_thorns), `Catalyst` (double_poison), `Choke` (apply_choke), `Cloak And Dagger` (add_shivs_to_hand), `Concentrate` (discard_x), `Corpse Explosion` (apply_poison, apply_corpse_explosion), `Crippling Poison` (apply_poison_all, apply_weak_2_all), `Dagger Spray` (damage_all_x_times), `Dagger Throw` (discard_1), `Deadly Poison` (apply_poison), `Distraction` (add_random_skill_cost_0), `Dodge and Roll` (block_next_turn), `Doppelganger` (draw_x_next_turn, gain_x_energy_next_turn), `Endless Agony` (copy_to_hand_when_drawn), `Envenom` (attacks_apply_poison), `Escape Plan` (if_skill_drawn_gain_block), `Eviscerate` (cost_reduces_per_discard), `Expertise` (draw_to_x_cards), `Finisher` (damage_per_attack_this_turn), `Flechettes` (damage_per_skill_in_hand), `Flying Knee` (gain_energy_next_turn_1), `Footwork` (gain_dexterity), `Glass Knife` (reduce_damage_by_2), `Grand Finale` (only_playable_if_draw_pile_empty), `Heel Hook` (if_target_weak_gain_energy_draw), `Infinite Blades` (add_shiv_each_turn), `Malaise` (apply_weak_x, apply_strength_down_x), `Masterful Stab` (cost_increases_when_damaged), `Night Terror` (copy_card_to_hand_next_turn), `Noxious Fumes` (apply_poison_all_each_turn), `Outmaneuver` (gain_energy_next_turn), `Phantasmal Killer` (double_damage_next_turn), `PiercingWail` (reduce_strength_all_enemies), `Poisoned Stab` (apply_poison), `Predator` (draw_2_next_turn), `Prepared` (draw_x, discard_x), `Reflex` (when_discarded_draw), `Setup` (put_card_on_draw_pile_cost_0), `Skewer` (damage_x_times_energy), `Storm of Steel` (discard_hand, add_shivs_equal_to_discarded), `Survivor` (discard_1), `Tactician` (when_discarded_gain_energy), `Tools of the Trade` (draw_1_discard_1_each_turn), `Underhanded Strike` (refund_2_energy_if_discarded_this_turn), `Unload` (discard_non_attacks), `Venomology` (obtain_random_potion), `Well Laid Plans` (retain_cards_each_turn), `Wraith Form` (gain_intangible, lose_1_dexterity_each_turn)
- Supported: `Adrenaline`, `Backflip`, `Backstab`, `Dash`, `Defend_G`, `Deflect`, `Die Die Die`, `Leg Sweep`, `Neutralize`, `Quick Slash`, `Riddle With Holes`, `Shiv`, `Slice`, `Strike_G`, `Sucker Punch`, `Terror`

### Defect
- Missing: `Aggregate` (gain_energy_per_x_cards_in_draw), `All For One` (return_all_0_cost_from_discard), `Amplify` (next_power_plays_twice), `Auto Shields` (only_if_no_block), `Ball Lightning` (channel_lightning), `Barrage` (damage_per_orb), `Biased Cognition` (gain_focus_lose_focus_each_turn), `Blizzard` (damage_per_frost_channeled), `Buffer` (prevent_next_hp_loss), `Capacitor` (increase_orb_slots), `Chaos` (channel_random_orb), `Chill` (channel_frost_per_enemy), `Claw` (increase_all_claw_damage), `Cold Snap` (channel_frost), `Compile Driver` (draw_per_unique_orb), `Conserve Battery` (gain_1_energy_next_turn), `Consume` (gain_focus_lose_orb_slot), `Coolheaded` (channel_frost), `Creative AI` (add_random_power_each_turn), `Darkness` (channel_dark), `Defragment` (gain_focus), `Doom and Gloom` (channel_dark), `Double Energy` (double_energy), `Dualcast` (evoke_orb_twice), `Echo Form` (play_first_card_twice), `Electrodynamics` (lightning_hits_all, channel_lightning), `FTL` (if_played_less_than_x_draw), `Fission` (remove_orbs_gain_energy_and_draw), `Force Field` (cost_reduces_per_power_played), `Fusion` (channel_plasma), `Genetic Algorithm` (block_increases_permanently), `Glacier` (channel_2_frost), `Go for the Eyes` (if_attacking_apply_weak), `Heatsinks` (draw_on_power_play), `Hello World` (add_common_card_each_turn), `Hologram` (return_card_from_discard), `Hyperbeam` (lose_focus), `Lockon` (apply_lockon), `Loop` (trigger_orb_passive_extra), `Machine Learning` (draw_extra_each_turn), `Melter` (remove_enemy_block), `Meteor Strike` (channel_3_plasma), `Multi-Cast` (evoke_first_orb_x_times), `Rainbow` (channel_lightning_frost_dark), `Reboot` (shuffle_hand_and_discard_draw), `Rebound` (next_card_on_top_of_draw), `Recycle` (exhaust_card_gain_energy), `Redo` (evoke_then_channel_same_orb), `Reinforced Body` (block_x_times), `Reprogram` (lose_focus_gain_strength_dex), `Rip and Tear` (damage_random_enemy_twice), `Scrape` (draw_discard_non_zero_cost), `Seek` (search_draw_pile), `Self Repair` (heal_at_end_of_combat), `Stack` (block_equals_discard_size), `Static Discharge` (channel_lightning_on_damage), `Steam` (lose_1_block_permanently), `Steam Power` (add_burn_to_discard), `Storm` (channel_lightning_on_power_play), `Streamline` (reduce_cost_permanently), `Sunder` (if_fatal_gain_3_energy), `Tempest` (channel_x_lightning), `Thunder Strike` (damage_per_lightning_channeled), `Turbo` (add_void_to_discard), `Undo` (retain_hand), `White Noise` (add_random_power_to_hand_cost_0), `Zap` (channel_lightning)
- Supported: `Beam Cell`, `BootSequence`, `Core Surge`, `Defend_B`, `Leap`, `Skim`, `Strike_B`, `Sweeping Beam`

### Colorless
- Supported: `Apotheosis`, `Bandage Up`, `Bite`, `Blind`, `Chrysalis`, `Dark Shackles`, `Deep Breath`, `Discovery`, `Dramatic Entrance`, `Enlightenment`, `Finesse`, `Flash of Steel`, `Forethought`, `Ghostly`, `Good Instincts`, `HandOfGreed`, `Impatience`, `J.A.X.`, `Jack Of All Trades`, `Madness`, `Magnetism`, `Master of Strategy`, `Mayhem`, `Metamorphosis`, `Mind Blast`, `Panacea`, `Panache`, `PanicButton`, `Purity`, `RitualDagger`, `Sadistic Nature`, `Secret Technique`, `Secret Weapon`, `Swift Strike`, `The Bomb`, `Thinking Ahead`, `Transmutation`, `Trip`, `Violence`

### Curse
- Supported: `AscendersBane`, `Clumsy`, `CurseOfTheBell`, `Decay`, `Doubt`, `Injury`, `Necronomicurse`, `Normality`, `Pain`, `Parasite`, `Pride`, `Regret`, `Shame`, `Writhe`

### Status
- Supported: `Burn`, `Dazed`, `Slimed`, `Void`, `Wound`

## Relics (Per-Entity Status)

Status model used here:
- **Active** = trigger registered in `registry/relics.py`
- **Passive** = flag defined in `registry/relics_passive.py`

Counts (180 total): active_only=109, passive_only=20, active_and_passive=7, missing_all=44.

- Active only: `Akabeko`, `Anchor`, `Ancient Tea Set`, `Art of War`, `Bag of Marbles`, `Bag of Preparation`, `Bird Faced Urn`, `Black Blood`, `Blood Vial`, `Bloody Idol`, `Boot`, `Bottled Flame`, `Bottled Lightning`, `Bottled Tornado`, `Brimstone`, `Bronze Scales`, `Burning Blood`, `Calipers`, `CaptainsWheel`, `Centennial Puzzle`, `Champion Belt`, `CloakClasp`, `ClockworkSouvenir`, `CultistMask`, `Damaru`, `Darkstone Periapt`, `Dead Branch`, `Du-Vu Doll`, `Emotion Chip`, `Enchiridion`, `FossilizedHelix`, `Frozen Egg 2`, `Gambling Chip`, `Gremlin Horn`, `GremlinMask`, `Happy Flower`, `HolyWater`, `HornCleat`, `HoveringKite`, `Incense Burner`, `InkBottle`, `Inserter`, `Kunai`, `Lantern`, `Lee's Waffle`, `Letter Opener`, `Lizard Tail`, `Magic Flower`, `Mango`, `Mark of Pain`, `Matryoshka`, `Meat on the Bone`, `Mercury Hourglass`, `Molten Egg 2`, `Mummified Hand`, `MutagenicStrength`, `Necronomicon`, `Nilry's Codex`, `Ninja Scroll`, `Nuclear Battery`, `Nunchaku`, `Oddly Smooth Stone`, `Old Coin`, `Omamori`, `OrangePellets`, `Orichalcum`, `Ornamental Fan`, `Pantograph`, `Pear`, `Pen Nib`, `Philosopher's Stone`, `Pocketwatch`, `PureWater`, `Red Mask`, `Red Skull`, `Ring of the Snake`, `Runic Cube`, `Self Forming Clay`, `Shuriken`, `SlaversCollar`, `Sling`, `Snake Skull`, `Snecko Eye`, `StoneCalendar`, `Strange Spoon`, `Strawberry`, `StrikeDummy`, `Sundial`, `Symbiotic Virus`, `TeardropLocket`, `The Specimen`, `TheAbacus`, `Thread and Needle`, `Tingsha`, `Tiny Chest`, `Torii`, `Tough Bandages`, `Toxic Egg 2`, `Toy Ornithopter`, `TungstenRod`, `TwistedFunnel`, `Unceasing Top`, `Vajra`, `Velvet Choker`, `VioletLotus`, `War Paint`, `Whetstone`, `WristBlade`, `Yang`
- Passive only: `Black Star`, `Coffee Dripper`, `Cursed Key`, `Dream Catcher`, `Ectoplasm`, `Fusion Hammer`, `Ginger`, `Juzu Bracelet`, `Mark of the Bloom`, `Melange`, `Membership Card`, `Odd Mushroom`, `Paper Frog`, `Peace Pipe`, `Regal Pillow`, `Runic Pyramid`, `Smiling Mask`, `Sozu`, `The Courier`, `Turnip`
- Active + passive: `Blue Candle`, `Girya`, `Golden Idol`, `Ice Cream`, `Medical Kit`, `Paper Crane`, `Shovel`
- Missing all: `Astrolabe`, `Busted Crown`, `Cables`, `Calling Bell`, `Cauldron`, `CeramicFish`, `Charon's Ashes`, `Chemical X`, `Circlet`, `Cracked Core`, `DataDisk`, `Discerning Monocle`, `DollysMirror`, `Empty Cage`, `Eternal Feather`, `FaceOfCleric`, `Frozen Eye`, `FrozenCore`, `GoldenEye`, `HandDrill`, `MawBank`, `MealTicket`, `NeowsBlessing`, `Nloth's Gift`, `NlothsMask`, `Orrery`, `Pandora's Box`, `Potion Belt`, `Prayer Wheel`, `PreservedInsect`, `PrismaticShard`, `Question Card`, `Red Circlet`, `Ring of the Serpent`, `Runic Capacitor`, `Runic Dome`, `SacredBark`, `Singing Bowl`, `Spirit Poop`, `SsserpentHead`, `Tiny House`, `WarpedTongs`, `White Beast Statue`, `WingedGreaves`

## Potions (Per-Entity Status)

All 42 potions have registry handlers, but several are **stubbed/partial** or have TODOs in the combat engine.

Potions with handlers (all): `Ambrosia`, `Ancient Potion`, `AttackPotion`, `BlessingOfTheForge`, `Block Potion`, `BloodPotion`, `BottledMiracle`, `ColorlessPotion`, `CultistPotion`, `CunningPotion`, `Dexterity Potion`, `DistilledChaos`, `DuplicationPotion`, `ElixirPotion`, `Energy Potion`, `EntropicBrew`, `EssenceOfDarkness`, `EssenceOfSteel`, `Explosive Potion`, `FairyPotion`, `FearPotion`, `Fire Potion`, `FocusPotion`, `Fruit Juice`, `GamblersBrew`, `GhostInAJar`, `HeartOfIron`, `LiquidBronze`, `LiquidMemories`, `Poison Potion`, `PotionOfCapacity`, `PowerPotion`, `Regen Potion`, `SkillPotion`, `SmokeBomb`, `SneckoOil`, `SpeedPotion`, `StancePotion`, `SteroidPotion`, `Strength Potion`, `Swift Potion`, `Weak Potion`

Partial/stub behaviors to address:
- `ElixirPotion`: simplified to exhaust all cards (no selection UX yet).
- `FairyPotion`: auto-revive supported in combat runner; manual use no-op.
- `DistilledChaos`: still simplified (draws cards, doesn’t auto-play).
- `LiquidMemories`: simplified (no discard selection UI).
- `EntropicBrew`: deterministic fill (no potion RNG parity yet).
- `SmokeBomb`: boss-fight restriction not enforced.
- Discovery potions (`AttackPotion`, `SkillPotion`, `PowerPotion`, `ColorlessPotion`) auto-select instead of offering a 3‑choice.

## Powers (Per-Entity Status)

Power data defined: 94 total. Registry triggers implemented: 30. Missing triggers: 64.

With triggers: `BattleHymn`, `Bias`, `Brutality`, `Buffer`, `Choked`, `Combust`, `Constricted`, `DevaForm`, `Dexterity`, `Energized`, `Envenom`, `Evolve`, `Frail`, `Heatsink`, `Intangible`, `Juggernaut`, `LoseDexterity`, `Metallicize`, `Nirvana`, `Panache`, `Plated Armor`, `Poison`, `Regeneration`, `Rupture`, `Strength`, `Study`, `Thorns`, `Vigor`, `Vulnerable`, `Weakened`

Missing triggers: `Accuracy`, `Rushdown`, `After Image`, `Angry`, `Artifact`, `Barricade`, `BeatOfDeath`, `BlockReturnPower`, `Blur`, `Burst`, `CannotChangeStancePower`, `Controlled`, `Corruption`, `Creative AI`, `Curiosity`, `Dark Embrace`, `Demon Form`, `DevotionPower`, `Double Damage`, `Double Tap`, `Draw`, `Draw Reduction`, `Echo Form`, `Electro`, `Entangled`, `Equilibrium`, `EstablishmentPower`, `Fading`, `Feel No Pain`, `Fire Breathing`, `Flame Barrier`, `Flex`, `Focus`, `FreeAttackPower`, `GrowthPower`, `Infinite Blades`, `IntangiblePlayer`, `Invincible`, `Life Link`, `LikeWaterPower`, `Lockon`, `Mantra`, `MasterRealityPower`, `Mode Shift`, `No Draw`, `NoBlockPower`, `Noxious Fumes`, `OmegaPower`, `PathToVictoryPower`, `Pen Nib`, `Repair`, `Retain Cards`, `Sadistic`, `Slow`, `Split`, `Static Discharge`, `Storm`, `Thievery`, `Thousand Cuts`, `Time Warp`, `WaveOfTheHandPower`, `Foresight`, `Wraith Form`, `WrathNextTurnPower`

## Enemies (Per-Entity Status)

All enemy classes are implemented (parity verified). IDs: `AcidSlime_L`, `AcidSlime_M`, `AcidSlime_S`, `AwakenedOne`, `BanditBear`, `BanditChild`, `BanditLeader`, `BookOfStabbing`, `BronzeAutomaton`, `BronzeOrb`, `Byrd`, `Centurion`, `Champ`, `Chosen`, `CorruptHeart`, `Cultist`, `Dagger`, `Darkling`, `Deca`, `Donu`, `Exploder`, `FungiBeast`, `FuzzyLouseDefensive`, `FuzzyLouseNormal`, `GiantHead`, `GremlinFat`, `GremlinLeader`, `GremlinNob`, `GremlinThief`, `GremlinTsundere`, `GremlinWarrior`, `GremlinWizard`, `Healer`, `Hexaghost`, `JawWorm`, `Lagavulin`, `Looter`, `Louse`, `Maw`, `Mugger`, `Nemesis`, `Orb Walker`, `Reptomancer`, `Repulsor`, `Sentry`, `Serpent`, `Shelled Parasite`, `SlaverBlue`, `SlaverBoss`, `SlaverRed`, `SlimeBoss`, `SnakePlant`, `Snecko`, `SphericGuardian`, `SpikeSlime_L`, `SpikeSlime_M`, `SpikeSlime_S`, `Spiker`, `SpireShield`, `SpireSpear`, `Taskmaster`, `TheCollector`, `TheGuardian`, `TimeEater`, `TorchHead`, `Transient`, `WrithingMass`

## Events (Per-Entity Status)

Data definitions: 51 (`content/events.py`). Runtime handler definitions: 50 (`handlers/event_handler.py`). Choice generators implemented: 17/50. Missing handlers: `GremlinMatchGame`, `GremlinWheelGame`.
Event ID alias normalization is now handled in `handlers/event_handler.py` to map content IDs to handler IDs.

Content-only event IDs (exist in `content/events.py` but not in handler definitions; mostly spacing/ID mismatches):
- `Accursed Blacksmith`, `Back to Basics`, `Big Fish`, `Bonfire Elementals`, `Cursed Tome`, `Dead Adventurer`, `Drug Dealer`, `Forgotten Altar`, `Fountain of Cleansing`, `Golden Idol`, `Golden Shrine`, `Golden Wing`, `Knowing Skull`, `Lab`, `Liars Game`, `Living Wall`, `Masked Bandits`, `Match and Keep!`, `Mysterious Sphere`, `N'loth`, `NoteForYourself`, `Scrap Ooze`, `Shining Light`, `The Cleric`, `The Joust`, `The Library`, `The Mausoleum`, `The Moai Head`, `The Woman in Blue`, `Tomb of Lord Red Mask`, `Transmorgrifier`, `Upgrade Shrine`, `Wheel of Change`, `Winding Halls`, `World of Goop`

Handler-only IDs (exist in `handlers/event_handler.py` but not in `content/events.py`):
- `AccursedBlacksmith`, `Augmenter`, `BackToBasics`, `BigFish`, `BonfireElementals`, `CursedTome`, `DeadAdventurer`, `ForgottenAltar`, `FountainOfCleansing`, `GoldenIdol`, `GoldenShrine`, `GremlinMatchGame`, `GremlinWheelGame`, `KnowingSkull`, `LivingWall`, `MaskedBandits`, `MoaiHead`, `MysteriousSphere`, `Nloth`, `ScrapOoze`, `ShiningLight`, `Sssserpent`, `TheCleric`, `TheJoust`, `TheLab`, `TheLibrary`, `TheMausoleum`, `TombOfLordRedMask`, `Transmogrifier`, `UpgradeShrine`, `WindingHalls`, `WingStatue`, `WomanInBlue`, `WorldOfGoop`

Handler-defined events missing choice generators:
- `AccursedBlacksmith`, `Addict`, `Augmenter`, `BackToBasics`, `Beggar`, `BonfireElementals`, `CursedTome`, `DeadAdventurer`, `Designer`, `FaceTrader`, `Falling`, `ForgottenAltar`, `FountainOfCleansing`, `Ghosts`, `GremlinMatchGame`, `GremlinWheelGame`, `MoaiHead`, `Mushrooms`, `MysteriousSphere`, `Nest`, `Nloth`, `SecretPortal`, `SensoryStone`, `ShiningLight`, `Sssserpent`, `TheJoust`, `TheLab`, `TombOfLordRedMask`, `Vampires`, `WeMeetAgain`, `WindingHalls`, `WingStatue`, `WomanInBlue`

## Stances

Watcher stances implemented: `Neutral`, `Calm`, `Wrath`, `Divinity`.

## Tests & Coverage

- **Coverage**: ~68% (tests cover `packages/engine` per `CLAUDE.md`).
- **Latest full test run** (`uv run pytest tests/ -ra`, 2026-02-04): collected 3950 tests; aborted during collection with 3 import errors:
  - `tests/test_ascension.py`: `WATCHER_BASE_GOLD` import no longer exists (replaced by `BASE_STARTING_GOLD`).
  - `tests/test_coverage_boost.py`: `EventHandler` moved out of `handlers.rooms`.
  - `tests/test_handlers.py`: `EventHandler` moved out of `handlers.rooms`.
- **XFail inventory (140 matches across 8 files)**:
  - `tests/test_relic_rest_site.py` (37)
  - `tests/test_relic_bottled.py` (20)
  - `tests/test_relic_pickup.py` (34)
  - `tests/test_relic_acquisition.py` (30)
  - `tests/test_relic_triggers_outofcombat.py` (14)
  - `tests/test_relic_card_rewards.py` (3)
  - `tests/test_audit_relics_cardplay.py` (1)
  - `tests/test_enemy_ai_parity.py` (1)
- **Incomplete tests**:
  - `tests/test_rng_parity.py` has a `NotImplementedError` placeholder for expected cards.
  - `tests/test_rng.py` has TODO placeholders for expected values.

## TODOs / Stubs Index

Known TODOs and pass stubs (non-exhaustive):
- `packages/engine/combat_engine.py`: Smoke Bomb boss check; Distilled Chaos; Liquid Memories; Entropic Brew.
- `packages/engine/registry/relics.py`: Defect orb TODOs (multiple relics); Ice Cream energy carry; Blue Candle/Medical Kit play behavior.
- `packages/engine/game.py`: enemy count tracking for rewards.
- `packages/engine/handlers/reward_handler.py`: action classes are `pass` (Gold/Potion/Relic/Key claim & proceed).
- `packages/engine/handlers/combat.py`: Snecko Eye cost randomization, Ice Cream energy carry, and other relic hooks are `pass`.
- `packages/engine/generation/rewards.py`: `on_uncommon` no-op.
- `packages/engine/generation/shop.py`: fallback `pass` when retrying shop card rolls.
- `tests/test_rng_parity.py`, `tests/test_rng.py`, `docs/vault/stsrlsolver-analysis.md`: TODO placeholders remain.
- `packages/engine/effects/cards.py`: several marker effects are `pass` but intentionally handled elsewhere.

## Docs Inventory

- `docs/ARCHITECTURE.md` (core architecture; currently references `core/` paths that no longer exist).
- `docs/vault/` (mechanics ground truth): RNG, damage, relic effects, map generation, events, card rewards, verified seeds, etc.
  - Notable: `rng-system-analysis.md`, `damage-mechanics.md`, `event-mechanics.md`, `relic-effects.md`, `shop-mechanics.md`, `map-generation.md`, `watcher-cards-complete.md`.

## Cleanup & Consolidation Opportunities

- **Path/packaging alignment**: README and `pyproject.toml` now reflect `packages/engine`, but keep docs and tooling consistent as the layout evolves.
- **Events duplication**: event definitions exist in both `content/events.py` and `handlers/event_handler.py` with mismatched IDs; alias normalization exists but full consolidation remains.
- **TDD placeholders**: many `xfail` tests are placeholders; either implement or remove/retag to avoid masking regressions.

## Work Units (Small-Model Tasks)

These unit-sized tasks are split by domain to keep scope manageable and parallelizable:

- Cards (Watcher): [docs/work_units/cards-watcher.md](docs/work_units/cards-watcher.md)
- Cards (Ironclad): [docs/work_units/cards-ironclad.md](docs/work_units/cards-ironclad.md)
- Cards (Silent): [docs/work_units/cards-silent.md](docs/work_units/cards-silent.md)
- Cards (Defect): [docs/work_units/cards-defect.md](docs/work_units/cards-defect.md)
- Potions: [docs/work_units/potions.md](docs/work_units/potions.md)
- Powers: [docs/work_units/powers.md](docs/work_units/powers.md)
- Events: [docs/work_units/events.md](docs/work_units/events.md)
- Rewards: [docs/work_units/rewards.md](docs/work_units/rewards.md)
- Relics: [docs/work_units/relics.md](docs/work_units/relics.md)

Ultra-granular checklists (per-category):
- Action spec (model-facing): [docs/work_units/granular-actions.md](docs/work_units/granular-actions.md)
- Agent interface: [docs/work_units/granular-agent-interface.md](docs/work_units/granular-agent-interface.md)
- Observation schema: [docs/work_units/granular-observation.md](docs/work_units/granular-observation.md)
- Determinism & RNG: [docs/work_units/granular-determinism.md](docs/work_units/granular-determinism.md)
- Phase flow: [docs/work_units/granular-phase-flow.md](docs/work_units/granular-phase-flow.md)
- Map visibility: [docs/work_units/granular-map-visibility.md](docs/work_units/granular-map-visibility.md)
- Cards (Watcher): [docs/work_units/granular-cards-watcher.md](docs/work_units/granular-cards-watcher.md)
- Cards (Ironclad): [docs/work_units/granular-cards-ironclad.md](docs/work_units/granular-cards-ironclad.md)
- Cards (Silent): [docs/work_units/granular-cards-silent.md](docs/work_units/granular-cards-silent.md)
- Cards (Defect): [docs/work_units/granular-cards-defect.md](docs/work_units/granular-cards-defect.md)
- Defect orbs: [docs/work_units/granular-orbs.md](docs/work_units/granular-orbs.md)
- Potions: [docs/work_units/granular-potions.md](docs/work_units/granular-potions.md)
- Powers: [docs/work_units/granular-powers.md](docs/work_units/granular-powers.md)
- Events: [docs/work_units/granular-events.md](docs/work_units/granular-events.md)
- Rewards: [docs/work_units/granular-rewards.md](docs/work_units/granular-rewards.md)
- Relics: [docs/work_units/granular-relics.md](docs/work_units/granular-relics.md)

Granular checklists incorporate the latest failed/skip test mappings (2026-02-04).
Model‑facing actions are prioritized over UI (choices should be traversable via explicit actions). Parameter signatures are explicit in the granular specs and defined in `granular-actions.md`.

## Agent Readiness Gates (minimum for RL)
1. **Action API**: `get_available_actions()` / `take_action()` adhere to `granular-actions.md` with deterministic IDs and no dead‑ends.
2. **Observation schema**: `get_observation()` returns stable, JSON‑serializable payloads per `granular-observation.md`.
3. **Determinism**: RNG streams and counters are synchronized; identical seed+actions yield identical outcomes (`granular-determinism.md`).
4. **Phase flow**: transitions obey the state machine (`granular-phase-flow.md`) and never strand the agent.
5. **Decision coverage**: rewards, events, and potions expose explicit selection actions (including boss relic skip).
6. **Map visibility**: current‑act map and `available_paths` are observable; `path_choice` actions align to map indices.

### Work Units Coverage Audit
- All `docs/work_units/granular-*.md` checklists include explicit `(action: ...)` tags.
- No legacy IDs are referenced in work_units docs (modern names only).
- All work_units docs are linked in this spec for discoverability.
- Map visibility spec is linked and covered by observation/action requirements.

## RL Readiness (Engine-Only Next Steps)

1. **Character completion**: run factories added for Ironclad/Silent/Defect; remaining work is effect parity (cards/powers/relics).
2. **Defect orbs system**: channel/evoke mechanics, orb slots, focus, and all orb-triggered relics/powers/cards.
3. **Card effects**: implement missing effect handlers for Ironclad/Silent/Defect and the single Watcher gap (`InnerPeace`).
4. **Relics**: implement missing active triggers (44 relics missing all + xfail buckets: bottled, pickup, acquisition, rest-site).
5. **Potions**: finish TODOs and discovery/selection logic for interactive potions.
6. **Events**: unify definitions and implement missing handlers/choice generators.
7. **Rewards/actions**: implement reward handler actions and ensure reward resolution mirrors Java.

### Watcher RL readiness (current max)
Safe (high parity): RNG, damage/block, enemy AI, Watcher stances.
Cautious (partial parity): potions, powers, relic triggers, events.
Risky (low fidelity): reward action processing, cross-class systems (Prismatic Shard/Defect orbs).

Suggested constraints if starting now:
- Prefer Watcher-only runs; avoid Prismatic Shard and cross-class dependencies.
- Treat event rooms as low-fidelity unless you implement missing choice generators.
- Avoid potions with incomplete behavior (discovery, Distilled Chaos, Liquid Memories, Entropic Brew, Smoke Bomb).

Training plan (doc-only): [docs/RL_TRAINING_PLAN.md](docs/RL_TRAINING_PLAN.md)

