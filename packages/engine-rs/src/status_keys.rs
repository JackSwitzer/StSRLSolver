//! Canonical status key constants for the Slay the Spire Rust engine.
//!
//! ALL status reads/writes should use these constants instead of raw strings.
//! This prevents silent key mismatches (e.g., "Like Water" vs "LikeWater").
//! The compiler catches typos — raw strings don't.
//!
//! Naming convention: PascalCase, no spaces. Each constant's VALUE is the
//! canonical key stored in the HashMap. We do NOT copy Java's inconsistent
//! naming — we use our own consistent format.

#![allow(dead_code)] // Many keys are defined before consumption is wired

/// Status key constants. Import as `use crate::status_keys::sk;`
pub mod sk {
    // ======================================================================
    // Core combat stats
    // ======================================================================
    pub const STRENGTH: &str = "Strength";
    pub const DEXTERITY: &str = "Dexterity";
    pub const FOCUS: &str = "Focus";
    pub const VIGOR: &str = "Vigor";
    pub const MANTRA: &str = "Mantra";

    // ======================================================================
    // Debuffs (applied to player or enemies)
    // ======================================================================
    pub const VULNERABLE: &str = "Vulnerable";
    pub const WEAKENED: &str = "Weakened";
    pub const FRAIL: &str = "Frail";
    pub const POISON: &str = "Poison";
    pub const CONSTRICTED: &str = "Constricted";
    pub const ENTANGLED: &str = "Entangled";
    pub const HEX: &str = "Hex";
    pub const CONFUSION: &str = "Confusion";
    pub const NO_DRAW: &str = "NoDraw";
    pub const DRAW_REDUCTION: &str = "DrawReduction";

    // ======================================================================
    // Ironclad powers
    // ======================================================================
    pub const BARRICADE: &str = "Barricade";
    pub const DEMON_FORM: &str = "DemonForm";
    pub const CORRUPTION: &str = "Corruption";
    pub const DARK_EMBRACE: &str = "DarkEmbrace";
    pub const FEEL_NO_PAIN: &str = "FeelNoPain";
    pub const BRUTALITY: &str = "Brutality";
    pub const COMBUST: &str = "Combust";
    pub const EVOLVE: &str = "Evolve";
    pub const FIRE_BREATHING: &str = "FireBreathing";
    pub const JUGGERNAUT: &str = "Juggernaut";
    pub const METALLICIZE: &str = "Metallicize";
    pub const RUPTURE: &str = "Rupture";
    pub const BERSERK: &str = "Berserk";
    pub const RAGE: &str = "Rage";
    pub const FLAME_BARRIER: &str = "FlameBarrier";

    // ======================================================================
    // Silent powers
    // ======================================================================
    pub const AFTER_IMAGE: &str = "AfterImage";
    pub const THOUSAND_CUTS: &str = "ThousandCuts";
    pub const NOXIOUS_FUMES: &str = "NoxiousFumes";
    pub const INFINITE_BLADES: &str = "InfiniteBlades";
    pub const ENVENOM: &str = "Envenom";
    pub const ACCURACY: &str = "Accuracy";
    pub const TOOLS_OF_THE_TRADE: &str = "ToolsOfTheTrade";
    pub const RETAIN_CARDS: &str = "RetainCards";
    pub const WELL_LAID_PLANS: &str = "RetainCards"; // alias — same power

    // ======================================================================
    // Watcher powers
    // ======================================================================
    pub const BATTLE_HYMN: &str = "BattleHymn";
    pub const DEVOTION: &str = "Devotion";
    pub const DEVA_FORM: &str = "DevaForm";
    pub const ESTABLISHMENT: &str = "Establishment";
    pub const FASTING: &str = "Fasting";
    pub const LIKE_WATER: &str = "LikeWater";
    pub const MASTER_REALITY: &str = "MasterReality";
    pub const MENTAL_FORTRESS: &str = "MentalFortress";
    pub const NIRVANA: &str = "Nirvana";
    pub const OMEGA: &str = "Omega";
    pub const RUSHDOWN: &str = "Rushdown";
    pub const STUDY: &str = "Study";
    pub const WAVE_OF_THE_HAND: &str = "WaveOfTheHand";
    pub const WRAITH_FORM: &str = "WraithForm";

    // ======================================================================
    // Defect powers
    // ======================================================================
    pub const BUFFER: &str = "Buffer";
    pub const CREATIVE_AI: &str = "CreativeAI";
    pub const ECHO_FORM: &str = "EchoForm";
    pub const ELECTRO: &str = "Electro";
    pub const ELECTRODYNAMICS: &str = "Electrodynamics";
    pub const HEATSINK: &str = "Heatsink";
    pub const HELLO_WORLD: &str = "HelloWorld";
    pub const LOOP: &str = "Loop";
    pub const STORM: &str = "Storm";
    pub const STATIC_DISCHARGE: &str = "StaticDischarge";

    // ======================================================================
    // Colorless / universal powers
    // ======================================================================
    pub const PANACHE: &str = "Panache";
    pub const SADISTIC: &str = "SadisticNature";
    pub const MAGNETISM: &str = "Magnetism";
    pub const MAYHEM: &str = "Mayhem";

    // ======================================================================
    // Temporary / turn-scoped effects
    // ======================================================================
    pub const TEMP_STRENGTH: &str = "TempStrength";
    pub const TEMP_STRENGTH_LOSS: &str = "TempStrengthLoss";
    pub const NEXT_ATTACK_FREE: &str = "NextAttackFree";
    pub const BULLET_TIME: &str = "BulletTime";
    pub const DOUBLE_TAP: &str = "DoubleTap";
    pub const BURST: &str = "Burst";
    pub const LOSE_STRENGTH: &str = "LoseStrength";
    pub const LOSE_DEXTERITY: &str = "LoseDexterity";
    pub const DOUBLE_DAMAGE: &str = "DoubleDamage";
    pub const NO_BLOCK: &str = "NoBlock";
    pub const EQUILIBRIUM: &str = "Equilibrium";
    pub const ENERGIZED: &str = "Energized";
    pub const ENERGY_DOWN: &str = "EnergyDown";
    pub const DRAW: &str = "Draw";
    pub const DRAW_CARD: &str = "DrawCard";
    pub const NEXT_TURN_BLOCK: &str = "NextTurnBlock";
    pub const WRATH_NEXT_TURN: &str = "WrathNextTurn";
    pub const CANNOT_CHANGE_STANCE: &str = "CannotChangeStance";
    pub const END_TURN_DEATH: &str = "EndTurnDeath";
    pub const FREE_ATTACK_POWER: &str = "FreeAttackPower";
    pub const NO_SKILLS_POWER: &str = "NoSkillsPower";
    pub const DOPPELGANGER_DRAW: &str = "DoppelgangerDraw";
    pub const DOPPELGANGER_ENERGY: &str = "DoppelgangerEnergy";

    // ======================================================================
    // Enemy powers
    // ======================================================================
    pub const ARTIFACT: &str = "Artifact";
    pub const BEAT_OF_DEATH: &str = "BeatOfDeath";
    pub const THORNS: &str = "Thorns";
    pub const RITUAL: &str = "Ritual";
    pub const CURL_UP: &str = "CurlUp";
    pub const ENRAGE: &str = "Enrage";
    pub const INTANGIBLE: &str = "Intangible";
    pub const PLATED_ARMOR: &str = "PlatedArmor";
    pub const SHARP_HIDE: &str = "SharpHide";
    pub const MODE_SHIFT: &str = "ModeShift";
    pub const INVINCIBLE: &str = "Invincible";
    pub const INVINCIBLE_DAMAGE_TAKEN: &str = "InvincibleDamageTaken";
    pub const MALLEABLE: &str = "Malleable";
    pub const REACTIVE: &str = "Reactive";
    pub const SLOW: &str = "Slow";
    pub const TIME_WARP: &str = "TimeWarp";
    pub const TIME_WARP_ACTIVE: &str = "TimeWarpActive";
    pub const SHIFTING: &str = "Shifting";
    pub const ANGRY: &str = "Angry";
    pub const CURIOSITY: &str = "Curiosity";
    pub const GENERIC_STRENGTH_UP: &str = "GenericStrengthUp";
    pub const FADING: &str = "Fading";
    pub const EXPLOSIVE: &str = "Explosive";
    pub const GROWTH: &str = "Growth";
    pub const SPORE_CLOUD: &str = "SporeCloud";
    pub const REGROW: &str = "Regrow";
    pub const REGENERATION: &str = "Regeneration";
    pub const THE_BOMB: &str = "TheBomb";
    pub const THE_BOMB_TURNS: &str = "TheBombTurns";
    pub const REBIRTH_PENDING: &str = "RebirthPending";
    pub const SLEEP_TURNS: &str = "SleepTurns";
    pub const PHASE: &str = "Phase";
    pub const THRESHOLD_REACHED: &str = "ThresholdReached";
    pub const SKILL_BURN: &str = "SkillBurn";
    pub const FORCEFIELD: &str = "Forcefield";
    pub const FLIGHT: &str = "Flight";
    pub const BLUR: &str = "Blur";
    pub const LOCK_ON: &str = "Lock-On";

    // ======================================================================
    // Card/mechanic tracking
    // ======================================================================
    pub const BLOCK_RETURN: &str = "BlockReturn";
    pub const MARK: &str = "Mark";
    pub const EXPUNGER_HITS: &str = "ExpungerHits";
    pub const MANTRA_GAINED: &str = "MantraGained";
    pub const LIVE_FOREVER: &str = "LiveForever";

    // ======================================================================
    // Relic counters & flags
    // ======================================================================
    pub const LANTERN_READY: &str = "LanternReady";
    pub const BAG_OF_PREP_DRAW: &str = "BagOfPrepDraw";
    pub const PEN_NIB_COUNTER: &str = "PenNibCounter";
    pub const ORNAMENTAL_FAN_COUNTER: &str = "OrnamentalFanCounter";
    pub const KUNAI_COUNTER: &str = "KunaiCounter";
    pub const SHURIKEN_COUNTER: &str = "ShurikenCounter";
    pub const NUNCHAKU_COUNTER: &str = "NunchakuCounter";
    pub const LETTER_OPENER_COUNTER: &str = "LetterOpenerCounter";
    pub const HAPPY_FLOWER_COUNTER: &str = "HappyFlowerCounter";
    pub const INCENSE_BURNER_COUNTER: &str = "IncenseBurnerCounter";
    pub const HORN_CLEAT_COUNTER: &str = "HornCleatCounter";
    pub const CAPTAINS_WHEEL_COUNTER: &str = "CaptainsWheelCounter";
    pub const STONE_CALENDAR_COUNTER: &str = "StoneCalendarCounter";
    pub const STONE_CALENDAR_FIRED: &str = "StoneCalendarFired";
    pub const VELVET_CHOKER_COUNTER: &str = "VelvetChokerCounter";
    pub const POCKETWATCH_COUNTER: &str = "PocketwatchCounter";
    pub const POCKETWATCH_FIRST_TURN: &str = "PocketwatchFirstTurn";
    pub const VIOLET_LOTUS: &str = "VioletLotus";
    pub const EMOTION_CHIP_READY: &str = "EmotionChipReady";
    pub const CENTENNIAL_PUZZLE_READY: &str = "CentennialPuzzleReady";
    pub const ART_OF_WAR_READY: &str = "ArtOfWarReady";
    pub const SNECKO_EYE: &str = "SneckoEye";
    pub const SLING_ELITE: &str = "SlingElite";
    pub const PRESERVED_INSECT_ELITE: &str = "PreservedInsectElite";
    pub const NEOWS_LAMENT_COUNTER: &str = "NeowsLamentCounter";
    pub const DU_VU_DOLL_CURSES: &str = "DuVuDollCurses";
    pub const GIRYA_COUNTER: &str = "GiryaCounter";
    pub const RED_SKULL_ACTIVE: &str = "RedSkullActive";
    pub const OP_ATTACK: &str = "OPAttack";
    pub const OP_SKILL: &str = "OPSkill";
    pub const OP_POWER: &str = "OPPower";
    pub const TURN_START_EXTRA_DRAW: &str = "TurnStartExtraDraw";
    pub const INK_BOTTLE_COUNTER: &str = "InkBottleCounter";
    pub const INK_BOTTLE_DRAW: &str = "InkBottleDraw";
    pub const MUMMIFIED_HAND_TRIGGER: &str = "MummifiedHandTrigger";
    pub const ENTER_DIVINITY: &str = "EnterDivinity";
    pub const INSERTER_COUNTER: &str = "InserterCounter";
    pub const ORB_SLOTS: &str = "OrbSlots";
    pub const FROZEN_CORE_TRIGGER: &str = "FrozenCoreTrigger";
    pub const MUTAGENIC_STRENGTH: &str = "MutagenicStrength";
    pub const PANACHE_COUNT: &str = "PanacheCount";
    pub const DEVA_FORM_ENERGY: &str = "DevaFormEnergy";
}
