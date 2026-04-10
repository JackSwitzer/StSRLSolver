//! Numeric status ID constants for the zero-alloc engine refactor.
//!
//! Parallel to `status_keys::sk` but using `StatusId` instead of `&str`.
//! Every constant in `status_keys::sk` has a corresponding entry here
//! with a sequential numeric ID.

use crate::ids::StatusId;

/// Numeric status IDs. Import as `use crate::status_ids::sid;`
pub mod sid {
    use super::StatusId;

    // ==================================================================
    // Core combat stats (0-4)
    // ==================================================================
    pub const STRENGTH: StatusId = StatusId(0);
    pub const DEXTERITY: StatusId = StatusId(1);
    pub const FOCUS: StatusId = StatusId(2);
    pub const VIGOR: StatusId = StatusId(3);
    pub const MANTRA: StatusId = StatusId(4);

    // ==================================================================
    // Debuffs (5-14)
    // ==================================================================
    pub const VULNERABLE: StatusId = StatusId(5);
    pub const WEAKENED: StatusId = StatusId(6);
    pub const FRAIL: StatusId = StatusId(7);
    pub const POISON: StatusId = StatusId(8);
    pub const CONSTRICTED: StatusId = StatusId(9);
    pub const ENTANGLED: StatusId = StatusId(10);
    pub const HEX: StatusId = StatusId(11);
    pub const CONFUSION: StatusId = StatusId(12);
    pub const NO_DRAW: StatusId = StatusId(13);
    pub const DRAW_REDUCTION: StatusId = StatusId(14);

    // ==================================================================
    // Ironclad powers (15-29)
    // ==================================================================
    pub const BARRICADE: StatusId = StatusId(15);
    pub const DEMON_FORM: StatusId = StatusId(16);
    pub const CORRUPTION: StatusId = StatusId(17);
    pub const DARK_EMBRACE: StatusId = StatusId(18);
    pub const FEEL_NO_PAIN: StatusId = StatusId(19);
    pub const BRUTALITY: StatusId = StatusId(20);
    pub const COMBUST: StatusId = StatusId(21);
    pub const EVOLVE: StatusId = StatusId(22);
    pub const FIRE_BREATHING: StatusId = StatusId(23);
    pub const JUGGERNAUT: StatusId = StatusId(24);
    pub const METALLICIZE: StatusId = StatusId(25);
    pub const RUPTURE: StatusId = StatusId(26);
    pub const BERSERK: StatusId = StatusId(27);
    pub const RAGE: StatusId = StatusId(28);
    pub const FLAME_BARRIER: StatusId = StatusId(29);

    // ==================================================================
    // Silent powers (30-38)
    // ==================================================================
    pub const AFTER_IMAGE: StatusId = StatusId(30);
    pub const THOUSAND_CUTS: StatusId = StatusId(31);
    pub const NOXIOUS_FUMES: StatusId = StatusId(32);
    pub const INFINITE_BLADES: StatusId = StatusId(33);
    pub const ENVENOM: StatusId = StatusId(34);
    pub const ACCURACY: StatusId = StatusId(35);
    pub const TOOLS_OF_THE_TRADE: StatusId = StatusId(36);
    pub const RETAIN_CARDS: StatusId = StatusId(37);
    // WELL_LAID_PLANS is an alias for RETAIN_CARDS (same power)
    pub const WELL_LAID_PLANS: StatusId = StatusId(37);

    // ==================================================================
    // Watcher powers (38-51)
    // ==================================================================
    pub const BATTLE_HYMN: StatusId = StatusId(38);
    pub const DEVOTION: StatusId = StatusId(39);
    pub const DEVA_FORM: StatusId = StatusId(40);
    pub const ESTABLISHMENT: StatusId = StatusId(41);
    pub const FASTING: StatusId = StatusId(42);
    pub const LIKE_WATER: StatusId = StatusId(43);
    pub const MASTER_REALITY: StatusId = StatusId(44);
    pub const MENTAL_FORTRESS: StatusId = StatusId(45);
    pub const NIRVANA: StatusId = StatusId(46);
    pub const OMEGA: StatusId = StatusId(47);
    pub const RUSHDOWN: StatusId = StatusId(48);
    pub const STUDY: StatusId = StatusId(49);
    pub const WAVE_OF_THE_HAND: StatusId = StatusId(50);
    pub const WRAITH_FORM: StatusId = StatusId(51);

    // ==================================================================
    // Defect powers (52-61)
    // ==================================================================
    pub const BUFFER: StatusId = StatusId(52);
    pub const CREATIVE_AI: StatusId = StatusId(53);
    pub const ECHO_FORM: StatusId = StatusId(54);
    pub const ELECTRO: StatusId = StatusId(55);
    pub const ELECTRODYNAMICS: StatusId = StatusId(56);
    pub const HEATSINK: StatusId = StatusId(57);
    pub const HELLO_WORLD: StatusId = StatusId(58);
    pub const LOOP: StatusId = StatusId(59);
    pub const STORM: StatusId = StatusId(60);
    pub const STATIC_DISCHARGE: StatusId = StatusId(61);

    // ==================================================================
    // Colorless / universal powers (62-65)
    // ==================================================================
    pub const PANACHE: StatusId = StatusId(62);
    pub const SADISTIC: StatusId = StatusId(63);
    pub const MAGNETISM: StatusId = StatusId(64);
    pub const MAYHEM: StatusId = StatusId(65);

    // ==================================================================
    // Temporary / turn-scoped effects (66-89)
    // ==================================================================
    pub const TEMP_STRENGTH: StatusId = StatusId(66);
    pub const TEMP_STRENGTH_LOSS: StatusId = StatusId(67);
    pub const NEXT_ATTACK_FREE: StatusId = StatusId(68);
    pub const BULLET_TIME: StatusId = StatusId(69);
    pub const DOUBLE_TAP: StatusId = StatusId(70);
    pub const BURST: StatusId = StatusId(71);
    pub const LOSE_STRENGTH: StatusId = StatusId(72);
    pub const LOSE_DEXTERITY: StatusId = StatusId(73);
    pub const DOUBLE_DAMAGE: StatusId = StatusId(74);
    pub const NO_BLOCK: StatusId = StatusId(75);
    pub const EQUILIBRIUM: StatusId = StatusId(76);
    pub const ENERGIZED: StatusId = StatusId(77);
    pub const ENERGY_DOWN: StatusId = StatusId(78);
    pub const DRAW: StatusId = StatusId(79);
    pub const DRAW_CARD: StatusId = StatusId(80);
    pub const NEXT_TURN_BLOCK: StatusId = StatusId(81);
    pub const WRATH_NEXT_TURN: StatusId = StatusId(82);
    pub const CANNOT_CHANGE_STANCE: StatusId = StatusId(83);
    pub const END_TURN_DEATH: StatusId = StatusId(84);
    pub const FREE_ATTACK_POWER: StatusId = StatusId(85);
    pub const NO_SKILLS_POWER: StatusId = StatusId(86);
    pub const DOPPELGANGER_DRAW: StatusId = StatusId(87);
    pub const DOPPELGANGER_ENERGY: StatusId = StatusId(88);

    // ==================================================================
    // Enemy powers (89-126)
    // ==================================================================
    pub const ARTIFACT: StatusId = StatusId(89);
    pub const BEAT_OF_DEATH: StatusId = StatusId(90);
    pub const THORNS: StatusId = StatusId(91);
    pub const RITUAL: StatusId = StatusId(92);
    pub const CURL_UP: StatusId = StatusId(93);
    pub const ENRAGE: StatusId = StatusId(94);
    pub const INTANGIBLE: StatusId = StatusId(95);
    pub const PLATED_ARMOR: StatusId = StatusId(96);
    pub const SHARP_HIDE: StatusId = StatusId(97);
    pub const MODE_SHIFT: StatusId = StatusId(98);
    pub const INVINCIBLE: StatusId = StatusId(99);
    pub const INVINCIBLE_DAMAGE_TAKEN: StatusId = StatusId(100);
    pub const MALLEABLE: StatusId = StatusId(101);
    pub const REACTIVE: StatusId = StatusId(102);
    pub const SLOW: StatusId = StatusId(103);
    pub const TIME_WARP: StatusId = StatusId(104);
    pub const TIME_WARP_ACTIVE: StatusId = StatusId(105);
    pub const SHIFTING: StatusId = StatusId(106);
    pub const ANGRY: StatusId = StatusId(107);
    pub const CURIOSITY: StatusId = StatusId(108);
    pub const GENERIC_STRENGTH_UP: StatusId = StatusId(109);
    pub const FADING: StatusId = StatusId(110);
    pub const EXPLOSIVE: StatusId = StatusId(111);
    pub const GROWTH: StatusId = StatusId(112);
    pub const SPORE_CLOUD: StatusId = StatusId(113);
    pub const REGROW: StatusId = StatusId(114);
    pub const REGENERATION: StatusId = StatusId(115);
    pub const THE_BOMB: StatusId = StatusId(116);
    pub const THE_BOMB_TURNS: StatusId = StatusId(117);
    pub const REBIRTH_PENDING: StatusId = StatusId(118);
    pub const SLEEP_TURNS: StatusId = StatusId(119);
    pub const PHASE: StatusId = StatusId(120);
    pub const THRESHOLD_REACHED: StatusId = StatusId(121);
    pub const SKILL_BURN: StatusId = StatusId(122);
    pub const FORCEFIELD: StatusId = StatusId(123);
    pub const FLIGHT: StatusId = StatusId(124);
    pub const BLUR: StatusId = StatusId(125);
    pub const LOCK_ON: StatusId = StatusId(126);

    // ==================================================================
    // Card/mechanic tracking (127-131)
    // ==================================================================
    pub const BLOCK_RETURN: StatusId = StatusId(127);
    pub const MARK: StatusId = StatusId(128);
    pub const EXPUNGER_HITS: StatusId = StatusId(129);
    pub const MANTRA_GAINED: StatusId = StatusId(130);
    pub const LIVE_FOREVER: StatusId = StatusId(131);

    // ==================================================================
    // Relic counters & flags (132-173)
    // ==================================================================
    pub const LANTERN_READY: StatusId = StatusId(132);
    pub const BAG_OF_PREP_DRAW: StatusId = StatusId(133);
    pub const PEN_NIB_COUNTER: StatusId = StatusId(134);
    pub const ORNAMENTAL_FAN_COUNTER: StatusId = StatusId(135);
    pub const KUNAI_COUNTER: StatusId = StatusId(136);
    pub const SHURIKEN_COUNTER: StatusId = StatusId(137);
    pub const NUNCHAKU_COUNTER: StatusId = StatusId(138);
    pub const LETTER_OPENER_COUNTER: StatusId = StatusId(139);
    pub const HAPPY_FLOWER_COUNTER: StatusId = StatusId(140);
    pub const INCENSE_BURNER_COUNTER: StatusId = StatusId(141);
    pub const HORN_CLEAT_COUNTER: StatusId = StatusId(142);
    pub const CAPTAINS_WHEEL_COUNTER: StatusId = StatusId(143);
    pub const STONE_CALENDAR_COUNTER: StatusId = StatusId(144);
    pub const STONE_CALENDAR_FIRED: StatusId = StatusId(145);
    pub const VELVET_CHOKER_COUNTER: StatusId = StatusId(146);
    pub const POCKETWATCH_COUNTER: StatusId = StatusId(147);
    pub const POCKETWATCH_FIRST_TURN: StatusId = StatusId(148);
    pub const VIOLET_LOTUS: StatusId = StatusId(149);
    pub const EMOTION_CHIP_READY: StatusId = StatusId(150);
    pub const CENTENNIAL_PUZZLE_READY: StatusId = StatusId(151);
    pub const ART_OF_WAR_READY: StatusId = StatusId(152);
    pub const SNECKO_EYE: StatusId = StatusId(153);
    pub const SLING_ELITE: StatusId = StatusId(154);
    pub const PRESERVED_INSECT_ELITE: StatusId = StatusId(155);
    pub const NEOWS_LAMENT_COUNTER: StatusId = StatusId(156);
    pub const DU_VU_DOLL_CURSES: StatusId = StatusId(157);
    pub const GIRYA_COUNTER: StatusId = StatusId(158);
    pub const RED_SKULL_ACTIVE: StatusId = StatusId(159);
    pub const OP_ATTACK: StatusId = StatusId(160);
    pub const OP_SKILL: StatusId = StatusId(161);
    pub const OP_POWER: StatusId = StatusId(162);
    pub const TURN_START_EXTRA_DRAW: StatusId = StatusId(163);
    pub const INK_BOTTLE_COUNTER: StatusId = StatusId(164);
    pub const INK_BOTTLE_DRAW: StatusId = StatusId(165);
    pub const MUMMIFIED_HAND_TRIGGER: StatusId = StatusId(166);
    pub const ENTER_DIVINITY: StatusId = StatusId(167);
    pub const INSERTER_COUNTER: StatusId = StatusId(168);
    pub const ORB_SLOTS: StatusId = StatusId(169);
    pub const FROZEN_CORE_TRIGGER: StatusId = StatusId(170);
    pub const MUTAGENIC_STRENGTH: StatusId = StatusId(171);
    pub const PANACHE_COUNT: StatusId = StatusId(172);
    pub const DEVA_FORM_ENERGY: StatusId = StatusId(173);

    // ==================================================================
    // Enemy AI tracking (174-220)
    // ==================================================================
    pub const ATTACK_COUNT: StatusId = StatusId(174);
    pub const BEAM_DMG: StatusId = StatusId(175);
    pub const BLOCK_AMT: StatusId = StatusId(176);
    pub const BLOOD_HIT_COUNT: StatusId = StatusId(177);
    pub const BUFF_COUNT: StatusId = StatusId(178);
    pub const CARD_COUNT: StatusId = StatusId(179);
    pub const CENTENNIAL_PUZZLE_DRAW: StatusId = StatusId(180);
    pub const COUNT: StatusId = StatusId(181);
    pub const DAMAGE_TAKEN_THIS_MODE: StatusId = StatusId(182);
    pub const DUPLICATION: StatusId = StatusId(183);
    pub const ECHO_DMG: StatusId = StatusId(184);
    pub const EMOTION_CHIP_TRIGGER: StatusId = StatusId(185);
    pub const FIERCE_BASH_DMG: StatusId = StatusId(186);
    pub const FIRE_TACKLE_DMG: StatusId = StatusId(187);
    pub const FIREBALL_DMG: StatusId = StatusId(188);
    pub const FIRST_MOVE: StatusId = StatusId(189);
    pub const FIRST_TURN: StatusId = StatusId(190);
    pub const FLAIL_DMG: StatusId = StatusId(191);
    pub const FORGE_AMT: StatusId = StatusId(192);
    pub const FORGE_TIMES: StatusId = StatusId(193);
    pub const GREMLIN_HORN_DRAW: StatusId = StatusId(194);
    pub const HEAD_SLAM_DMG: StatusId = StatusId(195);
    pub const INFERNO_DMG: StatusId = StatusId(196);
    pub const IS_FIRST_MOVE: StatusId = StatusId(197);
    pub const LIGHTNING_CHANNELED: StatusId = StatusId(198);
    pub const MOVE_COUNT: StatusId = StatusId(199);
    pub const NECRONOMICON_USED: StatusId = StatusId(200);
    pub const NUM_TURNS: StatusId = StatusId(201);
    pub const POTION_DRAW: StatusId = StatusId(202);
    pub const REGENERATE: StatusId = StatusId(203);
    pub const REVERB_DMG: StatusId = StatusId(204);
    pub const ROLL_DMG: StatusId = StatusId(205);
    pub const RUNIC_CUBE_DRAW: StatusId = StatusId(206);
    pub const SCYTHE_COOLDOWN: StatusId = StatusId(207);
    pub const SEAR_BURN_COUNT: StatusId = StatusId(208);
    pub const SKEWER_COUNT: StatusId = StatusId(209);
    pub const SLAP_DMG: StatusId = StatusId(210);
    pub const SLASH_DMG: StatusId = StatusId(211);
    pub const STAB_COUNT: StatusId = StatusId(212);
    pub const STARTING_DEATH_DMG: StatusId = StatusId(213);
    pub const STARTING_DMG: StatusId = StatusId(214);
    pub const STR_AMT: StatusId = StatusId(215);
    pub const SUNDIAL_COUNTER: StatusId = StatusId(216);
    pub const TURN_COUNT: StatusId = StatusId(217);
    pub const USED_HASTE: StatusId = StatusId(218);
    pub const USED_MEGA_DEBUFF: StatusId = StatusId(219);
    pub const WEAK: StatusId = StatusId(220);
    pub const MYSTIC_HEAL_USED: StatusId = StatusId(221);
    pub const HAS_GINGER: StatusId = StatusId(222);
    pub const HAS_TURNIP: StatusId = StatusId(223);
    pub const HAS_MARK_OF_BLOOM: StatusId = StatusId(224);
    pub const HAS_MAGIC_FLOWER: StatusId = StatusId(225);
    pub const LIZARD_TAIL_USED: StatusId = StatusId(226);
    pub const CHANNEL_DARK_START: StatusId = StatusId(227);
    pub const CHANNEL_LIGHTNING_START: StatusId = StatusId(228);
    pub const CHANNEL_PLASMA_START: StatusId = StatusId(229);
    pub const RING_OF_SERPENT_DRAW: StatusId = StatusId(230);
    pub const SLAVERS_COLLAR_ENERGY: StatusId = StatusId(231);
    pub const GAMBLING_CHIP_ACTIVE: StatusId = StatusId(232);
    pub const FORESIGHT: StatusId = StatusId(233);
    pub const DISCARDED_THIS_TURN: StatusId = StatusId(234);
    pub const PERSEVERANCE_BONUS: StatusId = StatusId(235);
    pub const WINDMILL_STRIKE_BONUS: StatusId = StatusId(236);
    pub const RAMPAGE_BONUS: StatusId = StatusId(237);
    pub const GLASS_KNIFE_PENALTY: StatusId = StatusId(238);
    pub const GENETIC_ALG_BONUS: StatusId = StatusId(239);
    pub const RITUAL_DAGGER_BONUS: StatusId = StatusId(240);

    /// Total number of defined status IDs (exclusive upper bound).
    pub const NUM_IDS: usize = 241;

    /// Array sizing constant (power of 2 for cache-friendly indexing).
    pub const MAX_STATUS_ID: usize = 256;
}

// =========================================================================
// Reverse lookup tables
// =========================================================================

/// StatusId -> canonical string name (for PyO3 bridge and debug output).
pub fn status_name(id: StatusId) -> &'static str {
    STATUS_NAMES.get(id.0 as usize).copied().unwrap_or("Unknown")
}

/// String name -> StatusId (for PyO3 bridge, test setup, deserialization).
pub fn status_id_from_name(name: &str) -> Option<StatusId> {
    STATUS_NAMES
        .iter()
        .position(|&n| n == name)
        .map(|i| StatusId(i as u16))
}

/// Reverse table indexed by StatusId.0. Must match sid:: constants exactly.
static STATUS_NAMES: &[&str] = &[
    // Core combat stats (0-4)
    "Strength",          // 0
    "Dexterity",         // 1
    "Focus",             // 2
    "Vigor",             // 3
    "Mantra",            // 4
    // Debuffs (5-14)
    "Vulnerable",        // 5
    "Weakened",          // 6
    "Frail",             // 7
    "Poison",            // 8
    "Constricted",       // 9
    "Entangled",         // 10
    "Hex",               // 11
    "Confusion",         // 12
    "NoDraw",            // 13
    "DrawReduction",     // 14
    // Ironclad powers (15-29)
    "Barricade",         // 15
    "DemonForm",         // 16
    "Corruption",        // 17
    "DarkEmbrace",       // 18
    "FeelNoPain",        // 19
    "Brutality",         // 20
    "Combust",           // 21
    "Evolve",            // 22
    "FireBreathing",     // 23
    "Juggernaut",        // 24
    "Metallicize",       // 25
    "Rupture",           // 26
    "Berserk",           // 27
    "Rage",              // 28
    "FlameBarrier",      // 29
    // Silent powers (30-37)
    "AfterImage",        // 30
    "ThousandCuts",      // 31
    "NoxiousFumes",      // 32
    "InfiniteBlades",    // 33
    "Envenom",           // 34
    "Accuracy",          // 35
    "ToolsOfTheTrade",   // 36
    "RetainCards",       // 37  (alias: WellLaidPlans)
    // Watcher powers (38-51)
    "BattleHymn",        // 38
    "Devotion",          // 39
    "DevaForm",          // 40
    "Establishment",     // 41
    "Fasting",           // 42
    "LikeWater",         // 43
    "MasterReality",     // 44
    "MentalFortress",    // 45
    "Nirvana",           // 46
    "Omega",             // 47
    "Rushdown",          // 48
    "Study",             // 49
    "WaveOfTheHand",     // 50
    "WraithForm",        // 51
    // Defect powers (52-61)
    "Buffer",            // 52
    "CreativeAI",        // 53
    "EchoForm",          // 54
    "Electro",           // 55
    "Electrodynamics",   // 56
    "Heatsink",          // 57
    "HelloWorld",        // 58
    "Loop",              // 59
    "Storm",             // 60
    "StaticDischarge",   // 61
    // Colorless / universal powers (62-65)
    "Panache",           // 62
    "SadisticNature",    // 63
    "Magnetism",         // 64
    "Mayhem",            // 65
    // Temporary / turn-scoped effects (66-88)
    "TempStrength",      // 66
    "TempStrengthLoss",  // 67
    "NextAttackFree",    // 68
    "BulletTime",        // 69
    "DoubleTap",         // 70
    "Burst",             // 71
    "LoseStrength",      // 72
    "LoseDexterity",     // 73
    "DoubleDamage",      // 74
    "NoBlock",           // 75
    "Equilibrium",       // 76
    "Energized",         // 77
    "EnergyDown",        // 78
    "Draw",              // 79
    "DrawCard",          // 80
    "NextTurnBlock",     // 81
    "WrathNextTurn",     // 82
    "CannotChangeStance", // 83
    "EndTurnDeath",      // 84
    "FreeAttackPower",   // 85
    "NoSkillsPower",     // 86
    "DoppelgangerDraw",  // 87
    "DoppelgangerEnergy", // 88
    // Enemy powers (89-126)
    "Artifact",          // 89
    "BeatOfDeath",       // 90
    "Thorns",            // 91
    "Ritual",            // 92
    "CurlUp",            // 93
    "Enrage",            // 94
    "Intangible",        // 95
    "PlatedArmor",       // 96
    "SharpHide",         // 97
    "ModeShift",         // 98
    "Invincible",        // 99
    "InvincibleDamageTaken", // 100
    "Malleable",         // 101
    "Reactive",          // 102
    "Slow",              // 103
    "TimeWarp",          // 104
    "TimeWarpActive",    // 105
    "Shifting",          // 106
    "Angry",             // 107
    "Curiosity",         // 108
    "GenericStrengthUp", // 109
    "Fading",            // 110
    "Explosive",         // 111
    "Growth",            // 112
    "SporeCloud",        // 113
    "Regrow",            // 114
    "Regeneration",      // 115
    "TheBomb",           // 116
    "TheBombTurns",      // 117
    "RebirthPending",    // 118
    "SleepTurns",        // 119
    "Phase",             // 120
    "ThresholdReached",  // 121
    "SkillBurn",         // 122
    "Forcefield",        // 123
    "Flight",            // 124
    "Blur",              // 125
    "Lock-On",           // 126
    // Card/mechanic tracking (127-131)
    "BlockReturn",       // 127
    "Mark",              // 128
    "ExpungerHits",      // 129
    "MantraGained",      // 130
    "LiveForever",       // 131
    // Relic counters & flags (132-173)
    "LanternReady",      // 132
    "BagOfPrepDraw",     // 133
    "PenNibCounter",     // 134
    "OrnamentalFanCounter", // 135
    "KunaiCounter",      // 136
    "ShurikenCounter",   // 137
    "NunchakuCounter",   // 138
    "LetterOpenerCounter", // 139
    "HappyFlowerCounter", // 140
    "IncenseBurnerCounter", // 141
    "HornCleatCounter",  // 142
    "CaptainsWheelCounter", // 143
    "StoneCalendarCounter", // 144
    "StoneCalendarFired", // 145
    "VelvetChokerCounter", // 146
    "PocketwatchCounter", // 147
    "PocketwatchFirstTurn", // 148
    "VioletLotus",       // 149
    "EmotionChipReady",  // 150
    "CentennialPuzzleReady", // 151
    "ArtOfWarReady",     // 152
    "SneckoEye",         // 153
    "SlingElite",        // 154
    "PreservedInsectElite", // 155
    "NeowsLamentCounter", // 156
    "DuVuDollCurses",    // 157
    "GiryaCounter",      // 158
    "RedSkullActive",    // 159
    "OPAttack",          // 160
    "OPSkill",           // 161
    "OPPower",           // 162
    "TurnStartExtraDraw", // 163
    "InkBottleCounter",  // 164
    "InkBottleDraw",     // 165
    "MummifiedHandTrigger", // 166
    "EnterDivinity",     // 167
    "InserterCounter",   // 168
    "OrbSlots",          // 169
    "FrozenCoreTrigger", // 170
    "MutagenicStrength", // 171
    "PanacheCount",      // 172
    "DevaFormEnergy",    // 173
    // Enemy AI tracking (174-220)
    "AttackCount",       // 174
    "BeamDmg",           // 175
    "BlockAmt",          // 176
    "BloodHitCount",     // 177
    "BuffCount",         // 178
    "CardCount",         // 179
    "CentennialPuzzleDraw", // 180
    "Count",             // 181
    "DamageTakenThisMode", // 182
    "Duplication",       // 183
    "EchoDmg",           // 184
    "EmotionChipTrigger", // 185
    "FierceBashDmg",     // 186
    "FireTackleDmg",     // 187
    "FireballDmg",       // 188
    "FirstMove",         // 189
    "FirstTurn",         // 190
    "FlailDmg",          // 191
    "ForgeAmt",          // 192
    "ForgeTimes",        // 193
    "GremlinHornDraw",   // 194
    "HeadSlamDmg",       // 195
    "InfernoDmg",        // 196
    "IsFirstMove",       // 197
    "LightningChanneled", // 198
    "MoveCount",         // 199
    "NecronomiconUsed",  // 200
    "NumTurns",          // 201
    "PotionDraw",        // 202
    "Regenerate",        // 203
    "ReverbDmg",         // 204
    "RollDmg",           // 205
    "RunicCubeDraw",     // 206
    "ScytheCooldown",    // 207
    "SearBurnCount",     // 208
    "SkewerCount",       // 209
    "SlapDmg",           // 210
    "SlashDmg",          // 211
    "StabCount",         // 212
    "StartingDeathDmg",  // 213
    "StartingDmg",       // 214
    "StrAmt",            // 215
    "SundialCounter",    // 216
    "TurnCount",         // 217
    "UsedHaste",         // 218
    "UsedMegaDebuff",    // 219
    "Weak",              // 220
    "MysticHealUsed",    // 221
    "HasGinger",         // 222
    "HasTurnip",         // 223
    "HasMarkOfBloom",    // 224
    "HasMagicFlower",    // 225
    "LizardTailUsed",   // 226
    "ChannelDarkStart",  // 227
    "ChannelLightningStart", // 228
    "ChannelPlasmaStart", // 229
    "RingOfSerpentDraw", // 230
    "SlaversCollarEnergy", // 231
    "GamblingChipActive", // 232
    "Foresight",          // 233
    "DiscardedThisTurn",  // 234
    "PerseveranceBonus",  // 235
    "WindmillStrikeBonus", // 236
    "RampageBonus",       // 237
    "GlassKnifePenalty",  // 238
    "GeneticAlgBonus",    // 239
    "RitualDaggerBonus",  // 240
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status_ids::sid;

    #[test]
    fn test_status_name_roundtrip() {
        assert_eq!(status_name(sid::STRENGTH), "Strength");
        assert_eq!(status_name(sid::VULNERABLE), "Vulnerable");
        assert_eq!(status_name(sid::LOCK_ON), "Lock-On");
        assert_eq!(status_name(sid::DEVA_FORM_ENERGY), "DevaFormEnergy");
    }

    #[test]
    fn test_status_id_from_name_roundtrip() {
        assert_eq!(status_id_from_name("Strength"), Some(sid::STRENGTH));
        assert_eq!(status_id_from_name("Vulnerable"), Some(sid::VULNERABLE));
        assert_eq!(status_id_from_name("Lock-On"), Some(sid::LOCK_ON));
        assert_eq!(status_id_from_name("Nonexistent"), None);
    }

    #[test]
    fn test_names_table_length() {
        assert_eq!(STATUS_NAMES.len(), sid::NUM_IDS);
    }

    #[test]
    fn test_well_laid_plans_alias() {
        // WELL_LAID_PLANS and RETAIN_CARDS share the same StatusId
        assert_eq!(sid::WELL_LAID_PLANS, sid::RETAIN_CARDS);
    }

    #[test]
    fn test_unknown_status_name() {
        assert_eq!(status_name(StatusId(999)), "Unknown");
    }
}
