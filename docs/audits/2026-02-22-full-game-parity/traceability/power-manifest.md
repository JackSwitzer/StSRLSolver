# Power Manifest

Deterministic Java-vs-Python manifest for power inventory and hook coverage.

- Java classes: `149`
- `exact`: `134`
- `alias`: `15`
- `missing`: `0`

## Missing Java Classes

- None

## Rows

| java_class | java_hooks_overridden | python_power_id | python_registry_hooks | status |
|---|---|---|---|---|
| `AccuracyPower` | `-` | `Accuracy` | `atDamageGive, onUseCard` | `exact` |
| `AfterImagePower` | `onUseCard` | `After Image` | `onUseCard` | `exact` |
| `AmplifyPower` | `atEndOfTurn, onUseCard` | `Amplify` | `-` | `exact` |
| `AngerPower` | `onUseCard` | `Anger` | `-` | `exact` |
| `AngryPower` | `onAttacked` | `Angry` | `-` | `exact` |
| `ArtifactPower` | `-` | `Artifact` | `-` | `exact` |
| `AttackBurnPower` | `atEndOfRound, onUseCard` | `AttackBurn` | `-` | `exact` |
| `BackAttackPower` | `-` | `BackAttack` | `-` | `exact` |
| `BarricadePower` | `-` | `Barricade` | `atStartOfTurnPostDraw` | `exact` |
| `BattleHymnPower` | `atStartOfTurn` | `BattleHymn` | `atStartOfTurn` | `exact` |
| `BeatOfDeathPower` | `onAfterUseCard` | `BeatOfDeath` | `onAfterUseCard` | `exact` |
| `BerserkPower` | `atStartOfTurn` | `Berserk` | `atStartOfTurn` | `exact` |
| `BiasPower` | `atStartOfTurn` | `Bias` | `atStartOfTurn` | `exact` |
| `BlockReturnPower` | `onAttacked` | `BlockReturnPower` | `-` | `exact` |
| `BlurPower` | `atEndOfRound` | `Blur` | `atStartOfTurn` | `exact` |
| `BrutalityPower` | `atStartOfTurnPostDraw` | `Brutality` | `atStartOfTurnPostDraw` | `exact` |
| `BufferPower` | `onAttackedToChangeDamage` | `Buffer` | `onAttackedToChangeDamage` | `exact` |
| `BurstPower` | `atEndOfTurn, onUseCard` | `Burst` | `atEndOfTurn, onUseCard` | `exact` |
| `CannotChangeStancePower` | `atEndOfTurn` | `CannotChangeStancePower` | `-` | `exact` |
| `ChokePower` | `atStartOfTurn, onUseCard` | `Choked` | `atStartOfTurn, onUseCard` | `alias` |
| `CollectPower` | `onEnergyRecharge` | `Collect` | `-` | `exact` |
| `CombustPower` | `atEndOfTurn` | `Combust` | `atEndOfTurn` | `exact` |
| `ConfusionPower` | `onCardDraw` | `Confusion` | `-` | `exact` |
| `ConservePower` | `atEndOfRound` | `Conserve` | `-` | `exact` |
| `ConstrictedPower` | `atEndOfTurn` | `Constricted` | `atEndOfTurn` | `exact` |
| `CorpseExplosionPower` | `onDeath` | `CorpseExplosion` | `onDeath` | `exact` |
| `CorruptionPower` | `onCardDraw, onUseCard` | `Corruption` | `onCardDraw, onUseCard` | `exact` |
| `CreativeAIPower` | `atStartOfTurn` | `Creative AI` | `atStartOfTurn` | `exact` |
| `CuriosityPower` | `onUseCard` | `Curiosity` | `-` | `exact` |
| `CurlUpPower` | `onAttacked` | `CurlUp` | `-` | `exact` |
| `DarkEmbracePower` | `onExhaust` | `Dark Embrace` | `onExhaust` | `exact` |
| `DemonFormPower` | `atStartOfTurnPostDraw` | `Demon Form` | `atStartOfTurnPostDraw` | `exact` |
| `DevaPower` | `onEnergyRecharge` | `DevaForm` | `onEnergyRecharge` | `alias` |
| `DevotionPower` | `atStartOfTurnPostDraw` | `DevotionPower` | `atStartOfTurnPostDraw` | `exact` |
| `DexterityPower` | `modifyBlock` | `Dexterity` | `modifyBlock` | `exact` |
| `DoubleDamagePower` | `atDamageGive, atEndOfRound` | `Double Damage` | `-` | `exact` |
| `DoubleTapPower` | `atEndOfTurn, onUseCard` | `Double Tap` | `onUseCard` | `exact` |
| `DrawCardNextTurnPower` | `atStartOfTurnPostDraw` | `DrawCardNextTurn` | `-` | `exact` |
| `DrawPower` | `-` | `Draw` | `-` | `exact` |
| `DrawReductionPower` | `atEndOfRound` | `Draw Reduction` | `-` | `exact` |
| `DuplicationPower` | `atEndOfRound, onUseCard` | `Duplication` | `onUseCard` | `exact` |
| `EchoPower` | `atStartOfTurn, onUseCard` | `Echo Form` | `-` | `alias` |
| `ElectroPower` | `-` | `Electro` | `-` | `exact` |
| `EndTurnDeathPower` | `atStartOfTurn` | `EndTurnDeath` | `-` | `exact` |
| `EnergizedBluePower` | `onEnergyRecharge` | `EnergizedBlue` | `-` | `exact` |
| `EnergizedPower` | `onEnergyRecharge` | `Energized` | `onEnergyRecharge` | `exact` |
| `EnergyDownPower` | `atStartOfTurn` | `EnergyDown` | `-` | `exact` |
| `EntanglePower` | `atEndOfTurn` | `Entangled` | `-` | `alias` |
| `EnvenomPower` | `onAttack` | `Envenom` | `onAttack` | `exact` |
| `EquilibriumPower` | `atEndOfRound, atEndOfTurn` | `Equilibrium` | `-` | `exact` |
| `EstablishmentPower` | `atEndOfTurn` | `EstablishmentPower` | `-` | `exact` |
| `EvolvePower` | `onCardDraw` | `Evolve` | `onCardDraw` | `exact` |
| `ExplosivePower` | `-` | `Explosive` | `-` | `exact` |
| `FadingPower` | `-` | `Fading` | `-` | `exact` |
| `FeelNoPainPower` | `onExhaust` | `Feel No Pain` | `onExhaust` | `exact` |
| `FireBreathingPower` | `onCardDraw` | `Fire Breathing` | `onCardDraw` | `exact` |
| `FlameBarrierPower` | `atStartOfTurn, onAttacked` | `Flame Barrier` | `atStartOfTurn, onAttacked` | `exact` |
| `FlightPower` | `atDamageFinalReceive, atStartOfTurn, onAttacked` | `Flight` | `-` | `exact` |
| `FocusPower` | `-` | `Focus` | `-` | `exact` |
| `ForcefieldPower` | `atDamageFinalReceive` | `Forcefield` | `-` | `exact` |
| `ForesightPower` | `atStartOfTurn` | `WireheadingPower` | `-` | `alias` |
| `FrailPower` | `atEndOfRound, modifyBlock` | `Frail` | `atEndOfRound, modifyBlock` | `exact` |
| `FreeAttackPower` | `onUseCard` | `FreeAttackPower` | `-` | `exact` |
| `GainStrengthPower` | `atEndOfTurn` | `GainStrength` | `-` | `exact` |
| `GenericStrengthUpPower` | `atEndOfRound` | `GenericStrengthUp` | `-` | `exact` |
| `GrowthPower` | `atEndOfRound` | `GrowthPower` | `-` | `exact` |
| `HeatsinkPower` | `onUseCard` | `Heatsink` | `onUseCard` | `exact` |
| `HelloPower` | `atStartOfTurn` | `Hello` | `-` | `exact` |
| `HexPower` | `onUseCard` | `Hex` | `-` | `exact` |
| `InfiniteBladesPower` | `atStartOfTurn` | `Infinite Blades` | `atStartOfTurn` | `exact` |
| `IntangiblePlayerPower` | `atDamageFinalReceive, atEndOfRound` | `IntangiblePlayer` | `-` | `exact` |
| `IntangiblePower` | `atDamageFinalReceive, atEndOfTurn` | `Intangible` | `atDamageFinalReceive, atEndOfTurn` | `exact` |
| `InvinciblePower` | `atStartOfTurn, onAttackedToChangeDamage` | `Invincible` | `-` | `exact` |
| `JuggernautPower` | `-` | `Juggernaut` | `onGainBlock` | `exact` |
| `LightningMasteryPower` | `-` | `LightningMastery` | `-` | `exact` |
| `LikeWaterPower` | `atEndOfTurnPreEndTurnCards` | `LikeWaterPower` | `atEndOfTurnPreEndTurnCards` | `exact` |
| `LiveForeverPower` | `atEndOfTurn` | `LiveForever` | `-` | `exact` |
| `LockOnPower` | `atEndOfRound` | `Lockon` | `-` | `exact` |
| `LoopPower` | `atStartOfTurn` | `Loop` | `atStartOfTurn` | `exact` |
| `LoseDexterityPower` | `atEndOfTurn` | `LoseDexterity` | `atEndOfTurn` | `exact` |
| `LoseStrengthPower` | `atEndOfTurn` | `Flex` | `-` | `alias` |
| `MagnetismPower` | `atStartOfTurn` | `Magnetism` | `atStartOfTurn` | `exact` |
| `MalleablePower` | `atEndOfRound, atEndOfTurn, onAttacked` | `Malleable` | `-` | `exact` |
| `MantraPower` | `-` | `Mantra` | `-` | `exact` |
| `MarkPower` | `-` | `PathToVictoryPower` | `-` | `alias` |
| `MasterRealityPower` | `-` | `MasterRealityPower` | `-` | `exact` |
| `MayhemPower` | `atStartOfTurn` | `Mayhem` | `atStartOfTurn` | `exact` |
| `MentalFortressPower` | `onChangeStance` | `Controlled` | `-` | `alias` |
| `MetallicizePower` | `atEndOfTurnPreEndTurnCards` | `Metallicize` | `atEndOfTurnPreEndTurnCards` | `exact` |
| `MinionPower` | `-` | `Minion` | `-` | `exact` |
| `ModeShiftPower` | `-` | `Mode Shift` | `-` | `exact` |
| `NextTurnBlockPower` | `atStartOfTurn` | `NextTurnBlock` | `atStartOfTurn` | `exact` |
| `NightmarePower` | `atStartOfTurn` | `Nightmare` | `-` | `exact` |
| `NirvanaPower` | `onScry` | `Nirvana` | `onScry` | `exact` |
| `NoBlockPower` | `atEndOfRound` | `NoBlockPower` | `-` | `exact` |
| `NoDrawPower` | `atEndOfTurn` | `No Draw` | `atEndOfTurn` | `exact` |
| `NoSkillsPower` | `atEndOfTurn` | `NoSkills` | `-` | `exact` |
| `NoxiousFumesPower` | `atStartOfTurnPostDraw` | `Noxious Fumes` | `atStartOfTurnPostDraw` | `exact` |
| `OmegaPower` | `atEndOfTurn` | `OmegaPower` | `atEndOfTurn` | `exact` |
| `OmnisciencePower` | `-` | `Omniscience` | `-` | `exact` |
| `PainfulStabsPower` | `-` | `PainfulStabs` | `-` | `exact` |
| `PanachePower` | `atStartOfTurn, onUseCard` | `Panache` | `onUseCard` | `exact` |
| `PenNibPower` | `atDamageGive, onUseCard` | `Pen Nib` | `-` | `exact` |
| `PhantasmalPower` | `atStartOfTurn` | `Phantasmal` | `-` | `exact` |
| `PlatedArmorPower` | `atEndOfTurnPreEndTurnCards, wasHPLost` | `Plated Armor` | `atEndOfTurnPreEndTurnCards, wasHPLost` | `exact` |
| `PoisonPower` | `atStartOfTurn` | `Poison` | `atStartOfTurn` | `exact` |
| `RagePower` | `atEndOfTurn, onUseCard` | `Rage` | `atStartOfTurnPostDraw, onUseCard` | `exact` |
| `ReactivePower` | `onAttacked` | `Reactive` | `-` | `exact` |
| `ReboundPower` | `atEndOfTurn, onAfterUseCard` | `Rebound` | `-` | `exact` |
| `RechargingCorePower` | `atStartOfTurn` | `RechargingCore` | `-` | `exact` |
| `RegenPower` | `atEndOfTurn` | `Regeneration` | `atStartOfTurn` | `alias` |
| `RegenerateMonsterPower` | `atEndOfTurn` | `RegenerateMonster` | `-` | `exact` |
| `RegrowPower` | `-` | `Life Link` | `-` | `alias` |
| `RepairPower` | `-` | `Repair` | `-` | `exact` |
| `ResurrectPower` | `-` | `Life Link` | `-` | `alias` |
| `RetainCardPower` | `atEndOfTurn` | `Retain Cards` | `-` | `alias` |
| `RitualPower` | `atEndOfRound, atEndOfTurn` | `Ritual` | `atEndOfTurn` | `exact` |
| `RupturePower` | `wasHPLost` | `Rupture` | `wasHPLost` | `exact` |
| `RushdownPower` | `onChangeStance` | `Adaptation` | `-` | `alias` |
| `SadisticPower` | `onApplyPower` | `Sadistic` | `-` | `exact` |
| `SharpHidePower` | `onUseCard` | `SharpHide` | `-` | `exact` |
| `ShiftingPower` | `onAttacked` | `Shifting` | `-` | `exact` |
| `SkillBurnPower` | `atEndOfRound, onUseCard` | `SkillBurn` | `-` | `exact` |
| `SlowPower` | `atDamageReceive, atEndOfRound, onAfterUseCard` | `Slow` | `atDamageReceive, atEndOfRound, onAfterUseCard` | `exact` |
| `SplitPower` | `-` | `Split` | `-` | `exact` |
| `SporeCloudPower` | `onDeath` | `SporeCloud` | `-` | `exact` |
| `StasisPower` | `onDeath` | `Stasis` | `-` | `exact` |
| `StaticDischargePower` | `onAttacked` | `Static Discharge` | `-` | `exact` |
| `StormPower` | `onUseCard` | `Storm` | `-` | `exact` |
| `StrengthPower` | `atDamageGive` | `Strength` | `atDamageGive` | `exact` |
| `StrikeUpPower` | `-` | `StrikeUp` | `-` | `exact` |
| `StudyPower` | `atEndOfTurn` | `Study` | `atEndOfTurn` | `exact` |
| `SurroundedPower` | `-` | `Surrounded` | `-` | `exact` |
| `TheBombPower` | `atEndOfTurn` | `TheBomb` | `-` | `exact` |
| `ThieveryPower` | `-` | `Thievery` | `-` | `exact` |
| `ThornsPower` | `onAttacked` | `Thorns` | `onAttack` | `exact` |
| `ThousandCutsPower` | `onAfterCardPlayed` | `Thousand Cuts` | `onAfterCardPlayed` | `exact` |
| `TimeMazePower` | `atStartOfTurn, onAfterUseCard` | `TimeMaze` | `-` | `exact` |
| `TimeWarpPower` | `onAfterUseCard` | `Time Warp` | `onAfterUseCard` | `exact` |
| `ToolsOfTheTradePower` | `atStartOfTurnPostDraw` | `ToolsOfTheTrade` | `atStartOfTurnPostDraw` | `exact` |
| `UnawakenedPower` | `-` | `Unawakened` | `-` | `exact` |
| `VaultPower` | `atEndOfRound` | `Vault` | `-` | `exact` |
| `VigorPower` | `atDamageGive, onUseCard` | `Vigor` | `atDamageGive, onUseCard` | `exact` |
| `VulnerablePower` | `atDamageReceive, atEndOfRound` | `Vulnerable` | `atDamageReceive, atEndOfRound` | `exact` |
| `WaveOfTheHandPower` | `atEndOfRound` | `WaveOfTheHandPower` | `onGainBlock` | `exact` |
| `WeakPower` | `atDamageGive, atEndOfRound` | `Weakened` | `atDamageGive, atEndOfRound` | `alias` |
| `WinterPower` | `atStartOfTurn` | `Winter` | `-` | `exact` |
| `WraithFormPower` | `atEndOfTurn` | `Wraith Form v2` | `-` | `alias` |
| `WrathNextTurnPower` | `atStartOfTurn` | `WrathNextTurnPower` | `-` | `exact` |
