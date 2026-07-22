//! Java-derived enemy construction, intent selection, and turn behavior.
//!
//! Every supported monster consumes the native dungeon RNG streams in Java
//! order, including deterministic move patterns whose ignored roll is still
//! observable in `aiRng`.

use crate::combat_types::{fx, mfx, Intent};
use crate::state::EnemyCombatState;
use crate::status_ids::sid;

pub mod act1;
pub mod act2;
pub mod act3;
pub mod act4;

/// Canonical IDs from each Java monster's `ID` constant.
///
/// This is the 68-row monster ledger minus `HexaghostBody` and
/// `HexaghostOrb`: both are visual helpers and neither is a targetable
/// `AbstractMonster`.
pub const CANONICAL_ENEMIES: &[(&str, &str)] = &[
    ("AcidSlime_L", "Acid Slime (L)"),
    ("AcidSlime_M", "Acid Slime (M)"),
    ("AcidSlime_S", "Acid Slime (S)"),
    ("Apology Slime", "Apology Slime"),
    ("AwakenedOne", "Awakened One"),
    ("BanditBear", "Bear"),
    ("BanditChild", "Pointy"),
    ("BanditLeader", "Romeo"),
    ("BookOfStabbing", "Book of Stabbing"),
    ("BronzeAutomaton", "Bronze Automaton"),
    ("BronzeOrb", "Bronze Orb"),
    ("Byrd", "Byrd"),
    ("Centurion", "Centurion"),
    ("Champ", "The Champ"),
    ("Chosen", "Chosen"),
    ("CorruptHeart", "Corrupt Heart"),
    ("Cultist", "Cultist"),
    ("Dagger", "Dagger"),
    ("Darkling", "Darkling"),
    ("Deca", "Deca"),
    ("Donu", "Donu"),
    ("Exploder", "Exploder"),
    ("FungiBeast", "Fungi Beast"),
    ("FuzzyLouseDefensive", "Green Louse"),
    ("FuzzyLouseNormal", "Red Louse"),
    ("GiantHead", "Giant Head"),
    ("GremlinFat", "Mad Gremlin"),
    ("GremlinLeader", "Gremlin Leader"),
    ("GremlinNob", "Gremlin Nob"),
    ("GremlinThief", "Sneaky Gremlin"),
    ("GremlinTsundere", "Shield Gremlin"),
    ("GremlinWarrior", "Angry Gremlin"),
    ("GremlinWizard", "Gremlin Wizard"),
    ("Healer", "Mystic"),
    ("Hexaghost", "Hexaghost"),
    ("JawWorm", "Jaw Worm"),
    ("Lagavulin", "Lagavulin"),
    ("Looter", "Looter"),
    ("Maw", "The Maw"),
    ("Mugger", "Mugger"),
    ("Nemesis", "Nemesis"),
    ("Orb Walker", "Orb Walker"),
    ("Reptomancer", "Reptomancer"),
    ("Repulsor", "Repulsor"),
    ("Sentry", "Sentry"),
    ("Serpent", "Spire Growth"),
    ("Shelled Parasite", "Shelled Parasite"),
    ("SlaverBlue", "Blue Slaver"),
    ("SlaverBoss", "Taskmaster"),
    ("SlaverRed", "Red Slaver"),
    ("SlimeBoss", "Slime Boss"),
    ("SnakePlant", "Snake Plant"),
    ("Snecko", "Snecko"),
    ("SphericGuardian", "Spheric Guardian"),
    ("SpikeSlime_L", "Spike Slime (L)"),
    ("SpikeSlime_M", "Spike Slime (M)"),
    ("SpikeSlime_S", "Spike Slime (S)"),
    ("Spiker", "Spiker"),
    ("SpireShield", "Spire Shield"),
    ("SpireSpear", "Spire Spear"),
    ("TheCollector", "The Collector"),
    ("TheGuardian", "The Guardian"),
    ("TimeEater", "Time Eater"),
    ("TorchHead", "Torch Head"),
    ("Transient", "Transient"),
    ("WrithingMass", "Writhing Mass"),
];

pub fn known_enemy_ids() -> &'static [(&'static str, &'static str)] {
    CANONICAL_ENEMIES
}

pub const ENEMY_ID_ALIASES: &[(&str, &str)] = &[
    ("RedLouse", "FuzzyLouseNormal"),
    ("GreenLouse", "FuzzyLouseDefensive"),
    ("BlueSlaver", "SlaverBlue"),
    ("RedSlaver", "SlaverRed"),
    ("GremlinSneaky", "GremlinTsundere"),
    ("Gremlin Nob", "GremlinNob"),
    ("ApologySlime", "Apology Slime"),
    ("ShelledParasite", "Shelled Parasite"),
    ("Mystic", "Healer"),
    ("Book of Stabbing", "BookOfStabbing"),
    ("Gremlin Leader", "GremlinLeader"),
    ("TaskMaster", "SlaverBoss"),
    ("Taskmaster", "SlaverBoss"),
    ("Spheric Guardian", "SphericGuardian"),
    ("Bear", "BanditBear"),
    ("BanditPointy", "BanditChild"),
    ("Pointy", "BanditChild"),
    ("Bronze Automaton", "BronzeAutomaton"),
    ("Bronze Orb", "BronzeOrb"),
    ("Torch Head", "TorchHead"),
    ("TheChamp", "Champ"),
    ("Collector", "TheCollector"),
    ("OrbWalker", "Orb Walker"),
    ("Writhing Mass", "WrithingMass"),
    ("SpireGrowth", "Serpent"),
    ("Spire Growth", "Serpent"),
    ("Giant Head", "GiantHead"),
    ("SnakeDagger", "Dagger"),
    ("Snake Dagger", "Dagger"),
    ("Awakened One", "AwakenedOne"),
    ("Time Eater", "TimeEater"),
    ("Spire Shield", "SpireShield"),
    ("Spire Spear", "SpireSpear"),
    ("Corrupt Heart", "CorruptHeart"),
];

fn resolve_enemy_alias(enemy_id: &str) -> Option<&'static str> {
    ENEMY_ID_ALIASES
        .iter()
        .find_map(|(alias, canonical)| (*alias == enemy_id).then_some(*canonical))
}

/// Resolve a canonical Java enemy ID or one of the factory's compatibility aliases.
pub fn canonical_enemy_id(enemy_id: &str) -> Option<&'static str> {
    CANONICAL_ENEMIES
        .iter()
        .find_map(|(canonical, _)| (*canonical == enemy_id).then_some(*canonical))
        .or_else(|| resolve_enemy_alias(enemy_id))
}

/// Deterministic libGDX draws performed synchronously by Java monster
/// constructors. Values are presentation-only, but their shared raw RNG state
/// can affect later gameplay-significant ambient draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ConstructorAmbientDraw {
    UnitFloat,
    FloatRange { start: f32, end: f32 },
}

const BASE_CONSTRUCTOR_DRAWS: &[ConstructorAmbientDraw] = &[ConstructorAmbientDraw::UnitFloat];
const BASE_AND_UNIT_FLOAT_CONSTRUCTOR_DRAWS: &[ConstructorAmbientDraw] = &[
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::UnitFloat,
];
const FUNGI_CONSTRUCTOR_DRAWS: &[ConstructorAmbientDraw] = &[
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::FloatRange {
        start: 0.7,
        end: 1.0,
    },
];
const DARKLING_CONSTRUCTOR_DRAWS: &[ConstructorAmbientDraw] = &[
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::FloatRange {
        start: 0.75,
        end: 1.0,
    },
];
const HEXAGHOST_CONSTRUCTOR_DRAWS: &[ConstructorAmbientDraw] = &[
    ConstructorAmbientDraw::UnitFloat,
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
    ConstructorAmbientDraw::FloatRange {
        start: -10.0,
        end: 10.0,
    },
];

pub(crate) fn constructor_ambient_sequence(
    enemy_id: &str,
) -> Option<&'static [ConstructorAmbientDraw]> {
    let canonical = canonical_enemy_id(enemy_id)?;
    Some(match canonical {
        // AbstractMonster constructs BobEffect before the subclass constructor;
        // BobEffect's timer initializer consumes one MathUtils float draw.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java:110
        // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/BobEffect.java:18
        // These subclasses add no synchronous constructor draw of their own.
        "BronzeOrb" | "CorruptHeart" | "TheGuardian" => BASE_CONSTRUCTOR_DRAWS,
        "FungiBeast" => FUNGI_CONSTRUCTOR_DRAWS,
        "Darkling" => DARKLING_CONSTRUCTOR_DRAWS,
        // Six HexaghostOrb constructors each choose x then y offsets.
        // Java: Hexaghost.java:96, HexaghostOrb.java:40.
        "Hexaghost" => HEXAGHOST_CONSTRUCTOR_DRAWS,
        _ => BASE_AND_UNIT_FLOAT_CONSTRUCTOR_DRAWS,
    })
}

pub(crate) fn create_enemy_with_ambient(
    enemy_id: &str,
    hp: i32,
    max_hp: i32,
    ambient_rng: &mut crate::seed::AmbientMathRng,
) -> EnemyCombatState {
    let sequence = constructor_ambient_sequence(enemy_id)
        .unwrap_or_else(|| panic!("unknown enemy id: {enemy_id}"));
    for draw in sequence {
        match *draw {
            ConstructorAmbientDraw::UnitFloat => {
                let _ = ambient_rng.random_f32();
            }
            ConstructorAmbientDraw::FloatRange { start, end } => {
                let _ = ambient_rng.random_f32_range(start, end);
            }
        }
    }
    create_enemy(enemy_id, hp, max_hp)
}

pub fn gameplay_def(enemy_id: &str) -> Option<&'static crate::gameplay::GameplayDef> {
    canonical_enemy_id(enemy_id).and_then(|id| crate::gameplay::global_registry().enemy(id))
}

pub fn gameplay_export_defs() -> Vec<crate::gameplay::GameplayDef> {
    known_enemy_ids()
        .iter()
        .map(|(id, name)| crate::gameplay::GameplayDef {
            domain: crate::gameplay::GameplayDomain::Enemy,
            id: (*id).to_string(),
            name: (*name).to_string(),
            tags: vec!["enemy".to_string()],
            schema: crate::gameplay::GameplaySchema::Enemy(crate::gameplay::EnemySchema {
                move_source: "enemies::roll_next_move".to_string(),
            }),
            handlers: Vec::new(),
            state_fields: Vec::new(),
            has_complex_hook: false,
        })
        .collect()
}

pub mod move_ids {
    // =====================================================================
    // Act 1 — Exordium
    // =====================================================================

    // Jaw Worm
    pub const JW_CHOMP: i32 = 1;
    pub const JW_BELLOW: i32 = 2;
    pub const JW_THRASH: i32 = 3;

    // Cultist
    pub const CULT_DARK_STRIKE: i32 = 1;
    pub const CULT_INCANTATION: i32 = 3;

    // Fungi Beast
    pub const FB_BITE: i32 = 1;
    pub const FB_GROW: i32 = 2;

    // Louse (Red/Green)
    pub const LOUSE_BITE: i32 = 3;
    pub const LOUSE_GROW: i32 = 4;
    pub const LOUSE_SPIT_WEB: i32 = 4;

    // Blue Slaver
    pub const BS_STAB: i32 = 1;
    pub const BS_RAKE: i32 = 4;

    // Red Slaver
    pub const RS_STAB: i32 = 1;
    pub const RS_ENTANGLE: i32 = 2;
    pub const RS_SCRAPE: i32 = 3;

    // Acid Slime S/M/L
    pub const AS_S_TACKLE: i32 = 1;
    pub const AS_S_LICK: i32 = 2;
    pub const AS_CORROSIVE_SPIT: i32 = 1;
    pub const AS_TACKLE: i32 = 2;
    pub const AS_LICK: i32 = 4;
    pub const AS_SPLIT: i32 = 3;

    // Spike Slime S/M/L
    pub const SS_TACKLE: i32 = 1;
    pub const SS_LICK: i32 = 4; // Frail
    pub const SS_SPLIT: i32 = 3;

    // Looter
    pub const LOOTER_MUG: i32 = 1;
    pub const LOOTER_SMOKE_BOMB: i32 = 2;
    pub const LOOTER_ESCAPE: i32 = 3;
    pub const LOOTER_LUNGE: i32 = 4;

    // Gremlin (Fat/Thief/Warrior/Wizard/Tsundere)
    pub const GREMLIN_ATTACK: i32 = 1;
    pub const GREMLIN_PROTECT: i32 = 2;
    pub const GREMLIN_FAT_SMASH: i32 = 2;
    pub const GREMLIN_ESCAPE: i32 = 99;
    pub const GREMLIN_TSUNDERE_PROTECT: i32 = 1;
    pub const GREMLIN_TSUNDERE_BASH: i32 = 2;

    // Gremlin Nob
    // Java: GremlinNob.java BULL_RUSH=1, SKULL_BASH=2, BELLOW=3.
    pub const NOB_RUSH: i32 = 1;
    pub const NOB_SKULL_BASH: i32 = 2;
    pub const NOB_BELLOW: i32 = 3;

    // Lagavulin
    pub const LAGA_SIPHON: i32 = 1;
    pub const LAGA_ATTACK: i32 = 3;
    pub const LAGA_STUN: i32 = 4;
    pub const LAGA_SLEEP: i32 = 5;
    pub const LAGA_OPEN_NATURAL: i32 = 6;

    // Sentry
    pub const SENTRY_BOLT: i32 = 3;
    pub const SENTRY_BEAM: i32 = 4;

    // The Guardian
    pub const GUARD_CHARGING_UP: i32 = 6;
    pub const GUARD_CLOSE_UP: i32 = 1;
    pub const GUARD_FIERCE_BASH: i32 = 2;
    pub const GUARD_ROLL_ATTACK: i32 = 3;
    pub const GUARD_TWIN_SLAM: i32 = 4;
    pub const GUARD_WHIRLWIND: i32 = 5;
    pub const GUARD_VENT_STEAM: i32 = 7;

    // Hexaghost
    pub const HEX_DIVIDER: i32 = 1;
    pub const HEX_TACKLE: i32 = 2;
    pub const HEX_INFLAME: i32 = 3;
    pub const HEX_SEAR: i32 = 4;
    pub const HEX_ACTIVATE: i32 = 5;
    pub const HEX_INFERNO: i32 = 6;

    // Apology Slime
    pub const APOLOGY_TACKLE: i32 = 1;
    pub const APOLOGY_DEBUFF: i32 = 2;

    // Slime Boss
    pub const SB_SLAM: i32 = 1;
    pub const SB_PREP_SLAM: i32 = 2;
    pub const SB_SPLIT: i32 = 3;
    pub const SB_STICKY: i32 = 4;

    // =====================================================================
    // Act 2 — The City
    // =====================================================================

    // Chosen
    pub const CHOSEN_POKE: i32 = 5;
    pub const CHOSEN_ZAP: i32 = 1;
    pub const CHOSEN_DRAIN: i32 = 2;
    pub const CHOSEN_DEBILITATE: i32 = 3;
    pub const CHOSEN_HEX: i32 = 4;

    // Mugger (same structure as Looter)
    pub const MUGGER_MUG: i32 = 1;
    pub const MUGGER_SMOKE_BOMB: i32 = 2;
    pub const MUGGER_ESCAPE: i32 = 3;
    pub const MUGGER_BIG_SWIPE: i32 = 4;

    // Byrd
    pub const BYRD_PECK: i32 = 1;
    pub const BYRD_FLY_UP: i32 = 2;
    pub const BYRD_SWOOP: i32 = 3;
    pub const BYRD_STUNNED: i32 = 4;
    pub const BYRD_HEADBUTT: i32 = 5;
    pub const BYRD_CAW: i32 = 6;

    // Shelled Parasite
    pub const SP_FELL: i32 = 1;
    pub const SP_DOUBLE_STRIKE: i32 = 2;
    pub const SP_LIFE_SUCK: i32 = 3;
    pub const SP_STUNNED: i32 = 4;

    // Snake Plant
    pub const SNAKE_CHOMP: i32 = 1;
    pub const SNAKE_SPORES: i32 = 2;

    // Centurion
    pub const CENT_SLASH: i32 = 1;
    pub const CENT_PROTECT: i32 = 2;
    pub const CENT_FURY: i32 = 3;

    // Mystic (Healer)
    pub const MYSTIC_ATTACK: i32 = 1;
    pub const MYSTIC_HEAL: i32 = 2;
    pub const MYSTIC_BUFF: i32 = 3;

    // Book of Stabbing
    pub const BOOK_STAB: i32 = 1;
    pub const BOOK_BIG_STAB: i32 = 2;

    // Gremlin Leader
    pub const GL_RALLY: i32 = 2;
    pub const GL_ENCOURAGE: i32 = 3;
    pub const GL_STAB: i32 = 4;

    // Taskmaster
    pub const TASK_SCOURING_WHIP: i32 = 2;

    // Spheric Guardian
    pub const SPHER_BIG_ATTACK: i32 = 1;
    pub const SPHER_INITIAL_BLOCK: i32 = 2;
    pub const SPHER_BLOCK_ATTACK: i32 = 3;
    pub const SPHER_FRAIL_ATTACK: i32 = 4;

    // Snecko
    pub const SNECKO_GLARE: i32 = 1;
    pub const SNECKO_BITE: i32 = 2;
    pub const SNECKO_TAIL: i32 = 3;

    // Bear (Bandit)
    pub const BEAR_MAUL: i32 = 1;
    pub const BEAR_HUG: i32 = 2;
    pub const BEAR_LUNGE: i32 = 3;

    // Bandit Leader (Pointy)
    pub const BANDIT_CROSS_SLASH: i32 = 1;
    pub const BANDIT_MOCK: i32 = 2;
    pub const BANDIT_AGONIZE: i32 = 3;

    // Bandit Pointy
    pub const POINTY_STAB: i32 = 1;

    // Bronze Automaton (Boss)
    pub const BA_FLAIL: i32 = 1;
    pub const BA_HYPER_BEAM: i32 = 2;
    pub const BA_STUNNED: i32 = 3;
    pub const BA_SPAWN_ORBS: i32 = 4;
    pub const BA_BOOST: i32 = 5;

    // Bronze Orb
    pub const BO_BEAM: i32 = 1;
    pub const BO_SUPPORT: i32 = 2;
    pub const BO_STASIS: i32 = 3;

    // Torch Head
    pub const TORCH_TACKLE: i32 = 1;

    // Champ (Boss)
    pub const CHAMP_HEAVY_SLASH: i32 = 1;
    pub const CHAMP_DEFENSIVE: i32 = 2;
    pub const CHAMP_EXECUTE: i32 = 3;
    pub const CHAMP_FACE_SLAP: i32 = 4;
    pub const CHAMP_GLOAT: i32 = 5;
    pub const CHAMP_TAUNT: i32 = 6;
    pub const CHAMP_ANGER: i32 = 7;

    // The Collector (Boss)
    pub const COLL_SPAWN: i32 = 1;
    pub const COLL_FIREBALL: i32 = 2;
    pub const COLL_BUFF: i32 = 3;
    pub const COLL_MEGA_DEBUFF: i32 = 4;
    pub const COLL_REVIVE: i32 = 5;

    // =====================================================================
    // Act 3 — Beyond
    // =====================================================================

    // Darkling
    pub const DARK_CHOMP: i32 = 1;
    pub const DARK_HARDEN: i32 = 2;
    pub const DARK_NIP: i32 = 3;
    pub const DARK_WAIT: i32 = 4;
    pub const DARK_REINCARNATE: i32 = 5;

    // Orb Walker
    pub const OW_LASER: i32 = 1;
    pub const OW_CLAW: i32 = 2;

    // Spiker
    pub const SPIKER_ATTACK: i32 = 1;
    pub const SPIKER_BUFF: i32 = 2;

    // Repulsor
    pub const REPULSOR_DAZE: i32 = 1;
    pub const REPULSOR_ATTACK: i32 = 2;

    // Exploder
    pub const EXPLODER_ATTACK: i32 = 1;
    pub const EXPLODER_EXPLODE: i32 = 2;

    // Writhing Mass
    pub const WM_BIG_HIT: i32 = 0;
    pub const WM_MULTI_HIT: i32 = 1;
    pub const WM_ATTACK_BLOCK: i32 = 2;
    pub const WM_ATTACK_DEBUFF: i32 = 3;
    pub const WM_MEGA_DEBUFF: i32 = 4;

    // Spire Growth
    pub const SG_QUICK_TACKLE: i32 = 1;
    pub const SG_CONSTRICT: i32 = 2;
    pub const SG_SMASH: i32 = 3;

    // Maw
    pub const MAW_ROAR: i32 = 2;
    pub const MAW_SLAM: i32 = 3;
    pub const MAW_DROOL: i32 = 4;
    pub const MAW_NOM: i32 = 5;

    // Transient
    pub const TRANSIENT_ATTACK: i32 = 1;

    // Giant Head (Elite)
    pub const GH_GLARE: i32 = 1;
    pub const GH_IT_IS_TIME: i32 = 2;
    pub const GH_COUNT: i32 = 3;

    // Nemesis (Elite)
    pub const NEM_TRI_ATTACK: i32 = 2;
    pub const NEM_SCYTHE: i32 = 3;
    pub const NEM_BURN: i32 = 4;

    // Reptomancer (Elite)
    pub const REPTO_SNAKE_STRIKE: i32 = 1;
    pub const REPTO_SPAWN: i32 = 2;
    pub const REPTO_BIG_BITE: i32 = 3;

    // Snake Dagger (Reptomancer minion)
    pub const SD_WOUND: i32 = 1;
    pub const SD_EXPLODE: i32 = 2;

    // Awakened One (Boss)
    pub const AO_SLASH: i32 = 1;
    pub const AO_SOUL_STRIKE: i32 = 2;
    pub const AO_REBIRTH: i32 = 3;
    pub const AO_DARK_ECHO: i32 = 5;
    pub const AO_SLUDGE: i32 = 6;
    pub const AO_TACKLE: i32 = 8;

    // Donu (Boss)
    pub const DONU_BEAM: i32 = 0;
    pub const DONU_CIRCLE: i32 = 2;

    // Deca (Boss)
    pub const DECA_BEAM: i32 = 0;
    pub const DECA_SQUARE: i32 = 2;

    // Time Eater (Boss)
    pub const TE_REVERBERATE: i32 = 2;
    pub const TE_RIPPLE: i32 = 3;
    pub const TE_HEAD_SLAM: i32 = 4;
    pub const TE_HASTE: i32 = 5;

    // =====================================================================
    // Act 4 — The Ending
    // =====================================================================

    // Spire Shield
    pub const SHIELD_BASH: i32 = 1;
    pub const SHIELD_FORTIFY: i32 = 2;
    pub const SHIELD_SMASH: i32 = 3;

    // Spire Spear
    pub const SPEAR_BURN_STRIKE: i32 = 1;
    pub const SPEAR_PIERCER: i32 = 2;
    pub const SPEAR_SKEWER: i32 = 3;

    // Corrupt Heart
    pub const HEART_BLOOD_SHOTS: i32 = 1;
    pub const HEART_ECHO: i32 = 2;
    pub const HEART_DEBILITATE: i32 = 3;
    pub const HEART_BUFF: i32 = 4;
}

pub fn create_enemy(enemy_id: &str, hp: i32, max_hp: i32) -> EnemyCombatState {
    let canonical_id =
        canonical_enemy_id(enemy_id).unwrap_or_else(|| panic!("unknown enemy id: {enemy_id}"));
    let mut enemy = EnemyCombatState::new(canonical_id, hp, max_hp);

    match canonical_id {
        // =================================================================
        // Act 1 — Exordium
        // =================================================================
        "JawWorm" => {
            // Source: reference/extracted/methods/monster/JawWorm.java.
            // The run layer patches these constructor values for ascension.
            enemy.entity.set_status(sid::STARTING_DMG, 11);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 6);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
        }
        "Cultist" => {
            // Java Cultist.java: getMove() firstMove -> INCANTATION (BUFF);
            // takeTurn() case 3 applies RitualPower(ritualAmount), where the
            // ctor sets ritualAmount = ascension >= 2 ? 4 : 3 and takeTurn adds
            // +1 at ascension >= 17. Base 3 here = ascension 0/1; the run layer
            // (run.rs enter_specific_combat) patches 4/5 for ascension >= 2/17.
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.set_move(move_ids::CULT_INCANTATION, 0, 0, 0);
            enemy.add_effect(mfx::RITUAL, 3);
        }
        "FungiBeast" => {
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
            enemy.entity.set_status(sid::SPORE_CLOUD, 2);
        }
        "FuzzyLouseNormal" => {
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status(sid::CURL_UP, 5);
        }
        "FuzzyLouseDefensive" => {
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            // Also marks the A17 getMove rule at value 4 in the run layer.
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status(sid::CURL_UP, 5);
        }
        "SlaverBlue" => {
            enemy.entity.set_status(sid::STARTING_DMG, 12);
            enemy.entity.set_status(sid::STR_AMT, 7);
            enemy.entity.set_status(sid::BLOCK_AMT, 1);
            enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
        }
        "SlaverRed" => {
            enemy.entity.set_status(sid::STARTING_DMG, 13);
            enemy.entity.set_status(sid::STR_AMT, 8);
            enemy.entity.set_status(sid::BLOCK_AMT, 1);
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 1);
            enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
        }
        "AcidSlime_S" => {
            enemy.entity.set_status(sid::STARTING_DMG, 3);
            enemy.entity.set_status(sid::STR_AMT, 0);
            enemy.set_move(move_ids::AS_S_TACKLE, 3, 1, 0);
        }
        "AcidSlime_M" => {
            enemy.entity.set_status(sid::STARTING_DMG, 7);
            enemy.entity.set_status(sid::STR_AMT, 10);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(
                move_ids::AS_CORROSIVE_SPIT,
                Intent::AttackDebuff {
                    damage: 7,
                    hits: 1,
                    effects: fx::SLIMED,
                },
            );
            enemy.add_effect(mfx::SLIMED, 1);
        }
        "AcidSlime_L" => {
            // Java constructor directly appends SplitPower before animation.
            // Java: monsters/exordium/AcidSlime_L.java:93.
            enemy.entity.set_status_direct(sid::SPLIT_POWER, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 11);
            enemy.entity.set_status(sid::STR_AMT, 16);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(
                move_ids::AS_CORROSIVE_SPIT,
                Intent::AttackDebuff {
                    damage: 11,
                    hits: 1,
                    effects: fx::SLIMED,
                },
            );
            enemy.add_effect(mfx::SLIMED, 2);
        }
        "SpikeSlime_S" => {
            enemy.entity.set_status(sid::STARTING_DMG, 5);
            enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
        }
        "SpikeSlime_M" => {
            enemy.entity.set_status(sid::STARTING_DMG, 8);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(
                move_ids::SS_TACKLE,
                Intent::AttackDebuff {
                    damage: 8,
                    hits: 1,
                    effects: fx::SLIMED,
                },
            );
            enemy.add_effect(mfx::SLIMED, 1);
        }
        "SpikeSlime_L" => {
            // Java constructor directly appends SplitPower before animation.
            // Java: monsters/exordium/SpikeSlime_L.java:85.
            enemy.entity.set_status_direct(sid::SPLIT_POWER, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 16);
            enemy.entity.set_status(sid::STR_AMT, 2);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(
                move_ids::SS_TACKLE,
                Intent::AttackDebuff {
                    damage: 16,
                    hits: 1,
                    effects: fx::SLIMED,
                },
            );
            enemy.add_effect(mfx::SLIMED, 2);
        }
        "Looter" => {
            enemy.entity.set_status(sid::STARTING_DMG, 10);
            enemy.entity.set_status(sid::STR_AMT, 12);
            enemy.entity.set_status(sid::BLOCK_AMT, 6);
            enemy.entity.set_status(sid::TURN_COUNT, 15);
            // Looter.usePreBattleAction applies a Java-visible ThieveryPower;
            // TURN_COUNT remains the private goldAmt backing value used by the
            // attack/reward path.
            // Source: Looter.java:59-85; ThieveryPower.java:13-27.
            enemy.entity.set_status(sid::THIEVERY, 15);
            enemy.set_move(move_ids::LOOTER_MUG, 10, 1, 0);
        }
        "GremlinFat" => {
            enemy.entity.set_status(sid::STARTING_DMG, 4);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(
                move_ids::GREMLIN_FAT_SMASH,
                Intent::AttackDebuff {
                    damage: 4,
                    hits: 1,
                    effects: fx::WEAK,
                },
            );
            enemy.add_effect(mfx::WEAK, 1);
        }
        "GremlinThief" => {
            enemy.entity.set_status(sid::STARTING_DMG, 9);
            enemy.set_move(move_ids::GREMLIN_ATTACK, 9, 1, 0);
        }
        "GremlinWarrior" => {
            enemy.entity.set_status(sid::STARTING_DMG, 4);
            enemy.entity.set_status(sid::ANGRY, 1);
            enemy.set_move(move_ids::GREMLIN_ATTACK, 4, 1, 0);
        }
        "GremlinWizard" => {
            enemy.entity.set_status(sid::STARTING_DMG, 25);
            enemy.entity.set_status(sid::COUNT, 1);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move_with_intent(move_ids::GREMLIN_PROTECT, Intent::Unknown);
        }
        "GremlinTsundere" => {
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            enemy.entity.set_status(sid::BLOCK_AMT, 7);
            enemy.set_move_with_intent(
                move_ids::GREMLIN_TSUNDERE_PROTECT,
                Intent::Block {
                    amount: 0,
                    effects: 0,
                },
            );
            enemy.add_effect(mfx::BLOCK_RANDOM_OTHER, 7);
        }
        "GremlinNob" => {
            enemy.entity.set_status(sid::STARTING_DMG, 14);
            enemy.entity.set_status(sid::STR_AMT, 6);
            enemy.entity.set_status(sid::TURN_COUNT, 2);
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.set_move(move_ids::NOB_BELLOW, 0, 0, 0);
            enemy.add_effect(mfx::ENRAGE, 2);
        }
        "Lagavulin" => {
            enemy.entity.block = 8;
            enemy.entity.set_status(sid::STARTING_DMG, 18);
            enemy.entity.set_status(sid::STR_AMT, 1);
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(sid::COUNT, 0);
            enemy.entity.set_status(sid::ATTACK_COUNT, 0);
            enemy.set_move_with_intent(move_ids::LAGA_SLEEP, Intent::Sleep);
            enemy.entity.set_status(sid::METALLICIZE, 8);
            enemy.entity.set_status(sid::SLEEP_TURNS, 1);
        }
        "Sentry" => {
            enemy.entity.set_status(sid::STARTING_DMG, 9);
            enemy.entity.set_status(sid::STR_AMT, 2);
            enemy.entity.set_status(sid::FIRST_MOVE, 0);
            enemy
                .entity
                .set_status(sid::IS_FIRST_MOVE, move_ids::SENTRY_BOLT);
            enemy.entity.set_status(sid::ARTIFACT, 1);
            enemy.set_move_with_intent(move_ids::SENTRY_BOLT, Intent::Debuff { effects: fx::DAZE });
            enemy.add_effect(mfx::DAZE, 2);
        }
        "TheGuardian" => {
            enemy.entity.set_status(sid::PHASE, 0);
            enemy.entity.set_status(sid::MODE_SHIFT, 30);
            enemy.entity.set_status(sid::DAMAGE_TAKEN_THIS_MODE, 0);
            enemy.entity.set_status(sid::FIERCE_BASH_DMG, 32);
            enemy.entity.set_status(sid::ROLL_DMG, 9);
            enemy.entity.set_status(sid::BLOCK_AMT, 9);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::TURN_COUNT, 20);
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }
        "Hexaghost" => {
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(sid::COUNT, 0);
            enemy.entity.set_status(sid::BUFF_COUNT, 0);
            enemy.entity.set_status(sid::STR_AMT, 2);
            enemy.entity.set_status(sid::SEAR_BURN_COUNT, 1);
            enemy.entity.set_status(sid::FIRE_TACKLE_DMG, 5);
            enemy.entity.set_status(sid::INFERNO_DMG, 2);
            enemy.set_move_with_intent(move_ids::HEX_ACTIVATE, Intent::Unknown);
        }
        "SlimeBoss" => {
            // SlimeBoss.java directly appends SplitPower in its constructor.
            enemy.entity.set_status_direct(sid::SPLIT_POWER, 1);
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
            enemy.entity.set_status(sid::FIRE_TACKLE_DMG, 9);
            enemy.entity.set_status(sid::SLAP_DMG, 35);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.set_move_with_intent(
                move_ids::SB_STICKY,
                Intent::StrongDebuff {
                    effects: fx::SLIMED,
                },
            );
            enemy.add_effect(mfx::SLIMED, 3);
        }
        "Apology Slime" => {
            enemy.set_move(move_ids::APOLOGY_TACKLE, 3, 1, 0);
        }

        // =================================================================
        // Act 2 — The City
        // =================================================================
        "Chosen" => {
            // Source: reference/extracted/methods/monster/Chosen.java.
            // The opening is rolled during combat initialization.
            enemy.set_move(move_ids::CHOSEN_POKE, 5, 2, 0);
            enemy.entity.set_status(sid::FIRST_TURN, 1);
            enemy.entity.set_status(sid::FIRST_MOVE, 0);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 18);
            enemy.entity.set_status(sid::STR_AMT, 10);
            enemy.entity.set_status(sid::SLAP_DMG, 5);
        }
        "Mugger" => {
            // Source: reference/extracted/methods/monster/Mugger.java.
            enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 10);
            enemy.entity.set_status(sid::STR_AMT, 16);
            enemy.entity.set_status(sid::BLOCK_AMT, 11);
            enemy.entity.set_status(sid::TURN_COUNT, 15);
            // Source: Mugger.java:60-87; ThieveryPower.java:13-27.
            enemy.entity.set_status(sid::THIEVERY, 15);
            enemy.entity.set_status(sid::ATTACK_COUNT, 0);
        }
        "Byrd" => {
            // Source: reference/extracted/methods/monster/Byrd.java.
            // Ascension-derived values are patched at the run spawn site.
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 1);
            enemy.entity.set_status(sid::STR_AMT, 5);
            enemy.entity.set_status(sid::SLASH_DMG, 12);
            enemy.entity.set_status(sid::HEAD_SLAM_DMG, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 3);
            enemy.entity.set_status(sid::FLIGHT, 3);
        }
        "Shelled Parasite" => {
            // Source: reference/extracted/methods/monster/ShelledParasite.java.
            enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
            enemy.entity.set_status(sid::PLATED_ARMOR, 14);
            enemy.entity.block = 14;
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            enemy.entity.set_status(sid::STR_AMT, 18);
            enemy.entity.set_status(sid::BLOCK_AMT, 10);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "SnakePlant" => {
            // Source: reference/extracted/methods/monster/SnakePlant.java.
            // The real opening intent is rolled during combat initialization.
            enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 7);
            enemy.entity.set_status(sid::MALLEABLE, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 3);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "Centurion" => {
            // Source: reference/extracted/methods/monster/Centurion.java.
            // The real opener is rolled during combat initialization.
            enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 12);
            enemy.entity.set_status(sid::STR_AMT, 6);
            enemy.entity.set_status(sid::ATTACK_COUNT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 15);
            enemy.entity.set_status(sid::COUNT, 2);
        }
        "Healer" => {
            // Source: reference/extracted/methods/monster/Healer.java.
            // The real opener is rolled during combat initialization.
            enemy.set_move_with_intent(
                move_ids::MYSTIC_ATTACK,
                Intent::AttackDebuff {
                    damage: 8,
                    hits: 1,
                    effects: fx::FRAIL,
                },
            );
            enemy.add_effect(mfx::FRAIL, 2);
            enemy.entity.set_status(sid::STARTING_DMG, 8);
            enemy.entity.set_status(sid::STR_AMT, 2);
            enemy.entity.set_status(sid::BLOCK_AMT, 16);
            enemy.entity.set_status(sid::COUNT, 0);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "BookOfStabbing" => {
            // A0 constructor and pre-battle values. The actual opener is
            // selected by getMove during CombatEngine::start_combat.
            // Java: reference/extracted/methods/monster/BookOfStabbing.java
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            enemy.entity.set_status(sid::STR_AMT, 21);
            // Java initializes stabCount to 1 as a field initializer; the
            // first STAB getMove branch increments it before setMove.
            enemy.entity.set_status(sid::STAB_COUNT, 1);
            enemy.entity.set_status(sid::BLOCK_AMT, 0);
            enemy.entity.set_status(sid::PAINFUL_STABS, 1);
            enemy.set_move(move_ids::BOOK_STAB, 6, 1, 0);
        }
        "GremlinLeader" => {
            // Source: reference/extracted/methods/monster/GremlinLeader.java.
            // The encounter spawn site patches ascension and alive-gremlin
            // state before the opening roll.
            enemy.set_move_with_intent(move_ids::GL_RALLY, Intent::Unknown);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 6);
            enemy.entity.set_status(sid::COUNT, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 0);
        }
        "SlaverBoss" => {
            // Source: reference/extracted/methods/monster/Taskmaster.java.
            enemy.set_move(move_ids::TASK_SCOURING_WHIP, 7, 1, 0);
            enemy.add_effect(mfx::WOUND, 1);
            enemy.intent = Intent::AttackDebuff {
                damage: 7,
                hits: 1,
                effects: fx::WOUND,
            };
            enemy.entity.set_status(sid::STARTING_DMG, 7);
            enemy.entity.set_status(sid::BLOCK_AMT, 1);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "SphericGuardian" => {
            // Source: reference/extracted/methods/monster/SphericGuardian.java.
            // Pre-battle grants Barricade, Artifact 3, and 40 Block. The
            // source-rolled opening Activate gains another 25 at A0.
            enemy.set_move(move_ids::SPHER_INITIAL_BLOCK, 0, 0, 25);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::FIRST_TURN, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 10);
            enemy.entity.set_status(sid::BLOCK_AMT, 25);
            enemy.entity.set_status(sid::STR_AMT, 15);
            enemy.entity.set_status(sid::BARRICADE, 1);
            enemy.entity.set_status(sid::ARTIFACT, 3);
            enemy.entity.block = 40;
        }
        "Snecko" => {
            // Source: reference/extracted/methods/monster/Snecko.java.
            // Provisional intent matches the forced Glare opener and is
            // replaced through the ordinary source-rolled move path.
            enemy.set_move_with_intent(move_ids::SNECKO_GLARE, Intent::StrongDebuff { effects: 0 });
            enemy.add_effect(mfx::CONFUSED, 1);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 15);
            enemy.entity.set_status(sid::STR_AMT, 8);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "BanditBear" => {
            // A0 constructor values. Ascension variants are patched at the
            // RunEngine spawn site, where ascension is available.
            // Java: reference/extracted/methods/monster/BanditBear.java
            enemy.entity.set_status(sid::STARTING_DMG, 18);
            enemy.entity.set_status(sid::STR_AMT, 9);
            enemy.entity.set_status(sid::BLOCK_AMT, 2);
            enemy.set_move_with_intent(move_ids::BEAR_HUG, Intent::StrongDebuff { effects: 0 });
            enemy.add_effect(mfx::DEX_DOWN, 2);
        }
        "BanditLeader" => {
            // A0 constructor values. Ascension variants are patched where the
            // run's ascension level is available.
            // Java: reference/extracted/methods/monster/BanditLeader.java
            enemy.entity.set_status(sid::STARTING_DMG, 15);
            enemy.entity.set_status(sid::STR_AMT, 10);
            enemy.entity.set_status(sid::BLOCK_AMT, 2);
            enemy.set_move_with_intent(move_ids::BANDIT_MOCK, Intent::Unknown);
        }
        "BanditChild" => {
            // The Java class is BanditPointy, but its canonical game ID is
            // BanditChild. A2 damage is patched at the RunEngine spawn site.
            // Java: reference/extracted/methods/monster/BanditPointy.java
            enemy.entity.set_status(sid::STARTING_DMG, 5);
            enemy.set_move(move_ids::POINTY_STAB, 5, 2, 0);
        }
        "BronzeAutomaton" => {
            // A0 constructor/pre-battle values. Independent A4/A9/A19 gates
            // are patched at the RunEngine spawn site.
            // Java: reference/extracted/methods/monster/BronzeAutomaton.java
            enemy.set_move_with_intent(move_ids::BA_SPAWN_ORBS, Intent::Unknown);
            enemy.entity.set_status(sid::FLAIL_DMG, 7);
            enemy.entity.set_status(sid::BEAM_DMG, 45);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 9);
            enemy.entity.set_status(sid::FIRST_TURN, 1);
            enemy.entity.set_status(sid::NUM_TURNS, 0);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
            enemy.entity.set_status(sid::ARTIFACT, 3);
        }
        "BronzeOrb" => {
            // Placeholder intent; SpawnMonsterAction/init performs the source
            // opening roll. FIRST_MOVE represents usedStasis.
            // Java: reference/extracted/methods/monster/BronzeOrb.java
            enemy.entity.set_status(sid::FIRST_MOVE, 0);
            enemy.set_move_with_intent(move_ids::BO_STASIS, Intent::StrongDebuff { effects: 0 });
            enemy.add_effect(mfx::STASIS, 1);
        }
        "TorchHead" => {
            // Always: Tackle (7 damage)
            enemy.set_move(move_ids::TORCH_TACKLE, 7, 1, 0);
        }
        "Champ" => {
            // A0 constructor values. Independent A4/A9/A19 thresholds are
            // patched at the run spawn site.
            // Java: reference/extracted/methods/monster/Champ.java.
            let (slash_dmg, slap_dmg, str_amt, forge_amt, block_amt) = (16, 12, 2, 5, 15);
            enemy.set_move_with_intent(
                move_ids::CHAMP_FACE_SLAP,
                Intent::AttackDebuff {
                    damage: slap_dmg as i16,
                    hits: 1,
                    effects: fx::FRAIL | fx::VULNERABLE,
                },
            );
            enemy.add_effect(mfx::FRAIL, 2);
            enemy.add_effect(mfx::VULNERABLE, 2);
            enemy.entity.set_status(sid::NUM_TURNS, 0);
            enemy.entity.set_status(sid::THRESHOLD_REACHED, 0);
            enemy.entity.set_status(sid::STR_AMT, str_amt);
            enemy.entity.set_status(sid::FORGE_AMT, forge_amt);
            enemy.entity.set_status(sid::BLOCK_AMT, block_amt);
            enemy.entity.set_status(sid::FORGE_TIMES, 0);
            enemy.entity.set_status(sid::SLASH_DMG, slash_dmg);
            enemy.entity.set_status(sid::SLAP_DMG, slap_dmg);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "TheCollector" => {
            // Source: reference/extracted/methods/monster/TheCollector.java.
            // A0 constructor state; independent A4/A9/A19 values are patched
            // at the run spawn site.
            enemy.set_move(move_ids::COLL_SPAWN, 0, 0, 0);
            enemy.intent = crate::combat_types::Intent::Unknown;
            enemy.entity.set_status(sid::FIREBALL_DMG, 18);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 15);
            enemy.entity.set_status(sid::STARTING_DMG, 3);
            enemy.entity.set_status(sid::TURN_COUNT, 0);
            enemy.entity.set_status(sid::USED_MEGA_DEBUFF, 0);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::COUNT, 0);
        }

        // =================================================================
        // Act 3 — Beyond
        // =================================================================
        "Darkling" => {
            // Source: reference/extracted/methods/monster/Darkling.java.
            // Ascension and randomized constructor values are patched at the
            // run spawn site; this is the A0 factory baseline.
            enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 8);
            enemy.entity.set_status(sid::STR_AMT, 8);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
            enemy.entity.set_status(sid::COUNT, 0);
            enemy.entity.set_status(sid::REGROW, 1);
        }
        "Orb Walker" => {
            // Source: reference/extracted/methods/monster/OrbWalker.java.
            enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
            enemy.add_effect(mfx::BURN_DRAW_DISCARD, 1);
            enemy.intent = Intent::AttackDebuff {
                damage: 10,
                hits: 1,
                effects: fx::BURN,
            };
            enemy.entity.set_status(sid::STARTING_DMG, 10);
            enemy.entity.set_status(sid::STR_AMT, 15);
            enemy.entity.set_status(sid::GENERIC_STRENGTH_UP, 3);
        }
        "Spiker" => {
            // Source: reference/extracted/methods/monster/Spiker.java.
            // The real opener is rolled during combat initialization.
            enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
            enemy.entity.set_status(sid::THORNS, 3);
            enemy.entity.set_status(sid::STARTING_DMG, 7);
            enemy.entity.set_status(sid::COUNT, 0);
        }
        "Repulsor" => {
            // Source: reference/extracted/methods/monster/Repulsor.java.
            enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
            enemy.add_effect(mfx::DAZE_DRAW, 2);
            enemy.intent = Intent::Debuff { effects: fx::DAZE };
            enemy.entity.set_status(sid::STARTING_DMG, 11);
        }
        "Exploder" => {
            // Source: reference/extracted/methods/monster/Exploder.java.
            enemy.set_move(move_ids::EXPLODER_ATTACK, 9, 1, 0);
            enemy.entity.set_status(sid::TURN_COUNT, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 9);
            enemy.entity.set_status(sid::EXPLOSIVE, 3);
        }
        "WrithingMass" => {
            // Base factory represents A0; RunEngine applies ascension stats.
            // AbstractMonster.init replaces this provisional intent with a random
            // first intent from WrithingMass.getMove.
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
            enemy.entity.set_status(sid::REACTIVE, 1);
            enemy.entity.set_status(sid::MALLEABLE, 3);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 32);
            enemy.entity.set_status(sid::STR_AMT, 7);
            enemy.entity.set_status(sid::BLOCK_AMT, 15);
            enemy.entity.set_status(sid::HEAD_SLAM_DMG, 10);
            enemy.entity.set_status(sid::USED_MEGA_DEBUFF, 0);
        }
        "Serpent" => {
            // Source: reference/extracted/methods/monster/SpireGrowth.java.
            enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 16);
            enemy.entity.set_status(sid::STR_AMT, 22);
            enemy.entity.set_status(sid::BLOCK_AMT, 10);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
            enemy.entity.set_status(sid::COUNT, 0);
        }
        "Maw" => {
            // Source: reference/extracted/methods/monster/Maw.java. The
            // opening getMove increments constructor turnCount 1 to 2; the
            // run-time initial roll performs that transition and consumes RNG.
            enemy.set_move_with_intent(move_ids::MAW_ROAR, Intent::StrongDebuff { effects: 0 });
            enemy.add_effect(mfx::WEAK, 3);
            enemy.add_effect(mfx::FRAIL, 3);
            enemy.entity.set_status(sid::TURN_COUNT, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 25);
            enemy.entity.set_status(sid::STR_AMT, 3);
            enemy.entity.set_status(sid::BLOCK_AMT, 3);
            enemy.entity.set_status(sid::FIRST_MOVE, 0);
        }
        "Transient" => {
            // Escalating damage. A2: starts at 40, else 30. +10 each turn.
            // Fading: A17 = 6 turns, else 5 turns. Has Shifting power.
            // startingDeathDmg stored as status for escalation.
            enemy.set_move(move_ids::TRANSIENT_ATTACK, 30, 1, 0);
            enemy.entity.set_status(sid::ATTACK_COUNT, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 30);
            // Base factory represents A0; RunEngine applies ascension thresholds.
            enemy.entity.set_status(sid::FADING, 5);
            enemy.entity.set_status(sid::SHIFTING, 1);
        }
        "GiantHead" => {
            // Countdown to It Is Time. Glare/Count cycle. Count starts at 5 (A18: 4).
            // startingDeathDmg: A3+ = 40, else 30. Has Slow power.
            // First getMove decrements count, so first turn is count=4 (or 3 at A18).
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
            enemy.entity.set_status(sid::COUNT, 5);
            enemy.entity.set_status(sid::STARTING_DEATH_DMG, 30);
            enemy.entity.set_status(sid::SLOW, 1);
        }
        "Nemesis" => {
            // Source: reference/extracted/methods/monster/Nemesis.java.
            enemy.set_move(move_ids::NEM_TRI_ATTACK, 6, 3, 0);
            enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 0);
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 6);
            enemy.entity.set_status(sid::BLOCK_AMT, 3);
        }
        "Reptomancer" => {
            // Source: reference/extracted/methods/monster/Reptomancer.java.
            enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
            enemy.intent = Intent::Unknown;
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
            enemy.entity.set_status(sid::STARTING_DMG, 13);
            enemy.entity.set_status(sid::STR_AMT, 30);
            enemy.entity.set_status(sid::BLOCK_AMT, 1);
            enemy.entity.set_status(sid::COUNT, 0);
        }
        "Dagger" => {
            // Source: reference/extracted/methods/monster/SnakeDagger.java.
            enemy.set_move(move_ids::SD_WOUND, 9, 1, 0);
            enemy.add_effect(mfx::WOUND, 1);
            enemy.intent = Intent::AttackDebuff {
                damage: 9,
                hits: 1,
                effects: fx::WOUND,
            };
            enemy.entity.set_status(sid::FIRST_MOVE, 1);
        }
        "AwakenedOne" => {
            // Ascension-dependent pre-battle values are patched at the run
            // spawn site. These are the A0 constructor/usePreBattle defaults.
            // Java: reference/extracted/methods/monster/AwakenedOne.java
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
            enemy.entity.set_status(sid::PHASE, 1);
            enemy.entity.set_status(sid::FIRST_TURN, 0);
            enemy.entity.set_status(sid::REGENERATION, 10);
            enemy.entity.set_status(sid::CURIOSITY, 1);
            enemy.entity.set_status(sid::UNAWAKENED, 1);
        }
        "Donu" => {
            // Source: reference/extracted/methods/monster/Donu.java. These are
            // A0 defaults; A4/A9/A19 are independent run-site thresholds.
            enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
            enemy.add_effect(mfx::STRENGTH, 3);
            enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 3);
            enemy.entity.set_status(sid::ARTIFACT, 2);
            enemy.entity.set_status(sid::BEAM_DMG, 10);
        }
        "Deca" => {
            // Source: reference/extracted/methods/monster/Deca.java. These are
            // A0 defaults; A4/A9/A19 are independent run-site thresholds.
            enemy.set_move_with_intent(
                move_ids::DECA_BEAM,
                Intent::AttackDebuff {
                    damage: 10,
                    hits: 2,
                    effects: fx::DAZE,
                },
            );
            enemy.add_effect(mfx::DAZE, 2);
            enemy.entity.set_status(sid::ARTIFACT, 2);
            enemy.entity.set_status(sid::BEAM_DMG, 10);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "TimeEater" => {
            // Source: reference/extracted/methods/monster/TimeEater.java.
            // A0 defaults; the run spawn site patches independent A4/A9/A19
            // thresholds before the source-random opening roll.
            enemy.set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
            enemy.entity.set_status(sid::CARD_COUNT, 0);
            enemy.entity.set_status(sid::USED_HASTE, 0);
            enemy.entity.set_status(sid::REVERB_DMG, 7);
            enemy.entity.set_status(sid::HEAD_SLAM_DMG, 26);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
            enemy.entity.set_status(sid::TIME_WARP_ACTIVE, 1);
        }

        // =================================================================
        // Act 4 — The Ending
        // =================================================================
        "SpireShield" => {
            // Source: reference/extracted/methods/monster/SpireShield.java.
            // These are the A0 constructor/pre-battle defaults; run spawning
            // patches the independent A3/A8/A18 thresholds.
            enemy.set_move(move_ids::SHIELD_BASH, 12, 1, 0);
            enemy.intent = crate::combat_types::Intent::AttackDebuff {
                damage: 12,
                hits: 1,
                effects: 0,
            };
            enemy.entity.set_status(sid::MOVE_COUNT, 0);
            enemy.entity.set_status(sid::STARTING_DMG, 12);
            enemy.entity.set_status(sid::STR_AMT, 34);
            enemy.entity.set_status(sid::ARTIFACT, 1);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "SpireSpear" => {
            // Source: reference/extracted/methods/monster/SpireSpear.java.
            // A0 constructor and pre-battle defaults; the run spawn site owns
            // the independent A3/A8/A18 thresholds.
            enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
            enemy.intent = crate::combat_types::Intent::AttackDebuff {
                damage: 5,
                hits: 2,
                effects: 0,
            };
            enemy.add_effect(mfx::BURN, 2);
            enemy.entity.set_status(sid::MOVE_COUNT, 0);
            enemy.entity.set_status(sid::SKEWER_COUNT, 3);
            enemy.entity.set_status(sid::STARTING_DMG, 5);
            enemy.entity.set_status(sid::ARTIFACT, 1);
            enemy.entity.set_status(sid::HIGH_ASCENSION_AI, 0);
        }
        "CorruptHeart" => {
            // Source: reference/extracted/methods/monster/CorruptHeart.java.
            // Ascension-dependent constructor/pre-battle values are patched at
            // the run spawn site because create_enemy has no ascension input.
            enemy.set_move_with_intent(
                move_ids::HEART_DEBILITATE,
                Intent::StrongDebuff { effects: 0 },
            );
            enemy.add_effect(mfx::VULNERABLE, 2);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::FRAIL, 2);
            enemy.add_effect(mfx::HEART_STATUS_CARDS, 1);
            enemy.entity.set_status(sid::MOVE_COUNT, 0);
            enemy.entity.set_status(sid::BUFF_COUNT, 0);
            enemy.entity.set_status(sid::IS_FIRST_MOVE, 1);
            enemy.entity.set_status(sid::INVINCIBLE, 300);
            enemy.entity.set_status(sid::BEAT_OF_DEATH, 1);
            enemy.entity.set_status(sid::BLOOD_HIT_COUNT, 12);
            enemy.entity.set_status(sid::ECHO_DMG, 40);
        }

        _ => unreachable!("canonical enemy catalog and factory arms must stay in sync"),
    }

    enemy.needs_initial_move_roll = true;
    enemy
}

/// Advance enemy intent for the next turn.
///
/// Mirrors Java `AbstractMonster.rollMove()`: consumes exactly one value from
/// the per-combat `ai_rng` (the equivalent of `AbstractDungeon.aiRng`) and
/// passes that 0..=99 integer to each enemy's `getMove(int num)` equivalent.
/// Deterministic enemies ignore `num` but the stream still advances, preserving
/// multi-enemy intent ordering parity with Java.
pub fn roll_next_move(enemy: &mut EnemyCombatState, ai_rng: &mut crate::seed::StsRandom) {
    let num: i32 = ai_rng.random_int(99);
    roll_next_move_with_num_and_rng(enemy, num, ai_rng);
}

/// Select a monster's opening intent without recording a placeholder move in
/// history. Java calls `rollMove()` from `AbstractMonster.init()` while history
/// is empty, including deterministic monsters that ignore the rolled value.
pub fn roll_initial_move(enemy: &mut EnemyCombatState, ai_rng: &mut crate::seed::StsRandom) {
    let num = ai_rng.random_int(99);
    roll_initial_move_with_num_and_rng(enemy, num, ai_rng);
    enemy.needs_initial_move_roll = false;
}

pub fn roll_initial_move_with_num_and_rng(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    enemy.move_history.clear();
    if enemy.id == "TorchHead" {
        // TorchHead is unusual: its constructor calls setMove(TACKLE) before
        // AbstractMonster.init calls rollMove(), and getMove then calls
        // setMove(TACKLE) again. Rust stores prior Java setMove entries in
        // move_history and projects the current move separately, so preserve
        // the constructor entry here.
        // Java: TorchHead.java::<init>/getMove and
        // AbstractMonster.java::init/setMove.
        enemy.move_history.push(move_ids::TORCH_TACKLE);
    }
    enemy.move_effects.clear();
    if enemy.id == "Cultist" {
        act1::roll_cultist_initial(enemy);
    } else if enemy.id == "JawWorm" && enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        act1::roll_jaw_worm_initial(enemy);
    } else {
        select_move(enemy, num, ai_rng);
    }
}

/// Test-friendly entry point: advance enemy intent using an explicit `num` (0..=99)
/// and a deterministic secondary RNG. Production code should call `roll_next_move`;
/// RNG-sensitive tests should call `roll_next_move_with_num_and_rng`.
#[cfg(test)]
pub(crate) fn roll_next_move_with_num(enemy: &mut EnemyCombatState, num: i32) {
    let mut secondary_rng = crate::seed::StsRandom::new(0);
    roll_next_move_with_num_and_rng(enemy, num, &mut secondary_rng);
}

/// Advance with an explicit Java `getMove` number while preserving conditional
/// RNG draws performed inside the enemy's AI routine.
pub fn roll_next_move_with_num_and_rng(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();
    select_move(enemy, num, ai_rng);
}

fn select_move(enemy: &mut EnemyCombatState, num: i32, ai_rng: &mut crate::seed::StsRandom) {
    match enemy.id.as_str() {
        // Act 1
        "JawWorm" => act1::roll_jaw_worm(enemy, num, ai_rng),
        "Cultist" => act1::roll_cultist(enemy, num),
        "FungiBeast" => act1::roll_fungi_beast(enemy, num),
        "FuzzyLouseNormal" => act1::roll_red_louse(enemy, num),
        "FuzzyLouseDefensive" => act1::roll_green_louse(enemy, num),
        "SlaverBlue" => act1::roll_blue_slaver(enemy, num),
        "SlaverRed" => act1::roll_red_slaver(enemy, num),
        "AcidSlime_S" => act1::roll_acid_slime_s(enemy, num, ai_rng),
        "AcidSlime_M" => act1::roll_acid_slime_m(enemy, num, ai_rng),
        "AcidSlime_L" => act1::roll_acid_slime_l(enemy, num, ai_rng),
        "SpikeSlime_S" => act1::roll_spike_slime_s(enemy, num),
        "SpikeSlime_M" => act1::roll_spike_slime_m(enemy, num),
        "SpikeSlime_L" => act1::roll_spike_slime_l(enemy, num),
        "Looter" => act1::roll_looter(enemy, num),
        "GremlinFat" => act1::roll_gremlin_fat(enemy),
        "GremlinThief" => act1::roll_gremlin_thief(enemy),
        "GremlinWarrior" => act1::roll_gremlin_warrior(enemy),
        "GremlinWizard" => act1::roll_gremlin_wizard(enemy),
        "GremlinTsundere" => act1::roll_gremlin_tsundere(enemy),
        "GremlinNob" => act1::roll_gremlin_nob(enemy, num),
        "Lagavulin" => act1::roll_lagavulin(enemy),
        "Sentry" => act1::roll_sentry(enemy),
        "TheGuardian" => act1::roll_guardian(enemy),
        "Hexaghost" => act1::roll_hexaghost(enemy),
        "SlimeBoss" => act1::roll_slime_boss(enemy),
        "Apology Slime" => act1::roll_apology_slime(enemy, ai_rng),
        // Act 2
        "Chosen" => act2::roll_chosen(enemy, num),
        "Mugger" => act2::roll_mugger(enemy, num),
        "Byrd" => act2::roll_byrd(enemy, num, ai_rng),
        "Shelled Parasite" => act2::roll_shelled_parasite(enemy, num, ai_rng),
        "SnakePlant" => act2::roll_snake_plant(enemy, num),
        "Centurion" => act2::roll_centurion(enemy, num),
        "Healer" => act2::roll_healer(enemy, num),
        "BookOfStabbing" => act2::roll_book_of_stabbing(enemy, num),
        "GremlinLeader" => act2::roll_gremlin_leader(enemy, num, ai_rng),
        "SlaverBoss" => act2::roll_taskmaster(enemy, num),
        "SphericGuardian" => act2::roll_spheric_guardian(enemy),
        "Snecko" => act2::roll_snecko(enemy, num),
        "BanditBear" => act2::roll_bear(enemy, num),
        "BanditLeader" => act2::roll_bandit_leader(enemy, num),
        "BanditChild" => act2::roll_bandit_pointy(enemy, num),
        "BronzeAutomaton" => act2::roll_bronze_automaton(enemy, num),
        "BronzeOrb" => act2::roll_bronze_orb(enemy, num),
        "TorchHead" => { /* Always Tackle 7 */ }
        "Champ" => act2::roll_champ(enemy, num),
        "TheCollector" => act2::roll_collector(enemy, num),
        // Act 3
        "Darkling" => act3::roll_darkling(enemy, num, ai_rng),
        "Orb Walker" => act3::roll_orb_walker(enemy, num),
        "Spiker" => act3::roll_spiker(enemy, num),
        "Repulsor" => act3::roll_repulsor(enemy, num),
        "Exploder" => act3::roll_exploder(enemy, num),
        "WrithingMass" => act3::roll_writhing_mass(enemy, num, ai_rng),
        "Serpent" => act3::roll_spire_growth(enemy, num),
        "Maw" => act3::roll_maw(enemy, num),
        "Transient" => act3::roll_transient(enemy, num),
        "GiantHead" => act3::roll_giant_head(enemy, num),
        "Nemesis" => act3::roll_nemesis(enemy, num, ai_rng),
        "Reptomancer" => act3::roll_reptomancer(enemy, num, ai_rng),
        "Dagger" => act3::roll_snake_dagger(enemy, num),
        "AwakenedOne" => act3::roll_awakened_one(enemy, num),
        "Donu" => act3::roll_donu(enemy, num),
        "Deca" => act3::roll_deca(enemy, num),
        "TimeEater" => act3::roll_time_eater(enemy, num, ai_rng),
        // Act 4
        "SpireShield" => act4::roll_spire_shield(enemy, ai_rng),
        "SpireSpear" => act4::roll_spire_spear(enemy, ai_rng),
        "CorruptHeart" => act4::roll_corrupt_heart(enemy, num, ai_rng),
        _ => panic!("unknown enemy id during intent selection: {}", enemy.id),
    }
}

// =========================================================================
// Helpers (shared by act modules)
// =========================================================================

pub(crate) fn last_move(enemy: &EnemyCombatState, move_id: i32) -> bool {
    enemy.move_history.last().copied() == Some(move_id)
}

pub(crate) fn last_two_moves(enemy: &EnemyCombatState, move_id: i32) -> bool {
    let len = enemy.move_history.len();
    if len < 2 {
        return false;
    }
    enemy.move_history[len - 1] == move_id && enemy.move_history[len - 2] == move_id
}

// Re-exports of pub functions from act modules
pub(crate) use act1::advance_acid_slime_s_after_turn;
pub use act1::guardian_check_mode_shift;
pub use act1::guardian_switch_to_offensive;
pub use act1::hexaghost_set_divider;
pub use act1::lagavulin_wake_up;
pub use act1::slime_boss_should_split;
pub use act3::awakened_one_rebirth;
pub use act3::writhing_mass_reactive_reroll;

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE_SUBCLASS_DRAW_CONSTRUCTORS: &[&str] = &[
        "AcidSlime_L",
        "AcidSlime_M",
        "AcidSlime_S",
        "Apology Slime",
        "AwakenedOne",
        "BanditBear",
        "BanditChild",
        "BanditLeader",
        "BookOfStabbing",
        "BronzeAutomaton",
        "Byrd",
        "Centurion",
        "Champ",
        "Chosen",
        "Cultist",
        "Dagger",
        "Deca",
        "Donu",
        "Exploder",
        "FuzzyLouseDefensive",
        "FuzzyLouseNormal",
        "GiantHead",
        "GremlinFat",
        "GremlinLeader",
        "GremlinNob",
        "GremlinThief",
        "GremlinTsundere",
        "GremlinWarrior",
        "GremlinWizard",
        "Healer",
        "JawWorm",
        "Lagavulin",
        "Looter",
        "Maw",
        "Mugger",
        "Nemesis",
        "Orb Walker",
        "Reptomancer",
        "Repulsor",
        "Sentry",
        "Serpent",
        "Shelled Parasite",
        "SlaverBlue",
        "SlaverBoss",
        "SlaverRed",
        "SlimeBoss",
        "SnakePlant",
        "Snecko",
        "SphericGuardian",
        "SpikeSlime_L",
        "SpikeSlime_M",
        "SpikeSlime_S",
        "Spiker",
        "SpireShield",
        "SpireSpear",
        "TheCollector",
        "TimeEater",
        "TorchHead",
        "Transient",
        "WrithingMass",
    ];

    #[test]
    fn constructor_ambient_sequence_table_is_complete() {
        // Every AbstractMonster first constructs BobEffect, whose timer field
        // consumes one MathUtils float draw. SnakeDagger's subclass call is
        // transitive through initializeAnimation; Hexaghost's twelve subclass
        // calls are the six HexaghostOrb x/y constructor pairs.
        // Java: AbstractMonster.java:110, BobEffect.java:18,
        // SnakeDagger.java:43, Hexaghost.java:96, HexaghostOrb.java:40, and each
        // class under monsters/{exordium,city,beyond,ending} listed above.
        let mut classified = std::collections::BTreeSet::new();
        for id in ["BronzeOrb", "CorruptHeart", "TheGuardian"] {
            assert_eq!(
                constructor_ambient_sequence(id),
                Some(BASE_CONSTRUCTOR_DRAWS)
            );
            assert!(classified.insert(id));
        }
        for id in SINGLE_SUBCLASS_DRAW_CONSTRUCTORS {
            assert_eq!(
                constructor_ambient_sequence(id),
                Some(BASE_AND_UNIT_FLOAT_CONSTRUCTOR_DRAWS)
            );
            assert!(classified.insert(*id));
        }
        for (id, expected) in [
            ("FungiBeast", FUNGI_CONSTRUCTOR_DRAWS),
            ("Darkling", DARKLING_CONSTRUCTOR_DRAWS),
            ("Hexaghost", HEXAGHOST_CONSTRUCTOR_DRAWS),
        ] {
            assert_eq!(constructor_ambient_sequence(id), Some(expected));
            assert!(classified.insert(id));
        }
        let canonical = CANONICAL_ENEMIES
            .iter()
            .map(|(id, _)| *id)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(classified, canonical);
        for (id, _) in CANONICAL_ENEMIES {
            assert_eq!(
                constructor_ambient_sequence(id).unwrap().first(),
                Some(&ConstructorAmbientDraw::UnitFloat),
                "{id} skipped AbstractMonster's BobEffect draw"
            );
        }
        assert_eq!(
            CANONICAL_ENEMIES
                .iter()
                .map(|(id, _)| constructor_ambient_sequence(id).unwrap().len())
                .sum::<usize>(),
            142
        );
    }

    #[test]
    fn constructor_ambient_aliases_match_canonical() {
        for (alias, canonical) in ENEMY_ID_ALIASES {
            assert_eq!(
                constructor_ambient_sequence(alias),
                constructor_ambient_sequence(canonical),
                "ambient constructor policy drifted for alias {alias}"
            );
        }
    }

    #[test]
    fn constructor_ambient_consumer_advances_exact_raw_state() {
        for (id, _) in CANONICAL_ENEMIES {
            let mut actual = crate::seed::AmbientMathRng::new(0x5eed);
            let mut expected = actual.clone();
            for draw in constructor_ambient_sequence(id).unwrap() {
                match *draw {
                    ConstructorAmbientDraw::UnitFloat => {
                        let _ = expected.random_f32();
                    }
                    ConstructorAmbientDraw::FloatRange { start, end } => {
                        let _ = expected.random_f32_range(start, end);
                    }
                }
            }
            let enemy = create_enemy_with_ambient(id, 1, 1, &mut actual);
            assert_eq!(actual.state_tuple(), expected.state_tuple(), "{id}");
            assert!(enemy.needs_initial_move_roll, "{id}");
        }
    }

    // ----- Act 1 -----

    #[test]
    fn test_create_jaw_worm() {
        let enemy = create_enemy("JawWorm", 44, 44);
        assert_eq!(enemy.id, "JawWorm");
        assert_eq!(enemy.entity.hp, 44);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
        assert_eq!(enemy.move_damage(), 11);
    }

    #[test]
    fn test_jaw_worm_pattern() {
        let mut enemy = create_enemy("JawWorm", 44, 44);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);

        // Source: reference/extracted/methods/monster/JawWorm.java (`getMove`).
        roll_next_move_with_num(&mut enemy, 30); // CHOMP -> THRASH
        assert_eq!(enemy.move_id, move_ids::JW_THRASH);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_block(), 5);

        roll_next_move_with_num(&mut enemy, 80); // THRASH -> BELLOW
        assert_eq!(enemy.move_id, move_ids::JW_BELLOW);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(3));

        roll_next_move_with_num(&mut enemy, 30); // BELLOW -> THRASH
        roll_next_move_with_num(&mut enemy, 10); // THRASH -> CHOMP
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
    }

    #[test]
    fn test_cultist_pattern() {
        let mut enemy = create_enemy("Cultist", 50, 50);
        assert_eq!(enemy.move_id, move_ids::CULT_INCANTATION);
        assert_eq!(enemy.move_damage(), 0);
        assert_eq!(enemy.effect(mfx::RITUAL), Some(3));

        // AbstractMonster.init calls rollMove; Cultist's first getMove keeps
        // Incantation while consuming the mandatory aiRng draw.
        roll_initial_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CULT_INCANTATION);
        assert_eq!(enemy.effect(mfx::RITUAL), Some(3));

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
        assert_eq!(enemy.move_damage(), 6);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
    }

    #[test]
    fn test_fungi_beast_anti_repeat() {
        let mut enemy = create_enemy("FungiBeast", 24, 24);
        // Source: reference/extracted/methods/monster/FungiBeast.java.
        roll_next_move_with_num(&mut enemy, 0);
        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::FB_GROW);

        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::FB_BITE);
    }

    #[test]
    fn test_sentry_alternating() {
        let mut enemy = create_enemy("Sentry", 38, 38);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);
        assert_eq!(enemy.effect(mfx::DAZE), Some(2));

        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut crate::seed::StsRandom::new(1));
        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::SENTRY_BEAM);
        assert_eq!(enemy.move_damage(), 9);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);
    }

    #[test]
    fn test_slime_boss_pattern() {
        // Source: reference/extracted/methods/monster/SlimeBoss.java (`takeTurn`).
        let mut enemy = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);

        act1::advance_slime_boss_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_PREP_SLAM);

        act1::advance_slime_boss_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_SLAM);
        assert_eq!(enemy.move_damage(), 35);

        act1::advance_slime_boss_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);
    }

    #[test]
    fn test_slime_boss_split_check() {
        let mut enemy = create_enemy("SlimeBoss", 140, 140);
        assert!(!slime_boss_should_split(&enemy));

        enemy.entity.hp = 70;
        assert!(slime_boss_should_split(&enemy));

        enemy.entity.hp = 69;
        assert!(slime_boss_should_split(&enemy));
    }

    #[test]
    fn test_guardian_offensive_pattern() {
        // Source: reference/extracted/methods/monster/TheGuardian.java
        // (`useChargeUp`, `useFierceBash`, and `useVentSteam`).
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);

        act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_FIERCE_BASH);
        assert_eq!(enemy.move_damage(), 32);

        act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_VENT_STEAM);
        assert_eq!(enemy.effect(mfx::WEAK), Some(2));
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(2));

        act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage(), 5);
        assert_eq!(enemy.move_hits(), 4);
    }

    #[test]
    fn test_guardian_mode_shift() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.entity.status(sid::MODE_SHIFT), 30);

        let shifted = guardian_check_mode_shift(&mut enemy, 30);
        assert!(shifted);
        assert_eq!(enemy.entity.status(sid::SHARP_HIDE), 0);
        assert_eq!(enemy.entity.status(sid::MODE_SHIFT), 40);
        assert_eq!(enemy.entity.block, 20);
        assert_eq!(enemy.move_id, move_ids::GUARD_CLOSE_UP);
        assert_eq!(enemy.effect(mfx::SHARP_HIDE), Some(3));

        act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_ROLL_ATTACK);
        act1::advance_guardian_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_TWIN_SLAM);
    }

    #[test]
    fn test_hexaghost_pattern() {
        // Source: reference/extracted/methods/monster/Hexaghost.java (`takeTurn`).
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        let mut rng = crate::seed::StsRandom::new(0);
        assert_eq!(enemy.move_id, move_ids::HEX_ACTIVATE);

        act1::advance_hexaghost_after_turn(&mut enemy, 80, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_hits(), 6);

        act1::advance_hexaghost_after_turn(&mut enemy, 38, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);
        assert_eq!(enemy.effect(mfx::BURN), Some(1));

        act1::advance_hexaghost_after_turn(&mut enemy, 32, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_hits(), 2);
    }

    #[test]
    fn test_hexaghost_divider_scaling() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        hexaghost_set_divider(&mut enemy, 80);
        // 80 / 12 + 1 = 7 (integer division)
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 6);

        hexaghost_set_divider(&mut enemy, 60);
        // 60 / 12 + 1 = 6
        assert_eq!(enemy.move_damage(), 6);
    }

    #[test]
    fn test_blue_slaver_pattern() {
        let mut enemy = create_enemy("SlaverBlue", 48, 48);
        // Source: reference/extracted/methods/monster/SlaverBlue.java.
        roll_initial_move_with_num_and_rng(&mut enemy, 40, &mut crate::seed::StsRandom::new(0));
        roll_next_move_with_num(&mut enemy, 40);
        assert_eq!(enemy.move_id, move_ids::BS_STAB);

        roll_next_move_with_num(&mut enemy, 40);
        assert_eq!(enemy.move_id, move_ids::BS_RAKE);
        assert_eq!(enemy.effect(mfx::WEAK), Some(1));
    }

    #[test]
    fn test_red_slaver_pattern() {
        let mut enemy = create_enemy("SlaverRed", 48, 48);
        // Source: reference/extracted/methods/monster/SlaverRed.java.
        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::RS_STAB);
        assert_eq!(enemy.move_damage(), 13);

        roll_next_move_with_num(&mut enemy, 75);
        assert_eq!(enemy.move_id, move_ids::RS_ENTANGLE);
        assert_eq!(enemy.effect(mfx::ENTANGLE), Some(1));

        roll_next_move_with_num(&mut enemy, 60);
        assert!(enemy.move_id == move_ids::RS_SCRAPE || enemy.move_id == move_ids::RS_STAB);
    }

    #[test]
    fn test_acid_slime_s_pattern() {
        let mut enemy = create_enemy("AcidSlime_S", 10, 10);
        assert_eq!(enemy.move_id, move_ids::AS_S_TACKLE);

        advance_acid_slime_s_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_S_LICK);
        assert_eq!(enemy.effect(mfx::WEAK), Some(1));

        advance_acid_slime_s_after_turn(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_S_TACKLE);
    }

    #[test]
    fn test_spike_slime_m_pattern() {
        // Source: reference/extracted/methods/monster/SpikeSlime_M.java
        // (`getMove`: <30 cannot Tackle three times; >=30 selects Lick).
        let mut enemy = create_enemy("SpikeSlime_M", 28, 28);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);
        assert_eq!(enemy.move_damage(), 8);
        assert_eq!(enemy.effect(mfx::SLIMED), Some(1));

        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);
        assert_eq!(enemy.effect(mfx::SLIMED), Some(1));

        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::SS_LICK);
        assert_eq!(enemy.effect(mfx::FRAIL), Some(1));
    }

    #[test]
    fn test_louse_curl_up() {
        let enemy = create_enemy("RedLouse", 12, 12);
        assert_eq!(enemy.entity.status(sid::CURL_UP), 5);
    }

    #[test]
    fn test_guardian_switch_to_offensive() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut enemy, 30);
        enemy.entity.set_status(sid::SHARP_HIDE, 3);

        guardian_switch_to_offensive(&mut enemy);
        assert_eq!(enemy.entity.status(sid::SHARP_HIDE), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
    }

    #[test]
    fn test_looter_escape() {
        // Source: reference/extracted/methods/monster/Looter.java (`takeTurn`).
        let mut enemy = create_enemy("Looter", 44, 44);
        assert_eq!(enemy.move_id, move_ids::LOOTER_MUG);
        let seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let _ = rng.random_f32();
                rng.random_f32() < 0.5
            })
            .unwrap();
        let mut rng = crate::seed::StsRandom::new(seed);

        act1::advance_looter_after_turn(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::LOOTER_MUG);

        act1::advance_looter_after_turn(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::LOOTER_SMOKE_BOMB);
        assert_eq!(enemy.move_block(), 6);
        assert_eq!(rng.counter, 2);

        act1::advance_looter_after_turn(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::LOOTER_ESCAPE);
        assert!(!enemy.is_escaping);
        act1::advance_looter_after_turn(&mut enemy, &mut rng);
        assert!(enemy.is_escaping);
        assert_eq!(enemy.entity.hp, 0);

        let lunge_seed = (1..10_000)
            .find(|&seed| {
                let mut rng = crate::seed::StsRandom::new(seed);
                let _ = rng.random_f32();
                rng.random_f32() >= 0.5
            })
            .unwrap();
        let mut lunge = create_enemy("Looter", 44, 44);
        let mut rng = crate::seed::StsRandom::new(lunge_seed);
        act1::advance_looter_after_turn(&mut lunge, &mut rng);
        act1::advance_looter_after_turn(&mut lunge, &mut rng);
        assert_eq!(lunge.move_id, move_ids::LOOTER_LUNGE);
        assert_eq!(lunge.move_damage(), 12);
        act1::advance_looter_after_turn(&mut lunge, &mut rng);
        assert_eq!(lunge.move_id, move_ids::LOOTER_SMOKE_BOMB);
    }

    // ----- Act 2 -----

    #[test]
    fn test_chosen_pattern() {
        // Source: reference/extracted/methods/monster/Chosen.java.
        let mut enemy = create_enemy("Chosen", 97, 97);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_POKE);
        assert_eq!(enemy.move_damage(), 5);
        assert_eq!(enemy.move_hits(), 2);

        roll_initial_move_with_num_and_rng(&mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CHOSEN_POKE);

        // After Poke: Hex.
        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_HEX);
        assert_eq!(enemy.effect(mfx::HEX), Some(1));

        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_DEBILITATE);

        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_ZAP);
        assert_eq!(enemy.move_damage(), 18);
    }

    #[test]
    fn test_byrd_source_ai_windows_and_conditional_rng() {
        // Source: reference/extracted/methods/monster/Byrd.java (`getMove`).
        let mut enemy = create_enemy("Byrd", 28, 28);
        assert_eq!(enemy.move_id, move_ids::BYRD_PECK);
        assert_eq!(enemy.move_damage(), 1);
        assert_eq!(enemy.move_hits(), 5);
        assert_eq!(enemy.entity.status(sid::FLIGHT), 3);
        assert_eq!(enemy.entity.status(sid::FIRST_MOVE), 1);

        let seed_for = |chance: f32, expected: bool| {
            (1..10_000)
                .find(|&seed| {
                    let mut rng = crate::seed::StsRandom::new(seed);
                    (rng.random_f32() < chance) == expected
                })
                .unwrap()
        };

        for (caws, expected) in [(true, move_ids::BYRD_CAW), (false, move_ids::BYRD_PECK)] {
            let mut opening = create_enemy("Byrd", 28, 28);
            let mut rng = crate::seed::StsRandom::new(seed_for(0.375, caws));
            roll_initial_move_with_num_and_rng(&mut opening, 99, &mut rng);
            assert_eq!(opening.move_id, expected);
            assert_eq!(rng.counter, 1);
        }

        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        roll_next_move_with_num(&mut enemy, 0);
        assert_eq!(enemy.move_id, move_ids::BYRD_PECK);
        roll_next_move_with_num(&mut enemy, 60);
        assert_eq!(enemy.move_id, move_ids::BYRD_SWOOP);
        assert_eq!(enemy.move_damage(), 12);

        for (swoops, expected) in [(true, move_ids::BYRD_SWOOP), (false, move_ids::BYRD_CAW)] {
            let mut repeated_peck = create_enemy("Byrd", 28, 28);
            repeated_peck.entity.set_status(sid::FIRST_MOVE, 0);
            repeated_peck.move_history.push(move_ids::BYRD_PECK);
            let mut rng = crate::seed::StsRandom::new(seed_for(0.4, swoops));
            roll_next_move_with_num_and_rng(&mut repeated_peck, 0, &mut rng);
            assert_eq!(repeated_peck.move_id, expected);
            assert_eq!(rng.counter, 1);
        }

        enemy.entity.set_status(sid::FLIGHT, 0);
        let mut rng = crate::seed::StsRandom::new(7);
        roll_next_move_with_num_and_rng(&mut enemy, 99, &mut rng);
        assert_eq!(enemy.move_id, move_ids::BYRD_HEADBUTT);
        assert_eq!(enemy.move_damage(), 3);
        assert_eq!(rng.counter, 0);
    }

    #[test]
    fn test_snake_plant_pattern() {
        // Source: reference/extracted/methods/monster/SnakePlant.java.
        let mut enemy = create_enemy("SnakePlant", 77, 77);
        assert_eq!(enemy.move_id, move_ids::SNAKE_CHOMP);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 3);
        assert_eq!(enemy.entity.status(sid::MALLEABLE), 3);

        roll_initial_move_with_num_and_rng(&mut enemy, 64, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::SNAKE_CHOMP);

        roll_next_move_with_num(&mut enemy, 64);
        assert_eq!(enemy.move_id, move_ids::SNAKE_CHOMP);
        roll_next_move_with_num(&mut enemy, 64);
        assert_eq!(enemy.move_id, move_ids::SNAKE_SPORES);
        assert_eq!(enemy.effect(mfx::WEAK), Some(2));
        assert_eq!(enemy.effect(mfx::FRAIL), Some(2));
    }

    #[test]
    fn test_book_of_stabbing_escalation() {
        let mut enemy = create_enemy("BookOfStabbing", 162, 162);
        // Source: reference/extracted/methods/monster/BookOfStabbing.java.
        roll_initial_move_with_num_and_rng(&mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BOOK_STAB);
        assert_eq!(enemy.move_hits(), 2);
        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::BOOK_STAB);
        assert_eq!(enemy.move_hits(), 3);
    }

    #[test]
    fn test_bronze_automaton_boss_pattern() {
        let mut enemy = create_enemy("BronzeAutomaton", 300, 300);
        // Source: reference/extracted/methods/monster/BronzeAutomaton.java.
        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_SPAWN_ORBS);

        // After spawn: Flail
        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 2);
    }

    #[test]
    fn test_champ_boss_pattern() {
        // Source: reference/extracted/methods/monster/Champ.java.
        let mut enemy = create_enemy("Champ", 420, 420);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);
        assert_eq!(enemy.move_damage(), 12);
        // Java: Face Slap gives Frail 2 + Vulnerable 2
        assert_eq!(enemy.effect(mfx::FRAIL), Some(2));
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(2));

        roll_initial_move_with_num_and_rng(&mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);
        assert_eq!(enemy.move_damage(), 16);

        // Exactly half does not trigger; strictly below half does.
        enemy.entity.hp = 210;
        roll_next_move_with_num(&mut enemy, 99);
        assert_ne!(enemy.move_id, move_ids::CHAMP_ANGER);
        enemy.entity.hp = 209;
        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::CHAMP_ANGER);
    }

    #[test]
    fn test_collector_boss_pattern() {
        // Source: reference/extracted/methods/monster/TheCollector.java.
        let mut enemy = create_enemy("TheCollector", 282, 282);
        assert_eq!(enemy.move_id, move_ids::COLL_SPAWN);
        assert_eq!(enemy.entity.status(sid::FIRST_MOVE), 1);
        assert_eq!(enemy.entity.status(sid::TURN_COUNT), 0);

        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        enemy.entity.set_status(sid::TURN_COUNT, 1);
        roll_next_move_with_num(&mut enemy, 70);
        assert_eq!(enemy.move_id, move_ids::COLL_FIREBALL);
        assert_eq!(enemy.move_damage(), 18);

        enemy.entity.set_status(sid::COUNT, 1);
        roll_next_move_with_num(&mut enemy, 25);
        assert_eq!(enemy.move_id, move_ids::COLL_REVIVE);

        enemy.entity.set_status(sid::COUNT, 0);
        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::COLL_BUFF);
        assert_eq!(enemy.move_block(), 15);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(3));
        assert_eq!(enemy.effect(mfx::STRENGTH_ALL_ALLIES), Some(3));

        enemy.entity.set_status(sid::TURN_COUNT, 3);
        roll_next_move_with_num(&mut enemy, 99);
        assert_eq!(enemy.move_id, move_ids::COLL_MEGA_DEBUFF);
        assert_eq!(
            enemy.intent,
            crate::combat_types::Intent::StrongDebuff { effects: 0 }
        );
        assert_eq!(enemy.effect(mfx::VULNERABLE), Some(3));
        assert_eq!(enemy.effect(mfx::WEAK), Some(3));
        assert_eq!(enemy.effect(mfx::FRAIL), Some(3));
    }

    // ----- Act 3 -----

    #[test]
    fn test_awakened_one_boss() {
        // Source-derived branch table for AwakenedOne.getMove. Phase one uses
        // a 25% Soul Strike branch with one-/two-move guards; phase two uses a
        // 50% Sludge branch and permits either normal attack twice, not three
        // times. Java: reference/extracted/methods/monster/AwakenedOne.java.
        let mut enemy = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        assert_eq!(enemy.move_damage(), 20);
        assert_eq!(enemy.entity.status(sid::PHASE), 1);
        assert_eq!(enemy.entity.status(sid::CURIOSITY), 1);
        assert_eq!(enemy.entity.status(sid::REGENERATION), 10);

        roll_next_move_with_num(&mut enemy, 80);
        assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        roll_next_move_with_num(&mut enemy, 80);
        assert_eq!(enemy.move_id, move_ids::AO_SOUL_STRIKE);
        assert_eq!(enemy.move_damage(), 6);
        assert_eq!(enemy.move_hits(), 4);
        roll_next_move_with_num(&mut enemy, 10);
        assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        roll_next_move_with_num(&mut enemy, 10);
        assert_eq!(enemy.move_id, move_ids::AO_SOUL_STRIKE);

        // damage() installs Rebirth and resets firstTurn; changeState then
        // heals before the queued RollMoveAction selects Dark Echo.
        enemy.set_move_with_intent(move_ids::AO_REBIRTH, Intent::Unknown);
        enemy.entity.set_status(sid::FIRST_TURN, 1);
        awakened_one_rebirth(&mut enemy);
        assert_eq!(enemy.entity.status(sid::PHASE), 2);
        assert_eq!(enemy.entity.hp, 300);
        roll_next_move_with_num(&mut enemy, 75);
        assert_eq!(enemy.move_id, move_ids::AO_DARK_ECHO);
        assert_eq!(enemy.move_damage(), 40);

        roll_next_move_with_num(&mut enemy, 75);
        assert_eq!(enemy.move_id, move_ids::AO_TACKLE);
        roll_next_move_with_num(&mut enemy, 75);
        assert_eq!(enemy.move_id, move_ids::AO_TACKLE);
        roll_next_move_with_num(&mut enemy, 75);
        assert_eq!(enemy.move_id, move_ids::AO_SLUDGE);
        assert_eq!(enemy.effect(mfx::VOID), Some(1));
        roll_next_move_with_num(&mut enemy, 10);
        assert_eq!(enemy.move_id, move_ids::AO_SLUDGE);
        roll_next_move_with_num(&mut enemy, 10);
        assert_eq!(enemy.move_id, move_ids::AO_TACKLE);
    }

    #[test]
    fn test_time_eater_boss() {
        // Source: reference/extracted/methods/monster/TimeEater.java.
        let mut enemy = create_enemy("TimeEater", 456, 456);
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);
        assert_eq!(enemy.move_damage(), 7);
        assert_eq!(enemy.move_hits(), 3);

        roll_initial_move_with_num_and_rng(&mut enemy, 44, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);

        roll_initial_move_with_num_and_rng(&mut enemy, 45, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_HEAD_SLAM);
        assert_eq!(enemy.move_damage(), 26);
        assert_eq!(enemy.effect(mfx::DRAW_REDUCTION), Some(1));

        roll_initial_move_with_num_and_rng(&mut enemy, 80, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_RIPPLE);
        assert_eq!(enemy.move_block(), 20);
        assert_eq!(
            enemy.intent,
            Intent::DefendDebuff {
                block: 20,
                effects: fx::VULNERABLE | fx::WEAK,
            },
            "TimeEater.java declares Ripple as DEFEND_DEBUFF"
        );

        enemy.entity.hp = 227;
        enemy.entity.set_status(sid::USED_HASTE, 0);
        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::TE_HASTE);
        assert_eq!(enemy.entity.status(sid::USED_HASTE), 1);
    }

    #[test]
    fn test_donu_deca_boss() {
        let mut donu = create_enemy("Donu", 250, 250);
        assert_eq!(donu.move_id, move_ids::DONU_CIRCLE);
        assert_eq!(donu.entity.status(sid::ARTIFACT), 2);

        roll_next_move(&mut donu, &mut crate::seed::StsRandom::new(0));
        assert_eq!(donu.move_id, move_ids::DONU_BEAM);
        assert_eq!(donu.move_damage(), 10);
        assert_eq!(donu.move_hits(), 2);

        let mut deca = create_enemy("Deca", 250, 250);
        // Java: Deca starts with isAttacking=true -> first move is Beam
        assert_eq!(deca.move_id, move_ids::DECA_BEAM);
        assert_eq!(deca.move_damage(), 10);
        assert_eq!(deca.effect(mfx::DAZE), Some(2));

        roll_next_move(&mut deca, &mut crate::seed::StsRandom::new(0));
        assert_eq!(deca.move_id, move_ids::DECA_SQUARE);
        assert_eq!(deca.move_block(), 16);
    }

    #[test]
    fn test_giant_head_elite() {
        let mut enemy = create_enemy("GiantHead", 500, 500);
        assert_eq!(enemy.move_id, move_ids::GH_COUNT);
        assert_eq!(enemy.move_damage(), 13);
        assert_eq!(enemy.entity.status(sid::COUNT), 5);

        // Roll moves. Count decrements each roll. After count reaches 1, It Is Time.
        // Count starts at 5, so after 4 rolls we should be in It Is Time territory.
        for _ in 0..5 {
            roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        }

        // Should eventually hit It Is Time
        let count = enemy.entity.status(sid::COUNT);
        assert!(count <= 0 || enemy.move_id == move_ids::GH_IT_IS_TIME);
    }

    #[test]
    fn test_nemesis_elite() {
        let mut enemy = create_enemy("Nemesis", 185, 185);
        assert_eq!(enemy.move_id, move_ids::NEM_TRI_ATTACK);
        assert_eq!(enemy.move_damage(), 6);
        assert_eq!(enemy.move_hits(), 3);

        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        // Second turn
        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        // Should eventually use Scythe
        let has_scythe = enemy.move_id == move_ids::NEM_SCYTHE
            || enemy
                .move_history
                .iter()
                .any(|&m| m == move_ids::NEM_SCYTHE);
        assert!(has_scythe || enemy.move_history.len() <= 3);
    }

    #[test]
    fn test_reptomancer_elite() {
        // Source: reference/extracted/methods/monster/Reptomancer.java.
        let mut enemy = create_enemy("Reptomancer", 185, 185);
        assert_eq!(enemy.move_id, move_ids::REPTO_SPAWN);
        assert!(matches!(enemy.intent, crate::combat_types::Intent::Unknown));

        roll_initial_move_with_num_and_rng(&mut enemy, 99, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::REPTO_SPAWN);

        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::REPTO_SNAKE_STRIKE);
        assert_eq!(enemy.move_damage(), 13);
        assert_eq!(enemy.move_hits(), 2);
        assert_eq!(enemy.effect(crate::combat_types::mfx::WEAK), Some(1));

        roll_next_move_with_num_and_rng(&mut enemy, 66, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::REPTO_BIG_BITE);
        assert_eq!(enemy.move_damage(), 30);
    }

    #[test]
    fn test_transient_escalation() {
        let mut enemy = create_enemy("Transient", 999, 999);
        assert_eq!(enemy.move_id, move_ids::TRANSIENT_ATTACK);
        assert_eq!(enemy.move_damage(), 30);

        // Source: Transient.java getMove reads count; takeTurn owns increment.
        for (count, damage) in [(1, 40), (2, 50), (3, 60)] {
            enemy.entity.set_status(sid::ATTACK_COUNT, count);
            roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
            assert_eq!(enemy.move_damage(), damage);
        }
    }

    // ----- Act 4 -----

    #[test]
    fn test_corrupt_heart_boss() {
        let mut enemy = create_enemy("CorruptHeart", 750, 750);
        assert_eq!(enemy.move_id, move_ids::HEART_DEBILITATE);
        assert!(matches!(enemy.intent, Intent::StrongDebuff { .. }));
        assert_eq!(enemy.entity.status(sid::INVINCIBLE), 300);
        assert_eq!(enemy.entity.status(sid::BEAT_OF_DEATH), 1);
        assert_eq!(enemy.entity.status(sid::BLOOD_HIT_COUNT), 12);

        // Source: reference/extracted/methods/monster/CorruptHeart.java.
        // The first getMove call selects Debilitate and does not advance the
        // cycle. Slot zero is random; slot one selects the other attack.
        let mut rng = crate::seed::StsRandom::new(0);
        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEART_DEBILITATE);
        assert!(matches!(enemy.intent, Intent::StrongDebuff { .. }));
        assert_eq!(enemy.entity.status(sid::MOVE_COUNT), 0);

        roll_next_move(&mut enemy, &mut rng);
        let first_attack = enemy.move_id;
        assert!(matches!(
            first_attack,
            move_ids::HEART_BLOOD_SHOTS | move_ids::HEART_ECHO
        ));
        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(
            enemy.move_id,
            if first_attack == move_ids::HEART_ECHO {
                move_ids::HEART_BLOOD_SHOTS
            } else {
                move_ids::HEART_ECHO
            }
        );

        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::HEART_BUFF);
        assert_eq!(enemy.entity.status(sid::BUFF_COUNT), 0);
    }

    #[test]
    fn test_spire_shield_boss() {
        let mut enemy = create_enemy("SpireShield", 110, 110);
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);
        // Source: reference/extracted/methods/monster/SpireShield.java.
        assert_eq!(enemy.move_damage(), 12);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 1);

        let false_seed = (1..10_000)
            .find(|&seed| !crate::seed::StsRandom::new(seed).random_bool())
            .unwrap();
        let mut rng = crate::seed::StsRandom::new(false_seed);
        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);
        assert_eq!(rng.counter, 1);

        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SHIELD_FORTIFY);
        assert_eq!(enemy.move_block(), 30);
        assert_eq!(enemy.effect(mfx::BLOCK_ALL_ALLIES), Some(30));

        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SHIELD_SMASH);
        assert_eq!(enemy.move_damage(), 34);
    }

    #[test]
    fn test_spire_spear_boss() {
        let mut enemy = create_enemy("SpireSpear", 160, 160);
        assert_eq!(enemy.move_id, move_ids::SPEAR_BURN_STRIKE);
        assert_eq!(enemy.move_damage(), 5);
        assert_eq!(enemy.move_hits(), 2);
        assert_eq!(enemy.entity.status(sid::SKEWER_COUNT), 3);
        assert_eq!(enemy.entity.status(sid::ARTIFACT), 1);

        // Source: reference/extracted/methods/monster/SpireSpear.java. The
        // empty-history opener is Burn Strike, then Skewer, then a boolean.
        let true_seed = (1..10_000)
            .find(|&seed| crate::seed::StsRandom::new(seed).random_bool())
            .unwrap();
        let mut rng = crate::seed::StsRandom::new(true_seed);
        roll_initial_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SPEAR_BURN_STRIKE);

        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SPEAR_SKEWER);
        assert_eq!(enemy.move_damage(), 10);
        assert_eq!(enemy.move_hits(), 3);

        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SPEAR_PIERCER);
        assert_eq!(enemy.effect(mfx::STRENGTH), Some(2));
        assert_eq!(enemy.effect(mfx::STRENGTH_ALL_ALLIES), Some(2));
        assert_eq!(rng.counter, 1);
    }

    #[test]
    fn test_snake_dagger_pattern() {
        let mut enemy = create_enemy("SnakeDagger", 22, 22);
        assert_eq!(enemy.move_id, move_ids::SD_WOUND);
        assert_eq!(enemy.move_damage(), 9);
        assert!(matches!(enemy.intent, Intent::AttackDebuff { .. }));

        let mut rng = crate::seed::StsRandom::new(0);
        roll_initial_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SD_WOUND);
        assert!(matches!(enemy.intent, Intent::AttackDebuff { .. }));
        assert_eq!(enemy.entity.status(sid::FIRST_MOVE), 0);

        roll_next_move(&mut enemy, &mut rng);
        assert_eq!(enemy.move_id, move_ids::SD_EXPLODE);
        assert_eq!(enemy.move_damage(), 25);
    }

    #[test]
    fn test_darkling_pattern() {
        let mut enemy = create_enemy("Darkling", 52, 52);
        assert_eq!(enemy.move_id, move_ids::DARK_NIP);
        assert_eq!(enemy.move_damage(), 8);

        let mut rng = crate::seed::StsRandom::new(0);
        // Java's first getMove call consumes the firstMove flag. Use an
        // explicit Nip opener before exercising the normal move table.
        roll_initial_move_with_num_and_rng(&mut enemy, 72, &mut rng);
        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_eq!(enemy.move_id, move_ids::DARK_CHOMP);
        assert_eq!(enemy.move_hits(), 2);

        // Java rejects a repeated Chomp and rerolls only within 40..=99.
        roll_next_move_with_num_and_rng(&mut enemy, 0, &mut rng);
        assert_ne!(enemy.move_id, move_ids::DARK_CHOMP);
        assert_eq!(rng.counter, 1);
    }

    #[test]
    fn test_exploder_timer() {
        let mut enemy = create_enemy("Exploder", 30, 30);
        assert_eq!(enemy.move_id, move_ids::EXPLODER_ATTACK);
        assert_eq!(enemy.entity.status(sid::EXPLOSIVE), 3);

        // Source: Exploder.java increments turnCount in takeTurn, not getMove.
        enemy.entity.set_status(sid::TURN_COUNT, 1);
        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::EXPLODER_ATTACK);

        enemy.entity.set_status(sid::TURN_COUNT, 2);
        roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
        assert_eq!(enemy.move_id, move_ids::EXPLODER_EXPLODE);
        assert_eq!(enemy.move_damage(), 0);
        assert_eq!(enemy.move_hits(), 0);
    }

    #[test]
    fn test_orb_walker_pattern() {
        let mut enemy = create_enemy("OrbWalker", 93, 93);
        assert_eq!(enemy.move_id, move_ids::OW_LASER);
        assert_eq!(enemy.move_damage(), 10);

        let mut rng = crate::seed::StsRandom::new(0);
        roll_next_move_with_num_and_rng(&mut enemy, 72, &mut rng);
        assert_eq!(enemy.move_id, move_ids::OW_LASER);

        // Java allows two Lasers, then forces Claw instead of a third.
        roll_next_move_with_num_and_rng(&mut enemy, 72, &mut rng);
        assert_eq!(enemy.move_id, move_ids::OW_CLAW);
        assert_eq!(enemy.move_damage(), 15);
    }

    fn java_intent_kind(intent: Intent) -> &'static str {
        match intent {
            Intent::Attack { .. } => "ATTACK",
            Intent::Block { .. } => "DEFEND",
            Intent::Buff { .. } => "BUFF",
            Intent::Debuff { .. } => "DEBUFF",
            Intent::StrongDebuff { .. } => "STRONG_DEBUFF",
            Intent::AttackBlock { .. } => "ATTACK_DEFEND",
            Intent::AttackBuff { .. } => "ATTACK_BUFF",
            Intent::AttackDebuff { .. } => "ATTACK_DEBUFF",
            Intent::DefendBuff { .. } => "DEFEND_BUFF",
            Intent::DefendDebuff { .. } => "DEFEND_DEBUFF",
            Intent::Spawn => "SPAWN",
            Intent::Escape => "ESCAPE",
            Intent::Sleep => "SLEEP",
            Intent::Stun => "STUN",
            Intent::Unknown => "UNKNOWN",
        }
    }

    #[test]
    fn acts_one_through_three_move_ids_match_java_constants() {
        // Source: the `private static final byte` declarations in every class
        // under decompiled/java-src/com/megacrit/cardcrawl/monsters/
        // {exordium,city,beyond}. This guards the numeric trace contract rather
        // than merely comparing Rust symbolic constants to other Rust code.
        macro_rules! ids {
            ($class:literal, [$($actual:expr),+ $(,)?] => [$($expected:expr),+ $(,)?]) => {
                assert_eq!([$($actual),+], [$($expected),+], "{} move IDs", $class);
            };
        }

        ids!("JawWorm", [move_ids::JW_CHOMP, move_ids::JW_BELLOW, move_ids::JW_THRASH] => [1, 2, 3]);
        ids!("Cultist", [move_ids::CULT_DARK_STRIKE, move_ids::CULT_INCANTATION] => [1, 3]);
        ids!("FungiBeast", [move_ids::FB_BITE, move_ids::FB_GROW] => [1, 2]);
        ids!("Louse", [move_ids::LOUSE_BITE, move_ids::LOUSE_GROW, move_ids::LOUSE_SPIT_WEB] => [3, 4, 4]);
        ids!("SlaverBlue", [move_ids::BS_STAB, move_ids::BS_RAKE] => [1, 4]);
        ids!("SlaverRed", [move_ids::RS_STAB, move_ids::RS_ENTANGLE, move_ids::RS_SCRAPE] => [1, 2, 3]);
        ids!("AcidSlime_S", [move_ids::AS_S_TACKLE, move_ids::AS_S_LICK] => [1, 2]);
        ids!("AcidSlime_M/L", [move_ids::AS_CORROSIVE_SPIT, move_ids::AS_TACKLE, move_ids::AS_SPLIT, move_ids::AS_LICK] => [1, 2, 3, 4]);
        ids!("SpikeSlime", [move_ids::SS_TACKLE, move_ids::SS_SPLIT, move_ids::SS_LICK] => [1, 3, 4]);
        ids!("Looter", [move_ids::LOOTER_MUG, move_ids::LOOTER_SMOKE_BOMB, move_ids::LOOTER_ESCAPE, move_ids::LOOTER_LUNGE] => [1, 2, 3, 4]);
        ids!("Gremlin", [move_ids::GREMLIN_ATTACK, move_ids::GREMLIN_PROTECT, move_ids::GREMLIN_ESCAPE] => [1, 2, 99]);
        ids!("GremlinTsundere", [move_ids::GREMLIN_TSUNDERE_PROTECT, move_ids::GREMLIN_TSUNDERE_BASH] => [1, 2]);
        ids!("GremlinNob", [move_ids::NOB_RUSH, move_ids::NOB_SKULL_BASH, move_ids::NOB_BELLOW] => [1, 2, 3]);
        ids!("Lagavulin", [move_ids::LAGA_SIPHON, move_ids::LAGA_ATTACK, move_ids::LAGA_STUN, move_ids::LAGA_SLEEP, move_ids::LAGA_OPEN_NATURAL] => [1, 3, 4, 5, 6]);
        ids!("Sentry", [move_ids::SENTRY_BOLT, move_ids::SENTRY_BEAM] => [3, 4]);
        ids!("TheGuardian", [move_ids::GUARD_CLOSE_UP, move_ids::GUARD_FIERCE_BASH, move_ids::GUARD_ROLL_ATTACK, move_ids::GUARD_TWIN_SLAM, move_ids::GUARD_WHIRLWIND, move_ids::GUARD_CHARGING_UP, move_ids::GUARD_VENT_STEAM] => [1, 2, 3, 4, 5, 6, 7]);
        ids!("Hexaghost", [move_ids::HEX_DIVIDER, move_ids::HEX_TACKLE, move_ids::HEX_INFLAME, move_ids::HEX_SEAR, move_ids::HEX_ACTIVATE, move_ids::HEX_INFERNO] => [1, 2, 3, 4, 5, 6]);
        ids!("ApologySlime", [move_ids::APOLOGY_TACKLE, move_ids::APOLOGY_DEBUFF] => [1, 2]);
        ids!("SlimeBoss", [move_ids::SB_SLAM, move_ids::SB_PREP_SLAM, move_ids::SB_SPLIT, move_ids::SB_STICKY] => [1, 2, 3, 4]);

        ids!("Chosen", [move_ids::CHOSEN_ZAP, move_ids::CHOSEN_DRAIN, move_ids::CHOSEN_DEBILITATE, move_ids::CHOSEN_HEX, move_ids::CHOSEN_POKE] => [1, 2, 3, 4, 5]);
        ids!("Mugger", [move_ids::MUGGER_MUG, move_ids::MUGGER_SMOKE_BOMB, move_ids::MUGGER_ESCAPE, move_ids::MUGGER_BIG_SWIPE] => [1, 2, 3, 4]);
        ids!("Byrd", [move_ids::BYRD_PECK, move_ids::BYRD_FLY_UP, move_ids::BYRD_SWOOP, move_ids::BYRD_STUNNED, move_ids::BYRD_HEADBUTT, move_ids::BYRD_CAW] => [1, 2, 3, 4, 5, 6]);
        ids!("ShelledParasite", [move_ids::SP_FELL, move_ids::SP_DOUBLE_STRIKE, move_ids::SP_LIFE_SUCK, move_ids::SP_STUNNED] => [1, 2, 3, 4]);
        ids!("SnakePlant", [move_ids::SNAKE_CHOMP, move_ids::SNAKE_SPORES] => [1, 2]);
        ids!("Centurion", [move_ids::CENT_SLASH, move_ids::CENT_PROTECT, move_ids::CENT_FURY] => [1, 2, 3]);
        ids!("Healer", [move_ids::MYSTIC_ATTACK, move_ids::MYSTIC_HEAL, move_ids::MYSTIC_BUFF] => [1, 2, 3]);
        ids!("BookOfStabbing", [move_ids::BOOK_STAB, move_ids::BOOK_BIG_STAB] => [1, 2]);
        ids!("GremlinLeader", [move_ids::GL_RALLY, move_ids::GL_ENCOURAGE, move_ids::GL_STAB] => [2, 3, 4]);
        ids!("Taskmaster", [move_ids::TASK_SCOURING_WHIP] => [2]);
        ids!("SphericGuardian", [move_ids::SPHER_BIG_ATTACK, move_ids::SPHER_INITIAL_BLOCK, move_ids::SPHER_BLOCK_ATTACK, move_ids::SPHER_FRAIL_ATTACK] => [1, 2, 3, 4]);
        ids!("Snecko", [move_ids::SNECKO_GLARE, move_ids::SNECKO_BITE, move_ids::SNECKO_TAIL] => [1, 2, 3]);
        ids!("BanditBear", [move_ids::BEAR_MAUL, move_ids::BEAR_HUG, move_ids::BEAR_LUNGE] => [1, 2, 3]);
        ids!("BanditLeader", [move_ids::BANDIT_CROSS_SLASH, move_ids::BANDIT_MOCK, move_ids::BANDIT_AGONIZE] => [1, 2, 3]);
        ids!("BanditPointy", [move_ids::POINTY_STAB] => [1]);
        ids!("BronzeAutomaton", [move_ids::BA_FLAIL, move_ids::BA_HYPER_BEAM, move_ids::BA_STUNNED, move_ids::BA_SPAWN_ORBS, move_ids::BA_BOOST] => [1, 2, 3, 4, 5]);
        ids!("BronzeOrb", [move_ids::BO_BEAM, move_ids::BO_SUPPORT, move_ids::BO_STASIS] => [1, 2, 3]);
        ids!("TorchHead", [move_ids::TORCH_TACKLE] => [1]);
        ids!("Champ", [move_ids::CHAMP_HEAVY_SLASH, move_ids::CHAMP_DEFENSIVE, move_ids::CHAMP_EXECUTE, move_ids::CHAMP_FACE_SLAP, move_ids::CHAMP_GLOAT, move_ids::CHAMP_TAUNT, move_ids::CHAMP_ANGER] => [1, 2, 3, 4, 5, 6, 7]);
        ids!("TheCollector", [move_ids::COLL_SPAWN, move_ids::COLL_FIREBALL, move_ids::COLL_BUFF, move_ids::COLL_MEGA_DEBUFF, move_ids::COLL_REVIVE] => [1, 2, 3, 4, 5]);

        ids!("Darkling", [move_ids::DARK_CHOMP, move_ids::DARK_HARDEN, move_ids::DARK_NIP, move_ids::DARK_WAIT, move_ids::DARK_REINCARNATE] => [1, 2, 3, 4, 5]);
        ids!("OrbWalker", [move_ids::OW_LASER, move_ids::OW_CLAW] => [1, 2]);
        ids!("Spiker", [move_ids::SPIKER_ATTACK, move_ids::SPIKER_BUFF] => [1, 2]);
        ids!("Repulsor", [move_ids::REPULSOR_DAZE, move_ids::REPULSOR_ATTACK] => [1, 2]);
        ids!("Exploder", [move_ids::EXPLODER_ATTACK, move_ids::EXPLODER_EXPLODE] => [1, 2]);
        ids!("WrithingMass", [move_ids::WM_BIG_HIT, move_ids::WM_MULTI_HIT, move_ids::WM_ATTACK_BLOCK, move_ids::WM_ATTACK_DEBUFF, move_ids::WM_MEGA_DEBUFF] => [0, 1, 2, 3, 4]);
        ids!("SpireGrowth", [move_ids::SG_QUICK_TACKLE, move_ids::SG_CONSTRICT, move_ids::SG_SMASH] => [1, 2, 3]);
        ids!("Maw", [move_ids::MAW_ROAR, move_ids::MAW_SLAM, move_ids::MAW_DROOL, move_ids::MAW_NOM] => [2, 3, 4, 5]);
        ids!("Transient", [move_ids::TRANSIENT_ATTACK] => [1]);
        ids!("GiantHead", [move_ids::GH_GLARE, move_ids::GH_IT_IS_TIME, move_ids::GH_COUNT] => [1, 2, 3]);
        ids!("Nemesis", [move_ids::NEM_TRI_ATTACK, move_ids::NEM_SCYTHE, move_ids::NEM_BURN] => [2, 3, 4]);
        ids!("Reptomancer", [move_ids::REPTO_SNAKE_STRIKE, move_ids::REPTO_SPAWN, move_ids::REPTO_BIG_BITE] => [1, 2, 3]);
        ids!("SnakeDagger", [move_ids::SD_WOUND, move_ids::SD_EXPLODE] => [1, 2]);
        ids!("AwakenedOne", [move_ids::AO_SLASH, move_ids::AO_SOUL_STRIKE, move_ids::AO_REBIRTH, move_ids::AO_DARK_ECHO, move_ids::AO_SLUDGE, move_ids::AO_TACKLE] => [1, 2, 3, 5, 6, 8]);
        ids!("Donu", [move_ids::DONU_BEAM, move_ids::DONU_CIRCLE] => [0, 2]);
        ids!("Deca", [move_ids::DECA_BEAM, move_ids::DECA_SQUARE] => [0, 2]);
        ids!("TimeEater", [move_ids::TE_REVERBERATE, move_ids::TE_RIPPLE, move_ids::TE_HEAD_SLAM, move_ids::TE_HASTE] => [2, 3, 4, 5]);
    }

    #[test]
    fn source_special_intent_constructor_table_matches_java() {
        // Numeric IDs and intent kinds come from each class's setMove call in
        // decompiled/java-src/com/megacrit/cardcrawl/monsters/{exordium,city,beyond}.
        // This table intentionally covers constructor/provisional moves whose
        // Java intent cannot be inferred from positive damage or self Block.
        let rows = [
            ("AcidSlime_M", 1, "ATTACK_DEBUFF"),
            ("AcidSlime_L", 1, "ATTACK_DEBUFF"),
            ("SpikeSlime_M", 1, "ATTACK_DEBUFF"),
            ("SpikeSlime_L", 1, "ATTACK_DEBUFF"),
            ("GremlinFat", 2, "ATTACK_DEBUFF"),
            ("GremlinNob", 3, "BUFF"),
            ("GremlinWizard", 2, "UNKNOWN"),
            ("GremlinTsundere", 1, "DEFEND"),
            ("Lagavulin", 5, "SLEEP"),
            ("Sentry", 3, "DEBUFF"),
            ("Hexaghost", 5, "UNKNOWN"),
            ("SlimeBoss", 4, "STRONG_DEBUFF"),
            ("Healer", 1, "ATTACK_DEBUFF"),
            ("GremlinLeader", 2, "UNKNOWN"),
            ("SlaverBoss", 2, "ATTACK_DEBUFF"),
            ("Snecko", 1, "STRONG_DEBUFF"),
            ("BanditBear", 2, "STRONG_DEBUFF"),
            ("BanditLeader", 2, "UNKNOWN"),
            ("BronzeAutomaton", 4, "UNKNOWN"),
            ("BronzeOrb", 3, "STRONG_DEBUFF"),
            ("Champ", 4, "ATTACK_DEBUFF"),
            ("TheCollector", 1, "UNKNOWN"),
            ("Orb Walker", 1, "ATTACK_DEBUFF"),
            ("Repulsor", 1, "DEBUFF"),
            ("Maw", 2, "STRONG_DEBUFF"),
            ("Reptomancer", 2, "UNKNOWN"),
            ("Dagger", 1, "ATTACK_DEBUFF"),
            ("Deca", 0, "ATTACK_DEBUFF"),
        ];

        for (enemy_id, move_id, expected_intent) in rows {
            let enemy = create_enemy(enemy_id, 100, 100);
            assert_eq!(enemy.move_id, move_id, "{enemy_id} Java move ID drifted");
            assert_eq!(
                java_intent_kind(enemy.intent),
                expected_intent,
                "{enemy_id} Java intent drifted"
            );
        }
    }

    #[test]
    fn source_special_intent_roll_branches_match_java() {
        // Source: getMove in SlaverBlue, SlaverRed, AcidSlime_M,
        // SpikeSlime_M, GremlinNob, Lagavulin, Hexaghost, SnakePlant, Healer,
        // GremlinLeader, SphericGuardian, BronzeAutomaton, BronzeOrb, Darkling,
        // Exploder, WrithingMass, SpireGrowth, GiantHead, Nemesis,
        // AwakenedOne, and Deca under the same Java monster directories.
        macro_rules! check {
            ($enemy:expr, $move_id:expr, $kind:literal) => {{
                let enemy = &$enemy;
                assert_eq!(enemy.move_id, $move_id);
                assert_eq!(java_intent_kind(enemy.intent), $kind);
            }};
        }

        let mut blue = create_enemy("SlaverBlue", 100, 100);
        act1::roll_blue_slaver(&mut blue, 0);
        check!(blue, move_ids::BS_RAKE, "ATTACK_DEBUFF");

        let mut red = create_enemy("SlaverRed", 100, 100);
        red.entity.set_status(sid::IS_FIRST_MOVE, 0);
        act1::roll_red_slaver(&mut red, 75);
        check!(red, move_ids::RS_ENTANGLE, "STRONG_DEBUFF");

        let mut acid = create_enemy("AcidSlime_M", 100, 100);
        act1::roll_acid_slime_m(&mut acid, 0, &mut crate::seed::StsRandom::new(0));
        check!(acid, move_ids::AS_CORROSIVE_SPIT, "ATTACK_DEBUFF");

        let mut spike = create_enemy("SpikeSlime_M", 100, 100);
        act1::roll_spike_slime_m(&mut spike, 99);
        check!(spike, move_ids::SS_LICK, "DEBUFF");

        let mut nob = create_enemy("GremlinNob", 100, 100);
        nob.entity.set_status(sid::IS_FIRST_MOVE, 1);
        act1::roll_gremlin_nob(&mut nob, 0);
        check!(nob, move_ids::NOB_SKULL_BASH, "ATTACK_DEBUFF");

        let mut lagavulin = create_enemy("Lagavulin", 100, 100);
        lagavulin.entity.set_status(sid::FIRST_MOVE, 1);
        act1::roll_lagavulin(&mut lagavulin);
        check!(lagavulin, move_ids::LAGA_SIPHON, "STRONG_DEBUFF");

        let mut hexaghost = create_enemy("Hexaghost", 100, 100);
        hexaghost.entity.set_status(sid::IS_FIRST_MOVE, 1);
        act1::roll_hexaghost(&mut hexaghost);
        check!(hexaghost, move_ids::HEX_SEAR, "ATTACK_DEBUFF");

        let mut snake = create_enemy("SnakePlant", 100, 100);
        act2::roll_snake_plant(&mut snake, 99);
        check!(snake, move_ids::SNAKE_SPORES, "STRONG_DEBUFF");

        let mut healer = create_enemy("Healer", 100, 100);
        act2::roll_healer(&mut healer, 99);
        check!(healer, move_ids::MYSTIC_ATTACK, "ATTACK_DEBUFF");

        let mut leader = create_enemy("GremlinLeader", 100, 100);
        leader.entity.set_status(sid::COUNT, 0);
        act2::roll_gremlin_leader(&mut leader, 0, &mut crate::seed::StsRandom::new(0));
        check!(leader, move_ids::GL_RALLY, "UNKNOWN");

        let mut guardian = create_enemy("SphericGuardian", 100, 100);
        guardian.entity.set_status(sid::FIRST_MOVE, 0);
        guardian.entity.set_status(sid::FIRST_TURN, 1);
        act2::roll_spheric_guardian(&mut guardian);
        check!(guardian, move_ids::SPHER_FRAIL_ATTACK, "ATTACK_DEBUFF");

        let mut automaton = create_enemy("BronzeAutomaton", 100, 100);
        act2::roll_bronze_automaton(&mut automaton, 0);
        check!(automaton, move_ids::BA_SPAWN_ORBS, "UNKNOWN");

        let mut orb = create_enemy("BronzeOrb", 100, 100);
        act2::roll_bronze_orb(&mut orb, 25);
        check!(orb, move_ids::BO_STASIS, "STRONG_DEBUFF");

        let mut darkling = create_enemy("Darkling", 100, 100);
        darkling.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        act3::roll_darkling(&mut darkling, 0, &mut crate::seed::StsRandom::new(0));
        check!(darkling, move_ids::DARK_HARDEN, "DEFEND_BUFF");

        let mut exploder = create_enemy("Exploder", 100, 100);
        exploder.entity.set_status(sid::TURN_COUNT, 2);
        act3::roll_exploder(&mut exploder, 0);
        check!(exploder, move_ids::EXPLODER_EXPLODE, "UNKNOWN");

        let mut mass = create_enemy("WrithingMass", 100, 100);
        mass.entity.set_status(sid::FIRST_MOVE, 0);
        act3::roll_writhing_mass(&mut mass, 10, &mut crate::seed::StsRandom::new(0));
        check!(mass, move_ids::WM_MEGA_DEBUFF, "STRONG_DEBUFF");

        let mut growth = create_enemy("Serpent", 100, 100);
        growth.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        act3::roll_spire_growth(&mut growth, 0);
        check!(growth, move_ids::SG_CONSTRICT, "STRONG_DEBUFF");

        let mut head = create_enemy("GiantHead", 100, 100);
        act3::roll_giant_head(&mut head, 0);
        check!(head, move_ids::GH_GLARE, "DEBUFF");

        let mut nemesis = create_enemy("Nemesis", 100, 100);
        act3::roll_nemesis(&mut nemesis, 99, &mut crate::seed::StsRandom::new(0));
        check!(nemesis, move_ids::NEM_BURN, "DEBUFF");

        let mut awakened = create_enemy("AwakenedOne", 100, 100);
        awakened.entity.set_status(sid::PHASE, 2);
        awakened.move_history.push(move_ids::AO_DARK_ECHO);
        act3::roll_awakened_one(&mut awakened, 0);
        check!(awakened, move_ids::AO_SLUDGE, "ATTACK_DEBUFF");

        let mut deca = create_enemy("Deca", 100, 100);
        deca.move_history.clear();
        act3::roll_deca(&mut deca, 0);
        check!(deca, move_ids::DECA_BEAM, "ATTACK_DEBUFF");
    }

    /// Test all enemy IDs can be created without panicking
    #[test]
    fn test_all_enemies_create() {
        for &(id, _) in known_enemy_ids() {
            let enemy = create_enemy(id, 100, 100);
            assert_eq!(enemy.id, id, "Enemy ID mismatch for {id}");
        }
        assert_eq!(known_enemy_ids().len(), 66);
    }

    /// Test all enemies can roll at least 5 moves without panicking
    #[test]
    fn test_all_enemies_roll() {
        for &(id, _) in known_enemy_ids() {
            let mut enemy = create_enemy(id, 100, 100);
            for _ in 0..5 {
                roll_next_move(&mut enemy, &mut crate::seed::StsRandom::new(0));
            }
        }
    }

    #[test]
    fn gameplay_lookup_uses_canonical_registry() {
        let def = super::gameplay_def("JawWorm").expect("enemy gameplay def");
        assert_eq!(def.domain, crate::gameplay::GameplayDomain::Enemy);
        assert_eq!(def.id, "JawWorm");
        assert!(def.enemy_schema().is_some());
    }
}
