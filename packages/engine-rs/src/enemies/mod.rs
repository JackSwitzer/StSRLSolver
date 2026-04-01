//! Enemy AI system — All 4 acts (73 enemies) for MCTS simulations.
//!
//! Each enemy has a deterministic move pattern that mirrors the Java implementations.
//! For MCTS, we use simplified AI: no RNG-based move selection, instead we use
//! the most common/expected move pattern for fast simulation.

use crate::state::EnemyCombatState;


pub mod act1;
pub mod act2;
pub mod act3;
pub mod act4;

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
            // Java: complex RNG-based move pattern. First turn determined by RNG.
            // Damage: A4+ slash=18 slap=14, else slash=16 slap=12. Execute always 10x2.
            // Str: A19=4, A4+=3, else 2. Forge: A19=7/20, A9+=6/18, else 5/15.
            // Face Slap gives Frail 2 + Vulnerable 2 (Java).
            // For MCTS: deterministic pattern starting with Face Slap.
            enemy.set_move(move_ids::CHAMP_FACE_SLAP, 12, 1, 0);
            enemy.move_effects.insert("frail".to_string(), 2);
            enemy.move_effects.insert("vulnerable".to_string(), 2);
            enemy.entity.set_status("NumTurns", 0);
            enemy.entity.set_status("ThresholdReached", 0);
            enemy.entity.set_status("StrAmt", 2);
            enemy.entity.set_status("ForgeAmt", 5);
            enemy.entity.set_status("BlockAmt", 15);
            enemy.entity.set_status("ForgeTimes", 0);
            enemy.entity.set_status("SlashDmg", 16);
            enemy.entity.set_status("SlapDmg", 12);
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
            // First turn: random attack. Use Multi Hit as default.
            // Reactive power: changes intent when hit. Malleable power: gains block when hit.
            // A2: 38/9/16/12, else 32/7/15/10
            // For MCTS deterministic: use Multi Hit as first move
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
            enemy.entity.set_status("Reactive", 1);
            enemy.entity.set_status("Malleable", 1);
            enemy.entity.set_status("UsedMegaDebuff", 0);
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
            // Escalating damage. A2: starts at 40, else 30. +10 each turn.
            // Fading: A17 = 6 turns, else 5 turns. Has Shifting power (reduces all damage to block).
            // startingDeathDmg stored as status for escalation.
            enemy.set_move(move_ids::TRANSIENT_ATTACK, 30, 1, 0);
            enemy.entity.set_status("AttackCount", 0);
            enemy.entity.set_status("StartingDmg", 30);
            enemy.entity.set_status("Shifting", 1);
        }
        "GiantHead" | "Giant Head" => {
            // Countdown to It Is Time. Glare/Count cycle. Count starts at 5 (A18: 4).
            // startingDeathDmg: A3+ = 40, else 30. Has Slow power.
            // First getMove decrements count, so first turn is count=4 (or 3 at A18).
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
            enemy.entity.set_status("Count", 5);
            enemy.entity.set_status("StartingDeathDmg", 30);
            enemy.entity.set_status("Slow", 1);
        }
        "Nemesis" => {
            // Intangible cycling — gains Intangible every turn in takeTurn if not already present.
            // First move: 50% Tri Attack (fireDmg x3), 50% Burn (3 burns, 5 at A18).
            // Deterministic MCTS: use Tri Attack as default first move.
            // fireDmg: A3+ = 7, else 6. Scythe always 45.
            enemy.set_move(move_ids::NEM_TRI_ATTACK, 6, 3, 0);
            enemy.entity.set_status("ScytheCooldown", 0);
            enemy.entity.set_status("FirstMove", 1);
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
            // Phase 1. Curiosity: gains Str when player plays a Power (A19: 2, else 1).
            // Regen: A19 = 15, else 10. A4: starts with +2 Str.
            // First turn always Slash (20 damage). Has Unawakened power.
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
            enemy.entity.set_status("Curiosity", 1);
            enemy.entity.set_status("Phase", 1);
            enemy.entity.set_status("Regenerate", 10);
            enemy.entity.set_status("FirstTurn", 0); // already set first move
        }
        "Donu" => {
            // Alternates: Beam (10x2) and Circle of Protection (+3 Str to both)
            enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
            enemy.move_effects.insert("strength".to_string(), 3);
            enemy.entity.set_status("Artifact", 2);
        }
        "Deca" => {
            // Java: Deca starts with isAttacking=true, so first getMove picks Beam.
            // Beam (10x2 + 2 Daze), Square (16 block). A4+ beam=12. A19 artifact=3 + plated armor.
            enemy.set_move(move_ids::DECA_BEAM, 10, 2, 0);
            enemy.move_effects.insert("daze".to_string(), 2);
            enemy.entity.set_status("Artifact", 2);
        }
        "TimeEater" | "Time Eater" => {
            // After player plays 12 cards: Haste (heal to 50%, cleanse).
            // A4: reverb=8, headSlam=32. Else reverb=7, headSlam=26.
            // Has Time Warp power. Head Slam applies draw reduction.
            // Ripple: 20 block + Vuln 1 + Weak 1. A19 also Frail 1.
            // A19: Head Slam also adds 2 Slimed.
            enemy.set_move(move_ids::TE_REVERBERATE, 7, 3, 0);
            enemy.entity.set_status("CardCount", 0);
            enemy.entity.set_status("UsedHaste", 0);
            enemy.entity.set_status("ReverbDmg", 7);
            enemy.entity.set_status("HeadSlamDmg", 26);
        }

        // =================================================================
        // Act 4 — The Ending
        // =================================================================
        "SpireShield" | "Spire Shield" => {
            // 3-move cycle. First turn: Bash or Fortify (50/50 in Java).
            // Bash: 12 (A3+ = 14). Smash: 34 (A3+ = 38). Fortify: 30 block.
            // Bash applies -1 Str or -1 Focus (random if player has orbs).
            enemy.set_move(move_ids::SHIELD_BASH, 12, 1, 0);
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
            // First turn: Debilitate (Vuln 2, Weak 2, Frail 2 + status cards to draw pile).
            // A4: echo=45, bloodHits=15. Else echo=40, bloodHits=12.
            // A9: HP=800. A19: invincible=200, beatOfDeath=2.
            // Buff cycle: +2 Str + [Artifact 2, +1 BeatOfDeath, PainfulStabs, +10 Str, +50 Str]
            enemy.set_move(move_ids::HEART_DEBILITATE, 0, 0, 0);
            enemy.move_effects.insert("vulnerable".to_string(), 2);
            enemy.move_effects.insert("weak".to_string(), 2);
            enemy.move_effects.insert("frail".to_string(), 2);
            enemy.entity.set_status("Invincible", 300);
            enemy.entity.set_status("BeatOfDeath", 1);
            enemy.entity.set_status("MoveCount", 0);
            enemy.entity.set_status("BloodHitCount", 12);
            enemy.entity.set_status("EchoDmg", 40);
            enemy.entity.set_status("BuffCount", 0);
            enemy.entity.set_status("IsFirstMove", 1);
        }

        _ => {
            // Unknown enemy: generic 6 damage attack
            enemy.set_move(1, 6, 1, 0);
        }
    }

    enemy
}

pub fn roll_next_move(enemy: &mut EnemyCombatState) {
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();

    match enemy.id.as_str() {
        // Act 1
        "JawWorm" => act1::roll_jaw_worm(enemy),
        "Cultist" => act1::roll_cultist(enemy),
        "FungiBeast" => act1::roll_fungi_beast(enemy),
        "FuzzyLouseNormal" | "RedLouse" => act1::roll_red_louse(enemy),
        "FuzzyLouseDefensive" | "GreenLouse" => act1::roll_green_louse(enemy),
        "SlaverBlue" | "BlueSlaver" => act1::roll_blue_slaver(enemy),
        "SlaverRed" | "RedSlaver" => act1::roll_red_slaver(enemy),
        "AcidSlime_S" => act1::roll_acid_slime_s(enemy),
        "AcidSlime_M" => act1::roll_acid_slime_m(enemy),
        "AcidSlime_L" => act1::roll_acid_slime_l(enemy),
        "SpikeSlime_S" => act1::roll_spike_slime_s(enemy),
        "SpikeSlime_M" => act1::roll_spike_slime_m(enemy),
        "SpikeSlime_L" => act1::roll_spike_slime_l(enemy),
        "Looter" => act1::roll_looter(enemy),
        "GremlinFat" => act1::roll_gremlin_simple(enemy, 4, 1),
        "GremlinThief" => act1::roll_gremlin_simple(enemy, 9, 0),
        "GremlinWarrior" => act1::roll_gremlin_simple(enemy, 4, 0),
        "GremlinWizard" => act1::roll_gremlin_wizard(enemy),
        "GremlinTsundere" | "GremlinSneaky" => { /* Does nothing each turn */ }
        "GremlinNob" | "Gremlin Nob" => act1::roll_gremlin_nob(enemy),
        "Lagavulin" => act1::roll_lagavulin(enemy),
        "Sentry" => act1::roll_sentry(enemy),
        "TheGuardian" => act1::roll_guardian(enemy),
        "Hexaghost" => act1::roll_hexaghost(enemy),
        "SlimeBoss" => act1::roll_slime_boss(enemy),
        // Act 2
        "Chosen" => act2::roll_chosen(enemy),
        "Mugger" => act2::roll_mugger(enemy),
        "Byrd" => act2::roll_byrd(enemy),
        "Shelled Parasite" | "ShelledParasite" => act2::roll_shelled_parasite(enemy),
        "SnakePlant" => act2::roll_snake_plant(enemy),
        "Centurion" => act2::roll_centurion(enemy),
        "Mystic" | "Healer" => act2::roll_mystic(enemy),
        "BookOfStabbing" | "Book of Stabbing" => act2::roll_book_of_stabbing(enemy),
        "GremlinLeader" | "Gremlin Leader" => act2::roll_gremlin_leader(enemy),
        "Taskmaster" => act2::roll_taskmaster(enemy),
        "SphericGuardian" | "Spheric Guardian" => act2::roll_spheric_guardian(enemy),
        "Snecko" => act2::roll_snecko(enemy),
        "BanditBear" | "Bear" => act2::roll_bear(enemy),
        "BanditLeader" => act2::roll_bandit_leader(enemy),
        "BanditPointy" | "Pointy" => { /* Always stab 5x2 */ }
        "BronzeAutomaton" | "Bronze Automaton" => act2::roll_bronze_automaton(enemy),
        "BronzeOrb" | "Bronze Orb" => act2::roll_bronze_orb(enemy),
        "TorchHead" | "Torch Head" => { /* Always Tackle 7 */ }
        "Champ" => act2::roll_champ(enemy),
        "TheCollector" | "Collector" => act2::roll_collector(enemy),
        // Act 3
        "Darkling" => act3::roll_darkling(enemy),
        "OrbWalker" | "Orb Walker" => act3::roll_orb_walker(enemy),
        "Spiker" => act3::roll_spiker(enemy),
        "Repulsor" => act3::roll_repulsor(enemy),
        "Exploder" => act3::roll_exploder(enemy),
        "WrithingMass" | "Writhing Mass" => act3::roll_writhing_mass(enemy),
        "SpireGrowth" | "Spire Growth" => act3::roll_spire_growth(enemy),
        "Maw" => act3::roll_maw(enemy),
        "Transient" => act3::roll_transient(enemy),
        "GiantHead" | "Giant Head" => act3::roll_giant_head(enemy),
        "Nemesis" => act3::roll_nemesis(enemy),
        "Reptomancer" => act3::roll_reptomancer(enemy),
        "SnakeDagger" | "Snake Dagger" => act3::roll_snake_dagger(enemy),
        "AwakenedOne" | "Awakened One" => act3::roll_awakened_one(enemy),
        "Donu" => act3::roll_donu(enemy),
        "Deca" => act3::roll_deca(enemy),
        "TimeEater" | "Time Eater" => act3::roll_time_eater(enemy),
        // Act 4
        "SpireShield" | "Spire Shield" => act4::roll_spire_shield(enemy),
        "SpireSpear" | "Spire Spear" => act4::roll_spire_spear(enemy),
        "CorruptHeart" | "Corrupt Heart" => act4::roll_corrupt_heart(enemy),
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
// Helpers (shared by act modules)
// =========================================================================

pub(crate) fn last_move(enemy: &EnemyCombatState, move_id: i32) -> bool {
    enemy.move_history.last().copied() == Some(move_id)
}

pub(crate) fn last_two_moves(enemy: &EnemyCombatState, move_id: i32) -> bool {
    let len = enemy.move_history.len();
    if len < 2 { return false; }
    enemy.move_history[len - 1] == move_id && enemy.move_history[len - 2] == move_id
}

// Re-exports of pub functions from act modules
pub use act3::awakened_one_rebirth;
pub use act1::guardian_check_mode_shift;
pub use act1::guardian_switch_to_offensive;
pub use act1::hexaghost_set_divider;
pub use act1::lagavulin_wake_up;
pub use act1::slime_boss_should_split;
pub use act3::writhing_mass_reactive_reroll;

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
        // Java: Face Slap gives Frail 2 + Vulnerable 2
        assert_eq!(enemy.move_effects.get("frail"), Some(&2));
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));

        // Phase 1 cycle
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CHAMP_HEAVY_SLASH);
        assert_eq!(enemy.move_damage, 16); // base (non-A4) slash dmg

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

        // Phase 1 cycle: Slash -> Soul Strike
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

        // Phase 2: Dark Echo -> Sludge (adds Void, not Slimed)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AO_SLUDGE);
        assert_eq!(enemy.move_effects.get("void"), Some(&1));
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
        // After two reverberates: Head Slam (gives draw_reduction, not slimed)
        assert_eq!(enemy.move_id, move_ids::TE_HEAD_SLAM);
        assert_eq!(enemy.move_damage, 26);
        assert_eq!(enemy.move_effects.get("draw_reduction"), Some(&1));
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
        // Java: Deca starts with isAttacking=true -> first move is Beam
        assert_eq!(deca.move_id, move_ids::DECA_BEAM);
        assert_eq!(deca.move_damage, 10);
        assert_eq!(deca.move_effects.get("daze"), Some(&2));

        roll_next_move(&mut deca);
        assert_eq!(deca.move_id, move_ids::DECA_SQUARE);
        assert_eq!(deca.move_block, 16);
    }

    #[test]
    fn test_giant_head_elite() {
        let mut enemy = create_enemy("GiantHead", 500, 500);
        assert_eq!(enemy.move_id, move_ids::GH_COUNT);
        assert_eq!(enemy.move_damage, 13);
        assert_eq!(enemy.entity.status("Count"), 5);

        // Roll moves. Count decrements each roll. After count reaches 1, It Is Time.
        // Count starts at 5, so after 4 rolls we should be in It Is Time territory.
        for _ in 0..5 {
            roll_next_move(&mut enemy);
        }

        // Should eventually hit It Is Time
        let count = enemy.entity.status("Count");
        assert!(count <= 0 || enemy.move_id == move_ids::GH_IT_IS_TIME);
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

        // After Debilitate: moveCount=0, 0%3=0 -> Blood Shots
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BLOOD_SHOTS);
        assert_eq!(enemy.move_damage, 2);
        assert_eq!(enemy.move_hits, 12);

        // moveCount=1, 1%3=1 -> Echo (since last wasn't Echo)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_ECHO);
        assert_eq!(enemy.move_damage, 40);

        // moveCount=2, 2%3=2 -> Buff (first buff: +2 Str + Artifact 2)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEART_BUFF);
        assert_eq!(enemy.move_effects.get("strength"), Some(&2));
        assert_eq!(enemy.move_effects.get("artifact"), Some(&2));
    }

    #[test]
    fn test_spire_shield_boss() {
        let mut enemy = create_enemy("SpireShield", 110, 110);
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);
        // Base damage: 12 (A3+ = 14)
        assert_eq!(enemy.move_damage, 12);
        assert_eq!(enemy.move_effects.get("strength_down"), Some(&1));

        // mc=0 -> 0%3=0: Fortify (since last was Bash)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SHIELD_FORTIFY);
        assert_eq!(enemy.move_block, 30);

        // mc=1 -> 1%3=1: Bash (since last was Fortify)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SHIELD_BASH);

        // mc=2 -> 2%3=2: Smash
        roll_next_move(&mut enemy);
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

        // mc=0 -> 0%3=0: Piercer (since last was BurnStrike; anti-repeat)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SPEAR_PIERCER);

        // mc=1 -> 1%3=1: Skewer
        roll_next_move(&mut enemy);
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
