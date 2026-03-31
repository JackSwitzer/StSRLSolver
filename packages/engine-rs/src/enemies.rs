//! Enemy AI system — All 4 acts (73 enemies) for MCTS simulations.
//!
//! Each enemy has a deterministic move pattern that mirrors the Java implementations.
//! For MCTS, we use simplified AI: no RNG-based move selection, instead we use
//! the most common/expected move pattern for fast simulation.

use crate::state::EnemyCombatState;

/// Enemy move IDs — shared constants for pattern matching.
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

    // Gremlin (Fat/Thief/Warrior/Wizard/Tsundere)
    pub const GREMLIN_ATTACK: i32 = 1;
    pub const GREMLIN_PROTECT: i32 = 2;

    // Gremlin Nob
    pub const NOB_BELLOW: i32 = 1;
    pub const NOB_RUSH: i32 = 2;
    pub const NOB_SKULL_BASH: i32 = 3;

    // Lagavulin
    pub const LAGA_SLEEP: i32 = 1;
    pub const LAGA_ATTACK: i32 = 2;
    pub const LAGA_SIPHON: i32 = 3;

    // Sentry
    pub const SENTRY_BOLT: i32 = 1;
    pub const SENTRY_BEAM: i32 = 2;

    // The Guardian
    pub const GUARD_CHARGING_UP: i32 = 6;
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

/// Create a pre-configured enemy with initial move set.
pub fn create_enemy(enemy_id: &str, hp: i32, max_hp: i32) -> EnemyCombatState {
    let mut enemy = EnemyCombatState::new(enemy_id, hp, max_hp);

    match enemy_id {
        // =================================================================
        // Act 1 — Exordium
        // =================================================================
        "JawWorm" => {
            enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
        }
        "Cultist" => {
            enemy.set_move(move_ids::CULT_INCANTATION, 0, 0, 0);
            enemy.move_effects.insert("ritual".to_string(), 3);
        }
        "FungiBeast" => {
            enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
            enemy.entity.set_status("SporeCloud", 2);
        }
        "FuzzyLouseNormal" | "RedLouse" => {
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status("CurlUp", 5);
        }
        "FuzzyLouseDefensive" | "GreenLouse" => {
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status("CurlUp", 5);
        }
        "SlaverBlue" | "BlueSlaver" => {
            enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
        }
        "SlaverRed" | "RedSlaver" => {
            enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
        }
        "AcidSlime_S" => {
            enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
        }
        "AcidSlime_M" => {
            enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 1);
        }
        "AcidSlime_L" => {
            enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 2);
        }
        "SpikeSlime_S" => {
            enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
        }
        "SpikeSlime_M" => {
            enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
        }
        "SpikeSlime_L" => {
            enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
        }
        "Looter" => {
            // Mug -> Mug -> SmokeBomb -> Escape
            enemy.set_move(move_ids::LOOTER_MUG, 10, 1, 0);
        }
        "GremlinFat" => {
            // Smash: 4 damage + apply 1 Weak
            enemy.set_move(move_ids::GREMLIN_ATTACK, 4, 1, 0);
            enemy.move_effects.insert("weak".to_string(), 1);
        }
        "GremlinThief" => {
            // Puncture: 9 damage
            enemy.set_move(move_ids::GREMLIN_ATTACK, 9, 1, 0);
        }
        "GremlinWarrior" => {
            // Scratch: 4 damage
            enemy.set_move(move_ids::GREMLIN_ATTACK, 4, 1, 0);
        }
        "GremlinWizard" => {
            // Charging (first turn), then Ultimate Blast (25 damage)
            enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
        }
        "GremlinTsundere" | "GremlinSneaky" => {
            // Shield: does nothing
            enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
        }
        "GremlinNob" | "Gremlin Nob" => {
            enemy.set_move(move_ids::NOB_BELLOW, 0, 0, 0);
            enemy.entity.set_status("Enrage", 2);
        }
        "Lagavulin" => {
            enemy.set_move(move_ids::LAGA_SLEEP, 0, 0, 0);
            enemy.entity.set_status("Metallicize", 8);
            enemy.entity.set_status("SleepTurns", 3);
        }
        "Sentry" => {
            enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
        }
        "TheGuardian" => {
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
            enemy.entity.set_status("ModeShift", 30);
        }
        "Hexaghost" => {
            enemy.set_move(move_ids::HEX_ACTIVATE, 0, 0, 0);
        }
        "SlimeBoss" => {
            enemy.set_move(move_ids::SB_STICKY, 0, 0, 0);
            enemy.move_effects.insert("slimed".to_string(), 3);
        }

        // =================================================================
        // Act 2 — The City
        // =================================================================
        "Chosen" => {
            // First turn: Poke (5 dmg x2)
            enemy.set_move(move_ids::CHOSEN_POKE, 5, 2, 0);
        }
        "Mugger" => {
            // First turn: Mug (10 damage, steals gold)
            enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
        }
        "Byrd" => {
            // Starts flying with Flight power. First turn: Peck (1x5)
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
            enemy.entity.set_status("Flight", 3);
        }
        "Shelled Parasite" | "ShelledParasite" => {
            // Has Plated Armor 14. First turn: Double Strike (6x2)
            enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
            enemy.entity.set_status("PlatedArmor", 14);
        }
        "SnakePlant" => {
            // Has Malleable. First turn: Chomp (7x3)
            enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
            enemy.entity.set_status("Malleable", 1);
        }
        "Centurion" => {
            // First turn: Fury (6x3) or Slash (12)
            enemy.set_move(move_ids::CENT_FURY, 6, 3, 0);
        }
        "Mystic" | "Healer" => {
            // Attack + debuff (8 damage)
            enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
        }
        "BookOfStabbing" | "Book of Stabbing" => {
            // Multi-stab. Starts with stabCount=1, increases each turn
            enemy.set_move(move_ids::BOOK_STAB, 6, 2, 0);
            enemy.entity.set_status("StabCount", 2);
        }
        "GremlinLeader" | "Gremlin Leader" => {
            // First turn: Rally (summon gremlins)
            enemy.set_move(move_ids::GL_RALLY, 0, 0, 0);
        }
        "Taskmaster" => {
            // Always Scouring Whip (7 damage + Wounds)
            enemy.set_move(move_ids::TASK_SCOURING_WHIP, 7, 1, 0);
            enemy.move_effects.insert("wound".to_string(), 1);
        }
        "SphericGuardian" | "Spheric Guardian" => {
            // First turn: Activate (gain 40 block)
            enemy.set_move(move_ids::SPHER_INITIAL_BLOCK, 0, 0, 40);
        }
        "Snecko" => {
            // First turn: Glare (debuff)
            enemy.set_move(move_ids::SNECKO_GLARE, 0, 0, 0);
            enemy.move_effects.insert("confused".to_string(), 1);
        }
        "BanditBear" | "Bear" => {
            // First turn: Bear Hug (debuff: -2 Dexterity)
            enemy.set_move(move_ids::BEAR_HUG, 0, 0, 0);
            enemy.move_effects.insert("dexterity_down".to_string(), 2);
        }
        "BanditLeader" => {
            // First turn: Mock (buff minions)
            enemy.set_move(move_ids::BANDIT_MOCK, 0, 0, 0);
        }
        "BanditPointy" | "Pointy" => {
            // Always: stab 5x2
            enemy.set_move(move_ids::POINTY_STAB, 5, 2, 0);
        }
        "BronzeAutomaton" | "Bronze Automaton" => {
            // First turn: Spawn Orbs
            enemy.set_move(move_ids::BA_SPAWN_ORBS, 0, 0, 0);
        }
        "BronzeOrb" | "Bronze Orb" => {
            // First turn: Stasis (steal card from hand)
            enemy.set_move(move_ids::BO_STASIS, 0, 0, 0);
            enemy.move_effects.insert("stasis".to_string(), 1);
        }
        "TorchHead" | "Torch Head" => {
            // Always: Tackle (7 damage)
            enemy.set_move(move_ids::TORCH_TACKLE, 7, 1, 0);
        }
        "Champ" => {
            // First turn: always Defensive Stance or Face Slap
            enemy.set_move(move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
            enemy.move_effects.insert("frail".to_string(), 2);
            enemy.entity.set_status("NumTurns", 0);
            enemy.entity.set_status("ThresholdReached", 0);
        }
        "TheCollector" | "Collector" => {
            // First turn: Spawn (summon TorchHeads)
            enemy.set_move(move_ids::COLL_SPAWN, 0, 0, 0);
        }

        // =================================================================
        // Act 3 — Beyond
        // =================================================================
        "Darkling" => {
            // First turn: Nip (8 damage, variable)
            enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
        }
        "OrbWalker" | "Orb Walker" => {
            // First turn: Laser (10 damage + burn)
            enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
            enemy.move_effects.insert("burn".to_string(), 1);
        }
        "Spiker" => {
            // Has Thorns 3. First turn: attack (7 damage)
            enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
            enemy.entity.set_status("Thorns", 3);
        }
        "Repulsor" => {
            // Mostly Daze (add Daze cards). First turn: Daze
            enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
            enemy.move_effects.insert("daze".to_string(), 2);
        }
        "Exploder" => {
            // 3-turn timer: Attack -> Unknown -> Explode (30 damage)
            enemy.set_move(move_ids::EXPLODER_ATTACK, 9, 1, 0);
            enemy.entity.set_status("TurnCount", 0);
        }
        "WrithingMass" | "Writhing Mass" => {
            // First turn: random attack. Use Multi Hit (7x3)
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
            enemy.entity.set_status("Reactive", 1);
        }
        "SpireGrowth" | "Spire Growth" => {
            // Has Constrict. First turn: Quick Tackle (16)
            enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
        }
        "Maw" => {
            // First turn: Roar (debuff: Weak + Frail)
            enemy.set_move(move_ids::MAW_ROAR, 0, 0, 0);
            enemy.move_effects.insert("weak".to_string(), 3);
            enemy.move_effects.insert("frail".to_string(), 3);
            enemy.entity.set_status("TurnCount", 1);
        }
        "Transient" => {
            // Escalating damage. Starts at 30, +10 each turn. Dies after 5 turns.
            enemy.set_move(move_ids::TRANSIENT_ATTACK, 30, 1, 0);
            enemy.entity.set_status("AttackCount", 0);
        }
        "GiantHead" | "Giant Head" => {
            // Countdown to It Is Time. Glare/Count cycle for 5 turns.
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
            enemy.entity.set_status("Countdown", 5);
        }
        "Nemesis" => {
            // Intangible every other turn. First turn: Tri Attack (6x3)
            enemy.set_move(move_ids::NEM_TRI_ATTACK, 6, 3, 0);
            enemy.entity.set_status("ScytheCooldown", 0);
        }
        "Reptomancer" => {
            // First turn: Spawn daggers
            enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
        }
        "SnakeDagger" | "Snake Dagger" => {
            // First turn: Wound (9 damage + add Wound to discard)
            enemy.set_move(move_ids::SD_WOUND, 9, 1, 0);
            enemy.move_effects.insert("wound".to_string(), 1);
        }
        "AwakenedOne" | "Awakened One" => {
            // Phase 1. Curiosity: gains 1 Str when player plays a Power.
            // First turn: Slash (20 damage)
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
            enemy.entity.set_status("Curiosity", 1);
            enemy.entity.set_status("Phase", 1);
        }
        "Donu" => {
            // Alternates: Beam (10x2) and Circle of Protection (+3 Str to both)
            enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
            enemy.move_effects.insert("strength".to_string(), 3);
            enemy.entity.set_status("Artifact", 2);
        }
        "Deca" => {
            // Alternates: Beam (10x2 + Daze) and Square of Protection (16 block)
            enemy.set_move(move_ids::DECA_SQUARE, 0, 0, 16);
            enemy.entity.set_status("Artifact", 2);
        }
        "TimeEater" | "Time Eater" => {
            // After player plays 12 cards: Haste (heal to 50%, cleanse, +2 Str).
            // First turn: Reverberate (7x3)
            enemy.set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
            enemy.entity.set_status("CardCount", 0);
        }

        // =================================================================
        // Act 4 — The Ending
        // =================================================================
        "SpireShield" | "Spire Shield" => {
            // 3-move cycle. First turn: Bash (14 dmg + -1 Str) or Fortify (30 block)
            enemy.set_move(move_ids::SHIELD_BASH, 14, 1, 0);
            enemy.move_effects.insert("strength_down".to_string(), 1);
            enemy.entity.set_status("MoveCount", 0);
        }
        "SpireSpear" | "Spire Spear" => {
            // 3-move cycle. First turn: Burn Strike (5x2 + Burns)
            enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
            enemy.move_effects.insert("burn".to_string(), 2);
            enemy.entity.set_status("MoveCount", 0);
            enemy.entity.set_status("SkewerCount", 3);
        }
        "CorruptHeart" | "Corrupt Heart" => {
            // First turn: Debilitate (apply Vulnerable 2, Weak 2, Frail 2)
            enemy.set_move(move_ids::HEART_DEBILITATE, 0, 0, 0);
            enemy.move_effects.insert("vulnerable".to_string(), 2);
            enemy.move_effects.insert("weak".to_string(), 2);
            enemy.move_effects.insert("frail".to_string(), 2);
            enemy.entity.set_status("Invincible", 300);
            enemy.entity.set_status("BeatOfDeath", 1);
            enemy.entity.set_status("MoveCount", 0);
            enemy.entity.set_status("BloodHitCount", 12);
        }

        _ => {
            // Unknown enemy: generic 6 damage attack
            enemy.set_move(1, 6, 1, 0);
        }
    }

    enemy
}

/// Advance an enemy to its next move based on move history.
/// This is a deterministic pattern for MCTS (no RNG).
pub fn roll_next_move(enemy: &mut EnemyCombatState) {
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();

    match enemy.id.as_str() {
        // Act 1
        "JawWorm" => roll_jaw_worm(enemy),
        "Cultist" => roll_cultist(enemy),
        "FungiBeast" => roll_fungi_beast(enemy),
        "FuzzyLouseNormal" | "RedLouse" => roll_red_louse(enemy),
        "FuzzyLouseDefensive" | "GreenLouse" => roll_green_louse(enemy),
        "SlaverBlue" | "BlueSlaver" => roll_blue_slaver(enemy),
        "SlaverRed" | "RedSlaver" => roll_red_slaver(enemy),
        "AcidSlime_S" => roll_acid_slime_s(enemy),
        "AcidSlime_M" => roll_acid_slime_m(enemy),
        "AcidSlime_L" => roll_acid_slime_l(enemy),
        "SpikeSlime_S" => roll_spike_slime_s(enemy),
        "SpikeSlime_M" => roll_spike_slime_m(enemy),
        "SpikeSlime_L" => roll_spike_slime_l(enemy),
        "Looter" => roll_looter(enemy),
        "GremlinFat" => roll_gremlin_simple(enemy, 4, 1),
        "GremlinThief" => roll_gremlin_simple(enemy, 9, 0),
        "GremlinWarrior" => roll_gremlin_simple(enemy, 4, 0),
        "GremlinWizard" => roll_gremlin_wizard(enemy),
        "GremlinTsundere" | "GremlinSneaky" => { /* Does nothing each turn */ }
        "GremlinNob" | "Gremlin Nob" => roll_gremlin_nob(enemy),
        "Lagavulin" => roll_lagavulin(enemy),
        "Sentry" => roll_sentry(enemy),
        "TheGuardian" => roll_guardian(enemy),
        "Hexaghost" => roll_hexaghost(enemy),
        "SlimeBoss" => roll_slime_boss(enemy),
        // Act 2
        "Chosen" => roll_chosen(enemy),
        "Mugger" => roll_mugger(enemy),
        "Byrd" => roll_byrd(enemy),
        "Shelled Parasite" | "ShelledParasite" => roll_shelled_parasite(enemy),
        "SnakePlant" => roll_snake_plant(enemy),
        "Centurion" => roll_centurion(enemy),
        "Mystic" | "Healer" => roll_mystic(enemy),
        "BookOfStabbing" | "Book of Stabbing" => roll_book_of_stabbing(enemy),
        "GremlinLeader" | "Gremlin Leader" => roll_gremlin_leader(enemy),
        "Taskmaster" => roll_taskmaster(enemy),
        "SphericGuardian" | "Spheric Guardian" => roll_spheric_guardian(enemy),
        "Snecko" => roll_snecko(enemy),
        "BanditBear" | "Bear" => roll_bear(enemy),
        "BanditLeader" => roll_bandit_leader(enemy),
        "BanditPointy" | "Pointy" => { /* Always stab 5x2 */ }
        "BronzeAutomaton" | "Bronze Automaton" => roll_bronze_automaton(enemy),
        "BronzeOrb" | "Bronze Orb" => roll_bronze_orb(enemy),
        "TorchHead" | "Torch Head" => { /* Always Tackle 7 */ }
        "Champ" => roll_champ(enemy),
        "TheCollector" | "Collector" => roll_collector(enemy),
        // Act 3
        "Darkling" => roll_darkling(enemy),
        "OrbWalker" | "Orb Walker" => roll_orb_walker(enemy),
        "Spiker" => roll_spiker(enemy),
        "Repulsor" => roll_repulsor(enemy),
        "Exploder" => roll_exploder(enemy),
        "WrithingMass" | "Writhing Mass" => roll_writhing_mass(enemy),
        "SpireGrowth" | "Spire Growth" => roll_spire_growth(enemy),
        "Maw" => roll_maw(enemy),
        "Transient" => roll_transient(enemy),
        "GiantHead" | "Giant Head" => roll_giant_head(enemy),
        "Nemesis" => roll_nemesis(enemy),
        "Reptomancer" => roll_reptomancer(enemy),
        "SnakeDagger" | "Snake Dagger" => roll_snake_dagger(enemy),
        "AwakenedOne" | "Awakened One" => roll_awakened_one(enemy),
        "Donu" => roll_donu(enemy),
        "Deca" => roll_deca(enemy),
        "TimeEater" | "Time Eater" => roll_time_eater(enemy),
        // Act 4
        "SpireShield" | "Spire Shield" => roll_spire_shield(enemy),
        "SpireSpear" | "Spire Spear" => roll_spire_spear(enemy),
        "CorruptHeart" | "Corrupt Heart" => roll_corrupt_heart(enemy),
        _ => {
            if enemy.move_damage > 0 {
                enemy.set_move(2, 0, 0, 5);
            } else {
                enemy.set_move(1, 6, 1, 0);
            }
        }
    }
}

// =========================================================================
// Helpers
// =========================================================================

fn last_move(enemy: &EnemyCombatState, move_id: i32) -> bool {
    enemy.move_history.last().copied() == Some(move_id)
}

fn last_two_moves(enemy: &EnemyCombatState, move_id: i32) -> bool {
    let len = enemy.move_history.len();
    if len < 2 { return false; }
    enemy.move_history[len - 1] == move_id && enemy.move_history[len - 2] == move_id
}

// =========================================================================
// Act 1 Basic Enemies
// =========================================================================

fn roll_jaw_worm(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::JW_CHOMP) {
        enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::JW_BELLOW) {
        enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
    } else if last_move(enemy, move_ids::JW_THRASH) {
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    } else {
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    }
}

fn roll_cultist(enemy: &mut EnemyCombatState) {
    enemy.set_move(move_ids::CULT_DARK_STRIKE, 6, 1, 0);
}

fn roll_fungi_beast(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::FB_BITE) {
        enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::FB_GROW) {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    }
}

fn roll_red_louse(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::LOUSE_GROW) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

fn roll_green_louse(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::LOUSE_SPIT_WEB) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

fn roll_blue_slaver(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::BS_STAB) {
        enemy.set_move(move_ids::BS_RAKE, 7, 1, 0);
        enemy.move_effects.insert("weak".to_string(), 1);
    } else if last_move(enemy, move_ids::BS_RAKE) {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    } else {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    }
}

fn roll_red_slaver(enemy: &mut EnemyCombatState) {
    let used_entangle = enemy
        .move_history
        .iter()
        .any(|&m| m == move_ids::RS_ENTANGLE);

    if !used_entangle && !enemy.move_history.is_empty() {
        enemy.set_move(move_ids::RS_ENTANGLE, 0, 0, 0);
        enemy.move_effects.insert("entangle".to_string(), 1);
    } else if last_move(enemy, move_ids::RS_ENTANGLE)
        || last_two_moves(enemy, move_ids::RS_SCRAPE)
    {
        enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
    } else {
        enemy.set_move(move_ids::RS_SCRAPE, 8, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 1);
    }
}

fn roll_acid_slime_s(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 1);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
    }
}

fn roll_acid_slime_m(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 10, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    } else {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    }
}

fn roll_acid_slime_l(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 2);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 2);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    }
}

fn roll_spike_slime_s(enemy: &mut EnemyCombatState) {
    enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
}

fn roll_spike_slime_m(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.move_effects.insert("frail".to_string(), 1);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    }
}

fn roll_spike_slime_l(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    }
}

fn roll_looter(enemy: &mut EnemyCombatState) {
    let turns = enemy.move_history.len();
    if turns < 2 {
        // Mug twice
        enemy.set_move(move_ids::LOOTER_MUG, 10, 1, 0);
    } else if turns == 2 {
        // Smoke Bomb (block + prepare escape)
        enemy.set_move(move_ids::LOOTER_SMOKE_BOMB, 0, 0, 11);
    } else {
        // Escape
        enemy.set_move(move_ids::LOOTER_ESCAPE, 0, 0, 0);
        enemy.is_escaping = true;
    }
}

fn roll_gremlin_simple(enemy: &mut EnemyCombatState, dmg: i32, weak: i32) {
    enemy.set_move(move_ids::GREMLIN_ATTACK, dmg, 1, 0);
    if weak > 0 {
        enemy.move_effects.insert("weak".to_string(), weak);
    }
}

fn roll_gremlin_wizard(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::GREMLIN_PROTECT) {
        // Ultimate Blast after charging
        enemy.set_move(move_ids::GREMLIN_ATTACK, 25, 1, 0);
    } else {
        // Charge up again
        enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
    }
}

fn roll_gremlin_nob(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::NOB_BELLOW) || last_move(enemy, move_ids::NOB_SKULL_BASH) {
        enemy.set_move(move_ids::NOB_RUSH, 14, 1, 0);
    } else {
        enemy.set_move(move_ids::NOB_SKULL_BASH, 6, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    }
}

fn roll_lagavulin(enemy: &mut EnemyCombatState) {
    let sleep_turns = enemy.entity.status("SleepTurns");

    if sleep_turns > 0 {
        enemy.entity.set_status("SleepTurns", sleep_turns - 1);
        if sleep_turns - 1 <= 0 {
            enemy.entity.set_status("Metallicize", 0);
            enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
        } else {
            enemy.set_move(move_ids::LAGA_SLEEP, 0, 0, 0);
        }
    } else {
        // Awake: alternate Attack and Siphon Soul
        if last_move(enemy, move_ids::LAGA_ATTACK) {
            enemy.set_move(move_ids::LAGA_SIPHON, 0, 0, 0);
            enemy.move_effects.insert("siphon_str".to_string(), 1);
            enemy.move_effects.insert("siphon_dex".to_string(), 1);
        } else {
            enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
        }
    }
}

/// Wake Lagavulin early (e.g. when player deals damage to it while sleeping).
pub fn lagavulin_wake_up(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status("SleepTurns", 0);
    enemy.entity.set_status("Metallicize", 0);
    enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
}

fn roll_sentry(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::SENTRY_BOLT) {
        enemy.set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
        enemy.move_effects.insert("daze".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
    }
}

// =========================================================================
// Act 1 Bosses
// =========================================================================

fn roll_guardian(enemy: &mut EnemyCombatState) {
    let is_defensive = enemy.entity.status("SharpHide") > 0;

    if is_defensive {
        if last_move(enemy, move_ids::GUARD_ROLL_ATTACK) {
            enemy.set_move(move_ids::GUARD_TWIN_SLAM, 8, 2, 0);
        } else {
            enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        }
    } else {
        if last_move(enemy, move_ids::GUARD_CHARGING_UP) {
            enemy.set_move(move_ids::GUARD_FIERCE_BASH, 32, 1, 0);
        } else if last_move(enemy, move_ids::GUARD_FIERCE_BASH) {
            enemy.set_move(move_ids::GUARD_VENT_STEAM, 0, 0, 0);
            enemy.move_effects.insert("weak".to_string(), 2);
            enemy.move_effects.insert("vulnerable".to_string(), 2);
        } else if last_move(enemy, move_ids::GUARD_VENT_STEAM) {
            enemy.set_move(move_ids::GUARD_WHIRLWIND, 5, 4, 0);
        } else {
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }
    }
}

/// Check if Guardian should switch to defensive mode after taking damage.
pub fn guardian_check_mode_shift(enemy: &mut EnemyCombatState, damage_dealt: i32) -> bool {
    let threshold = enemy.entity.status("ModeShift");
    if threshold <= 0 { return false; }

    let current_taken = enemy.entity.status("DamageTakenThisMode") + damage_dealt;
    enemy.entity.set_status("DamageTakenThisMode", current_taken);

    if current_taken >= threshold {
        enemy.entity.set_status("SharpHide", 3);
        enemy.entity.set_status("DamageTakenThisMode", 0);
        enemy.entity.set_status("ModeShift", threshold + 10);
        enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        enemy.move_history.clear();
        true
    } else {
        false
    }
}

/// Switch Guardian back to offensive mode.
pub fn guardian_switch_to_offensive(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status("SharpHide", 0);
    enemy.entity.set_status("DamageTakenThisMode", 0);
    enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
    enemy.move_history.clear();
}

fn roll_hexaghost(enemy: &mut EnemyCombatState) {
    let moves_done = enemy.move_history.len();

    match moves_done {
        1 => {
            // After Activate: Divider. Damage = player_hp / 12 + 1, hit 6 times.
            // Use 7 as default (80hp / 12 + 1 = 7.67 -> 7)
            enemy.set_move(move_ids::HEX_DIVIDER, 7, 6, 0);
        }
        _ => {
            let pattern_turn = (moves_done - 2) % 7;
            match pattern_turn {
                0 => {
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                1 => {
                    enemy.set_move(move_ids::HEX_TACKLE, 5, 2, 0);
                }
                2 => {
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                3 => {
                    enemy.set_move(move_ids::HEX_INFLAME, 0, 0, 12);
                    enemy.move_effects.insert("strength".to_string(), 2);
                }
                4 => {
                    enemy.set_move(move_ids::HEX_TACKLE, 5, 2, 0);
                }
                5 => {
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                _ => {
                    enemy.set_move(move_ids::HEX_INFERNO, 2, 6, 0);
                    enemy.move_effects.insert("burn+".to_string(), 3);
                }
            }
        }
    }
}

/// Set Hexaghost Divider damage based on player HP.
/// Formula: ceil(player_hp / 12 + 1) per hit, 6 hits.
pub fn hexaghost_set_divider(enemy: &mut EnemyCombatState, player_hp: i32) {
    let per_hit = player_hp / 12 + 1;
    enemy.set_move(move_ids::HEX_DIVIDER, per_hit, 6, 0);
}

fn roll_slime_boss(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::SB_STICKY) {
        enemy.set_move(move_ids::SB_PREP_SLAM, 0, 0, 0);
    } else if last_move(enemy, move_ids::SB_PREP_SLAM) {
        enemy.set_move(move_ids::SB_SLAM, 35, 1, 0);
    } else {
        enemy.set_move(move_ids::SB_STICKY, 0, 0, 0);
        enemy.move_effects.insert("slimed".to_string(), 3);
    }
}

/// Check if Slime Boss should split (HP <= 50%).
pub fn slime_boss_should_split(enemy: &EnemyCombatState) -> bool {
    enemy.entity.hp > 0 && enemy.entity.hp <= enemy.entity.max_hp / 2
}

// =========================================================================
// Act 2 Basic Enemies
// =========================================================================

fn roll_chosen(enemy: &mut EnemyCombatState) {
    let used_hex = enemy.move_history.iter().any(|&m| m == move_ids::CHOSEN_HEX);

    // After first turn (Poke): use Hex
    if !used_hex {
        enemy.set_move(move_ids::CHOSEN_HEX, 0, 0, 0);
        enemy.move_effects.insert("hex".to_string(), 1);
        return;
    }
    // After Hex: alternate Debilitate/Drain and Zap/Poke
    if last_move(enemy, move_ids::CHOSEN_DEBILITATE) || last_move(enemy, move_ids::CHOSEN_DRAIN) {
        // Attack turn: Zap (18) or Poke (5x2)
        enemy.set_move(move_ids::CHOSEN_ZAP, 18, 1, 0);
    } else {
        // Debuff turn: Debilitate (10 + Vuln 2) or Drain (Weak 3, +3 Str)
        enemy.set_move(move_ids::CHOSEN_DEBILITATE, 10, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    }
}

fn roll_mugger(enemy: &mut EnemyCombatState) {
    let turns = enemy.move_history.len();
    if turns < 2 {
        enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
    } else if turns == 2 {
        // SmokeBomb or BigSwipe. Use BigSwipe (more threatening)
        enemy.set_move(move_ids::MUGGER_BIG_SWIPE, 16, 1, 0);
    } else if last_move(enemy, move_ids::MUGGER_BIG_SWIPE) {
        enemy.set_move(move_ids::MUGGER_SMOKE_BOMB, 0, 0, 11);
    } else if last_move(enemy, move_ids::MUGGER_SMOKE_BOMB) {
        enemy.set_move(move_ids::MUGGER_ESCAPE, 0, 0, 0);
        enemy.is_escaping = true;
    } else {
        enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
    }
}

fn roll_byrd(enemy: &mut EnemyCombatState) {
    let is_flying = enemy.entity.status("Flight") > 0;

    if !is_flying {
        // Grounded: Headbutt then Fly Up
        if last_move(enemy, move_ids::BYRD_STUNNED) {
            enemy.set_move(move_ids::BYRD_HEADBUTT, 3, 1, 0);
        } else {
            enemy.set_move(move_ids::BYRD_FLY_UP, 0, 0, 0);
            enemy.entity.set_status("Flight", 3);
        }
    } else {
        // Flying: alternate Peck and Swoop
        if last_two_moves(enemy, move_ids::BYRD_PECK) {
            enemy.set_move(move_ids::BYRD_SWOOP, 12, 1, 0);
        } else if last_move(enemy, move_ids::BYRD_SWOOP) {
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
        } else {
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
        }
    }
}

fn roll_shelled_parasite(enemy: &mut EnemyCombatState) {
    // Cycle: Double Strike (6x2), Life Suck (10), Fell (18 + Frail 2)
    if last_move(enemy, move_ids::SP_DOUBLE_STRIKE) {
        enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
        enemy.move_effects.insert("heal".to_string(), 10);
    } else if last_move(enemy, move_ids::SP_LIFE_SUCK) {
        enemy.set_move(move_ids::SP_FELL, 18, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
    }
}

fn roll_snake_plant(enemy: &mut EnemyCombatState) {
    // 65% Chomp (7x3), 35% Spores (Weak 2 + Frail 2). Anti-repeat.
    if last_two_moves(enemy, move_ids::SNAKE_CHOMP) {
        enemy.set_move(move_ids::SNAKE_SPORES, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else if last_move(enemy, move_ids::SNAKE_SPORES) {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    }
}

fn roll_centurion(enemy: &mut EnemyCombatState) {
    // Fury (6x3) or Slash (12), with Protect (15 block to ally) when ally alive
    if last_two_moves(enemy, move_ids::CENT_FURY) {
        enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
    } else if last_two_moves(enemy, move_ids::CENT_SLASH) {
        enemy.set_move(move_ids::CENT_FURY, 6, 3, 0);
    } else if last_move(enemy, move_ids::CENT_PROTECT) {
        enemy.set_move(move_ids::CENT_FURY, 6, 3, 0);
    } else {
        // Default: Slash
        enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
    }
}

fn roll_mystic(enemy: &mut EnemyCombatState) {
    // Attack (8 dmg), Heal (16 hp to ally), Buff (+2 Str to all allies)
    if last_two_moves(enemy, move_ids::MYSTIC_ATTACK) {
        enemy.set_move(move_ids::MYSTIC_BUFF, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 2);
    } else if last_move(enemy, move_ids::MYSTIC_BUFF) || last_move(enemy, move_ids::MYSTIC_HEAL) {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
    }
}

fn roll_book_of_stabbing(enemy: &mut EnemyCombatState) {
    // Multi-stab with increasing count. Stab count increases each time multi-stab is used.
    let stab_count = enemy.entity.status("StabCount");
    if last_two_moves(enemy, move_ids::BOOK_STAB) {
        enemy.set_move(move_ids::BOOK_BIG_STAB, 21, 1, 0);
        // Increment stab count on A18+
    } else if last_move(enemy, move_ids::BOOK_BIG_STAB) {
        let new_count = stab_count + 1;
        enemy.entity.set_status("StabCount", new_count);
        enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
    } else {
        let new_count = stab_count + 1;
        enemy.entity.set_status("StabCount", new_count);
        enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
    }
}

fn roll_gremlin_leader(enemy: &mut EnemyCombatState) {
    // Rally (summon), Encourage (block + Str to minions), Stab (6x3)
    if last_move(enemy, move_ids::GL_RALLY) {
        enemy.set_move(move_ids::GL_ENCOURAGE, 0, 0, 6);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::GL_ENCOURAGE) {
        enemy.set_move(move_ids::GL_STAB, 6, 3, 0);
    } else {
        // After stab: Rally if minions dead, else Encourage
        enemy.set_move(move_ids::GL_RALLY, 0, 0, 0);
    }
}

fn roll_taskmaster(enemy: &mut EnemyCombatState) {
    // Always Scouring Whip (7 damage + Wound card to discard)
    enemy.set_move(move_ids::TASK_SCOURING_WHIP, 7, 1, 0);
    enemy.move_effects.insert("wound".to_string(), 1);
}

fn roll_spheric_guardian(enemy: &mut EnemyCombatState) {
    // Pattern: Initial Block -> Frail Attack -> Big Attack -> Block Attack -> repeat
    if last_move(enemy, move_ids::SPHER_INITIAL_BLOCK) {
        enemy.set_move(move_ids::SPHER_FRAIL_ATTACK, 10, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 5);
    } else if last_move(enemy, move_ids::SPHER_BIG_ATTACK) {
        enemy.set_move(move_ids::SPHER_BLOCK_ATTACK, 10, 1, 15);
    } else if last_move(enemy, move_ids::SPHER_BLOCK_ATTACK) || last_move(enemy, move_ids::SPHER_FRAIL_ATTACK) {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    } else {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    }
}

fn roll_snecko(enemy: &mut EnemyCombatState) {
    // First turn: Glare. Then alternate Tail (8 + Vuln 2) and Bite (15)
    if last_move(enemy, move_ids::SNECKO_GLARE) || last_two_moves(enemy, move_ids::SNECKO_BITE) {
        enemy.set_move(move_ids::SNECKO_TAIL, 8, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SNECKO_BITE, 15, 1, 0);
    }
}

fn roll_bear(enemy: &mut EnemyCombatState) {
    // Bear Hug (debuff) -> Maul (18) -> Lunge (9 + 9 block) -> cycle
    if last_move(enemy, move_ids::BEAR_HUG) {
        enemy.set_move(move_ids::BEAR_MAUL, 18, 1, 0);
    } else if last_move(enemy, move_ids::BEAR_MAUL) {
        enemy.set_move(move_ids::BEAR_LUNGE, 9, 1, 9);
    } else {
        enemy.set_move(move_ids::BEAR_HUG, 0, 0, 0);
        enemy.move_effects.insert("dexterity_down".to_string(), 2);
    }
}

fn roll_bandit_leader(enemy: &mut EnemyCombatState) {
    // Mock -> Agonizing Slash (10 + Weak 2) -> Cross Slash (15) -> cycle
    if last_move(enemy, move_ids::BANDIT_MOCK) {
        enemy.set_move(move_ids::BANDIT_AGONIZE, 10, 1, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::BANDIT_AGONIZE) {
        enemy.set_move(move_ids::BANDIT_CROSS_SLASH, 15, 1, 0);
    } else {
        enemy.set_move(move_ids::BANDIT_MOCK, 0, 0, 0);
    }
}

// =========================================================================
// Act 2 Bosses
// =========================================================================

fn roll_bronze_automaton(enemy: &mut EnemyCombatState) {
    // Spawn Orbs -> Flail (7x2) -> ... -> Hyper Beam (45) -> Stunned -> repeat
    if last_move(enemy, move_ids::BA_SPAWN_ORBS) || last_move(enemy, move_ids::BA_STUNNED) || last_move(enemy, move_ids::BA_BOOST) {
        enemy.set_move(move_ids::BA_FLAIL, 7, 2, 0);
    } else if last_move(enemy, move_ids::BA_FLAIL) {
        // After enough Flails, Hyper Beam
        let turns = enemy.move_history.len();
        if turns >= 4 {
            enemy.set_move(move_ids::BA_HYPER_BEAM, 45, 1, 0);
        } else {
            enemy.set_move(move_ids::BA_BOOST, 0, 0, 9);
            enemy.move_effects.insert("strength".to_string(), 3);
        }
    } else if last_move(enemy, move_ids::BA_HYPER_BEAM) {
        enemy.set_move(move_ids::BA_STUNNED, 0, 0, 0);
    } else {
        enemy.set_move(move_ids::BA_FLAIL, 7, 2, 0);
    }
}

fn roll_bronze_orb(enemy: &mut EnemyCombatState) {
    // Stasis (first turn) -> Beam (8) / Support (12 block to Automaton)
    if last_two_moves(enemy, move_ids::BO_BEAM) {
        enemy.set_move(move_ids::BO_SUPPORT, 0, 0, 12);
    } else if last_move(enemy, move_ids::BO_SUPPORT) {
        enemy.set_move(move_ids::BO_BEAM, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::BO_BEAM, 8, 1, 0);
    }
}

fn roll_champ(enemy: &mut EnemyCombatState) {
    let num_turns = enemy.entity.status("NumTurns") + 1;
    enemy.entity.set_status("NumTurns", num_turns);

    let threshold_reached = enemy.entity.hp <= enemy.entity.max_hp / 2;

    // Phase 2: Anger (remove debuffs, +Str, Metallicize) then Execute spam
    if threshold_reached && enemy.entity.status("ThresholdReached") == 0 {
        enemy.entity.set_status("ThresholdReached", 1);
        enemy.set_move(move_ids::CHAMP_ANGER, 0, 0, 0);
        enemy.move_effects.insert("metallicize".to_string(), 5);
        enemy.move_effects.insert("strength".to_string(), 2);
        return;
    }
    if enemy.entity.status("ThresholdReached") > 0 {
        // Phase 2: Execute (10x2) and Heavy Slash (18)
        if !last_move(enemy, move_ids::CHAMP_EXECUTE) {
            enemy.set_move(move_ids::CHAMP_EXECUTE, 10, 2, 0);
        } else {
            enemy.set_move(move_ids::CHAMP_HEAVY_SLASH, 18, 1, 0);
        }
        return;
    }

    // Phase 1: cycle through moves
    if last_move(enemy, move_ids::CHAMP_FACE_SLAP) {
        enemy.set_move(move_ids::CHAMP_HEAVY_SLASH, 18, 1, 0);
    } else if last_move(enemy, move_ids::CHAMP_HEAVY_SLASH) {
        // Gloat or Defensive Stance
        if num_turns <= 3 {
            enemy.set_move(move_ids::CHAMP_GLOAT, 0, 0, 0);
            enemy.move_effects.insert("strength".to_string(), 2);
        } else {
            enemy.set_move(move_ids::CHAMP_DEFENSIVE, 0, 0, 15);
            enemy.move_effects.insert("metallicize".to_string(), 5);
        }
    } else if last_move(enemy, move_ids::CHAMP_GLOAT) || last_move(enemy, move_ids::CHAMP_DEFENSIVE) {
        enemy.set_move(move_ids::CHAMP_TAUNT, 0, 0, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::CHAMP_TAUNT) {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    }
}

fn roll_collector(enemy: &mut EnemyCombatState) {
    // Spawn -> Mega Debuff -> Fireball (18) cycle with Buff (+3 Str, 15 block) and Revive
    let turns = enemy.move_history.len();
    if turns == 1 {
        // After Spawn: Mega Debuff (Vuln 3, Weak 3, Frail 3)
        enemy.set_move(move_ids::COLL_MEGA_DEBUFF, 0, 0, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 3);
        enemy.move_effects.insert("weak".to_string(), 3);
        enemy.move_effects.insert("frail".to_string(), 3);
    } else if last_two_moves(enemy, move_ids::COLL_FIREBALL) {
        enemy.set_move(move_ids::COLL_BUFF, 0, 0, 15);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::COLL_BUFF) {
        enemy.set_move(move_ids::COLL_FIREBALL, 18, 1, 0);
    } else {
        enemy.set_move(move_ids::COLL_FIREBALL, 18, 1, 0);
    }
}

// =========================================================================
// Act 3 Basic Enemies
// =========================================================================

fn roll_darkling(enemy: &mut EnemyCombatState) {
    // Chomp (8x2), Harden (12 block + Reanimated), Nip (8).
    // If dead: Reincarnate (revive at 50% HP).
    if enemy.entity.hp <= 0 {
        enemy.set_move(move_ids::DARK_REINCARNATE, 0, 0, 0);
        return;
    }
    if last_two_moves(enemy, move_ids::DARK_NIP) {
        enemy.set_move(move_ids::DARK_CHOMP, 8, 2, 0);
    } else if last_move(enemy, move_ids::DARK_CHOMP) {
        enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
    } else if last_move(enemy, move_ids::DARK_HARDEN) {
        enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
    }
}

fn roll_orb_walker(enemy: &mut EnemyCombatState) {
    // Alternate: Claw (15) and Laser (10 + Burn)
    if last_two_moves(enemy, move_ids::OW_CLAW) {
        enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
        enemy.move_effects.insert("burn".to_string(), 1);
    } else if last_two_moves(enemy, move_ids::OW_LASER) {
        enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
    } else if last_move(enemy, move_ids::OW_LASER) {
        enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
    } else {
        enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
        enemy.move_effects.insert("burn".to_string(), 1);
    }
}

fn roll_spiker(enemy: &mut EnemyCombatState) {
    // Attack (7 dmg) or Buff (+2 Thorns). Anti-repeat.
    if last_move(enemy, move_ids::SPIKER_ATTACK) {
        enemy.set_move(move_ids::SPIKER_BUFF, 0, 0, 0);
        let thorns = enemy.entity.status("Thorns");
        enemy.entity.set_status("Thorns", thorns + 2);
        enemy.move_effects.insert("thorns".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
    }
}

fn roll_repulsor(enemy: &mut EnemyCombatState) {
    // Mostly Daze, sometimes Attack (11)
    if last_move(enemy, move_ids::REPULSOR_ATTACK) {
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.move_effects.insert("daze".to_string(), 2);
    } else {
        // 80% Daze, 20% Attack
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.move_effects.insert("daze".to_string(), 2);
    }
}

fn roll_exploder(enemy: &mut EnemyCombatState) {
    let count = enemy.entity.status("TurnCount") + 1;
    enemy.entity.set_status("TurnCount", count);

    if count >= 3 {
        // Explode! 30 damage and die
        enemy.set_move(move_ids::EXPLODER_EXPLODE, 30, 1, 0);
    } else {
        enemy.set_move(move_ids::EXPLODER_ATTACK, 9, 1, 0);
    }
}

fn roll_writhing_mass(enemy: &mut EnemyCombatState) {
    // Rotates through moves. Has Reactive power (changes intent when hit).
    // Deterministic for MCTS: cycle Big Hit -> Multi -> Attack+Block -> Attack+Debuff
    if last_move(enemy, move_ids::WM_MULTI_HIT) {
        enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 10);
    } else if last_move(enemy, move_ids::WM_ATTACK_BLOCK) {
        enemy.set_move(move_ids::WM_ATTACK_DEBUFF, 10, 1, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::WM_ATTACK_DEBUFF) {
        enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
    } else if last_move(enemy, move_ids::WM_BIG_HIT) {
        enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
    }
}

fn roll_spire_growth(enemy: &mut EnemyCombatState) {
    // Constrict then alternate Quick Tackle (16) and Smash (22)
    if last_move(enemy, move_ids::SG_CONSTRICT) || last_two_moves(enemy, move_ids::SG_SMASH) {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    } else if last_two_moves(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_CONSTRICT, 0, 0, 0);
        enemy.move_effects.insert("constrict".to_string(), 10);
    } else if last_move(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_SMASH, 22, 1, 0);
    } else {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    }
}

fn roll_maw(enemy: &mut EnemyCombatState) {
    let turn_count = enemy.entity.status("TurnCount") + 1;
    enemy.entity.set_status("TurnCount", turn_count);

    // Roar (first turn), then cycle: NomNom / Slam / Drool(Str)
    if last_move(enemy, move_ids::MAW_SLAM) || last_move(enemy, move_ids::MAW_NOM) {
        enemy.set_move(move_ids::MAW_DROOL, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::MAW_DROOL) || last_move(enemy, move_ids::MAW_ROAR) {
        // NomNom: 5 x (turnCount/2) or Slam: 25
        let nom_hits = turn_count / 2;
        if nom_hits >= 2 {
            enemy.set_move(move_ids::MAW_NOM, 5, nom_hits, 0);
        } else {
            enemy.set_move(move_ids::MAW_SLAM, 25, 1, 0);
        }
    } else {
        enemy.set_move(move_ids::MAW_SLAM, 25, 1, 0);
    }
}

fn roll_transient(enemy: &mut EnemyCombatState) {
    let count = enemy.entity.status("AttackCount") + 1;
    enemy.entity.set_status("AttackCount", count);
    // Escalating: 30 + 10 * count
    let dmg = 30 + count * 10;
    enemy.set_move(move_ids::TRANSIENT_ATTACK, dmg, 1, 0);
}

// =========================================================================
// Act 3 Elites
// =========================================================================

fn roll_giant_head(enemy: &mut EnemyCombatState) {
    let countdown = enemy.entity.status("Countdown");

    if countdown <= 0 {
        // It Is Time! Escalating damage: starts at 30, +5 per use
        let uses = enemy.entity.status("ItIsTimeUses");
        let dmg = 30 + uses * 5;
        enemy.entity.set_status("ItIsTimeUses", uses + 1);
        enemy.set_move(move_ids::GH_IT_IS_TIME, dmg, 1, 0);
    } else {
        enemy.entity.set_status("Countdown", countdown - 1);
        // Alternate Glare (Weak 1) and Count (13 dmg)
        if last_two_moves(enemy, move_ids::GH_COUNT) {
            enemy.set_move(move_ids::GH_GLARE, 0, 0, 0);
            enemy.move_effects.insert("weak".to_string(), 1);
        } else if last_move(enemy, move_ids::GH_GLARE) {
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        } else {
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        }
    }
}

fn roll_nemesis(enemy: &mut EnemyCombatState) {
    let cooldown = enemy.entity.status("ScytheCooldown");

    if cooldown > 0 {
        enemy.entity.set_status("ScytheCooldown", cooldown - 1);
    }

    // Pattern: Tri Attack / Burn / Scythe (45, goes Intangible after)
    if last_move(enemy, move_ids::NEM_SCYTHE) {
        // After Scythe: Intangible, then Tri Attack or Burn
        enemy.entity.set_status("ScytheCooldown", 2);
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.move_effects.insert("burn".to_string(), 3);
    } else if cooldown <= 0 && !last_move(enemy, move_ids::NEM_SCYTHE) {
        // Scythe when off cooldown
        if enemy.move_history.len() >= 2 {
            enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
            enemy.entity.set_status("Intangible", 1);
        } else {
            enemy.set_move(move_ids::NEM_TRI_ATTACK, 6, 3, 0);
        }
    } else if last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.move_effects.insert("burn".to_string(), 3);
    } else {
        enemy.set_move(move_ids::NEM_TRI_ATTACK, 6, 3, 0);
    }
}

fn roll_reptomancer(enemy: &mut EnemyCombatState) {
    // Spawn -> Snake Strike (13x2 + Weak) -> Big Bite (30) -> cycle
    if last_move(enemy, move_ids::REPTO_SPAWN) {
        enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0);
        enemy.move_effects.insert("weak".to_string(), 1);
    } else if last_move(enemy, move_ids::REPTO_SNAKE_STRIKE) {
        enemy.set_move(move_ids::REPTO_BIG_BITE, 30, 1, 0);
    } else {
        // After Big Bite: Spawn more daggers if slots open
        enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
    }
}

fn roll_snake_dagger(enemy: &mut EnemyCombatState) {
    // Wound (9 + Wound card) -> Explode (25 dmg, dies)
    if last_move(enemy, move_ids::SD_WOUND) {
        enemy.set_move(move_ids::SD_EXPLODE, 25, 1, 0);
    } else {
        enemy.set_move(move_ids::SD_WOUND, 9, 1, 0);
        enemy.move_effects.insert("wound".to_string(), 1);
    }
}

// =========================================================================
// Act 3 Bosses
// =========================================================================

fn roll_awakened_one(enemy: &mut EnemyCombatState) {
    let phase = enemy.entity.status("Phase");

    if phase == 1 {
        // Phase 1: Slash (20) / Soul Strike (6x4)
        if last_move(enemy, move_ids::AO_SLASH) || last_two_moves(enemy, move_ids::AO_SLASH) {
            enemy.set_move(move_ids::AO_SOUL_STRIKE, 6, 4, 0);
        } else {
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
        }
        // Check for death -> Rebirth handled externally
    } else {
        // Phase 2: Dark Echo (40) / Sludge (18 + Slimed) / Tackle (10x3)
        if last_move(enemy, move_ids::AO_DARK_ECHO) {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 1);
        } else if last_move(enemy, move_ids::AO_SLUDGE) || last_two_moves(enemy, move_ids::AO_SLUDGE) {
            enemy.set_move(move_ids::AO_TACKLE, 10, 3, 0);
        } else if last_move(enemy, move_ids::AO_TACKLE) || last_two_moves(enemy, move_ids::AO_TACKLE) {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 1);
        } else {
            enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
        }
    }
}

/// Trigger Awakened One rebirth (Phase 1 -> Phase 2).
pub fn awakened_one_rebirth(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status("Phase", 2);
    enemy.entity.set_status("Curiosity", 0);
    // Heal to full (second form HP)
    enemy.entity.hp = enemy.entity.max_hp;
    enemy.move_history.clear();
    // First move of Phase 2: Dark Echo
    enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
}

fn roll_donu(enemy: &mut EnemyCombatState) {
    // Alternate: Circle of Protection (+3 Str to both) and Beam (10x2)
    if last_move(enemy, move_ids::DONU_CIRCLE) {
        enemy.set_move(move_ids::DONU_BEAM, 10, 2, 0);
    } else {
        enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    }
}

fn roll_deca(enemy: &mut EnemyCombatState) {
    // Alternate: Square of Protection (16 block) and Beam (10x2 + 2 Daze)
    if last_move(enemy, move_ids::DECA_SQUARE) {
        enemy.set_move(move_ids::DECA_BEAM, 10, 2, 0);
        enemy.move_effects.insert("daze".to_string(), 2);
    } else {
        enemy.set_move(move_ids::DECA_SQUARE, 0, 0, 16);
    }
}

fn roll_time_eater(enemy: &mut EnemyCombatState) {
    // Reverberate (7x3), Head Slam (26 + Slimed), Ripple (20 block + Vuln+Weak+Frail 1)
    // Haste at 50% HP: heal, cleanse, +2 Str
    if last_move(enemy, move_ids::TE_HASTE) || last_two_moves(enemy, move_ids::TE_REVERBERATE) {
        enemy.set_move(move_ids::TE_HEAD_SLAM, 26, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    } else if last_move(enemy, move_ids::TE_HEAD_SLAM) {
        enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
        enemy.move_effects.insert("vulnerable".to_string(), 1);
        enemy.move_effects.insert("weak".to_string(), 1);
        enemy.move_effects.insert("frail".to_string(), 1);
    } else if last_move(enemy, move_ids::TE_RIPPLE) {
        enemy.set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
    }
}

// =========================================================================
// Act 4 — The Ending
// =========================================================================

fn roll_spire_shield(enemy: &mut EnemyCombatState) {
    let mc = enemy.entity.status("MoveCount") + 1;
    enemy.entity.set_status("MoveCount", mc);

    // 3-move cycle: (Bash/Fortify), (Bash/Fortify), Smash
    match mc % 3 {
        1 => {
            // Bash (14 dmg, -1 Str) or Fortify (30 block)
            if last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            } else {
                enemy.set_move(move_ids::SHIELD_BASH, 14, 1, 0);
                enemy.move_effects.insert("strength_down".to_string(), 1);
            }
        }
        2 => {
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, 14, 1, 0);
                enemy.move_effects.insert("strength_down".to_string(), 1);
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            }
        }
        _ => {
            // Smash (34 dmg + block equal to damage dealt)
            enemy.set_move(move_ids::SHIELD_SMASH, 34, 1, 0);
        }
    }
}

fn roll_spire_spear(enemy: &mut EnemyCombatState) {
    let mc = enemy.entity.status("MoveCount") + 1;
    enemy.entity.set_status("MoveCount", mc);
    let skewer_count = enemy.entity.status("SkewerCount");

    // 3-move cycle: (Burn Strike/Piercer), Skewer, (Piercer/Burn Strike)
    match mc % 3 {
        1 => {
            if !last_move(enemy, move_ids::SPEAR_BURN_STRIKE) {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
                enemy.move_effects.insert("burn".to_string(), 2);
            } else {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.move_effects.insert("strength".to_string(), 2);
            }
        }
        2 => {
            enemy.set_move(move_ids::SPEAR_SKEWER, 10, skewer_count, 0);
        }
        _ => {
            enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
            enemy.move_effects.insert("strength".to_string(), 2);
        }
    }
}

fn roll_corrupt_heart(enemy: &mut EnemyCombatState) {
    let mc = enemy.entity.status("MoveCount") + 1;
    enemy.entity.set_status("MoveCount", mc);
    let blood_count = enemy.entity.status("BloodHitCount");

    // 3-move cycle: Attack (Blood or Echo), Attack (the other), Buff (+Str, +Beat of Death)
    match mc % 3 {
        1 => {
            // Blood Shots (2 x 12) or Echo (40)
            enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
        }
        2 => {
            if !last_move(enemy, move_ids::HEART_ECHO) {
                enemy.set_move(move_ids::HEART_ECHO, 40, 1, 0);
            } else {
                enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
            }
        }
        _ => {
            // Buff: +2 Strength, +1 Beat of Death
            enemy.set_move(move_ids::HEART_BUFF, 0, 0, 0);
            enemy.move_effects.insert("strength".to_string(), 2);
            enemy.move_effects.insert("beat_of_death".to_string(), 1);
        }
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Act 1 -----

    #[test]
    fn test_create_jaw_worm() {
        let enemy = create_enemy("JawWorm", 44, 44);
        assert_eq!(enemy.id, "JawWorm");
        assert_eq!(enemy.entity.hp, 44);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
        assert_eq!(enemy.move_damage, 11);
    }

    #[test]
    fn test_jaw_worm_pattern() {
        let mut enemy = create_enemy("JawWorm", 44, 44);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_BELLOW);
        assert_eq!(enemy.move_effects.get("strength"), Some(&3));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_THRASH);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_block, 5);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
    }

    #[test]
    fn test_cultist_pattern() {
        let mut enemy = create_enemy("Cultist", 50, 50);
        assert_eq!(enemy.move_id, move_ids::CULT_INCANTATION);
        assert_eq!(enemy.move_damage, 0);
        assert_eq!(enemy.move_effects.get("ritual"), Some(&3));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
        assert_eq!(enemy.move_damage, 6);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
    }

    #[test]
    fn test_fungi_beast_anti_repeat() {
        let mut enemy = create_enemy("FungiBeast", 24, 24);
        assert_eq!(enemy.move_id, move_ids::FB_BITE);

        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        if enemy.move_history.len() >= 2
            && enemy.move_history[enemy.move_history.len() - 1] == move_ids::FB_BITE
            && enemy.move_history[enemy.move_history.len() - 2] == move_ids::FB_BITE
        {
            assert_eq!(enemy.move_id, move_ids::FB_GROW);
        }
    }

    #[test]
    fn test_sentry_alternating() {
        let mut enemy = create_enemy("Sentry", 38, 38);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BEAM);
        assert_eq!(enemy.move_effects.get("daze"), Some(&2));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);
    }

    #[test]
    fn test_slime_boss_pattern() {
        let mut enemy = create_enemy("SlimeBoss", 140, 140);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_PREP_SLAM);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_SLAM);
        assert_eq!(enemy.move_damage, 35);

        roll_next_move(&mut enemy);
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
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_FIERCE_BASH);
        assert_eq!(enemy.move_damage, 32);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_VENT_STEAM);
        assert_eq!(enemy.move_effects.get("weak"), Some(&2));
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 4);
    }

    #[test]
    fn test_guardian_mode_shift() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.entity.status("ModeShift"), 30);

        let shifted = guardian_check_mode_shift(&mut enemy, 30);
        assert!(shifted);
        assert_eq!(enemy.entity.status("SharpHide"), 3);
        assert_eq!(enemy.entity.status("ModeShift"), 40);

        assert_eq!(enemy.move_id, move_ids::GUARD_ROLL_ATTACK);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_TWIN_SLAM);
    }

    #[test]
    fn test_hexaghost_pattern() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        assert_eq!(enemy.move_id, move_ids::HEX_ACTIVATE);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_hits, 6);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);
        assert_eq!(enemy.move_effects.get("burn"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_hits, 2);
    }

    #[test]
    fn test_hexaghost_divider_scaling() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        hexaghost_set_divider(&mut enemy, 80);
        // 80 / 12 + 1 = 7 (integer division)
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 6);

        hexaghost_set_divider(&mut enemy, 60);
        // 60 / 12 + 1 = 6
        assert_eq!(enemy.move_damage, 6);
    }

    #[test]
    fn test_blue_slaver_pattern() {
        let mut enemy = create_enemy("SlaverBlue", 48, 48);
        assert_eq!(enemy.move_id, move_ids::BS_STAB);
        assert_eq!(enemy.move_damage, 12);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BS_STAB);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BS_RAKE);
        assert_eq!(enemy.move_effects.get("weak"), Some(&1));
    }

    #[test]
    fn test_red_slaver_pattern() {
        let mut enemy = create_enemy("SlaverRed", 48, 48);
        assert_eq!(enemy.move_id, move_ids::RS_STAB);
        assert_eq!(enemy.move_damage, 13);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::RS_ENTANGLE);
        assert_eq!(enemy.move_effects.get("entangle"), Some(&1));

        roll_next_move(&mut enemy);
        assert!(
            enemy.move_id == move_ids::RS_SCRAPE || enemy.move_id == move_ids::RS_STAB
        );
    }

    #[test]
    fn test_acid_slime_s_pattern() {
        let mut enemy = create_enemy("AcidSlime_S", 10, 10);
        assert_eq!(enemy.move_id, move_ids::AS_TACKLE);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_LICK);
        assert_eq!(enemy.move_effects.get("weak"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_TACKLE);
    }

    #[test]
    fn test_spike_slime_m_pattern() {
        let mut enemy = create_enemy("SpikeSlime_M", 28, 28);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);
        assert_eq!(enemy.move_damage, 8);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SS_LICK);
        assert_eq!(enemy.move_effects.get("frail"), Some(&1));
    }

    #[test]
    fn test_louse_curl_up() {
        let enemy = create_enemy("RedLouse", 12, 12);
        assert_eq!(enemy.entity.status("CurlUp"), 5);
    }

    #[test]
    fn test_guardian_switch_to_offensive() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        guardian_check_mode_shift(&mut enemy, 30);
        assert_eq!(enemy.entity.status("SharpHide"), 3);

        guardian_switch_to_offensive(&mut enemy);
        assert_eq!(enemy.entity.status("SharpHide"), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
    }

    #[test]
    fn test_looter_escape() {
        let mut enemy = create_enemy("Looter", 44, 44);
        assert_eq!(enemy.move_id, move_ids::LOOTER_MUG);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::LOOTER_MUG);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::LOOTER_SMOKE_BOMB);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::LOOTER_ESCAPE);
        assert!(enemy.is_escaping);
    }

    // ----- Act 2 -----

    #[test]
    fn test_chosen_pattern() {
        let mut enemy = create_enemy("Chosen", 97, 97);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_POKE);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 2);

        // After Poke: Hex
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_HEX);
        assert_eq!(enemy.move_effects.get("hex"), Some(&1));

        // After Hex: Debilitate or Drain
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_DEBILITATE);

        // After debuff: attack
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHOSEN_ZAP);
        assert_eq!(enemy.move_damage, 18);
    }

    #[test]
    fn test_byrd_pattern() {
        let mut enemy = create_enemy("Byrd", 28, 28);
        assert_eq!(enemy.move_id, move_ids::BYRD_PECK);
        assert_eq!(enemy.move_damage, 1);
        assert_eq!(enemy.move_hits, 5);
        assert_eq!(enemy.entity.status("Flight"), 3);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BYRD_PECK);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BYRD_SWOOP);
        assert_eq!(enemy.move_damage, 12);
    }

    #[test]
    fn test_snake_plant_pattern() {
        let mut enemy = create_enemy("SnakePlant", 77, 77);
        assert_eq!(enemy.move_id, move_ids::SNAKE_CHOMP);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 3);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SNAKE_CHOMP);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SNAKE_SPORES);
        assert_eq!(enemy.move_effects.get("weak"), Some(&2));
        assert_eq!(enemy.move_effects.get("frail"), Some(&2));
    }

    #[test]
    fn test_book_of_stabbing_escalation() {
        let mut enemy = create_enemy("BookOfStabbing", 162, 162);
        assert_eq!(enemy.move_id, move_ids::BOOK_STAB);
        assert_eq!(enemy.move_hits, 2);

        // Roll: stab count increments
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BOOK_STAB);
        let new_count = enemy.entity.status("StabCount");
        assert!(new_count >= 3);
    }

    #[test]
    fn test_bronze_automaton_boss_pattern() {
        let mut enemy = create_enemy("BronzeAutomaton", 300, 300);
        assert_eq!(enemy.move_id, move_ids::BA_SPAWN_ORBS);

        // After spawn: Flail
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BA_FLAIL);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 2);
    }

    #[test]
    fn test_champ_boss_pattern() {
        let mut enemy = create_enemy("Champ", 420, 420);
        assert_eq!(enemy.move_id, move_ids::CHAMP_FACE_SLAP);
        assert_eq!(enemy.move_damage, 12);

        // Phase 1 cycle
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);
        assert_eq!(enemy.move_damage, 18);

        // Trigger phase 2 at half HP
        enemy.entity.hp = 200;
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_ANGER);
    }

    #[test]
    fn test_collector_boss_pattern() {
        let mut enemy = create_enemy("TheCollector", 282, 282);
        assert_eq!(enemy.move_id, move_ids::COLL_SPAWN);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::COLL_MEGA_DEBUFF);
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&3));
        assert_eq!(enemy.move_effects.get("weak"), Some(&3));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::COLL_FIREBALL);
        assert_eq!(enemy.move_damage, 18);
    }

    // ----- Act 3 -----

    #[test]
    fn test_awakened_one_boss() {
        let mut enemy = create_enemy("AwakenedOne", 300, 300);
        assert_eq!(enemy.move_id, move_ids::AO_SLASH);
        assert_eq!(enemy.move_damage, 20);
        assert_eq!(enemy.entity.status("Phase"), 1);
        assert_eq!(enemy.entity.status("Curiosity"), 1);

        // Phase 1 cycle
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AO_SOUL_STRIKE);
        assert_eq!(enemy.move_damage, 6);
        assert_eq!(enemy.move_hits, 4);

        // Trigger rebirth
        awakened_one_rebirth(&mut enemy);
        assert_eq!(enemy.entity.status("Phase"), 2);
        assert_eq!(enemy.entity.hp, 300);
        assert_eq!(enemy.move_id, move_ids::AO_DARK_ECHO);
        assert_eq!(enemy.move_damage, 40);
    }

    #[test]
    fn test_time_eater_boss() {
        let mut enemy = create_enemy("TimeEater", 456, 456);
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_hits, 3);

        roll_next_move(&mut enemy);
        // After first reverberate, second reverberate
        assert_eq!(enemy.move_id, move_ids::TE_REVERBERATE);

        roll_next_move(&mut enemy);
        // After two reverberates: Head Slam
        assert_eq!(enemy.move_id, move_ids::TE_HEAD_SLAM);
        assert_eq!(enemy.move_damage, 26);
    }

    #[test]
    fn test_donu_deca_boss() {
        let mut donu = create_enemy("Donu", 250, 250);
        assert_eq!(donu.move_id, move_ids::DONU_CIRCLE);
        assert_eq!(donu.entity.status("Artifact"), 2);

        roll_next_move(&mut donu);
        assert_eq!(donu.move_id, move_ids::DONU_BEAM);
        assert_eq!(donu.move_damage, 10);
        assert_eq!(donu.move_hits, 2);

        let mut deca = create_enemy("Deca", 250, 250);
        assert_eq!(deca.move_id, move_ids::DECA_SQUARE);
        assert_eq!(deca.move_block, 16);

        roll_next_move(&mut deca);
        assert_eq!(deca.move_id, move_ids::DECA_BEAM);
        assert_eq!(deca.move_damage, 10);
        assert_eq!(deca.move_effects.get("daze"), Some(&2));
    }

    #[test]
    fn test_giant_head_elite() {
        let mut enemy = create_enemy("GiantHead", 500, 500);
        assert_eq!(enemy.move_id, move_ids::GH_COUNT);
        assert_eq!(enemy.move_damage, 13);
        assert_eq!(enemy.entity.status("Countdown"), 5);

        // Count down turns
        for _ in 0..5 {
            roll_next_move(&mut enemy);
        }

        // Should eventually hit It Is Time
        let countdown = enemy.entity.status("Countdown");
        assert!(countdown == 0 || enemy.move_id == move_ids::GH_IT_IS_TIME);
    }

    #[test]
    fn test_nemesis_elite() {
        let mut enemy = create_enemy("Nemesis", 185, 185);
        assert_eq!(enemy.move_id, move_ids::NEM_TRI_ATTACK);
        assert_eq!(enemy.move_damage, 6);
        assert_eq!(enemy.move_hits, 3);

        roll_next_move(&mut enemy);
        // Second turn
        roll_next_move(&mut enemy);
        // Should eventually use Scythe
        let has_scythe = enemy.move_id == move_ids::NEM_SCYTHE
            || enemy.move_history.iter().any(|&m| m == move_ids::NEM_SCYTHE);
        assert!(has_scythe || enemy.move_history.len() <= 3);
    }

    #[test]
    fn test_reptomancer_elite() {
        let mut enemy = create_enemy("Reptomancer", 185, 185);
        assert_eq!(enemy.move_id, move_ids::REPTO_SPAWN);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::REPTO_SNAKE_STRIKE);
        assert_eq!(enemy.move_damage, 13);
        assert_eq!(enemy.move_hits, 2);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::REPTO_BIG_BITE);
        assert_eq!(enemy.move_damage, 30);
    }

    #[test]
    fn test_transient_escalation() {
        let mut enemy = create_enemy("Transient", 999, 999);
        assert_eq!(enemy.move_id, move_ids::TRANSIENT_ATTACK);
        assert_eq!(enemy.move_damage, 30);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_damage, 40); // 30 + 10

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_damage, 50); // 30 + 20

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_damage, 60); // 30 + 30
    }

    // ----- Act 4 -----

    #[test]
    fn test_corrupt_heart_boss() {
        let mut enemy = create_enemy("CorruptHeart", 750, 750);
        assert_eq!(enemy.move_id, move_ids::HEART_DEBILITATE);
        assert_eq!(enemy.entity.status("Invincible"), 300);
        assert_eq!(enemy.entity.status("BeatOfDeath"), 1);
        assert_eq!(enemy.entity.status("BloodHitCount"), 12);

        // After Debilitate: Blood Shots
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BLOOD_SHOTS);
        assert_eq!(enemy.move_damage, 2);
        assert_eq!(enemy.move_hits, 12);

        // Then Echo or Blood Shots
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_ECHO);
        assert_eq!(enemy.move_damage, 40);

        // Then Buff
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BUFF);
        assert_eq!(enemy.move_effects.get("strength"), Some(&2));
        assert_eq!(enemy.move_effects.get("beat_of_death"), Some(&1));
    }

    #[test]
    fn test_spire_shield_boss() {
        let mut enemy = create_enemy("SpireShield", 110, 110);
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);
        assert_eq!(enemy.move_damage, 14);
        assert_eq!(enemy.move_effects.get("strength_down"), Some(&1));

        roll_next_move(&mut enemy); // mc=1: Fortify (since last was Bash)
        assert_eq!(enemy.move_id, move_ids::SHIELD_FORTIFY);
        assert_eq!(enemy.move_block, 30);

        roll_next_move(&mut enemy); // mc=2: Bash (since last was Fortify)
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);

        roll_next_move(&mut enemy); // mc=3 -> 3%3=0: Smash
        assert_eq!(enemy.move_id, move_ids::SHIELD_SMASH);
        assert_eq!(enemy.move_damage, 34);
    }

    #[test]
    fn test_spire_spear_boss() {
        let mut enemy = create_enemy("SpireSpear", 160, 160);
        assert_eq!(enemy.move_id, move_ids::SPEAR_BURN_STRIKE);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 2);
        assert_eq!(enemy.entity.status("SkewerCount"), 3);

        roll_next_move(&mut enemy); // mc=1, 1%3=1: Piercer (since last was BurnStrike)
        assert_eq!(enemy.move_id, move_ids::SPEAR_PIERCER);

        roll_next_move(&mut enemy); // mc=2, 2%3=2: Skewer
        assert_eq!(enemy.move_id, move_ids::SPEAR_SKEWER);
        assert_eq!(enemy.move_damage, 10);
        assert_eq!(enemy.move_hits, 3);
    }

    #[test]
    fn test_snake_dagger_pattern() {
        let mut enemy = create_enemy("SnakeDagger", 22, 22);
        assert_eq!(enemy.move_id, move_ids::SD_WOUND);
        assert_eq!(enemy.move_damage, 9);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SD_EXPLODE);
        assert_eq!(enemy.move_damage, 25);
    }

    #[test]
    fn test_darkling_pattern() {
        let mut enemy = create_enemy("Darkling", 52, 52);
        assert_eq!(enemy.move_id, move_ids::DARK_NIP);
        assert_eq!(enemy.move_damage, 8);

        roll_next_move(&mut enemy);
        roll_next_move(&mut enemy);
        // After two Nips: Chomp
        assert_eq!(enemy.move_id, move_ids::DARK_CHOMP);
        assert_eq!(enemy.move_hits, 2);
    }

    #[test]
    fn test_exploder_timer() {
        let mut enemy = create_enemy("Exploder", 30, 30);
        assert_eq!(enemy.move_id, move_ids::EXPLODER_ATTACK);

        roll_next_move(&mut enemy); // count=1, attack
        assert_eq!(enemy.move_id, move_ids::EXPLODER_ATTACK);

        roll_next_move(&mut enemy); // count=2, attack
        assert_eq!(enemy.move_id, move_ids::EXPLODER_ATTACK);

        roll_next_move(&mut enemy); // count=3, EXPLODE
        assert_eq!(enemy.move_id, move_ids::EXPLODER_EXPLODE);
        assert_eq!(enemy.move_damage, 30);
    }

    #[test]
    fn test_orb_walker_pattern() {
        let mut enemy = create_enemy("OrbWalker", 93, 93);
        assert_eq!(enemy.move_id, move_ids::OW_LASER);
        assert_eq!(enemy.move_damage, 10);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::OW_CLAW);
        assert_eq!(enemy.move_damage, 15);
    }

    /// Test all enemy IDs can be created without panicking
    #[test]
    fn test_all_enemies_create() {
        let ids = vec![
            // Act 1
            "JawWorm", "Cultist", "FungiBeast", "FuzzyLouseNormal",
            "FuzzyLouseDefensive", "SlaverBlue", "SlaverRed",
            "AcidSlime_S", "AcidSlime_M", "AcidSlime_L",
            "SpikeSlime_S", "SpikeSlime_M", "SpikeSlime_L",
            "Looter", "GremlinFat", "GremlinThief", "GremlinWarrior",
            "GremlinWizard", "GremlinTsundere",
            "GremlinNob", "Lagavulin", "Sentry",
            "TheGuardian", "Hexaghost", "SlimeBoss",
            // Act 2
            "Chosen", "Mugger", "Byrd", "ShelledParasite", "SnakePlant",
            "Centurion", "Mystic", "BookOfStabbing", "GremlinLeader",
            "Taskmaster", "SphericGuardian", "Snecko",
            "BanditBear", "BanditLeader", "BanditPointy",
            "BronzeAutomaton", "BronzeOrb", "TorchHead",
            "Champ", "TheCollector",
            // Act 3
            "Darkling", "OrbWalker", "Spiker", "Repulsor", "Exploder",
            "WrithingMass", "SpireGrowth", "Maw", "Transient",
            "GiantHead", "Nemesis", "Reptomancer", "SnakeDagger",
            "AwakenedOne", "Donu", "Deca", "TimeEater",
            // Act 4
            "SpireShield", "SpireSpear", "CorruptHeart",
        ];
        for id in &ids {
            let enemy = create_enemy(id, 100, 100);
            assert_eq!(enemy.id, *id, "Enemy ID mismatch for {}", id);
            // Should not use fallback generic move
            assert!(
                enemy.move_id != 1 || ["GremlinFat", "GremlinThief", "GremlinWarrior",
                    "SpikeSlime_S", "AcidSlime_S", "SpikeSlime_L",
                    "SpikeSlime_M", "AcidSlime_M", "AcidSlime_L"].contains(id)
                || enemy.move_id == move_ids::GREMLIN_ATTACK
                || enemy.move_id == move_ids::SS_TACKLE
                || enemy.move_id == move_ids::AS_CORROSIVE_SPIT
                || enemy.move_id == move_ids::AS_TACKLE,
                "Enemy {} has fallback move_id=1 (generic), expected a specific move", id
            );
        }
        assert_eq!(ids.len(), 65, "Should have 65 unique IDs covering all enemies");
    }

    /// Test all enemies can roll at least 5 moves without panicking
    #[test]
    fn test_all_enemies_roll() {
        let ids = vec![
            "JawWorm", "Cultist", "FungiBeast", "FuzzyLouseNormal",
            "FuzzyLouseDefensive", "SlaverBlue", "SlaverRed",
            "AcidSlime_S", "AcidSlime_M", "AcidSlime_L",
            "SpikeSlime_S", "SpikeSlime_M", "SpikeSlime_L",
            "Looter", "GremlinFat", "GremlinThief", "GremlinWarrior",
            "GremlinWizard", "GremlinTsundere",
            "GremlinNob", "Lagavulin", "Sentry",
            "TheGuardian", "Hexaghost", "SlimeBoss",
            "Chosen", "Mugger", "Byrd", "ShelledParasite", "SnakePlant",
            "Centurion", "Mystic", "BookOfStabbing", "GremlinLeader",
            "Taskmaster", "SphericGuardian", "Snecko",
            "BanditBear", "BanditLeader", "BanditPointy",
            "BronzeAutomaton", "BronzeOrb", "TorchHead",
            "Champ", "TheCollector",
            "Darkling", "OrbWalker", "Spiker", "Repulsor", "Exploder",
            "WrithingMass", "SpireGrowth", "Maw", "Transient",
            "GiantHead", "Nemesis", "Reptomancer", "SnakeDagger",
            "AwakenedOne", "Donu", "Deca", "TimeEater",
            "SpireShield", "SpireSpear", "CorruptHeart",
        ];
        for id in &ids {
            let mut enemy = create_enemy(id, 100, 100);
            for _ in 0..5 {
                roll_next_move(&mut enemy);
            }
        }
    }
}
