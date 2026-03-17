# Java Power Implementations -- Comprehensive Audit

Generated for the Slay the Spire engine parity project.

- **Source:** `decompiled/java-src/com/megacrit/cardcrawl/powers/`
- **Cross-reference:** `packages/engine/registry/powers.py`
- **Note:** Java POWER_ID often differs from class name (e.g., `RushdownPower` has ID `Adaptation`)

## Summary

| Metric | Count |
|--------|-------|
| Total Java powers | 149 |
| Python IMPLEMENTED | 83 |
| Python PARTIAL | 12 |
| Python MISSING | 38 |
| PASSIVE (no behavioral hooks) | 16 |
| Watcher-specific powers | 26 |

Implementation rate (excluding passives): 83/133 = 62%

## Complete Power Reference

| # | Class Name | Java POWER_ID | Type | Watcher | Behavioral Hooks | Python Status |
|---|------------|---------------|------|---------|------------------|---------------|
| 1 | AccuracyPower | Accuracy | BUFF |  | onDrawOrDiscard | PARTIAL |
| 2 | AfterImagePower | After Image | BUFF |  | onUseCard | IMPLEMENTED |
| 3 | AmplifyPower | Amplify | BUFF |  | atEndOfTurn, onUseCard | MISSING |
| 4 | AngerPower | Anger | BUFF |  | onUseCard | MISSING |
| 5 | AngryPower | Angry | BUFF |  | onAttacked | IMPLEMENTED |
| 6 | ArtifactPower | Artifact | BUFF |  | onSpecificTrigger | MISSING |
| 7 | AttackBurnPower | Attack Burn | DEBUFF |  | atEndOfRound, onUseCard | MISSING |
| 8 | BackAttackPower | BackAttack | BUFF |  | (passive) | PASSIVE |
| 9 | BarricadePower | Barricade | BUFF |  | (passive) | PASSIVE |
| 10 | BattleHymnPower | BattleHymn | BUFF | Yes | atStartOfTurn | IMPLEMENTED |
| 11 | BeatOfDeathPower | BeatOfDeath | BUFF |  | onAfterUseCard | IMPLEMENTED |
| 12 | BerserkPower | Berserk | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 13 | BiasPower | Bias | DEBUFF |  | atStartOfTurn | IMPLEMENTED |
| 14 | BlockReturnPower | BlockReturnPower | DEBUFF | Yes | onAttacked | IMPLEMENTED |
| 15 | BlurPower | Blur | BUFF |  | atEndOfRound | PARTIAL |
| 16 | BrutalityPower | Brutality | BUFF |  | atStartOfTurnPostDraw | IMPLEMENTED |
| 17 | BufferPower | Buffer | BUFF |  | onAttackedToChangeDamage | IMPLEMENTED |
| 18 | BurstPower | Burst | BUFF |  | atEndOfTurn, onUseCard | IMPLEMENTED |
| 19 | CannotChangeStancePower | CannotChangeStancePower | DEBUFF | Yes | atEndOfTurn | IMPLEMENTED |
| 20 | ChokePower | Choked | DEBUFF |  | atStartOfTurn, onUseCard | IMPLEMENTED |
| 21 | CollectPower | Collect | BUFF |  | onEnergyRecharge | MISSING |
| 22 | CombustPower | Combust | BUFF |  | atEndOfTurn | IMPLEMENTED |
| 23 | ConfusionPower | Confusion | DEBUFF |  | onCardDraw | MISSING |
| 24 | ConservePower | Conserve | BUFF |  | atEndOfRound | MISSING |
| 25 | ConstrictedPower | Constricted | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 26 | CorpseExplosionPower | CorpseExplosionPower | DEBUFF |  | onDeath | IMPLEMENTED |
| 27 | CorruptionPower | Corruption | BUFF |  | onUseCard, onCardDraw | IMPLEMENTED |
| 28 | CreativeAIPower | Creative AI | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 29 | CuriosityPower | Curiosity | BUFF |  | onUseCard | IMPLEMENTED |
| 30 | CurlUpPower | Curl Up | BUFF |  | onAttacked | MISSING |
| 31 | DarkEmbracePower | Dark Embrace | BUFF |  | onExhaust | IMPLEMENTED |
| 32 | DemonFormPower | Demon Form | BUFF |  | atStartOfTurnPostDraw | IMPLEMENTED |
| 33 | DevaPower | DevaForm | BUFF | Yes | onEnergyRecharge | IMPLEMENTED |
| 34 | DevotionPower | DevotionPower | BUFF | Yes | atStartOfTurnPostDraw | IMPLEMENTED |
| 35 | DexterityPower | Dexterity | DEBUFF |  | modifyBlock | IMPLEMENTED |
| 36 | DoubleDamagePower | Double Damage | BUFF |  | atEndOfRound, atDamageGive | IMPLEMENTED |
| 37 | DoubleTapPower | Double Tap | BUFF |  | atEndOfTurn, onUseCard | PARTIAL |
| 38 | DrawCardNextTurnPower | Draw Card | BUFF |  | atStartOfTurnPostDraw | IMPLEMENTED |
| 39 | DrawPower | Draw | DEBUFF |  | (passive) | MISSING |
| 40 | DrawReductionPower | Draw Reduction | DEBUFF |  | atEndOfRound | MISSING |
| 41 | DuplicationPower | DuplicationPower | BUFF |  | atEndOfRound, onUseCard | PARTIAL |
| 42 | EchoPower | Echo Form | BUFF |  | atStartOfTurn, onUseCard | IMPLEMENTED |
| 43 | ElectroPower | Electro | BUFF |  | (passive) | PASSIVE |
| 44 | EndTurnDeathPower | EndTurnDeath | BUFF | Yes | atStartOfTurn | MISSING |
| 45 | EnergizedBluePower | EnergizedBlue | BUFF |  | onEnergyRecharge | MISSING |
| 46 | EnergizedPower | Energized | BUFF |  | onEnergyRecharge | IMPLEMENTED |
| 47 | EnergyDownPower | EnergyDownPower | DEBUFF | Yes | atStartOfTurn | MISSING |
| 48 | EntanglePower | Entangled | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 49 | EnvenomPower | Envenom | BUFF |  | onAttack | IMPLEMENTED |
| 50 | EquilibriumPower | Equilibrium | BUFF |  | atEndOfTurn, atEndOfRound | IMPLEMENTED |
| 51 | EstablishmentPower | EstablishmentPower | BUFF | Yes | atEndOfTurn | IMPLEMENTED |
| 52 | EvolvePower | Evolve | BUFF |  | onCardDraw | IMPLEMENTED |
| 53 | ExplosivePower | Explosive | BUFF |  | duringTurn | MISSING |
| 54 | FadingPower | Fading | BUFF |  | duringTurn | PARTIAL |
| 55 | FeelNoPainPower | Feel No Pain | BUFF |  | onExhaust | IMPLEMENTED |
| 56 | FireBreathingPower | Fire Breathing | BUFF |  | onCardDraw | IMPLEMENTED |
| 57 | FlameBarrierPower | Flame Barrier | BUFF |  | atStartOfTurn, onAttacked | IMPLEMENTED |
| 58 | FlightPower | Flight | BUFF |  | atStartOfTurn, atDamageFinalReceive, onAttacked | PARTIAL |
| 59 | FocusPower | Focus | DEBUFF |  | (passive) | PASSIVE |
| 60 | ForcefieldPower | Nullify Attack | BUFF |  | atDamageFinalReceive | MISSING |
| 61 | ForesightPower | WireheadingPower | BUFF | Yes | atStartOfTurn | IMPLEMENTED |
| 62 | FrailPower | Frail | DEBUFF |  | atEndOfRound, modifyBlock | IMPLEMENTED |
| 63 | FreeAttackPower | FreeAttackPower | BUFF | Yes | onUseCard | IMPLEMENTED |
| 64 | GainStrengthPower | Shackled | DEBUFF |  | atEndOfTurn | MISSING |
| 65 | GenericStrengthUpPower | Generic Strength Up Power | BUFF |  | atEndOfRound | MISSING |
| 66 | GrowthPower | GrowthPower | BUFF |  | atEndOfRound | IMPLEMENTED |
| 67 | HeatsinkPower | Heatsink | BUFF |  | onUseCard | IMPLEMENTED |
| 68 | HelloPower | Hello | BUFF |  | atStartOfTurn | MISSING |
| 69 | HexPower | Hex | DEBUFF |  | onUseCard | MISSING |
| 70 | InfiniteBladesPower | Infinite Blades | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 71 | IntangiblePlayerPower | IntangiblePlayer | BUFF |  | atEndOfRound, atDamageFinalReceive | IMPLEMENTED |
| 72 | IntangiblePower | Intangible | BUFF |  | atEndOfTurn, atDamageFinalReceive | IMPLEMENTED |
| 73 | InvinciblePower | Invincible | BUFF |  | atStartOfTurn, onAttackedToChangeDamage | IMPLEMENTED |
| 74 | JuggernautPower | Juggernaut | BUFF |  | onGainedBlock | PARTIAL |
| 75 | LightningMasteryPower | Lightning Mastery | BUFF |  | (passive) | PASSIVE |
| 76 | LikeWaterPower | LikeWaterPower | BUFF | Yes | atEndOfTurnPreEndTurnCards | IMPLEMENTED |
| 77 | LiveForeverPower | AngelForm | BUFF | Yes | atEndOfTurn | MISSING |
| 78 | LockOnPower | Lockon | DEBUFF |  | atEndOfRound | IMPLEMENTED |
| 79 | LoopPower | Loop | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 80 | LoseDexterityPower | DexLoss | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 81 | LoseStrengthPower | Flex | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 82 | MagnetismPower | Magnetism | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 83 | MalleablePower | Malleable | BUFF |  | atEndOfTurn, atEndOfRound, onAttacked | IMPLEMENTED |
| 84 | MantraPower | Mantra | BUFF | Yes | (passive) | PASSIVE |
| 85 | MarkPower | PathToVictoryPower | DEBUFF | Yes | (passive) | PASSIVE |
| 86 | MasterRealityPower | MasterRealityPower | BUFF | Yes | (passive) | PASSIVE |
| 87 | MayhemPower | Mayhem | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 88 | MentalFortressPower | Controlled | BUFF | Yes | onChangeStance | IMPLEMENTED |
| 89 | MetallicizePower | Metallicize | BUFF |  | atEndOfTurnPreEndTurnCards | IMPLEMENTED |
| 90 | MinionPower | Minion | BUFF |  | (passive) | PASSIVE |
| 91 | ModeShiftPower | Mode Shift | BUFF |  | (passive) | PASSIVE |
| 92 | NextTurnBlockPower | Next Turn Block | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 93 | NightmarePower | Night Terror | BUFF |  | atStartOfTurn | MISSING |
| 94 | NirvanaPower | Nirvana | BUFF | Yes | onScry | IMPLEMENTED |
| 95 | NoBlockPower | NoBlockPower | DEBUFF |  | atEndOfRound, modifyBlockLast | IMPLEMENTED |
| 96 | NoDrawPower | No Draw | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 97 | NoSkillsPower | NoSkills | DEBUFF | Yes | atEndOfTurn | MISSING |
| 98 | NoxiousFumesPower | Noxious Fumes | BUFF |  | atStartOfTurnPostDraw | IMPLEMENTED |
| 99 | OmegaPower | OmegaPower | BUFF | Yes | atEndOfTurn | IMPLEMENTED |
| 100 | OmnisciencePower | OmnisciencePower | BUFF | Yes | (passive) | PASSIVE |
| 101 | PainfulStabsPower | Painful Stabs | BUFF |  | onInflictDamage | MISSING |
| 102 | PanachePower | Panache | BUFF |  | atStartOfTurn, onUseCard | IMPLEMENTED |
| 103 | PenNibPower | Pen Nib | BUFF |  | onUseCard, atDamageGive | IMPLEMENTED |
| 104 | PhantasmalPower | Phantasmal | BUFF |  | atStartOfTurn | IMPLEMENTED |
| 105 | PlatedArmorPower | Plated Armor | BUFF |  | atEndOfTurnPreEndTurnCards, wasHPLost | PARTIAL |
| 106 | PoisonPower | Poison | DEBUFF |  | atStartOfTurn | IMPLEMENTED |
| 107 | RagePower | Rage | BUFF |  | atEndOfTurn, onUseCard | PARTIAL |
| 108 | ReactivePower | Compulsive | BUFF |  | onAttacked | MISSING |
| 109 | ReboundPower | Rebound | BUFF |  | atEndOfTurn, onAfterUseCard | MISSING |
| 110 | RechargingCorePower | RechargingCore | BUFF |  | atStartOfTurn | MISSING |
| 111 | RegenPower | Regeneration | BUFF |  | atEndOfTurn | PARTIAL |
| 112 | RegenerateMonsterPower | Regenerate | BUFF |  | atEndOfTurn | MISSING |
| 113 | RegrowPower | Life Link | BUFF |  | (passive) | PASSIVE |
| 114 | RepairPower | Repair | BUFF |  | onVictory | IMPLEMENTED |
| 115 | ResurrectPower | Life Link | BUFF |  | (passive) | PASSIVE |
| 116 | RetainCardPower | Retain Cards | BUFF |  | atEndOfTurn | IMPLEMENTED |
| 117 | RitualPower | Ritual | BUFF |  | atEndOfTurn, atEndOfRound | PARTIAL |
| 118 | RupturePower | Rupture | BUFF |  | wasHPLost | IMPLEMENTED |
| 119 | RushdownPower | Adaptation | BUFF | Yes | onChangeStance | IMPLEMENTED |
| 120 | SadisticPower | Sadistic | DEBUFF |  | onApplyPower | IMPLEMENTED |
| 121 | SharpHidePower | Sharp Hide | BUFF |  | onUseCard | MISSING |
| 122 | ShiftingPower | Shifting | BUFF |  | onAttacked | MISSING |
| 123 | SkillBurnPower | Skill Burn | DEBUFF |  | atEndOfRound, onUseCard | MISSING |
| 124 | SlowPower | Slow | DEBUFF |  | atEndOfRound, onAfterUseCard, atDamageReceive | IMPLEMENTED |
| 125 | SplitPower | Split | BUFF |  | (passive) | PASSIVE |
| 126 | SporeCloudPower | Spore Cloud | BUFF |  | onDeath | MISSING |
| 127 | StasisPower | Stasis | BUFF |  | onDeath | MISSING |
| 128 | StaticDischargePower | StaticDischarge | BUFF |  | onAttacked | IMPLEMENTED |
| 129 | StormPower | Storm | BUFF |  | onUseCard | IMPLEMENTED |
| 130 | StrengthPower | Strength | DEBUFF |  | atDamageGive | IMPLEMENTED |
| 131 | StrikeUpPower | StrikeUp | BUFF |  | onDrawOrDiscard | MISSING |
| 132 | StudyPower | Study | BUFF | Yes | atEndOfTurn | IMPLEMENTED |
| 133 | SurroundedPower | Surrounded | BUFF |  | (passive) | PASSIVE |
| 134 | TheBombPower | TheBomb | BUFF |  | atEndOfTurn | MISSING |
| 135 | ThieveryPower | Thievery | BUFF |  | (passive) | IMPLEMENTED |
| 136 | ThornsPower | Thorns | BUFF |  | onAttacked | IMPLEMENTED |
| 137 | ThousandCutsPower | Thousand Cuts | BUFF |  | onAfterCardPlayed | IMPLEMENTED |
| 138 | TimeMazePower | TimeMazePower | BUFF |  | atStartOfTurn, onAfterUseCard | MISSING |
| 139 | TimeWarpPower | Time Warp | BUFF |  | onAfterUseCard | IMPLEMENTED |
| 140 | ToolsOfTheTradePower | Tools Of The Trade | BUFF |  | atStartOfTurnPostDraw | IMPLEMENTED |
| 141 | UnawakenedPower | Unawakened | BUFF |  | (passive) | PASSIVE |
| 142 | VaultPower | Vault | BUFF | Yes | atEndOfRound | MISSING |
| 143 | VigorPower | Vigor | BUFF | Yes | onUseCard, atDamageGive | IMPLEMENTED |
| 144 | VulnerablePower | Vulnerable | DEBUFF |  | atEndOfRound, atDamageReceive | IMPLEMENTED |
| 145 | WaveOfTheHandPower | WaveOfTheHandPower | BUFF | Yes | atEndOfRound, onGainedBlock | PARTIAL |
| 146 | WeakPower | Weakened | DEBUFF |  | atEndOfRound, atDamageGive | IMPLEMENTED |
| 147 | WinterPower | Winter | BUFF |  | atStartOfTurn | MISSING |
| 148 | WraithFormPower | Wraith Form v2 | DEBUFF |  | atEndOfTurn | IMPLEMENTED |
| 149 | WrathNextTurnPower | WrathNextTurnPower | BUFF | Yes | atStartOfTurn | MISSING |

## Detailed Power Breakdown

### AccuracyPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `stackPower`: Custom stack/reduce logic
  - `onDrawOrDiscard`: (complex - see Java source)

### AfterImagePower (ID: `After Image`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Gain block equal to amount

### AmplifyPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `onUseCard`: Remove this power; Queue copy of card to play

### AngerPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onUseCard`: Apply Strength

### AngryPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttacked`: Apply Strength

### ArtifactPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onSpecificTrigger`: Remove this power

### AttackBurnPower (ID: `Attack Burn`)
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Reduce stacks by 1
  - `onUseCard`: Force exhaust played card

### BackAttackPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### BarricadePower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### BattleHymnPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Add temp card to hand
  - `stackPower`: Custom stack/reduce logic

### BeatOfDeathPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAfterUseCard`: Deal damage to target

### BerserkPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Gain energy equal to amount

### BiasPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Apply Focus
  - `stackPower`: Custom stack/reduce logic

### BlockReturnPower [WATCHER]
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttacked`: Gain block equal to amount
  - `stackPower`: Custom stack/reduce logic

### BlurPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfRound`: Remove this power

### BrutalityPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Draw cards; Lose HP equal to amount

### BufferPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttackedToChangeDamage`: Reduce stacks by 1
  - `stackPower`: Custom stack/reduce logic

### BurstPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `onUseCard`: Remove this power; Queue copy of card to play

### CannotChangeStancePower [WATCHER]
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Remove this power

### ChokePower (ID: `Choked`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Remove this power
  - `onUseCard`: Lose HP equal to amount

### CollectPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onEnergyRecharge`: Remove this power; Add temp card to hand
  - `stackPower`: Custom stack/reduce logic

### CombustPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Lose HP equal to amount; Deal damage to ALL enemies
  - `stackPower`: Custom stack/reduce logic

### ConfusionPower
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onCardDraw`: Randomize card cost 0-3

### ConservePower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Remove this power

### ConstrictedPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Deal damage to target

### CorpseExplosionPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onDeath`: Deal damage to ALL enemies

### CorruptionPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Force exhaust played card
  - `onCardDraw`: Set card cost for turn

### CreativeAIPower (ID: `Creative AI`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Add temp card to hand

### CuriosityPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Apply Strength

### CurlUpPower (ID: `Curl Up`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onAttacked`: Gain block equal to amount; Remove this power; Change enemy animation state

### DarkEmbracePower (ID: `Dark Embrace`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onExhaust`: Draw cards

### DemonFormPower (ID: `Demon Form`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Apply Strength

### DevaPower [WATCHER] (ID: `DevaForm`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onEnergyRecharge`: Gain energy directly
  - `stackPower`: Custom stack/reduce logic

### DevotionPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Apply Mantra

### DexterityPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `modifyBlock`: Add/subtract Dex to block
  - `stackPower`: Remove this power
  - `reducePower`: Remove this power

### DoubleDamagePower (ID: `Double Damage`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `atDamageGive`: Double damage output

### DoubleTapPower (ID: `Double Tap`)
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `onUseCard`: Remove this power; Queue copy of card to play

### DrawCardNextTurnPower (ID: `Draw Card`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Draw cards; Remove this power

### DrawPower
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onRemove`: Modify draw hand size
  - `reducePower`: Remove this power

### DrawReductionPower (ID: `Draw Reduction`)
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Reduce stacks by 1
  - `onRemove`: Modify draw hand size
  - `onInitialApplication`: Modify draw hand size

### DuplicationPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `onUseCard`: Remove this power; Queue copy of card to play

### EchoPower (ID: `Echo Form`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Echo first N cards per turn
  - `onUseCard`: Queue copy of card to play; Echo first N cards per turn

### ElectroPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### EndTurnDeathPower [WATCHER]
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: Lose HP equal to amount; Remove this power

### EnergizedBluePower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onEnergyRecharge`: Remove this power; Gain energy directly
  - `stackPower`: Custom stack/reduce logic

### EnergizedPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onEnergyRecharge`: Remove this power; Gain energy directly
  - `stackPower`: Custom stack/reduce logic

### EnergyDownPower [WATCHER]
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)

### EntanglePower (ID: `Entangled`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Remove this power

### EnvenomPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttack`: Apply Poison

### EquilibriumPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)
  - `atEndOfRound`: Remove this power

### EstablishmentPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)

### EvolvePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onCardDraw`: Draw cards

### ExplosivePower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `duringTurn`: Deal damage to target; Reduce stacks by 1; Kill self (suicide)

### FadingPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `duringTurn`: Reduce stacks by 1; Kill self (suicide)

### FeelNoPainPower (ID: `Feel No Pain`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onExhaust`: Gain block equal to amount

### FireBreathingPower (ID: `Fire Breathing`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onCardDraw`: Deal damage to ALL enemies

### FlameBarrierPower (ID: `Flame Barrier`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Remove this power
  - `onAttacked`: Deal damage to target
  - `stackPower`: Custom stack/reduce logic

### FlightPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atStartOfTurn`: Reset stacks to stored amount
  - `atDamageFinalReceive`: (complex - see Java source)
  - `onAttacked`: Reduce stacks by 1
  - `onRemove`: Change enemy animation state

### FocusPower
- **Type:** DEBUFF
- **Python Status:** PASSIVE
- **Hooks:**
  - `stackPower`: Remove this power
  - `reducePower`: Remove this power

### ForcefieldPower (ID: `Nullify Attack`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atDamageFinalReceive`: (complex - see Java source)

### ForesightPower [WATCHER] (ID: `WireheadingPower`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)

### FrailPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `modifyBlock`: Reduce block by 25%

### FreeAttackPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Remove this power

### GainStrengthPower (ID: `Shackled`)
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Apply Strength; Remove this power
  - `stackPower`: Remove this power
  - `reducePower`: Remove this power

### GenericStrengthUpPower (ID: `Generic Strength Up Power`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Apply Strength

### GrowthPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Apply Strength

### HeatsinkPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Draw cards
  - `stackPower`: Custom stack/reduce logic

### HelloPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: Add temp card to hand
  - `stackPower`: Custom stack/reduce logic

### HexPower
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onUseCard`: (complex - see Java source)

### InfiniteBladesPower (ID: `Infinite Blades`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Add temp card to hand
  - `stackPower`: Custom stack/reduce logic

### IntangiblePlayerPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `atDamageFinalReceive`: (complex - see Java source)

### IntangiblePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `atDamageFinalReceive`: (complex - see Java source)

### InvinciblePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)
  - `onAttackedToChangeDamage`: (complex - see Java source)

### JuggernautPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `onGainedBlock`: (complex - see Java source)

### LightningMasteryPower (ID: `Lightning Mastery`)
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### LikeWaterPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurnPreEndTurnCards`: Gain block equal to amount
  - `stackPower`: Custom stack/reduce logic

### LiveForeverPower [WATCHER] (ID: `AngelForm`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Apply PlatedArmor

### LockOnPower (ID: `Lockon`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power

### LoopPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)

### LoseDexterityPower (ID: `DexLoss`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Apply Dexterity; Remove this power

### LoseStrengthPower (ID: `Flex`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Apply Strength; Remove this power

### MagnetismPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Add temp card to hand
  - `stackPower`: Custom stack/reduce logic

### MalleablePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)
  - `atEndOfRound`: (complex - see Java source)
  - `onAttacked`: Gain block equal to amount
  - `stackPower`: Custom stack/reduce logic

### MantraPower [WATCHER]
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:**
  - `stackPower`: Remove this power

### MarkPower [WATCHER] (ID: `PathToVictoryPower`)
- **Type:** DEBUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### MasterRealityPower [WATCHER]
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### MayhemPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)

### MentalFortressPower [WATCHER] (ID: `Controlled`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onChangeStance`: Gain block equal to amount

### MetallicizePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurnPreEndTurnCards`: Gain block equal to amount

### MinionPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### ModeShiftPower (ID: `Mode Shift`)
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### NextTurnBlockPower (ID: `Next Turn Block`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Gain block equal to amount; Remove this power

### NightmarePower (ID: `Night Terror`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: Remove this power; Add temp card to hand

### NirvanaPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onScry`: Gain block equal to amount

### NoBlockPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `modifyBlockLast`: (complex - see Java source)

### NoDrawPower (ID: `No Draw`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Remove this power

### NoSkillsPower [WATCHER]
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Remove this power

### NoxiousFumesPower (ID: `Noxious Fumes`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Apply Poison
  - `stackPower`: Custom stack/reduce logic

### OmegaPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Deal damage to ALL enemies

### OmnisciencePower [WATCHER]
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### PainfulStabsPower (ID: `Painful Stabs`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onInflictDamage`: (complex - see Java source)

### PanachePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)
  - `onUseCard`: Deal damage to ALL enemies
  - `stackPower`: Custom stack/reduce logic

### PenNibPower (ID: `Pen Nib`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Remove this power
  - `atDamageGive`: Double damage output

### PhantasmalPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: Apply DoubleDamage; Reduce stacks by 1

### PlatedArmorPower (ID: `Plated Armor`)
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfTurnPreEndTurnCards`: Gain block equal to amount
  - `wasHPLost`: Reduce stacks by 1
  - `onRemove`: Change enemy animation state
  - `stackPower`: Custom stack/reduce logic

### PoisonPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)
  - `stackPower`: Custom stack/reduce logic

### RagePower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `onUseCard`: Gain block equal to amount

### ReactivePower (ID: `Compulsive`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onAttacked`: (complex - see Java source)

### ReboundPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Remove this power
  - `onAfterUseCard`: Reduce stacks by 1

### RechargingCorePower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: Gain energy equal to amount

### RegenPower (ID: `Regeneration`)
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)
  - `stackPower`: Custom stack/reduce logic

### RegenerateMonsterPower (ID: `Regenerate`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)

### RegrowPower (ID: `Life Link`)
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### RepairPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onVictory`: (complex - see Java source)

### ResurrectPower (ID: `Life Link`)
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### RetainCardPower (ID: `Retain Cards`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)

### RitualPower
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfTurn`: Apply Strength
  - `atEndOfRound`: Apply Strength

### RupturePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `wasHPLost`: Apply Strength

### RushdownPower [WATCHER] (ID: `Adaptation`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onChangeStance`: Draw cards

### SadisticPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onApplyPower`: Deal damage to target

### SharpHidePower (ID: `Sharp Hide`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onUseCard`: Deal damage to target

### ShiftingPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onAttacked`: Apply Strength

### SkillBurnPower (ID: `Skill Burn`)
- **Type:** DEBUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Reduce stacks by 1
  - `onUseCard`: Force exhaust played card

### SlowPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: (complex - see Java source)
  - `onAfterUseCard`: Apply Slow
  - `atDamageReceive`: (complex - see Java source)

### SplitPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### SporeCloudPower (ID: `Spore Cloud`)
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onDeath`: Apply Vulnerable

### StasisPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `onDeath`: Add temp card to hand

### StaticDischargePower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttacked`: (complex - see Java source)

### StormPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: (complex - see Java source)

### StrengthPower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atDamageGive`: (complex - see Java source)
  - `stackPower`: Remove this power
  - `reducePower`: Remove this power

### StrikeUpPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `stackPower`: Custom stack/reduce logic
  - `onDrawOrDiscard`: (complex - see Java source)

### StudyPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: (complex - see Java source)

### SurroundedPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### TheBombPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfTurn`: Deal damage to ALL enemies; Reduce stacks by 1

### ThieveryPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:** None (passive power -- effect handled inline in engine)

### ThornsPower
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAttacked`: Deal damage to target
  - `stackPower`: Custom stack/reduce logic

### ThousandCutsPower (ID: `Thousand Cuts`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAfterCardPlayed`: Deal damage to ALL enemies
  - `stackPower`: Custom stack/reduce logic

### TimeMazePower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)
  - `onAfterUseCard`: (complex - see Java source)

### TimeWarpPower (ID: `Time Warp`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onAfterUseCard`: Apply Strength

### ToolsOfTheTradePower (ID: `Tools Of The Trade`)
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atStartOfTurnPostDraw`: Draw cards

### UnawakenedPower
- **Type:** BUFF
- **Python Status:** PASSIVE
- **Hooks:** None (passive power -- effect handled inline in engine)

### VaultPower [WATCHER]
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atEndOfRound`: Deal damage to target; Remove this power

### VigorPower [WATCHER]
- **Type:** BUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `onUseCard`: Remove this power
  - `atDamageGive`: (complex - see Java source)

### VulnerablePower
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `atDamageReceive`: Increase damage taken by 50%

### WaveOfTheHandPower [WATCHER]
- **Type:** BUFF
- **Python Status:** PARTIAL
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `onGainedBlock`: Apply Weak

### WeakPower (ID: `Weakened`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfRound`: Remove this power
  - `atDamageGive`: Reduce damage dealt by 25%

### WinterPower
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: (complex - see Java source)
  - `stackPower`: Custom stack/reduce logic

### WraithFormPower (ID: `Wraith Form v2`)
- **Type:** DEBUFF
- **Python Status:** IMPLEMENTED
- **Hooks:**
  - `atEndOfTurn`: Apply Dexterity
  - `stackPower`: Custom stack/reduce logic

### WrathNextTurnPower [WATCHER]
- **Type:** BUFF
- **Python Status:** MISSING
- **Hooks:**
  - `atStartOfTurn`: Remove this power

## Missing/Partial Powers by Priority

### HIGH PRIORITY: Watcher Powers Missing/Partial

| Class | Java ID | Status | Hooks Needed |
|-------|---------|--------|-------------|
| EndTurnDeathPower | EndTurnDeath | MISSING | atStartOfTurn |
| EnergyDownPower | EnergyDownPower | MISSING | atStartOfTurn |
| LiveForeverPower | AngelForm | MISSING | atEndOfTurn |
| NoSkillsPower | NoSkills | MISSING | atEndOfTurn |
| VaultPower | Vault | MISSING | atEndOfRound |
| WaveOfTheHandPower | WaveOfTheHandPower | PARTIAL | atEndOfRound, onGainedBlock |
| WrathNextTurnPower | WrathNextTurnPower | MISSING | atStartOfTurn |

### MEDIUM PRIORITY: Common Enemy/Player Powers Missing/Partial

| Class | Java ID | Type | Status | Hooks Needed |
|-------|---------|------|--------|-------------|
| AccuracyPower | Accuracy | BUFF | PARTIAL | onDrawOrDiscard |
| AmplifyPower | Amplify | BUFF | MISSING | atEndOfTurn, onUseCard |
| AngerPower | Anger | BUFF | MISSING | onUseCard |
| ArtifactPower | Artifact | BUFF | MISSING | onSpecificTrigger |
| AttackBurnPower | Attack Burn | DEBUFF | MISSING | atEndOfRound, onUseCard |
| BlurPower | Blur | BUFF | PARTIAL | atEndOfRound |
| CollectPower | Collect | BUFF | MISSING | onEnergyRecharge |
| ConfusionPower | Confusion | DEBUFF | MISSING | onCardDraw |
| ConservePower | Conserve | BUFF | MISSING | atEndOfRound |
| CurlUpPower | Curl Up | BUFF | MISSING | onAttacked |
| DoubleTapPower | Double Tap | BUFF | PARTIAL | atEndOfTurn, onUseCard |
| DrawPower | Draw | DEBUFF | MISSING | (passive) |
| DrawReductionPower | Draw Reduction | DEBUFF | MISSING | atEndOfRound |
| DuplicationPower | DuplicationPower | BUFF | PARTIAL | atEndOfRound, onUseCard |
| EnergizedBluePower | EnergizedBlue | BUFF | MISSING | onEnergyRecharge |
| ExplosivePower | Explosive | BUFF | MISSING | duringTurn |
| FadingPower | Fading | BUFF | PARTIAL | duringTurn |
| FlightPower | Flight | BUFF | PARTIAL | atStartOfTurn, atDamageFinalReceive, onAttacked |
| ForcefieldPower | Nullify Attack | BUFF | MISSING | atDamageFinalReceive |
| GainStrengthPower | Shackled | DEBUFF | MISSING | atEndOfTurn |
| GenericStrengthUpPower | Generic Strength Up Power | BUFF | MISSING | atEndOfRound |
| HelloPower | Hello | BUFF | MISSING | atStartOfTurn |
| HexPower | Hex | DEBUFF | MISSING | onUseCard |
| JuggernautPower | Juggernaut | BUFF | PARTIAL | onGainedBlock |
| NightmarePower | Night Terror | BUFF | MISSING | atStartOfTurn |
| PainfulStabsPower | Painful Stabs | BUFF | MISSING | onInflictDamage |
| PlatedArmorPower | Plated Armor | BUFF | PARTIAL | atEndOfTurnPreEndTurnCards, wasHPLost |
| RagePower | Rage | BUFF | PARTIAL | atEndOfTurn, onUseCard |
| ReactivePower | Compulsive | BUFF | MISSING | onAttacked |
| ReboundPower | Rebound | BUFF | MISSING | atEndOfTurn, onAfterUseCard |
| RechargingCorePower | RechargingCore | BUFF | MISSING | atStartOfTurn |
| RegenPower | Regeneration | BUFF | PARTIAL | atEndOfTurn |
| RegenerateMonsterPower | Regenerate | BUFF | MISSING | atEndOfTurn |
| RitualPower | Ritual | BUFF | PARTIAL | atEndOfTurn, atEndOfRound |
| SharpHidePower | Sharp Hide | BUFF | MISSING | onUseCard |
| ShiftingPower | Shifting | BUFF | MISSING | onAttacked |
| SkillBurnPower | Skill Burn | DEBUFF | MISSING | atEndOfRound, onUseCard |
| SporeCloudPower | Spore Cloud | BUFF | MISSING | onDeath |
| StasisPower | Stasis | BUFF | MISSING | onDeath |
| StrikeUpPower | StrikeUp | BUFF | MISSING | onDrawOrDiscard |
| TheBombPower | TheBomb | BUFF | MISSING | atEndOfTurn |
| TimeMazePower | TimeMazePower | BUFF | MISSING | atStartOfTurn, onAfterUseCard |
| WinterPower | Winter | BUFF | MISSING | atStartOfTurn |

## Powers Grouped by Hook

### `atEndOfTurn` (27 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| AmplifyPower | Amplify | BUFF | Remove this power | No |
| BurstPower | Burst | BUFF | Remove this power | Yes |
| CannotChangeStancePower | CannotChangeStancePower | DEBUFF | Remove this power | Yes |
| CombustPower | Combust | BUFF | Lose HP equal to amount; Deal damage to ALL enemies | Yes |
| ConstrictedPower | Constricted | DEBUFF | Deal damage to target | Yes |
| DoubleTapPower | Double Tap | BUFF | Remove this power | No |
| EntanglePower | Entangled | DEBUFF | Remove this power | Yes |
| EquilibriumPower | Equilibrium | BUFF | (complex - see Java source) | Yes |
| EstablishmentPower | EstablishmentPower | BUFF | (complex - see Java source) | Yes |
| GainStrengthPower | Shackled | DEBUFF | Apply Strength; Remove this power | No |
| IntangiblePower | Intangible | BUFF | Remove this power | Yes |
| LiveForeverPower | AngelForm | BUFF | Apply PlatedArmor | No |
| LoseDexterityPower | DexLoss | DEBUFF | Apply Dexterity; Remove this power | Yes |
| LoseStrengthPower | Flex | DEBUFF | Apply Strength; Remove this power | Yes |
| MalleablePower | Malleable | BUFF | (complex - see Java source) | Yes |
| NoDrawPower | No Draw | DEBUFF | Remove this power | Yes |
| NoSkillsPower | NoSkills | DEBUFF | Remove this power | No |
| OmegaPower | OmegaPower | BUFF | Deal damage to ALL enemies | Yes |
| RagePower | Rage | BUFF | Remove this power | No |
| ReboundPower | Rebound | BUFF | Remove this power | No |
| RegenPower | Regeneration | BUFF | (complex - see Java source) | No |
| RegenerateMonsterPower | Regenerate | BUFF | (complex - see Java source) | No |
| RetainCardPower | Retain Cards | BUFF | (complex - see Java source) | Yes |
| RitualPower | Ritual | BUFF | Apply Strength | Yes |
| StudyPower | Study | BUFF | (complex - see Java source) | Yes |
| TheBombPower | TheBomb | BUFF | Deal damage to ALL enemies; Reduce stacks by 1 | No |
| WraithFormPower | Wraith Form v2 | DEBUFF | Apply Dexterity | Yes |

### `atStartOfTurn` (26 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| BattleHymnPower | BattleHymn | BUFF | Add temp card to hand | Yes |
| BerserkPower | Berserk | BUFF | Gain energy equal to amount | Yes |
| BiasPower | Bias | DEBUFF | Apply Focus | Yes |
| ChokePower | Choked | DEBUFF | Remove this power | Yes |
| CreativeAIPower | Creative AI | BUFF | Add temp card to hand | Yes |
| EchoPower | Echo Form | BUFF | Echo first N cards per turn | Yes |
| EndTurnDeathPower | EndTurnDeath | BUFF | Lose HP equal to amount; Remove this power | No |
| EnergyDownPower | EnergyDownPower | DEBUFF | (complex - see Java source) | No |
| FlameBarrierPower | Flame Barrier | BUFF | Remove this power | Yes |
| FlightPower | Flight | BUFF | Reset stacks to stored amount | Yes |
| ForesightPower | WireheadingPower | BUFF | (complex - see Java source) | Yes |
| HelloPower | Hello | BUFF | Add temp card to hand | No |
| InfiniteBladesPower | Infinite Blades | BUFF | Add temp card to hand | Yes |
| InvinciblePower | Invincible | BUFF | (complex - see Java source) | Yes |
| LoopPower | Loop | BUFF | (complex - see Java source) | Yes |
| MagnetismPower | Magnetism | BUFF | Add temp card to hand | Yes |
| MayhemPower | Mayhem | BUFF | (complex - see Java source) | Yes |
| NextTurnBlockPower | Next Turn Block | BUFF | Gain block equal to amount; Remove this power | Yes |
| NightmarePower | Night Terror | BUFF | Remove this power; Add temp card to hand | No |
| PanachePower | Panache | BUFF | (complex - see Java source) | Yes |
| PhantasmalPower | Phantasmal | BUFF | Apply DoubleDamage; Reduce stacks by 1 | Yes |
| PoisonPower | Poison | DEBUFF | (complex - see Java source) | Yes |
| RechargingCorePower | RechargingCore | BUFF | Gain energy equal to amount | No |
| TimeMazePower | TimeMazePower | BUFF | (complex - see Java source) | No |
| WinterPower | Winter | BUFF | (complex - see Java source) | No |
| WrathNextTurnPower | WrathNextTurnPower | BUFF | Remove this power | No |

### `onUseCard` (21 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| AfterImagePower | After Image | BUFF | Gain block equal to amount | Yes |
| AmplifyPower | Amplify | BUFF | Remove this power; Queue copy of card to play | No |
| AngerPower | Anger | BUFF | Apply Strength | No |
| AttackBurnPower | Attack Burn | DEBUFF | Force exhaust played card | No |
| BurstPower | Burst | BUFF | Remove this power; Queue copy of card to play | Yes |
| ChokePower | Choked | DEBUFF | Lose HP equal to amount | Yes |
| CorruptionPower | Corruption | BUFF | Force exhaust played card | Yes |
| CuriosityPower | Curiosity | BUFF | Apply Strength | Yes |
| DoubleTapPower | Double Tap | BUFF | Remove this power; Queue copy of card to play | Yes |
| DuplicationPower | DuplicationPower | BUFF | Remove this power; Queue copy of card to play | Yes |
| EchoPower | Echo Form | BUFF | Queue copy of card to play; Echo first N cards per turn | Yes |
| FreeAttackPower | FreeAttackPower | BUFF | Remove this power | Yes |
| HeatsinkPower | Heatsink | BUFF | Draw cards | Yes |
| HexPower | Hex | DEBUFF | (complex - see Java source) | No |
| PanachePower | Panache | BUFF | Deal damage to ALL enemies | Yes |
| PenNibPower | Pen Nib | BUFF | Remove this power | Yes |
| RagePower | Rage | BUFF | Gain block equal to amount | Yes |
| SharpHidePower | Sharp Hide | BUFF | Deal damage to target | No |
| SkillBurnPower | Skill Burn | DEBUFF | Force exhaust played card | No |
| StormPower | Storm | BUFF | (complex - see Java source) | Yes |
| VigorPower | Vigor | BUFF | Remove this power | Yes |

### `atEndOfRound` (21 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| AttackBurnPower | Attack Burn | DEBUFF | Reduce stacks by 1 | No |
| BlurPower | Blur | BUFF | Remove this power | No |
| ConservePower | Conserve | BUFF | Remove this power | No |
| DoubleDamagePower | Double Damage | BUFF | Remove this power | Yes |
| DrawReductionPower | Draw Reduction | DEBUFF | Reduce stacks by 1 | No |
| DuplicationPower | DuplicationPower | BUFF | Remove this power | No |
| EquilibriumPower | Equilibrium | BUFF | Remove this power | Yes |
| FrailPower | Frail | DEBUFF | Remove this power | Yes |
| GenericStrengthUpPower | Generic Strength Up Power | BUFF | Apply Strength | No |
| GrowthPower | GrowthPower | BUFF | Apply Strength | Yes |
| IntangiblePlayerPower | IntangiblePlayer | BUFF | Remove this power | Yes |
| LockOnPower | Lockon | DEBUFF | Remove this power | Yes |
| MalleablePower | Malleable | BUFF | (complex - see Java source) | Yes |
| NoBlockPower | NoBlockPower | DEBUFF | Remove this power | Yes |
| RitualPower | Ritual | BUFF | Apply Strength | No |
| SkillBurnPower | Skill Burn | DEBUFF | Reduce stacks by 1 | No |
| SlowPower | Slow | DEBUFF | (complex - see Java source) | Yes |
| VaultPower | Vault | BUFF | Deal damage to target; Remove this power | No |
| VulnerablePower | Vulnerable | DEBUFF | Remove this power | Yes |
| WaveOfTheHandPower | WaveOfTheHandPower | BUFF | Remove this power | Yes |
| WeakPower | Weakened | DEBUFF | Remove this power | Yes |

### `onAttacked` (10 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| AngryPower | Angry | BUFF | Apply Strength | Yes |
| BlockReturnPower | BlockReturnPower | DEBUFF | Gain block equal to amount | Yes |
| CurlUpPower | Curl Up | BUFF | Gain block equal to amount; Remove this power; Change enemy animation state | No |
| FlameBarrierPower | Flame Barrier | BUFF | Deal damage to target | Yes |
| FlightPower | Flight | BUFF | Reduce stacks by 1 | Yes |
| MalleablePower | Malleable | BUFF | Gain block equal to amount | Yes |
| ReactivePower | Compulsive | BUFF | (complex - see Java source) | No |
| ShiftingPower | Shifting | BUFF | Apply Strength | No |
| StaticDischargePower | StaticDischarge | BUFF | (complex - see Java source) | Yes |
| ThornsPower | Thorns | BUFF | Deal damage to target | Yes |

### `atStartOfTurnPostDraw` (6 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| BrutalityPower | Brutality | BUFF | Draw cards; Lose HP equal to amount | Yes |
| DemonFormPower | Demon Form | BUFF | Apply Strength | Yes |
| DevotionPower | DevotionPower | BUFF | Apply Mantra | Yes |
| DrawCardNextTurnPower | Draw Card | BUFF | Draw cards; Remove this power | Yes |
| NoxiousFumesPower | Noxious Fumes | BUFF | Apply Poison | Yes |
| ToolsOfTheTradePower | Tools Of The Trade | BUFF | Draw cards | Yes |

### `onAfterUseCard` (5 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| BeatOfDeathPower | BeatOfDeath | BUFF | Deal damage to target | Yes |
| ReboundPower | Rebound | BUFF | Reduce stacks by 1 | No |
| SlowPower | Slow | DEBUFF | Apply Slow | Yes |
| TimeMazePower | TimeMazePower | BUFF | (complex - see Java source) | No |
| TimeWarpPower | Time Warp | BUFF | Apply Strength | Yes |

### `atDamageGive` (5 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| DoubleDamagePower | Double Damage | BUFF | Double damage output | Yes |
| PenNibPower | Pen Nib | BUFF | Double damage output | Yes |
| StrengthPower | Strength | DEBUFF | (complex - see Java source) | Yes |
| VigorPower | Vigor | BUFF | (complex - see Java source) | Yes |
| WeakPower | Weakened | DEBUFF | Reduce damage dealt by 25% | Yes |

### `onEnergyRecharge` (4 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| CollectPower | Collect | BUFF | Remove this power; Add temp card to hand | No |
| DevaPower | DevaForm | BUFF | Gain energy directly | Yes |
| EnergizedBluePower | EnergizedBlue | BUFF | Remove this power; Gain energy directly | No |
| EnergizedPower | Energized | BUFF | Remove this power; Gain energy directly | Yes |

### `onCardDraw` (4 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| ConfusionPower | Confusion | DEBUFF | Randomize card cost 0-3 | No |
| CorruptionPower | Corruption | BUFF | Set card cost for turn | Yes |
| EvolvePower | Evolve | BUFF | Draw cards | Yes |
| FireBreathingPower | Fire Breathing | BUFF | Deal damage to ALL enemies | Yes |

### `onRemove` (4 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| DrawPower | Draw | DEBUFF | Modify draw hand size | No |
| DrawReductionPower | Draw Reduction | DEBUFF | Modify draw hand size | No |
| FlightPower | Flight | BUFF | Change enemy animation state | No |
| PlatedArmorPower | Plated Armor | BUFF | Change enemy animation state | No |

### `atDamageFinalReceive` (4 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| FlightPower | Flight | BUFF | (complex - see Java source) | Yes |
| ForcefieldPower | Nullify Attack | BUFF | (complex - see Java source) | No |
| IntangiblePlayerPower | IntangiblePlayer | BUFF | (complex - see Java source) | Yes |
| IntangiblePower | Intangible | BUFF | (complex - see Java source) | Yes |

### `onDeath` (3 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| CorpseExplosionPower | CorpseExplosionPower | DEBUFF | Deal damage to ALL enemies | Yes |
| SporeCloudPower | Spore Cloud | BUFF | Apply Vulnerable | No |
| StasisPower | Stasis | BUFF | Add temp card to hand | No |

### `atEndOfTurnPreEndTurnCards` (3 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| LikeWaterPower | LikeWaterPower | BUFF | Gain block equal to amount | Yes |
| MetallicizePower | Metallicize | BUFF | Gain block equal to amount | Yes |
| PlatedArmorPower | Plated Armor | BUFF | Gain block equal to amount | Yes |

### `onDrawOrDiscard` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| AccuracyPower | Accuracy | BUFF | (complex - see Java source) | No |
| StrikeUpPower | StrikeUp | BUFF | (complex - see Java source) | No |

### `onAttackedToChangeDamage` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| BufferPower | Buffer | BUFF | Reduce stacks by 1 | Yes |
| InvinciblePower | Invincible | BUFF | (complex - see Java source) | Yes |

### `onExhaust` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| DarkEmbracePower | Dark Embrace | BUFF | Draw cards | Yes |
| FeelNoPainPower | Feel No Pain | BUFF | Gain block equal to amount | Yes |

### `modifyBlock` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| DexterityPower | Dexterity | DEBUFF | Add/subtract Dex to block | Yes |
| FrailPower | Frail | DEBUFF | Reduce block by 25% | Yes |

### `duringTurn` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| ExplosivePower | Explosive | BUFF | Deal damage to target; Reduce stacks by 1; Kill self (suicide) | No |
| FadingPower | Fading | BUFF | Reduce stacks by 1; Kill self (suicide) | No |

### `onGainedBlock` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| JuggernautPower | Juggernaut | BUFF | (complex - see Java source) | No |
| WaveOfTheHandPower | WaveOfTheHandPower | BUFF | Apply Weak | No |

### `onChangeStance` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| MentalFortressPower | Controlled | BUFF | Gain block equal to amount | Yes |
| RushdownPower | Adaptation | BUFF | Draw cards | Yes |

### `wasHPLost` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| PlatedArmorPower | Plated Armor | BUFF | Reduce stacks by 1 | Yes |
| RupturePower | Rupture | BUFF | Apply Strength | Yes |

### `atDamageReceive` (2 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| SlowPower | Slow | DEBUFF | (complex - see Java source) | Yes |
| VulnerablePower | Vulnerable | DEBUFF | Increase damage taken by 50% | Yes |

### `onSpecificTrigger` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| ArtifactPower | Artifact | BUFF | Remove this power | No |

### `onInitialApplication` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| DrawReductionPower | Draw Reduction | DEBUFF | Modify draw hand size | No |

### `onAttack` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| EnvenomPower | Envenom | BUFF | Apply Poison | Yes |

### `onScry` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| NirvanaPower | Nirvana | BUFF | Gain block equal to amount | Yes |

### `modifyBlockLast` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| NoBlockPower | NoBlockPower | DEBUFF | (complex - see Java source) | Yes |

### `onInflictDamage` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| PainfulStabsPower | Painful Stabs | BUFF | (complex - see Java source) | No |

### `onVictory` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| RepairPower | Repair | BUFF | (complex - see Java source) | Yes |

### `onApplyPower` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| SadisticPower | Sadistic | DEBUFF | Deal damage to target | Yes |

### `onAfterCardPlayed` (1 powers)

| Class | Java ID | Type | Description | Python |
|-------|---------|------|-------------|--------|
| ThousandCutsPower | Thousand Cuts | BUFF | Deal damage to ALL enemies | Yes |
