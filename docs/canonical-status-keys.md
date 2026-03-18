# Canonical Status/Power Key Mapping

Generated 2026-03-11 from decompiled Java source and Python engine analysis.

**Ground truth**: Java `POWER_ID` strings from `com.megacrit.cardcrawl.powers/*.java`

## Summary

- **148 Java POWER_IDs** (non-deprecated)
- **148 Python POWER_DATA keys** (match count, but 25 use different names)
- **118 unique runtime status keys** found in Python engine code
- **36 mismatches** identified (keys used at runtime that don't match Java POWER_ID)

---

## Section 1: Powers Where Python POWER_DATA Key Differs from Java POWER_ID

The Python POWER_DATA chose a different canonical key than Java's POWER_ID string.
In every case, the Java POWER_ID should be considered ground truth.

| Java POWER_ID | Python POWER_DATA Key | Java File | Action |
|---|---|---|---|
| `"Attack Burn"` | `AttackBurn` | AttackBurnPower.java | Rename to `"Attack Burn"` |
| `"CorpseExplosionPower"` | `CorpseExplosion` | CorpseExplosionPower.java | Rename to `"CorpseExplosionPower"` |
| `"Curl Up"` | `CurlUp` | CurlUpPower.java | Rename to `"Curl Up"` |
| `"DexLoss"` | `LoseDexterity` | LoseDexterityPower.java | Rename to `"DexLoss"` |
| `"Draw Card"` | `DrawCardNextTurn` | DrawCardNextTurnPower.java | Rename to `"Draw Card"` |
| `"DuplicationPower"` | `Duplication` | DuplicationPower.java | Rename to `"DuplicationPower"` |
| `"EnergyDownPower"` | `EnergyDown` | EnergyDownPower.java | Already `EnergyDownPower` in Java; Python auto-gen has `EnergyDown` |
| `"Generic Strength Up Power"` | `GenericStrengthUp` | GenericStrengthUpPower.java | Rename to `"Generic Strength Up Power"` |
| `"Lightning Mastery"` | `LightningMastery` | LightningMasteryPower.java | Rename to `"Lightning Mastery"` |
| `"AngelForm"` | `LiveForever` | LiveForeverPower.java | Rename to `"AngelForm"` |
| `"Next Turn Block"` | `NextTurnBlock` | NextTurnBlockPower.java | Rename to `"Next Turn Block"` |
| `"Night Terror"` | `Nightmare` | NightmarePower.java | Rename to `"Night Terror"` |
| `"Nullify Attack"` | `Forcefield` | ForcefieldPower.java | Rename to `"Nullify Attack"` |
| `"OmnisciencePower"` | `Omniscience` | OmnisciencePower.java | Rename to `"OmnisciencePower"` |
| `"Painful Stabs"` | `PainfulStabs` | PainfulStabsPower.java | Rename to `"Painful Stabs"` |
| `"Compulsive"` | `Reactive` | ReactivePower.java | Rename to `"Compulsive"` |
| `"Regenerate"` | `RegenerateMonster` | RegenerateMonsterPower.java | Rename to `"Regenerate"` |
| `"Shackled"` | `GainStrength` | GainStrengthPower.java | Rename to `"Shackled"` |
| `"Sharp Hide"` | `SharpHide` | SharpHidePower.java | Rename to `"Sharp Hide"` |
| `"Skill Burn"` | `SkillBurn` | SkillBurnPower.java | Rename to `"Skill Burn"` |
| `"Spore Cloud"` | `SporeCloud` | SporeCloudPower.java | Rename to `"Spore Cloud"` |
| `"StaticDischarge"` | `Static Discharge` | StaticDischargePower.java | Rename to `"StaticDischarge"` |
| `"TimeMazePower"` | `TimeMaze` | TimeMazePower.java | Rename to `"TimeMazePower"` |
| `"Tools Of The Trade"` | `ToolsOfTheTrade` | ToolsOfTheTradePower.java | Rename to `"Tools Of The Trade"` |
| `"Winter"` | `Winter` | WinterPower.java | OK (already matches) |

**Note**: `"Winter"` appears in both sets because the auto-gen inventory put it under `Winter` which is correct.

---

## Section 2: Runtime Status Keys That Don't Match Java POWER_ID

These are keys used directly in `entity.statuses` dicts at runtime that differ from the Java canonical POWER_ID. This is the highest-impact problem -- the actual status dict keys at runtime are wrong.

### 2a. Wrong key used for an existing Java power

| Java POWER_ID | Runtime Key Used | Files Using Wrong Key | Fix |
|---|---|---|---|
| `"Weakened"` | `"Weak"` | combat_engine.py:608,1867; effects/registry.py:386; effects/cards.py:282,384,961,2110,2123,2131,2488,2949,3075; state/combat.py:83; calc/combat_sim.py:583; handlers/combat.py:380 | Replace `"Weak"` with `"Weakened"` |
| `"No Draw"` | `"NoDraw"` | combat_engine.py:379; effects/cards.py:1906,3020; calc/combat_sim.py:867 | Replace `"NoDraw"` with `"No Draw"` |
| `"Controlled"` (MentalFortress) | `"MentalFortress"` | effects/cards.py:644; combat_engine.py:1680; calc/combat_sim.py:647; registry/potions.py:460 | Replace `"MentalFortress"` with `"Controlled"` |
| `"Adaptation"` (Rushdown) | `"Rushdown"` | effects/cards.py:658; combat_engine.py:1681,1682; calc/combat_sim.py:652 | Replace `"Rushdown"` with `"Adaptation"` |
| `"LikeWaterPower"` | `"LikeWater"` | effects/cards.py:665; calc/combat_sim.py:715 | Replace `"LikeWater"` with `"LikeWaterPower"` |
| `"DevotionPower"` | `"Devotion"` | effects/cards.py:672; calc/combat_sim.py:880 | Replace `"Devotion"` with `"DevotionPower"` |
| `"EstablishmentPower"` | `"Establishment"` | effects/cards.py:684 | Replace `"Establishment"` with `"EstablishmentPower"` |
| `"WireheadingPower"` (Foresight) | `"Foresight"` | effects/cards.py:691 | Replace `"Foresight"` with `"WireheadingPower"` |
| `"MasterRealityPower"` | `"MasterReality"` | effects/cards.py:716,1052,1371,1449,2533,2558; effects/executor.py:452 | Replace `"MasterReality"` with `"MasterRealityPower"` |
| `"WrathNextTurnPower"` | `"WrathNextTurn"` | effects/cards.py:740; combat_engine.py:404,406,1311,1315; effects/executor.py:454,615,618 | Replace `"WrathNextTurn"` with `"WrathNextTurnPower"` |
| `"Draw Card"` | `"DrawCardNextTurn"` | effects/cards.py:741; combat_engine.py:374,377,1312,1313,1316; effects/executor.py:454,622,625 | Replace `"DrawCardNextTurn"` with `"Draw Card"` |
| `"OmegaPower"` | `"Omega"` | effects/cards.py:876 | Replace `"Omega"` with `"OmegaPower"` |
| `"WaveOfTheHandPower"` | `"WaveOfTheHand"` | effects/cards.py:754; registry/powers.py:1021,1022 | Replace `"WaveOfTheHand"` with `"WaveOfTheHandPower"` |
| `"After Image"` | `"AfterImage"` | effects/cards.py:2724 | Replace `"AfterImage"` with `"After Image"` |
| `"Thousand Cuts"` | `"ThousandCuts"` | effects/cards.py:2838 | Replace `"ThousandCuts"` with `"Thousand Cuts"` |
| `"Noxious Fumes"` | `"NoxiousFumes"` | effects/cards.py:2515 | Replace `"NoxiousFumes"` with `"Noxious Fumes"` |
| `"CorpseExplosionPower"` | `"CorpseExplosion"` | effects/cards.py:2521 | Replace `"CorpseExplosion"` with `"CorpseExplosionPower"` |
| `"Infinite Blades"` | `"InfiniteBlades"` | effects/cards.py:2543 | Replace `"InfiniteBlades"` with `"Infinite Blades"` |
| `"Tools Of The Trade"` | `"ToolsOfTheTrade"` | effects/cards.py:2700 | Replace `"ToolsOfTheTrade"` with `"Tools Of The Trade"` |
| `"Feel No Pain"` | `"FeelNoPain"` | effects/cards.py:2205 | Replace `"FeelNoPain"` with `"Feel No Pain"` |
| `"Fire Breathing"` | `"FireBreathing"` | effects/cards.py:2212 | Replace `"FireBreathing"` with `"Fire Breathing"` |
| `"Flame Barrier"` | `"FlameBarrier"` | effects/cards.py:2219; registry/powers.py:170,171 | Replace `"FlameBarrier"` with `"Flame Barrier"` |
| `"Demon Form"` | `"DemonForm"` | effects/cards.py:2226 | Replace `"DemonForm"` with `"Demon Form"` |
| `"Dark Embrace"` | `"DarkEmbrace"` | effects/cards.py:2191 | Replace `"DarkEmbrace"` with `"Dark Embrace"` |
| `"Double Tap"` | `"DoubleTap"` | effects/cards.py:2268; registry/powers.py:1146-1159 | Replace `"DoubleTap"` with `"Double Tap"` |
| `"Flex"` (LoseStrength) | `"LoseStrength"` | effects/cards.py:1728; registry/powers.py:382; registry/relics.py:205 | Replace `"LoseStrength"` with `"Flex"` |
| `"DexLoss"` | `"LoseDexterity"` | registry/powers.py:391; registry/relics.py:863 | Replace `"LoseDexterity"` with `"DexLoss"` |
| `"Next Turn Block"` | `"NextTurnBlock"` | effects/cards.py:2712; registry/powers.py:136; registry/relics.py:739,740; combat_engine.py:1274-1277 (via Equilibrium) | Replace `"NextTurnBlock"` with `"Next Turn Block"` |
| `"Wraith Form v2"` | `"WraithFormPower"` | effects/cards.py:2908 | Replace `"WraithFormPower"` with `"Wraith Form v2"` |
| `"Phantasmal"` | `"PhantasmalKiller"` | effects/cards.py:2914; registry/powers.py:1207 | Replace `"PhantasmalKiller"` with `"Phantasmal"` |
| `"Retain Cards"` | `"WellLaidPlans"` | effects/cards.py:2889 | Replace `"WellLaidPlans"` with `"Retain Cards"` |
| `"Plated Armor"` | `"PlatedArmor"` | effects/cards.py:920 | Replace `"PlatedArmor"` with `"Plated Armor"` (space) |
| `"Lockon"` | `"Lock-On"` | effects/orbs.py:351 | Replace `"Lock-On"` with `"Lockon"` |
| `"Regeneration"` | `"Regen"` | combat_engine.py:453; combat_engine.py:2239,2242 | Replace `"Regen"` with `"Regeneration"` |
| `"Draw Reduction"` | `"DrawReduction"` | registry/powers.py:1994 | Replace `"DrawReduction"` with `"Draw Reduction"` |
| `"Night Terror"` | `"Night Terror"` | registry/powers.py:2144 | Already correct (matches Java) |
| `"DuplicationPower"` | `"Duplication"` | registry/powers.py:562-564 | Replace `"Duplication"` with `"DuplicationPower"` |

### 2b. Python-only status keys (no Java POWER_ID equivalent)

These are synthetic/internal keys the Python engine uses that don't correspond to any Java power:

| Key | Purpose | Notes |
|---|---|---|
| `"RetainHand"` | Boolean marker for "retain all cards" | Used by Equilibrium/Runic Pyramid. No Java equivalent -- Java handles via Equilibrium/BarricadePower logic |
| `"PanacheCounter"` | Tracks cards-played-this-turn for Panache | Java uses Panache.amount as counter. Python should use power amount, not separate key |
| `"TempStrengthDown"` | Tracks temporary strength reduction from Piercing Wail etc. | Java uses `Shackled` (GainStrengthPower) |
| `"TempStrengthLoss"` | Tracks temp strength loss on enemies | Same as TempStrengthDown, should use `"Shackled"` |
| `"OrbSlots"` | Bonus orb slots beyond base 3 | Python internal bookkeeping for Defect. No direct Java POWER_ID |
| `"NightmareCard"` | Stores card ID for Nightmare power to replay | Python internal. Java stores in NightmarePower.card field |
| `"NextTurnDraw"` | Draw extra cards next turn | Distinct from `"Draw Card"` (DrawCardNextTurnPower). Appears to be a duplicate -- should use `"Draw Card"` |
| `"NextTurnEnergy"` | Gain extra energy next turn | Should be `"Energized"` (Java POWER_ID) |
| `"Confused"` | Confusion debuff applied by relics | Should be `"Confusion"` (Java POWER_ID) |
| `"ZeroCostCards"` | Cards cost 0 this turn | No Java equivalent. May correspond to an unnamed mechanic |
| `"Normality"` | Cannot play >3 cards per turn | No Java power; this is a Curse card effect, not a power in Java |
| `"Pain"` | Take damage when playing cards | No Java power; this is a Curse card effect in Java |
| `"Blasphemy"` | Die at end of turn marker | Java uses `"EndTurnDeath"` POWER_ID |

---

## Section 3: Complete Java POWER_ID Reference

All 148 non-deprecated Java POWER_IDs sorted alphabetically with their Java class, Python POWER_DATA key, and match status.

| # | Java POWER_ID | Java Class | Python POWER_DATA Key | Status |
|---|---|---|---|---|
| 1 | `"Accuracy"` | AccuracyPower | `Accuracy` | MATCH |
| 2 | `"Adaptation"` | RushdownPower | `Adaptation` | MATCH (but runtime uses `"Rushdown"`) |
| 3 | `"After Image"` | AfterImagePower | `After Image` | MATCH (but runtime uses `"AfterImage"`) |
| 4 | `"Amplify"` | AmplifyPower | `Amplify` | MATCH |
| 5 | `"AngelForm"` | LiveForeverPower | `LiveForever` | MISMATCH -- Python uses `LiveForever` |
| 6 | `"Anger"` | AngerPower | `Anger` | MATCH |
| 7 | `"Angry"` | AngryPower | `Angry` | MATCH |
| 8 | `"Artifact"` | ArtifactPower | `Artifact` | MATCH |
| 9 | `"Attack Burn"` | AttackBurnPower | `AttackBurn` | MISMATCH -- Python removes space |
| 10 | `"BackAttack"` | BackAttackPower | `BackAttack` | MATCH |
| 11 | `"Barricade"` | BarricadePower | `Barricade` | MATCH |
| 12 | `"BattleHymn"` | BattleHymnPower | `BattleHymn` | MATCH |
| 13 | `"BeatOfDeath"` | BeatOfDeathPower | `BeatOfDeath` | MATCH |
| 14 | `"Berserk"` | BerserkPower | `Berserk` | MATCH |
| 15 | `"Bias"` | BiasPower | `Bias` | MATCH |
| 16 | `"BlockReturnPower"` | BlockReturnPower | `BlockReturnPower` | MATCH (but runtime uses `"BlockReturn"`) |
| 17 | `"Blur"` | BlurPower | `Blur` | MATCH |
| 18 | `"Brutality"` | BrutalityPower | `Brutality` | MATCH |
| 19 | `"Buffer"` | BufferPower | `Buffer` | MATCH |
| 20 | `"Burst"` | BurstPower | `Burst` | MATCH |
| 21 | `"CannotChangeStancePower"` | CannotChangeStancePower | `CannotChangeStancePower` | MATCH |
| 22 | `"Choked"` | ChokePower | `Choked` | MATCH |
| 23 | `"Collect"` | CollectPower | `Collect` | MATCH |
| 24 | `"Combust"` | CombustPower | `Combust` | MATCH |
| 25 | `"Compulsive"` | ReactivePower | `Reactive` | MISMATCH -- Python renamed |
| 26 | `"Confusion"` | ConfusionPower | `Confusion` | MATCH (but runtime uses `"Confused"`) |
| 27 | `"Conserve"` | ConservePower | `Conserve` | MATCH |
| 28 | `"Constricted"` | ConstrictedPower | `Constricted` | MATCH |
| 29 | `"Controlled"` | MentalFortressPower | `Controlled` | MATCH (but runtime uses `"MentalFortress"`) |
| 30 | `"CorpseExplosionPower"` | CorpseExplosionPower | `CorpseExplosion` | MISMATCH -- Python drops `Power` suffix |
| 31 | `"Corruption"` | CorruptionPower | `Corruption` | MATCH |
| 32 | `"Creative AI"` | CreativeAIPower | `Creative AI` | MATCH |
| 33 | `"Curiosity"` | CuriosityPower | `Curiosity` | MATCH |
| 34 | `"Curl Up"` | CurlUpPower | `CurlUp` | MISMATCH -- Python removes space |
| 35 | `"Dark Embrace"` | DarkEmbracePower | `Dark Embrace` | MATCH (but runtime uses `"DarkEmbrace"`) |
| 36 | `"Demon Form"` | DemonFormPower | `Demon Form` | MATCH (but runtime uses `"DemonForm"`) |
| 37 | `"DevaForm"` | DevaPower | `DevaForm` | MATCH |
| 38 | `"DevotionPower"` | DevotionPower | `DevotionPower` | MATCH (but runtime uses `"Devotion"`) |
| 39 | `"DexLoss"` | LoseDexterityPower | `LoseDexterity` | MISMATCH -- Python renamed |
| 40 | `"Dexterity"` | DexterityPower | `Dexterity` | MATCH |
| 41 | `"Double Damage"` | DoubleDamagePower | `Double Damage` | MATCH |
| 42 | `"Double Tap"` | DoubleTapPower | `Double Tap` | MATCH (but runtime uses `"DoubleTap"`) |
| 43 | `"Draw"` | DrawPower | `Draw` | MATCH |
| 44 | `"Draw Card"` | DrawCardNextTurnPower | `DrawCardNextTurn` | MISMATCH -- Python renamed |
| 45 | `"Draw Reduction"` | DrawReductionPower | `Draw Reduction` | MATCH (but runtime also uses `"DrawReduction"`) |
| 46 | `"DuplicationPower"` | DuplicationPower | `Duplication` | MISMATCH -- Python drops `Power` suffix |
| 47 | `"Echo Form"` | EchoPower | `Echo Form` | MATCH |
| 48 | `"Electro"` | ElectroPower | `Electro` | MATCH |
| 49 | `"EndTurnDeath"` | EndTurnDeathPower | `EndTurnDeath` | MATCH (but runtime also uses `"Blasphemy"`) |
| 50 | `"Energized"` | EnergizedPower | `Energized` | MATCH (but runtime also uses `"NextTurnEnergy"`) |
| 51 | `"EnergizedBlue"` | EnergizedBluePower | `EnergizedBlue` | MATCH |
| 52 | `"EnergyDownPower"` | EnergyDownPower | `EnergyDown` | MISMATCH -- Python drops `Power` suffix |
| 53 | `"Entangled"` | EntanglePower | `Entangled` | MATCH |
| 54 | `"Envenom"` | EnvenomPower | `Envenom` | MATCH |
| 55 | `"Equilibrium"` | EquilibriumPower | `Equilibrium` | MATCH |
| 56 | `"EstablishmentPower"` | EstablishmentPower | `EstablishmentPower` | MATCH (but runtime uses `"Establishment"`) |
| 57 | `"Evolve"` | EvolvePower | `Evolve` | MATCH |
| 58 | `"Explosive"` | ExplosivePower | `Explosive` | MATCH |
| 59 | `"Fading"` | FadingPower | `Fading` | MATCH |
| 60 | `"Feel No Pain"` | FeelNoPainPower | `Feel No Pain` | MATCH (but runtime uses `"FeelNoPain"`) |
| 61 | `"Fire Breathing"` | FireBreathingPower | `Fire Breathing` | MATCH (but runtime uses `"FireBreathing"`) |
| 62 | `"Flame Barrier"` | FlameBarrierPower | `Flame Barrier` | MATCH (but runtime uses `"FlameBarrier"`) |
| 63 | `"Flex"` | LoseStrengthPower | `Flex` | MATCH (but runtime uses `"LoseStrength"`) |
| 64 | `"Flight"` | FlightPower | `Flight` | MATCH |
| 65 | `"Focus"` | FocusPower | `Focus` | MATCH |
| 66 | `"Frail"` | FrailPower | `Frail` | MATCH |
| 67 | `"FreeAttackPower"` | FreeAttackPower | `FreeAttackPower` | MATCH |
| 68 | `"Generic Strength Up Power"` | GenericStrengthUpPower | `GenericStrengthUp` | MISMATCH -- Python shortened |
| 69 | `"GrowthPower"` | GrowthPower | `GrowthPower` | MATCH |
| 70 | `"Heatsink"` | HeatsinkPower | `Heatsink` | MATCH |
| 71 | `"Hello"` | HelloPower | `Hello` | MATCH |
| 72 | `"Hex"` | HexPower | `Hex` | MATCH |
| 73 | `"Infinite Blades"` | InfiniteBladesPower | `Infinite Blades` | MATCH (but runtime uses `"InfiniteBlades"`) |
| 74 | `"Intangible"` | IntangiblePower | `Intangible` | MATCH |
| 75 | `"IntangiblePlayer"` | IntangiblePlayerPower | `IntangiblePlayer` | MATCH |
| 76 | `"Invincible"` | InvinciblePower | `Invincible` | MATCH |
| 77 | `"Juggernaut"` | JuggernautPower | `Juggernaut` | MATCH |
| 78 | `"Life Link"` | RegrowPower/ResurrectPower | `Life Link` | MATCH |
| 79 | `"Lightning Mastery"` | LightningMasteryPower | `LightningMastery` | MISMATCH -- Python removes space |
| 80 | `"LikeWaterPower"` | LikeWaterPower | `LikeWaterPower` | MATCH (but runtime uses `"LikeWater"`) |
| 81 | `"Lockon"` | LockOnPower | `Lockon` | MATCH (but runtime uses `"Lock-On"`) |
| 82 | `"Loop"` | LoopPower | `Loop` | MATCH |
| 83 | `"Magnetism"` | MagnetismPower | `Magnetism` | MATCH |
| 84 | `"Malleable"` | MalleablePower | `Malleable` | MATCH |
| 85 | `"Mantra"` | MantraPower | `Mantra` | MATCH |
| 86 | `"MasterRealityPower"` | MasterRealityPower | `MasterRealityPower` | MATCH (but runtime uses `"MasterReality"`) |
| 87 | `"Mayhem"` | MayhemPower | `Mayhem` | MATCH |
| 88 | `"Metallicize"` | MetallicizePower | `Metallicize` | MATCH |
| 89 | `"Minion"` | MinionPower | `Minion` | MATCH |
| 90 | `"Mode Shift"` | ModeShiftPower | `Mode Shift` | MATCH |
| 91 | `"Next Turn Block"` | NextTurnBlockPower | `NextTurnBlock` | MISMATCH -- Python removes space |
| 92 | `"Night Terror"` | NightmarePower | `Nightmare` | MISMATCH -- Python renamed |
| 93 | `"Nirvana"` | NirvanaPower | `Nirvana` | MATCH |
| 94 | `"No Draw"` | NoDrawPower | `No Draw` | MATCH (but runtime uses `"NoDraw"`) |
| 95 | `"NoBlockPower"` | NoBlockPower | `NoBlockPower` | MATCH |
| 96 | `"NoSkills"` | NoSkillsPower | `NoSkills` | MATCH |
| 97 | `"Noxious Fumes"` | NoxiousFumesPower | `Noxious Fumes` | MATCH (but runtime uses `"NoxiousFumes"`) |
| 98 | `"Nullify Attack"` | ForcefieldPower | `Forcefield` | MISMATCH -- Python renamed |
| 99 | `"OmegaPower"` | OmegaPower | `OmegaPower` | MATCH (but runtime uses `"Omega"`) |
| 100 | `"OmnisciencePower"` | OmnisciencePower | `Omniscience` | MISMATCH -- Python drops `Power` suffix |
| 101 | `"Painful Stabs"` | PainfulStabsPower | `PainfulStabs` | MISMATCH -- Python removes space |
| 102 | `"Panache"` | PanachePower | `Panache` | MATCH |
| 103 | `"PathToVictoryPower"` | MarkPower | `PathToVictoryPower` | MATCH (but runtime uses `"Mark"`) |
| 104 | `"Pen Nib"` | PenNibPower | `Pen Nib` | MATCH |
| 105 | `"Phantasmal"` | PhantasmalPower | `Phantasmal` | MATCH (but runtime uses `"PhantasmalKiller"`) |
| 106 | `"Plated Armor"` | PlatedArmorPower | `Plated Armor` | MATCH (but runtime also uses `"PlatedArmor"`) |
| 107 | `"Poison"` | PoisonPower | `Poison` | MATCH |
| 108 | `"Rage"` | RagePower | `Rage` | MATCH |
| 109 | `"Rebound"` | ReboundPower | `Rebound` | MATCH |
| 110 | `"RechargingCore"` | RechargingCorePower | `RechargingCore` | MATCH |
| 111 | `"Regenerate"` | RegenerateMonsterPower | `RegenerateMonster` | MISMATCH -- Python renamed |
| 112 | `"Regeneration"` | RegenPower | `Regeneration` | MATCH (but runtime uses `"Regen"`) |
| 113 | `"Repair"` | RepairPower | `Repair` | MATCH |
| 114 | `"Retain Cards"` | RetainCardPower | `Retain Cards` | MATCH (but runtime uses `"WellLaidPlans"`) |
| 115 | `"Ritual"` | RitualPower | `Ritual` | MATCH |
| 116 | `"Rupture"` | RupturePower | `Rupture` | MATCH |
| 117 | `"Sadistic"` | SadisticPower | `Sadistic` | MATCH |
| 118 | `"Shackled"` | GainStrengthPower | `GainStrength` | MISMATCH -- Python renamed |
| 119 | `"Sharp Hide"` | SharpHidePower | `SharpHide` | MISMATCH -- Python removes space |
| 120 | `"Shifting"` | ShiftingPower | `Shifting` | MATCH |
| 121 | `"Skill Burn"` | SkillBurnPower | `SkillBurn` | MISMATCH -- Python removes space |
| 122 | `"Slow"` | SlowPower | `Slow` | MATCH |
| 123 | `"Split"` | SplitPower | `Split` | MATCH |
| 124 | `"Spore Cloud"` | SporeCloudPower | `SporeCloud` | MISMATCH -- Python removes space |
| 125 | `"Stasis"` | StasisPower | `Stasis` | MATCH |
| 126 | `"StaticDischarge"` | StaticDischargePower | `Static Discharge` | MISMATCH -- Python adds space |
| 127 | `"Storm"` | StormPower | `Storm` | MATCH |
| 128 | `"Strength"` | StrengthPower | `Strength` | MATCH |
| 129 | `"StrikeUp"` | StrikeUpPower | `StrikeUp` | MATCH |
| 130 | `"Study"` | StudyPower | `Study` | MATCH |
| 131 | `"Surrounded"` | SurroundedPower | `Surrounded` | MATCH |
| 132 | `"TheBomb"` | TheBombPower | `TheBomb` | MATCH |
| 133 | `"Thievery"` | ThieveryPower | `Thievery` | MATCH |
| 134 | `"Thorns"` | ThornsPower | `Thorns` | MATCH |
| 135 | `"Thousand Cuts"` | ThousandCutsPower | `Thousand Cuts` | MATCH (but runtime uses `"ThousandCuts"`) |
| 136 | `"Time Warp"` | TimeWarpPower | `Time Warp` | MATCH |
| 137 | `"TimeMazePower"` | TimeMazePower | `TimeMaze` | MISMATCH -- Python drops `Power` suffix |
| 138 | `"Tools Of The Trade"` | ToolsOfTheTradePower | `ToolsOfTheTrade` | MISMATCH -- Python removes spaces |
| 139 | `"Unawakened"` | UnawakenedPower | `Unawakened` | MATCH |
| 140 | `"Vault"` | VaultPower | `Vault` | MATCH |
| 141 | `"Vigor"` | VigorPower | `Vigor` | MATCH |
| 142 | `"Vulnerable"` | VulnerablePower | `Vulnerable` | MATCH |
| 143 | `"WaveOfTheHandPower"` | WaveOfTheHandPower | `WaveOfTheHandPower` | MATCH (but runtime uses `"WaveOfTheHand"`) |
| 144 | `"Weakened"` | WeakPower | `Weakened` | MATCH (but runtime uses `"Weak"`) |
| 145 | `"Winter"` | WinterPower | `Winter` | MATCH |
| 146 | `"WireheadingPower"` | ForesightPower | `WireheadingPower` | MATCH (but runtime uses `"Foresight"`) |
| 147 | `"Wraith Form v2"` | WraithFormPower | `Wraith Form v2` | MATCH (but runtime uses `"WraithFormPower"`) |
| 148 | `"WrathNextTurnPower"` | WrathNextTurnPower | `WrathNextTurnPower` | MATCH (but runtime uses `"WrathNextTurn"`) |

---

## Section 4: Priority Fix List

### Priority 1: HIGH (breaks parity, triggers fire for wrong key)

These are cases where `@power_trigger(power="X")` uses a different key than what's stored in `entity.statuses`. The trigger will never fire because the registry can't find the power.

| @power_trigger power= | Runtime statuses key | Java POWER_ID | Impact |
|---|---|---|---|
| `"Foresight"` | `"Foresight"` | `"WireheadingPower"` | Trigger registered as Foresight, alias resolves; but inconsistent |
| `"MentalFortress"` | `"MentalFortress"` | `"Controlled"` | Same as above |
| `"Rushdown"` | `"Rushdown"` | `"Adaptation"` | Same |
| `"LikeWater"` | `"LikeWater"` | `"LikeWaterPower"` | Same |
| `"NextTurnDraw"` | `"NextTurnDraw"` | No Java equivalent | **Dead power** -- should be `"Draw Card"` |
| `"NextTurnEnergy"` | `"NextTurnEnergy"` | No Java equivalent | **Dead power** -- should be `"Energized"` |
| `"NextTurnBlock"` | `"NextTurnBlock"` | `"Next Turn Block"` | Space mismatch |
| `"NoDraw"` | `"NoDraw"` | `"No Draw"` | Space mismatch |
| `"DoubleTap"` | `"DoubleTap"` | `"Double Tap"` | Space mismatch |
| `"FlameBarrier"` | `"FlameBarrier"` | `"Flame Barrier"` | Space mismatch |

### Priority 2: MEDIUM (functional via alias resolution but messy)

These work because `resolve_power_id()` maps them, but the statuses dict stores a non-canonical key, causing O(n) alias lookups and potential bugs in direct dict access.

All 30+ runtime key mismatches from Section 2a fall here.

### Priority 3: LOW (Python-only internal keys)

- `"RetainHand"` -- internal marker, no Java equivalent needed
- `"PanacheCounter"` -- should be folded into Panache power amount
- `"OrbSlots"` -- internal Defect bookkeeping
- `"NightmareCard"` -- internal storage for Night Terror

---

## Section 5: @power_trigger Registration vs Java POWER_ID

These are the `power=` arguments in `@power_trigger()` decorators that don't match the Java POWER_ID exactly.

| @power_trigger power= | Java POWER_ID | Registered? | Notes |
|---|---|---|---|
| `"AfterImage"` | `"After Image"` | Via alias | Missing space |
| `"BeatOfDeath"` | `"BeatOfDeath"` | Direct | OK |
| `"BlockReturnPower"` | `"BlockReturnPower"` | Direct | OK |
| `"CreativeAI"` | `"Creative AI"` | Via alias | Missing space |
| `"DarkEmbrace"` | `"Dark Embrace"` | Via alias | Missing space |
| `"DemonForm"` | `"Demon Form"` | Via alias | Missing space |
| `"DoubleTap"` | `"Double Tap"` | Via alias | Missing space |
| `"DrawCardNextTurn"` | `"Draw Card"` | Via alias | Different name |
| `"EnergyDownPower"` | `"EnergyDownPower"` | Direct | OK |
| `"FeelNoPain"` | `"Feel No Pain"` | Via alias | Missing space |
| `"FireBreathing"` | `"Fire Breathing"` | Via alias | Missing space |
| `"FlameBarrier"` | `"Flame Barrier"` | Via alias | Missing space |
| `"Foresight"` | `"WireheadingPower"` | Via alias | Completely renamed |
| `"GrowthPower"` | `"GrowthPower"` | Direct | OK |
| `"InfiniteBlades"` | `"Infinite Blades"` | Via alias | Missing space |
| `"LikeWater"` | `"LikeWaterPower"` | Via alias | Missing suffix |
| `"Lock-On"` | `"Lockon"` | Via alias | Hyphen vs none |
| `"LoseDexterity"` | `"DexLoss"` | Via alias | Completely renamed |
| `"LoseStrength"` | `"Flex"` | Via alias | Completely renamed |
| `"MentalFortress"` | `"Controlled"` | Via alias | Completely renamed |
| `"NextTurnBlock"` | `"Next Turn Block"` | Via alias | Missing spaces |
| `"NextTurnDraw"` | N/A | NEW | No Java equivalent |
| `"NextTurnEnergy"` | N/A | NEW | No Java equivalent |
| `"NoDraw"` | `"No Draw"` | Via alias | Missing space |
| `"NoxiousFumes"` | `"Noxious Fumes"` | Via alias | Missing space |
| `"PhantasmalKiller"` | `"Phantasmal"` | Via alias | Different name |
| `"Rushdown"` | `"Adaptation"` | Via alias | Completely renamed |
| `"SadisticNature"` | `"Sadistic"` | Via alias | Added "Nature" |
| `"Shackled"` | `"Shackled"` | Direct | OK (Java POWER_ID, not Python's GainStrength) |
| `"StaticDischarge"` | `"StaticDischarge"` | Direct | OK |
| `"ThousandCuts"` | `"Thousand Cuts"` | Via alias | Missing space |
| `"ToolsOfTheTrade"` | `"Tools Of The Trade"` | Via alias | Missing spaces |
| `"WaveOfTheHand"` | `"WaveOfTheHandPower"` | Via alias | Missing suffix |
| `"WraithFormPower"` | `"Wraith Form v2"` | Via alias | Different name |
| `"ZeroCostCards"` | N/A | NEW | No Java equivalent |

---

## Section 6: Deprecated Java Powers (excluded from mapping)

These exist in `decompiled/java-src/com/megacrit/cardcrawl/powers/deprecated/` and should NOT be implemented:

| POWER_ID | Java File |
|---|---|
| `"Grounded"` | DEPRECATEDGroundedPower.java |
| `"FlowPower"` | DEPRECATEDFlowPower.java |
| `"Retribution"` | DEPRECATEDRetributionPower.java |
| `"DisciplinePower"` | DEPRECATEDDisciplinePower.java |
| `"HotHot"` | DEPRECATEDHotHotPower.java |
| `"MasterRealityPower"` (deprecated version) | DEPRECATEDMasterRealityPower.java |
| `"Mastery"` | DEPRECATEDMasteryPower.java |
| `"EmotionalTurmoilPower"` | DEPRECATEDEmotionalTurmoilPower.java |
| `"Serenity"` | DEPRECATEDSerenityPower.java |
| `"AlwaysMad"` | DEPRECATEDAlwaysMadPower.java |
| `"FlickPower"` | DEPRECATEDFlickedPower.java |
| `"DEPRECATEDCondense"` | DEPRECATEDCondensePower.java |

**Note**: `"DisciplinePower"` appears in both deprecated Java AND in the Python engine as an active power. The Python implementation was likely added intentionally for some custom mechanic; verify if this matches any non-deprecated Java behavior.

---

## Section 7: Recommended Normalization Strategy

1. **Make POWER_DATA keys match Java POWER_IDs exactly** (25 renames in Section 1)
2. **Make all runtime status access use Java POWER_IDs** (35+ renames in Section 2a)
3. **Merge Python-only keys into Java equivalents**:
   - `"NextTurnDraw"` -> use `"Draw Card"` (Java DrawCardNextTurnPower.POWER_ID)
   - `"NextTurnEnergy"` -> use `"Energized"` (Java EnergizedPower.POWER_ID)
   - `"Confused"` -> use `"Confusion"` (Java ConfusionPower.POWER_ID)
   - `"Blasphemy"` -> use `"EndTurnDeath"` (Java EndTurnDeathPower.POWER_ID)
   - `"Regen"` -> use `"Regeneration"` (Java RegenPower.POWER_ID)
   - `"TempStrengthDown"` / `"TempStrengthLoss"` -> use `"Shackled"` (Java GainStrengthPower.POWER_ID)
4. **Keep Python-only internal keys** that have no Java equivalent:
   - `"RetainHand"` (internal marker)
   - `"PanacheCounter"` (internal counter)
   - `"OrbSlots"` (Defect bookkeeping)
   - `"NightmareCard"` (card storage)
   - `"ZeroCostCards"` (turn-based effect)
   - `"Normality"` (curse effect, not a Java power)
   - `"Pain"` (curse effect, not a Java power)
5. **Update all `@power_trigger(power=...)` to use Java POWER_IDs**
6. **Remove alias resolution overhead** once all keys are canonical
