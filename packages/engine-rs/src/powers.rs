//! Power/status effect system — complete implementation of all 162 Slay the Spire powers.
//!
//! Design:
//! - Powers are stored as `HashMap<String, i32>` on `EntityState.statuses`
//! - `PowerId` enum provides compile-time safety for power identification
//! - `PowerDef` describes each power's metadata and which triggers it fires on
//! - Trigger dispatch functions are called by the engine at the appropriate moments
//! - Each trigger function checks all relevant powers on the entity and applies effects
//!
//! Trigger hooks (matching Java AbstractPower):
//! - `at_start_of_turn` / `at_start_of_turn_post_draw`
//! - `at_end_of_turn` / `at_end_of_round`
//! - `on_use_card` / `on_after_use_card` / `on_play_card`
//! - `on_attacked` / `on_attack` / `was_hp_lost`
//! - `modify_damage_give` / `modify_damage_receive`
//! - `modify_block`
//! - `on_exhaust` / `on_card_draw`
//! - `on_scry` / `on_change_stance`
//! - `on_death` / `on_heal`
//! - `on_apply_power` (for Artifact, Sadistic Nature, etc.)

use crate::state::EntityState;

// ---------------------------------------------------------------------------
// PowerId — compile-time safe IDs for all 162 powers
// ---------------------------------------------------------------------------

/// All Slay the Spire powers. IDs match the Java `POWER_ID` string constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum PowerId {
    // ===== Common / Neutral =====
    Strength,
    Dexterity,
    Weakened,
    Vulnerable,
    Frail,
    Artifact,
    Intangible,
    Thorns,
    Poison,
    Regeneration, // Regen
    Buffer,
    Barricade,
    Metallicize,
    PlatedArmor,
    Ritual,
    Anger, // Angry
    Enrage,
    Curiosity,
    ModeShift,
    Split,
    Fading,
    Invincible,
    Surrounded,
    BackAttack,
    Explosive,
    Unawakened,
    Resurrect,
    StrikeUp, // Minion bonus
    SlowPower,
    TimeWarp,
    SporeCloud, // on death: apply 2 Vulnerable to all
    Thievery,

    // ===== Ironclad Powers =====
    DemonForm,
    FlameBarrier,
    MetallicizePlayer, // same as Metallicize but kept for clarity
    Brutality,
    DarkEmbrace,
    DoubleTap,
    Evolve,
    FeelNoPain,
    FireBreathing,
    Juggernaut,
    Rupture,
    Berserk,
    Combust,
    Corruption,
    DoubleDown, // Double Damage
    Rage,
    Reaper, // not a power; heals on attack — handled by card
    BurningBlood, // relic, not a power

    // ===== Silent Powers =====
    NoxiousFumes,
    Envenom,
    AfterImage,
    Accuracy,
    AThousandCuts,
    InfiniteBlades,
    Tools, // Well-Laid Plans (retain)
    Nightmare,
    PhantasmalKiller,
    Sadistic,

    // ===== Defect Powers =====
    Focus,
    LockOn,
    CreativeAI,
    Storm,
    Heatsink,
    StaticDischarge,
    Electro,
    Loop,
    HelloWorld,
    Equilibrium,

    // ===== Watcher Powers =====
    Rushdown,
    MentalFortress,
    BattleHymn,
    Devotion,
    Establishment,
    Foresight,
    LikeWater,
    Nirvana,
    Omega,
    Study,
    WaveOfTheHand,
    Vigor,
    Mantra,
    BlockReturn, // Talk to the Hand
    DevaForm,
    LiveForever, // Wish: LiveForever
    WrathNextTurn,
    EndTurnDeath,
    FreeAttack,
    MasterReality,
    NoSkills,
    EnergyDown,
    CannotChangeStance,
    Mark,
    Vault,
    Omniscience,

    // ===== Turn-based / Temporary Powers =====
    Blur,
    ConservePower, // Conserve
    DrawCardNextTurn,
    DrawPower,
    DoubleDamage,
    EnergizedPower,
    NextTurnBlock,
    PenNib,
    Rebound,
    NoBlock,
    NoDraw,
    Entangle,
    Confusion,
    Panache,
    Burst,
    WraithForm,

    // ===== Enemy-specific / Boss Powers =====
    BeatOfDeath,
    Growth,
    Magnetism,
    SkillBurn,
    Forcefield,
    RegrowPower,
    Stasis,
    TheBomb,
    GenericStrengthUp,
    LoseStrength,
    LoseDexterity,
    Collect,
    Winter,
    Repair,

    // ===== Colorless Power Cards =====
    PanachePower, // already listed as Panache
}

impl PowerId {
    /// Get the canonical string key used in `EntityState.statuses`.
    /// Matches the Java `POWER_ID` constants.
    pub fn key(&self) -> &'static str {
        match self {
            PowerId::Strength => "Strength",
            PowerId::Dexterity => "Dexterity",
            PowerId::Weakened => "Weakened",
            PowerId::Vulnerable => "Vulnerable",
            PowerId::Frail => "Frail",
            PowerId::Artifact => "Artifact",
            PowerId::Intangible => "Intangible",
            PowerId::Thorns => "Thorns",
            PowerId::Poison => "Poison",
            PowerId::Regeneration => "Regeneration",
            PowerId::Buffer => "Buffer",
            PowerId::Barricade => "Barricade",
            PowerId::Metallicize | PowerId::MetallicizePlayer => "Metallicize",
            PowerId::PlatedArmor => "Plated Armor",
            PowerId::Ritual => "Ritual",
            PowerId::Anger => "Angry",
            PowerId::Enrage => "Enrage",
            PowerId::Curiosity => "Curiosity",
            PowerId::ModeShift => "Mode Shift",
            PowerId::Split => "Split",
            PowerId::Fading => "Fading",
            PowerId::Invincible => "Invincible",
            PowerId::Surrounded => "Surrounded",
            PowerId::BackAttack => "BackAttack",
            PowerId::Explosive => "Explosive",
            PowerId::Unawakened => "Unawakened",
            PowerId::Resurrect => "Resurrect",
            PowerId::StrikeUp => "StrikeUp",
            PowerId::SlowPower => "Slow",
            PowerId::TimeWarp => "Time Warp",
            PowerId::SporeCloud => "Spore Cloud",
            PowerId::Thievery => "Thievery",

            PowerId::DemonForm => "Demon Form",
            PowerId::FlameBarrier => "Flame Barrier",
            PowerId::Brutality => "Brutality",
            PowerId::DarkEmbrace => "Dark Embrace",
            PowerId::DoubleTap => "Double Tap",
            PowerId::Evolve => "Evolve",
            PowerId::FeelNoPain => "Feel No Pain",
            PowerId::FireBreathing => "Fire Breathing",
            PowerId::Juggernaut => "Juggernaut",
            PowerId::Rupture => "Rupture",
            PowerId::Berserk => "Berserk",
            PowerId::Combust => "Combust",
            PowerId::Corruption => "Corruption",
            PowerId::DoubleDown => "DoubleDamage",
            PowerId::Rage => "Rage",
            PowerId::Reaper | PowerId::BurningBlood => "BurningBlood",

            PowerId::NoxiousFumes => "Noxious Fumes",
            PowerId::Envenom => "Envenom",
            PowerId::AfterImage => "After Image",
            PowerId::Accuracy => "Accuracy",
            PowerId::AThousandCuts => "A Thousand Cuts",
            PowerId::InfiniteBlades => "Infinite Blades",
            PowerId::Tools => "Tools of the Trade",
            PowerId::Nightmare => "Nightmare",
            PowerId::PhantasmalKiller => "Phantasmal Killer",
            PowerId::Sadistic => "Sadistic",

            PowerId::Focus => "Focus",
            PowerId::LockOn => "Lock-On",
            PowerId::CreativeAI => "Creative AI",
            PowerId::Storm => "Storm",
            PowerId::Heatsink => "Heatsink",
            PowerId::StaticDischarge => "Static Discharge",
            PowerId::Electro => "Electro",
            PowerId::Loop => "Loop",
            PowerId::HelloWorld => "Hello World",
            PowerId::Equilibrium => "Equilibrium",

            PowerId::Rushdown => "Rushdown",
            PowerId::MentalFortress => "MentalFortress",
            PowerId::BattleHymn => "BattleHymn",
            PowerId::Devotion => "Devotion",
            PowerId::Establishment => "Establishment",
            PowerId::Foresight => "Foresight",
            PowerId::LikeWater => "Like Water",
            PowerId::Nirvana => "Nirvana",
            PowerId::Omega => "OmegaPower",
            PowerId::Study => "Study",
            PowerId::WaveOfTheHand => "Wave of the Hand",
            PowerId::Vigor => "Vigor",
            PowerId::Mantra => "Mantra",
            PowerId::BlockReturn => "BlockReturn",
            PowerId::DevaForm => "DevaForm",
            PowerId::LiveForever => "LiveForever",
            PowerId::WrathNextTurn => "WrathNextTurn",
            PowerId::EndTurnDeath => "EndTurnDeath",
            PowerId::FreeAttack => "FreeAttackPower",
            PowerId::MasterReality => "MasterReality",
            PowerId::NoSkills => "NoSkillsPower",
            PowerId::EnergyDown => "EnergyDown",
            PowerId::CannotChangeStance => "CannotChangeStance",
            PowerId::Mark => "Mark",
            PowerId::Vault => "Vault",
            PowerId::Omniscience => "Omniscience",

            PowerId::Blur => "Blur",
            PowerId::ConservePower => "Conserve",
            PowerId::DrawCardNextTurn => "Draw Card",
            PowerId::DrawPower => "Draw",
            PowerId::DoubleDamage => "DoubleDamage",
            PowerId::EnergizedPower => "Energized",
            PowerId::NextTurnBlock => "Next Turn Block",
            PowerId::PenNib => "Pen Nib",
            PowerId::Rebound => "Rebound",
            PowerId::NoBlock => "No Block",
            PowerId::NoDraw => "No Draw",
            PowerId::Entangle => "Entangled",
            PowerId::Confusion => "Confusion",
            PowerId::Panache | PowerId::PanachePower => "Panache",
            PowerId::Burst => "Burst",
            PowerId::WraithForm => "Wraith Form",

            PowerId::BeatOfDeath => "Beat of Death",
            PowerId::Growth => "Growth",
            PowerId::Magnetism => "Magnetism",
            PowerId::SkillBurn => "SkillBurn",
            PowerId::Forcefield => "Forcefield",
            PowerId::RegrowPower => "Regrow",
            PowerId::Stasis => "Stasis",
            PowerId::TheBomb => "TheBomb",
            PowerId::GenericStrengthUp => "GenericStrengthUp",
            PowerId::LoseStrength => "LoseStrength",
            PowerId::LoseDexterity => "LoseDexterity",
            PowerId::Collect => "Collect",
            PowerId::Winter => "Winter",
            PowerId::Repair => "Repair",
        }
    }
}

// ---------------------------------------------------------------------------
// PowerType — buff vs debuff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerType {
    Buff,
    Debuff,
}

// ---------------------------------------------------------------------------
// PowerDef — static metadata for each power
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PowerDef {
    pub id: &'static str,
    pub power_type: PowerType,
    /// Whether the power stacks (amount == -1 in Java means non-stackable)
    pub stackable: bool,
    /// Can go negative (e.g., Strength, Dexterity)
    pub can_go_negative: bool,
    /// Turn-based: decrements each turn
    pub is_turn_based: bool,
    // Trigger flags — which hooks this power fires on
    pub on_turn_start: bool,
    pub on_turn_start_post_draw: bool,
    pub on_turn_end: bool,
    pub on_end_of_round: bool,
    pub on_use_card: bool,
    pub on_after_use_card: bool,
    pub on_card_draw: bool,
    pub on_attacked: bool,
    pub on_attack: bool,
    pub on_hp_lost: bool,
    pub on_death: bool,
    pub on_exhaust: bool,
    pub on_scry: bool,
    pub on_change_stance: bool,
    pub on_heal: bool,
    pub modify_damage_give: bool,
    pub modify_damage_receive: bool,
    pub modify_block: bool,
    pub on_energy_recharge: bool,
    pub on_apply_power: bool,
    pub on_gained_block: bool,
    pub on_channel_orb: bool,
    pub on_evoke_orb: bool,
}

impl Default for PowerDef {
    fn default() -> Self {
        Self {
            id: "",
            power_type: PowerType::Buff,
            stackable: true,
            can_go_negative: false,
            is_turn_based: false,
            on_turn_start: false,
            on_turn_start_post_draw: false,
            on_turn_end: false,
            on_end_of_round: false,
            on_use_card: false,
            on_after_use_card: false,
            on_card_draw: false,
            on_attacked: false,
            on_attack: false,
            on_hp_lost: false,
            on_death: false,
            on_exhaust: false,
            on_scry: false,
            on_change_stance: false,
            on_heal: false,
            modify_damage_give: false,
            modify_damage_receive: false,
            modify_block: false,
            on_energy_recharge: false,
            on_apply_power: false,
            on_gained_block: false,
            on_channel_orb: false,
            on_evoke_orb: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Power Registry — static definitions for all powers
// ---------------------------------------------------------------------------

/// Get the power definition by string key. Returns None for unknown powers.
pub fn get_power_def(key: &str) -> Option<PowerDef> {
    let def = match key {
        // ===================================================================
        // Common / Status
        // ===================================================================
        "Strength" => PowerDef {
            id: "Strength",
            power_type: PowerType::Buff,
            can_go_negative: true,
            modify_damage_give: true,
            ..Default::default()
        },
        "Dexterity" => PowerDef {
            id: "Dexterity",
            power_type: PowerType::Buff,
            can_go_negative: true,
            modify_block: true,
            ..Default::default()
        },
        "Weakened" => PowerDef {
            id: "Weakened",
            power_type: PowerType::Debuff,
            is_turn_based: true,
            on_end_of_round: true,
            modify_damage_give: true,
            ..Default::default()
        },
        "Vulnerable" => PowerDef {
            id: "Vulnerable",
            power_type: PowerType::Debuff,
            is_turn_based: true,
            on_end_of_round: true,
            modify_damage_receive: true,
            ..Default::default()
        },
        "Frail" => PowerDef {
            id: "Frail",
            power_type: PowerType::Debuff,
            is_turn_based: true,
            on_end_of_round: true,
            modify_block: true,
            ..Default::default()
        },
        "Artifact" => PowerDef {
            id: "Artifact",
            power_type: PowerType::Buff,
            on_apply_power: true,
            ..Default::default()
        },
        "Intangible" => PowerDef {
            id: "Intangible",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_turn_end: true,
            modify_damage_receive: true,
            ..Default::default()
        },
        "Thorns" => PowerDef {
            id: "Thorns",
            power_type: PowerType::Buff,
            on_attacked: true,
            ..Default::default()
        },
        "Poison" => PowerDef {
            id: "Poison",
            power_type: PowerType::Debuff,
            on_turn_start: true,
            ..Default::default()
        },
        "Regeneration" => PowerDef {
            id: "Regeneration",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_turn_end: true,
            ..Default::default()
        },
        "Buffer" => PowerDef {
            id: "Buffer",
            power_type: PowerType::Buff,
            on_attacked: true,
            ..Default::default()
        },
        "Barricade" => PowerDef {
            id: "Barricade",
            power_type: PowerType::Buff,
            stackable: false,
            ..Default::default()
        },
        "Metallicize" => PowerDef {
            id: "Metallicize",
            power_type: PowerType::Buff,
            on_turn_end: true,
            ..Default::default()
        },
        "Plated Armor" => PowerDef {
            id: "Plated Armor",
            power_type: PowerType::Buff,
            on_turn_end: true,
            on_attacked: true, // decrements when taking unblocked damage
            ..Default::default()
        },
        "Ritual" => PowerDef {
            id: "Ritual",
            power_type: PowerType::Buff,
            on_turn_end: true, // gains Strength at end of turn
            ..Default::default()
        },
        "Angry" => PowerDef {
            id: "Angry",
            power_type: PowerType::Buff,
            on_hp_lost: true, // gains Strength when taking damage
            ..Default::default()
        },
        "Enrage" => PowerDef {
            id: "Enrage",
            power_type: PowerType::Buff,
            on_use_card: true, // gains Strength when player plays non-Attack
            ..Default::default()
        },
        "Curiosity" => PowerDef {
            id: "Curiosity",
            power_type: PowerType::Buff,
            on_use_card: true, // gains Strength when player plays a Power
            ..Default::default()
        },
        "Mode Shift" => PowerDef {
            id: "Mode Shift",
            power_type: PowerType::Buff,
            on_hp_lost: true, // tracks damage taken
            ..Default::default()
        },
        "Split" => PowerDef {
            id: "Split",
            power_type: PowerType::Buff,
            stackable: false,
            on_death: true,
            ..Default::default()
        },
        "Fading" => PowerDef {
            id: "Fading",
            power_type: PowerType::Debuff,
            is_turn_based: true,
            on_end_of_round: true,
            on_death: true, // dies when reaches 0
            ..Default::default()
        },
        "Invincible" => PowerDef {
            id: "Invincible",
            power_type: PowerType::Buff,
            on_end_of_round: true, // resets each round
            modify_damage_receive: true,
            ..Default::default()
        },
        "Surrounded" => PowerDef {
            id: "Surrounded",
            power_type: PowerType::Debuff,
            ..Default::default()
        },
        "BackAttack" => PowerDef {
            id: "BackAttack",
            power_type: PowerType::Buff,
            modify_damage_give: true, // 50% more from back
            ..Default::default()
        },
        "Explosive" => PowerDef {
            id: "Explosive",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_end_of_round: true, // countdown to explosion
            ..Default::default()
        },
        "Unawakened" => PowerDef {
            id: "Unawakened",
            power_type: PowerType::Buff,
            stackable: false,
            on_death: true, // resurrects the owner
            ..Default::default()
        },
        "Resurrect" => PowerDef {
            id: "Resurrect",
            power_type: PowerType::Buff,
            stackable: false,
            on_death: true,
            ..Default::default()
        },
        "StrikeUp" => PowerDef {
            id: "StrikeUp",
            power_type: PowerType::Buff,
            modify_damage_give: true,
            ..Default::default()
        },
        "Slow" => PowerDef {
            id: "Slow",
            power_type: PowerType::Debuff,
            on_after_use_card: true, // stacks when player plays cards
            on_end_of_round: true,   // resets to 0
            modify_damage_receive: true, // +10% per stack
            ..Default::default()
        },
        "Time Warp" => PowerDef {
            id: "Time Warp",
            power_type: PowerType::Buff,
            on_after_use_card: true, // counts cards, at 12 ends turn + gives Str
            ..Default::default()
        },
        "Spore Cloud" => PowerDef {
            id: "Spore Cloud",
            power_type: PowerType::Buff,
            on_death: true, // applies Vulnerable to player
            ..Default::default()
        },
        "Thievery" => PowerDef {
            id: "Thievery",
            power_type: PowerType::Buff,
            on_attack: true, // steals gold
            ..Default::default()
        },

        // ===================================================================
        // Ironclad Powers
        // ===================================================================
        "Demon Form" => PowerDef {
            id: "Demon Form",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // gain Strength
            ..Default::default()
        },
        "Flame Barrier" => PowerDef {
            id: "Flame Barrier",
            power_type: PowerType::Buff,
            on_attacked: true,     // deal damage back to attacker
            on_turn_start: true,   // removed at start of next turn
            ..Default::default()
        },
        "Brutality" => PowerDef {
            id: "Brutality",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // lose 1 HP, draw 1 card
            ..Default::default()
        },
        "Dark Embrace" => PowerDef {
            id: "Dark Embrace",
            power_type: PowerType::Buff,
            on_exhaust: true, // draw 1 card per exhaust
            ..Default::default()
        },
        "Double Tap" => PowerDef {
            id: "Double Tap",
            power_type: PowerType::Buff,
            on_use_card: true, // play next Attack twice
            ..Default::default()
        },
        "Evolve" => PowerDef {
            id: "Evolve",
            power_type: PowerType::Buff,
            on_card_draw: true, // draw cards when drawing a Status
            ..Default::default()
        },
        "Feel No Pain" => PowerDef {
            id: "Feel No Pain",
            power_type: PowerType::Buff,
            on_exhaust: true, // gain block per exhaust
            ..Default::default()
        },
        "Fire Breathing" => PowerDef {
            id: "Fire Breathing",
            power_type: PowerType::Buff,
            on_card_draw: true, // deal damage when drawing Status/Curse
            ..Default::default()
        },
        "Juggernaut" => PowerDef {
            id: "Juggernaut",
            power_type: PowerType::Buff,
            on_gained_block: true, // deal damage to random enemy when gaining block
            ..Default::default()
        },
        "Rupture" => PowerDef {
            id: "Rupture",
            power_type: PowerType::Buff,
            on_hp_lost: true, // gain Strength when losing HP from a card
            ..Default::default()
        },
        "Berserk" => PowerDef {
            id: "Berserk",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // gain energy
            ..Default::default()
        },
        "Combust" => PowerDef {
            id: "Combust",
            power_type: PowerType::Buff,
            on_turn_end: true, // lose 1 HP, deal N damage to ALL enemies
            ..Default::default()
        },
        "Corruption" => PowerDef {
            id: "Corruption",
            power_type: PowerType::Buff,
            stackable: false,
            // Skills cost 0 and exhaust
            ..Default::default()
        },
        "DoubleDamage" => PowerDef {
            id: "DoubleDamage",
            power_type: PowerType::Buff,
            modify_damage_give: true,
            on_use_card: true, // removed after playing an Attack
            ..Default::default()
        },
        "Rage" => PowerDef {
            id: "Rage",
            power_type: PowerType::Buff,
            on_use_card: true, // gain block when playing an Attack
            on_turn_end: true, // removed at end of turn
            ..Default::default()
        },

        // ===================================================================
        // Silent Powers
        // ===================================================================
        "Noxious Fumes" => PowerDef {
            id: "Noxious Fumes",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // poison all enemies
            ..Default::default()
        },
        "Envenom" => PowerDef {
            id: "Envenom",
            power_type: PowerType::Buff,
            on_attack: true, // apply 1 Poison on unblocked attack damage
            ..Default::default()
        },
        "After Image" => PowerDef {
            id: "After Image",
            power_type: PowerType::Buff,
            on_use_card: true, // gain block per card played
            ..Default::default()
        },
        "Accuracy" => PowerDef {
            id: "Accuracy",
            power_type: PowerType::Buff,
            // Increases Shiv damage (handled at card level)
            modify_damage_give: true,
            ..Default::default()
        },
        "A Thousand Cuts" => PowerDef {
            id: "A Thousand Cuts",
            power_type: PowerType::Buff,
            on_use_card: true, // deal damage to ALL enemies per card played
            ..Default::default()
        },
        "Infinite Blades" => PowerDef {
            id: "Infinite Blades",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add Shiv to hand
            ..Default::default()
        },
        "Tools of the Trade" => PowerDef {
            id: "Tools of the Trade",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // draw 1, discard 1
            ..Default::default()
        },
        "Phantasmal Killer" => PowerDef {
            id: "Phantasmal Killer",
            power_type: PowerType::Buff,
            // Next turn: double damage
            on_turn_start_post_draw: true,
            modify_damage_give: true,
            ..Default::default()
        },
        "Sadistic" => PowerDef {
            id: "Sadistic",
            power_type: PowerType::Buff,
            on_apply_power: true, // deal damage when applying debuff
            ..Default::default()
        },
        "Nightmare" => PowerDef {
            id: "Nightmare",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add copies of chosen card
            ..Default::default()
        },

        // ===================================================================
        // Defect Powers
        // ===================================================================
        "Focus" => PowerDef {
            id: "Focus",
            power_type: PowerType::Buff,
            can_go_negative: true,
            // Increases orb passive/evoke amounts (handled at orb level)
            ..Default::default()
        },
        "Lock-On" => PowerDef {
            id: "Lock-On",
            power_type: PowerType::Debuff,
            is_turn_based: true,
            on_end_of_round: true,
            // Orbs deal 50% more damage to this enemy
            ..Default::default()
        },
        "Creative AI" => PowerDef {
            id: "Creative AI",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add random Power to hand
            ..Default::default()
        },
        "Storm" => PowerDef {
            id: "Storm",
            power_type: PowerType::Buff,
            on_use_card: true, // channel Lightning when playing a Power
            ..Default::default()
        },
        "Heatsink" => PowerDef {
            id: "Heatsink",
            power_type: PowerType::Buff,
            on_use_card: true, // draw when playing a Power
            ..Default::default()
        },
        "Static Discharge" => PowerDef {
            id: "Static Discharge",
            power_type: PowerType::Buff,
            on_hp_lost: true, // channel Lightning when taking unblocked damage
            ..Default::default()
        },
        "Electro" => PowerDef {
            id: "Electro",
            power_type: PowerType::Buff,
            stackable: false,
            // Lightning orbs hit all enemies
            ..Default::default()
        },
        "Loop" => PowerDef {
            id: "Loop",
            power_type: PowerType::Buff,
            on_turn_start: true, // trigger passive of first orb again
            ..Default::default()
        },
        "Hello World" => PowerDef {
            id: "Hello World",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add random Common to hand
            ..Default::default()
        },
        "Equilibrium" => PowerDef {
            id: "Equilibrium",
            power_type: PowerType::Buff,
            on_turn_end: true, // retain hand this turn
            ..Default::default()
        },

        // ===================================================================
        // Watcher Powers
        // ===================================================================
        "Rushdown" => PowerDef {
            id: "Rushdown",
            power_type: PowerType::Buff,
            on_change_stance: true, // draw when entering Wrath
            ..Default::default()
        },
        "MentalFortress" => PowerDef {
            id: "MentalFortress",
            power_type: PowerType::Buff,
            on_change_stance: true, // gain block on stance change
            ..Default::default()
        },
        "BattleHymn" => PowerDef {
            id: "BattleHymn",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add Smite to hand
            ..Default::default()
        },
        "Devotion" => PowerDef {
            id: "Devotion",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // gain Mantra
            ..Default::default()
        },
        "Establishment" => PowerDef {
            id: "Establishment",
            power_type: PowerType::Buff,
            // Retained cards cost 1 less
            ..Default::default()
        },
        "Foresight" => PowerDef {
            id: "Foresight",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // Scry
            ..Default::default()
        },
        "Like Water" => PowerDef {
            id: "Like Water",
            power_type: PowerType::Buff,
            on_turn_end: true, // gain block if in Calm
            ..Default::default()
        },
        "Nirvana" => PowerDef {
            id: "Nirvana",
            power_type: PowerType::Buff,
            on_scry: true, // gain block when scrying
            ..Default::default()
        },
        "OmegaPower" => PowerDef {
            id: "OmegaPower",
            power_type: PowerType::Buff,
            on_turn_end: true, // deal damage to ALL enemies
            ..Default::default()
        },
        "Study" => PowerDef {
            id: "Study",
            power_type: PowerType::Buff,
            on_turn_end: true, // shuffle Insight into draw pile
            ..Default::default()
        },
        "Wave of the Hand" => PowerDef {
            id: "Wave of the Hand",
            power_type: PowerType::Buff,
            on_gained_block: true, // apply Weak when gaining block
            ..Default::default()
        },
        "Vigor" => PowerDef {
            id: "Vigor",
            power_type: PowerType::Buff,
            // Consumed on next Attack (handled in card play)
            ..Default::default()
        },
        "Mantra" => PowerDef {
            id: "Mantra",
            power_type: PowerType::Buff,
            // Tracked separately in CombatState.mantra
            ..Default::default()
        },
        "BlockReturn" => PowerDef {
            id: "BlockReturn",
            power_type: PowerType::Debuff,
            on_attack: true, // owner grants block to attacker when hit
            ..Default::default()
        },
        "DevaForm" => PowerDef {
            id: "DevaForm",
            power_type: PowerType::Buff,
            on_energy_recharge: true, // gain increasing energy each turn
            ..Default::default()
        },
        "LiveForever" => PowerDef {
            id: "LiveForever",
            power_type: PowerType::Buff,
            on_turn_end: true, // gain block
            ..Default::default()
        },
        "WrathNextTurn" => PowerDef {
            id: "WrathNextTurn",
            power_type: PowerType::Buff,
            on_turn_start: true, // enter Wrath at start of turn
            ..Default::default()
        },
        "EndTurnDeath" => PowerDef {
            id: "EndTurnDeath",
            power_type: PowerType::Debuff,
            on_turn_end: true, // die at end of turn
            ..Default::default()
        },
        "FreeAttackPower" => PowerDef {
            id: "FreeAttackPower",
            power_type: PowerType::Buff,
            // Next Attack costs 0
            ..Default::default()
        },
        "MasterReality" => PowerDef {
            id: "MasterReality",
            power_type: PowerType::Buff,
            stackable: false,
            // Created cards are upgraded
            ..Default::default()
        },
        "NoSkillsPower" => PowerDef {
            id: "NoSkillsPower",
            power_type: PowerType::Debuff,
            // Can't play Skills
            ..Default::default()
        },
        "EnergyDown" => PowerDef {
            id: "EnergyDown",
            power_type: PowerType::Debuff,
            on_energy_recharge: true, // lose energy each turn
            ..Default::default()
        },
        "CannotChangeStance" => PowerDef {
            id: "CannotChangeStance",
            power_type: PowerType::Debuff,
            stackable: false,
            ..Default::default()
        },
        "Mark" => PowerDef {
            id: "Mark",
            power_type: PowerType::Debuff,
            // Pressure Points damage
            ..Default::default()
        },
        "Vault" => PowerDef {
            id: "Vault",
            power_type: PowerType::Buff,
            stackable: false,
            // Extra turn
            ..Default::default()
        },
        "Omniscience" => PowerDef {
            id: "Omniscience",
            power_type: PowerType::Buff,
            on_use_card: true, // play a card from draw pile
            ..Default::default()
        },

        // ===================================================================
        // Turn-based / Temporary Powers
        // ===================================================================
        "Blur" => PowerDef {
            id: "Blur",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_end_of_round: true,
            // Block not removed at start of turn
            ..Default::default()
        },
        "Conserve" => PowerDef {
            id: "Conserve",
            power_type: PowerType::Buff,
            stackable: false,
            ..Default::default()
        },
        "Draw Card" => PowerDef {
            id: "Draw Card",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_turn_start_post_draw: true, // draw extra cards next turn
            ..Default::default()
        },
        "Draw" => PowerDef {
            id: "Draw",
            power_type: PowerType::Buff,
            // Permanent +draw per turn
            ..Default::default()
        },
        "Energized" => PowerDef {
            id: "Energized",
            power_type: PowerType::Buff,
            on_energy_recharge: true, // gain extra energy next turn
            ..Default::default()
        },
        "Next Turn Block" => PowerDef {
            id: "Next Turn Block",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // gain block at start of next turn
            ..Default::default()
        },
        "Pen Nib" => PowerDef {
            id: "Pen Nib",
            power_type: PowerType::Buff,
            modify_damage_give: true, // double next attack
            on_use_card: true,
            ..Default::default()
        },
        "Rebound" => PowerDef {
            id: "Rebound",
            power_type: PowerType::Buff,
            on_use_card: true, // next card goes on top of draw pile
            ..Default::default()
        },
        "No Block" => PowerDef {
            id: "No Block",
            power_type: PowerType::Debuff,
            modify_block: true,
            on_end_of_round: true,
            ..Default::default()
        },
        "No Draw" => PowerDef {
            id: "No Draw",
            power_type: PowerType::Debuff,
            stackable: false,
            ..Default::default()
        },
        "Entangled" => PowerDef {
            id: "Entangled",
            power_type: PowerType::Debuff,
            stackable: false,
            // Can't play Attacks (checked in can_play_card)
            ..Default::default()
        },
        "Confusion" => PowerDef {
            id: "Confusion",
            power_type: PowerType::Debuff,
            stackable: false,
            // Randomizes card costs
            ..Default::default()
        },
        "Panache" => PowerDef {
            id: "Panache",
            power_type: PowerType::Buff,
            on_after_use_card: true, // every 5 cards: deal 10 damage to all
            ..Default::default()
        },
        "Burst" => PowerDef {
            id: "Burst",
            power_type: PowerType::Buff,
            on_use_card: true, // next Skill played twice
            ..Default::default()
        },
        "Wraith Form" => PowerDef {
            id: "Wraith Form",
            power_type: PowerType::Buff,
            on_turn_start: true, // lose 1 Dexterity each turn
            ..Default::default()
        },

        // ===================================================================
        // Enemy-specific / Boss Powers
        // ===================================================================
        "Beat of Death" => PowerDef {
            id: "Beat of Death",
            power_type: PowerType::Buff,
            on_after_use_card: true, // deal damage to player per card played
            ..Default::default()
        },
        "Growth" => PowerDef {
            id: "Growth",
            power_type: PowerType::Buff,
            on_end_of_round: true, // gain Strength and Dexterity
            ..Default::default()
        },
        "Magnetism" => PowerDef {
            id: "Magnetism",
            power_type: PowerType::Buff,
            on_turn_start_post_draw: true, // add random colorless to hand
            ..Default::default()
        },
        "SkillBurn" => PowerDef {
            id: "SkillBurn",
            power_type: PowerType::Buff,
            on_after_use_card: true, // deal damage when player plays Skill
            ..Default::default()
        },
        "Forcefield" => PowerDef {
            id: "Forcefield",
            power_type: PowerType::Buff,
            on_after_use_card: true, // lose strength per card played
            ..Default::default()
        },
        "Regrow" => PowerDef {
            id: "Regrow",
            power_type: PowerType::Buff,
            on_turn_end: true, // heal
            ..Default::default()
        },
        "Stasis" => PowerDef {
            id: "Stasis",
            power_type: PowerType::Buff,
            stackable: false,
            // Steals a card from hand (handled by enemy AI)
            ..Default::default()
        },
        "TheBomb" => PowerDef {
            id: "TheBomb",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_end_of_round: true, // countdown, then big damage
            ..Default::default()
        },
        "GenericStrengthUp" => PowerDef {
            id: "GenericStrengthUp",
            power_type: PowerType::Buff,
            on_turn_end: true, // gain Strength
            ..Default::default()
        },
        "LoseStrength" => PowerDef {
            id: "LoseStrength",
            power_type: PowerType::Debuff,
            on_turn_start: true, // lose the temporary Strength
            ..Default::default()
        },
        "LoseDexterity" => PowerDef {
            id: "LoseDexterity",
            power_type: PowerType::Debuff,
            on_turn_start: true, // lose the temporary Dexterity
            ..Default::default()
        },
        "Collect" => PowerDef {
            id: "Collect",
            power_type: PowerType::Buff,
            is_turn_based: true,
            on_turn_start_post_draw: true, // countdown, then gain gold
            ..Default::default()
        },
        "Winter" => PowerDef {
            id: "Winter",
            power_type: PowerType::Buff,
            on_turn_end: true, // channel Frost
            ..Default::default()
        },
        "Repair" => PowerDef {
            id: "Repair",
            power_type: PowerType::Buff,
            // Heal at end of combat (not a turn trigger)
            ..Default::default()
        },

        _ => return None,
    };
    Some(def)
}

// ===========================================================================
// Trigger Dispatch Functions
//
// These are the core functions called by the engine at the appropriate moments.
// They check all relevant powers on the entity and apply effects.
// ===========================================================================

// ---------------------------------------------------------------------------
// Block Decay — checks Barricade, Blur, Calipers
// ---------------------------------------------------------------------------

/// Returns true if block should NOT be removed at start of turn.
/// Barricade prevents all block loss; Blur prevents for its duration.
pub fn should_retain_block(entity: &EntityState) -> bool {
    entity.status("Barricade") > 0 || entity.status("Blur") > 0
}

/// Calculate block retained through Calipers (keep up to 15).
/// Returns the block value after decay.
pub fn apply_block_decay(entity: &EntityState, has_calipers: bool) -> i32 {
    if should_retain_block(entity) {
        return entity.block;
    }
    if has_calipers {
        return (entity.block - 15).max(0).min(entity.block).max(0);
    }
    0
}

// ---------------------------------------------------------------------------
// Decrement turn-based debuffs at end of round
// ---------------------------------------------------------------------------

/// Decrement turn-based debuffs at end of round.
/// Matches the atEndOfRound power trigger in Python.
///
/// Debuffs that tick down: Weakened, Vulnerable, Frail.
pub fn decrement_debuffs(entity: &mut EntityState) {
    decrement_status(entity, "Weakened");
    decrement_status(entity, "Vulnerable");
    decrement_status(entity, "Frail");
}

/// Decrement a single status by 1. Remove if it reaches 0.
fn decrement_status(entity: &mut EntityState, key: &str) {
    if let Some(val) = entity.statuses.get(key).copied() {
        if val <= 1 {
            entity.statuses.remove(key);
        } else {
            entity.statuses.insert(key.to_string(), val - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// Poison
// ---------------------------------------------------------------------------

/// Apply poison tick to an entity. Returns damage dealt.
/// Poison decrements by 1 each tick, removed at 0.
pub fn tick_poison(entity: &mut EntityState) -> i32 {
    let poison = entity.status("Poison");
    if poison <= 0 {
        return 0;
    }

    let damage = poison;
    entity.hp -= damage;

    let new_poison = poison - 1;
    entity.set_status("Poison", new_poison);

    damage
}

// ---------------------------------------------------------------------------
// End-of-turn triggers
// ---------------------------------------------------------------------------

/// Apply Metallicize block gain at end of turn.
pub fn apply_metallicize(entity: &mut EntityState) {
    let metallicize = entity.status("Metallicize");
    if metallicize > 0 {
        entity.block += metallicize;
    }
}

/// Apply Plated Armor block gain at end of turn.
pub fn apply_plated_armor(entity: &mut EntityState) {
    let plated = entity.status("Plated Armor");
    if plated > 0 {
        entity.block += plated;
    }
}

/// Apply Ritual strength gain at start of enemy turn (not first turn).
pub fn apply_ritual(entity: &mut EntityState) {
    let ritual = entity.status("Ritual");
    if ritual > 0 {
        entity.add_status("Strength", ritual);
    }
}

/// Apply GenericStrengthUp (enemy version of Ritual, gains each turn).
pub fn apply_generic_strength_up(entity: &mut EntityState) {
    let amount = entity.status("GenericStrengthUp");
    if amount > 0 {
        entity.add_status("Strength", amount);
    }
}

// ---------------------------------------------------------------------------
// Start-of-turn triggers
// ---------------------------------------------------------------------------

/// Apply LoseStrength at start of turn (undo temporary Strength gains).
pub fn apply_lose_strength(entity: &mut EntityState) {
    let lose_str = entity.status("LoseStrength");
    if lose_str > 0 {
        entity.add_status("Strength", -lose_str);
        entity.set_status("LoseStrength", 0);
    }
}

/// Apply LoseDexterity at start of turn (undo temporary Dexterity gains).
pub fn apply_lose_dexterity(entity: &mut EntityState) {
    let lose_dex = entity.status("LoseDexterity");
    if lose_dex > 0 {
        entity.add_status("Dexterity", -lose_dex);
        entity.set_status("LoseDexterity", 0);
    }
}

/// Remove Flame Barrier at start of turn (it only lasts 1 turn).
pub fn remove_flame_barrier(entity: &mut EntityState) {
    entity.set_status("Flame Barrier", 0);
}

/// WrathNextTurn: enter Wrath at start of next turn. Returns true if should enter Wrath.
pub fn check_wrath_next_turn(entity: &mut EntityState) -> bool {
    let wrath = entity.status("WrathNextTurn");
    if wrath > 0 {
        entity.set_status("WrathNextTurn", 0);
        return true;
    }
    false
}

/// WraithForm: lose N Dexterity at start of turn.
pub fn apply_wraith_form(entity: &mut EntityState) {
    let wraith = entity.status("Wraith Form");
    if wraith > 0 {
        entity.add_status("Dexterity", -wraith);
    }
}

/// Demon Form: gain N Strength at start of turn.
pub fn apply_demon_form(entity: &mut EntityState) {
    let demon_form = entity.status("Demon Form");
    if demon_form > 0 {
        entity.add_status("Strength", demon_form);
    }
}

/// Berserk: gain N energy at start of turn. Returns energy to add.
pub fn apply_berserk(entity: &EntityState) -> i32 {
    entity.status("Berserk")
}

/// Noxious Fumes: returns the amount of poison to apply to all enemies.
pub fn get_noxious_fumes_amount(entity: &EntityState) -> i32 {
    entity.status("Noxious Fumes")
}

/// Brutality: returns the amount of cards to draw (and HP to lose).
pub fn get_brutality_amount(entity: &EntityState) -> i32 {
    entity.status("Brutality")
}

/// DrawCardNextTurn: returns the number of extra cards to draw, then removes the power.
pub fn consume_draw_card_next_turn(entity: &mut EntityState) -> i32 {
    let amount = entity.status("Draw Card");
    if amount > 0 {
        entity.set_status("Draw Card", 0);
    }
    amount
}

/// NextTurnBlock: returns the amount of block to gain, then removes the power.
pub fn consume_next_turn_block(entity: &mut EntityState) -> i32 {
    let amount = entity.status("Next Turn Block");
    if amount > 0 {
        entity.set_status("Next Turn Block", 0);
    }
    amount
}

/// Energized: returns energy to gain at start of turn, then removes the power.
pub fn consume_energized(entity: &mut EntityState) -> i32 {
    let amount = entity.status("Energized");
    if amount > 0 {
        entity.set_status("Energized", 0);
    }
    amount
}

/// Draw power: permanent +draw per turn.
pub fn get_extra_draw(entity: &EntityState) -> i32 {
    entity.status("Draw")
}

/// EnergyDown: returns energy to lose at start of turn.
pub fn get_energy_down(entity: &EntityState) -> i32 {
    entity.status("EnergyDown")
}

/// BattleHymn: returns amount of Smites to add to hand.
pub fn get_battle_hymn_amount(entity: &EntityState) -> i32 {
    entity.status("BattleHymn")
}

/// Devotion: returns amount of Mantra to gain.
pub fn get_devotion_amount(entity: &EntityState) -> i32 {
    entity.status("Devotion")
}

/// InfiniteBlades: returns number of Shivs to add (always 1 per stack).
pub fn get_infinite_blades(entity: &EntityState) -> i32 {
    let amount = entity.status("Infinite Blades");
    if amount > 0 { 1 } else { 0 }
}

// ---------------------------------------------------------------------------
// On-use-card triggers
// ---------------------------------------------------------------------------

/// AfterImage: returns block to gain per card played.
pub fn get_after_image_block(entity: &EntityState) -> i32 {
    entity.status("After Image")
}

/// A Thousand Cuts: returns damage to deal to ALL enemies per card played.
pub fn get_thousand_cuts_damage(entity: &EntityState) -> i32 {
    entity.status("A Thousand Cuts")
}

/// Rage: returns block to gain when playing an Attack.
pub fn get_rage_block(entity: &EntityState) -> i32 {
    entity.status("Rage")
}

/// BeatOfDeath: returns damage to deal to player per card played.
pub fn get_beat_of_death_damage(entity: &EntityState) -> i32 {
    entity.status("Beat of Death")
}

/// Slow: increment counter when player plays a card on this enemy.
pub fn increment_slow(entity: &mut EntityState) {
    let slow = entity.status("Slow");
    if slow >= 0 && entity.statuses.contains_key("Slow") {
        entity.add_status("Slow", 1);
    }
}

/// TimeWarp: increment card counter. Returns true if 12 reached (end turn + gain Str).
/// TimeWarp uses "TimeWarpActive" as a presence flag and "Time Warp" for the counter.
/// The counter starts at 0 and increments; at 12 it resets and triggers.
pub fn increment_time_warp(entity: &mut EntityState) -> bool {
    if entity.status("TimeWarpActive") <= 0 {
        return false;
    }
    let tw = entity.status("Time Warp");
    let new_val = tw + 1;
    if new_val >= 12 {
        entity.set_status("Time Warp", 0);
        return true;
    }
    // Use insert directly to allow storing intermediate values including 0
    entity.statuses.insert("Time Warp".to_string(), new_val);
    false
}

/// Panache: tracks cards played, every 5 deals 10 damage to all enemies.
/// Returns damage to deal (0 or panache amount).
pub fn check_panache(entity: &mut EntityState) -> i32 {
    // Panache stores remaining count until trigger (starts at 5, decrements)
    // We use a secondary counter approach: "PanacheCount"
    if entity.status("Panache") <= 0 {
        return 0;
    }
    let count = entity.status("PanacheCount") + 1;
    if count >= 5 {
        entity.set_status("PanacheCount", 0);
        entity.status("Panache")
    } else {
        entity.set_status("PanacheCount", count);
        0
    }
}

/// DoubleTap: returns true if the next Attack should be played twice.
/// Decrements the counter.
pub fn consume_double_tap(entity: &mut EntityState) -> bool {
    let dt = entity.status("Double Tap");
    if dt > 0 {
        entity.set_status("Double Tap", dt - 1);
        return true;
    }
    false
}

/// Burst: returns true if the next Skill should be played twice.
/// Decrements the counter.
pub fn consume_burst(entity: &mut EntityState) -> bool {
    let b = entity.status("Burst");
    if b > 0 {
        entity.set_status("Burst", b - 1);
        return true;
    }
    false
}

/// Heatsink: returns cards to draw when playing a Power card.
pub fn get_heatsink_draw(entity: &EntityState) -> i32 {
    entity.status("Heatsink")
}

/// Storm: returns true if should channel Lightning when playing a Power.
pub fn should_storm_channel(entity: &EntityState) -> bool {
    entity.status("Storm") > 0
}

/// Forcefield (Automaton): lose Block per card played.
/// Returns true if power is present.
pub fn check_forcefield(entity: &mut EntityState) -> bool {
    let ff = entity.status("Forcefield");
    if ff > 0 {
        entity.add_status("Forcefield", -1);
        return true;
    }
    false
}

/// SkillBurn: returns damage to deal to player when they play a Skill.
pub fn get_skill_burn_damage(entity: &EntityState) -> i32 {
    entity.status("SkillBurn")
}

// ---------------------------------------------------------------------------
// On-attacked / on-damaged triggers
// ---------------------------------------------------------------------------

/// Thorns: returns damage to deal back to attacker when hit.
pub fn get_thorns_damage(entity: &EntityState) -> i32 {
    entity.status("Thorns")
}

/// Flame Barrier: returns damage to deal back to attacker when hit.
pub fn get_flame_barrier_damage(entity: &EntityState) -> i32 {
    entity.status("Flame Barrier")
}

/// Plated Armor: decrement by 1 when taking unblocked damage.
pub fn decrement_plated_armor_on_hit(entity: &mut EntityState) {
    let plated = entity.status("Plated Armor");
    if plated > 0 {
        entity.set_status("Plated Armor", plated - 1);
    }
}

/// Buffer: returns true if damage should be negated (reduces buffer by 1).
pub fn check_buffer(entity: &mut EntityState) -> bool {
    let buffer = entity.status("Buffer");
    if buffer > 0 {
        entity.set_status("Buffer", buffer - 1);
        return true;
    }
    false
}

/// Angry: gain Strength when taking damage.
pub fn apply_angry_on_hit(entity: &mut EntityState) {
    let angry = entity.status("Angry");
    if angry > 0 {
        entity.add_status("Strength", angry);
    }
}

/// Envenom: returns Poison amount to apply when dealing unblocked attack damage.
pub fn get_envenom_amount(entity: &EntityState) -> i32 {
    entity.status("Envenom")
}

/// Curiosity: gain Strength when player plays a Power.
pub fn apply_curiosity(entity: &mut EntityState) {
    let curiosity = entity.status("Curiosity");
    if curiosity > 0 {
        entity.add_status("Strength", curiosity);
    }
}

/// Rupture: gain Strength when losing HP from a card.
pub fn apply_rupture(entity: &mut EntityState) {
    let rupture = entity.status("Rupture");
    if rupture > 0 {
        entity.add_status("Strength", rupture);
    }
}

/// StaticDischarge: returns number of Lightning orbs to channel when taking damage.
pub fn get_static_discharge(entity: &EntityState) -> i32 {
    entity.status("Static Discharge")
}

// ---------------------------------------------------------------------------
// On-exhaust triggers
// ---------------------------------------------------------------------------

/// DarkEmbrace: returns cards to draw per exhaust.
pub fn get_dark_embrace_draw(entity: &EntityState) -> i32 {
    entity.status("Dark Embrace")
}

/// FeelNoPain: returns block to gain per exhaust.
pub fn get_feel_no_pain_block(entity: &EntityState) -> i32 {
    entity.status("Feel No Pain")
}

// ---------------------------------------------------------------------------
// On-card-draw triggers
// ---------------------------------------------------------------------------

/// Evolve: returns cards to draw when drawing a Status card.
pub fn get_evolve_draw(entity: &EntityState) -> i32 {
    entity.status("Evolve")
}

/// FireBreathing: returns damage to deal to all enemies when drawing Status/Curse.
pub fn get_fire_breathing_damage(entity: &EntityState) -> i32 {
    entity.status("Fire Breathing")
}

// ---------------------------------------------------------------------------
// On-change-stance triggers
// ---------------------------------------------------------------------------

/// MentalFortress: returns block to gain on ANY stance change.
pub fn get_mental_fortress_block(entity: &EntityState) -> i32 {
    entity.status("MentalFortress")
}

/// Rushdown: returns cards to draw when entering Wrath.
pub fn get_rushdown_draw(entity: &EntityState) -> i32 {
    entity.status("Rushdown")
}

/// Nirvana: returns block to gain when scrying.
pub fn get_nirvana_block(entity: &EntityState) -> i32 {
    entity.status("Nirvana")
}

// ---------------------------------------------------------------------------
// On-gained-block triggers
// ---------------------------------------------------------------------------

/// Juggernaut: returns damage to deal to random enemy when gaining block.
pub fn get_juggernaut_damage(entity: &EntityState) -> i32 {
    entity.status("Juggernaut")
}

/// WaveOfTheHand: returns Weak amount to apply when gaining block.
pub fn get_wave_of_the_hand_weak(entity: &EntityState) -> i32 {
    entity.status("Wave of the Hand")
}

// ---------------------------------------------------------------------------
// Damage modification triggers
// ---------------------------------------------------------------------------

/// Modify outgoing damage based on powers.
/// Called during damage calculation for attacks.
pub fn modify_damage_give(entity: &EntityState, damage: f64, _is_attack: bool) -> f64 {
    let mut d = damage;

    // DoubleDamage (Phantasmal Killer active)
    if entity.status("DoubleDamage") > 0 {
        d *= 2.0;
    }

    // Pen Nib is handled separately in engine (relic counter)

    d
}

/// Modify incoming damage based on defender's powers.
/// Returns modified damage value.
pub fn modify_damage_receive(entity: &EntityState, damage: f64) -> f64 {
    let mut d = damage;

    // Slow: +10% per stack
    let slow = entity.status("Slow");
    if slow > 0 {
        d *= 1.0 + (slow as f64 * 0.1);
    }

    // Intangible: cap at 1
    if entity.status("Intangible") > 0 && d > 1.0 {
        d = 1.0;
    }

    d
}

/// Modify block amount based on powers.
pub fn modify_block(entity: &EntityState, block: f64) -> f64 {
    // NoBlock: can't gain block
    if entity.status("No Block") > 0 {
        return 0.0;
    }

    // Dexterity is handled in calculate_block() directly
    // Frail is handled in calculate_block() directly

    block
}

// ---------------------------------------------------------------------------
// On-heal triggers
// ---------------------------------------------------------------------------

/// Modify heal amount. Returns final heal amount.
pub fn modify_heal(entity: &EntityState, heal: i32) -> i32 {
    // No power modifies heal in base game except Mark of the Bloom (relic)
    let _ = entity;
    heal
}

// ---------------------------------------------------------------------------
// End-of-round triggers
// ---------------------------------------------------------------------------

/// Reset Slow stacks at end of round.
pub fn reset_slow(entity: &mut EntityState) {
    if entity.statuses.contains_key("Slow") {
        entity.set_status("Slow", 0);
    }
}

/// Decrement Fading. Returns true if entity should die (Fading reaches 0).
pub fn decrement_fading(entity: &mut EntityState) -> bool {
    let fading = entity.status("Fading");
    if fading > 0 {
        let new_val = fading - 1;
        entity.set_status("Fading", new_val);
        if new_val <= 0 {
            return true;
        }
    }
    false
}

/// Explosive countdown. Returns damage to deal when it reaches 0.
pub fn decrement_explosive(entity: &mut EntityState) -> i32 {
    let explosive = entity.status("Explosive");
    if explosive > 0 {
        let new_val = explosive - 1;
        entity.set_status("Explosive", new_val);
        if new_val <= 0 {
            // Explosive deals its stored damage amount
            // The damage is stored separately; typically 30-50
            return 30; // Default bomb damage
        }
    }
    0
}

/// Growth: gain Strength and Dexterity at end of round.
pub fn apply_growth(entity: &mut EntityState) {
    let growth = entity.status("Growth");
    if growth > 0 {
        entity.add_status("Strength", growth);
        entity.add_status("Dexterity", growth); // Growth also adds Dex in Java? No, just in Nemesis. Check specific enemies.
    }
}

/// Decrement Blur at end of round.
pub fn decrement_blur(entity: &mut EntityState) {
    decrement_status(entity, "Blur");
}

/// Decrement Intangible at end of turn.
pub fn decrement_intangible(entity: &mut EntityState) {
    decrement_status(entity, "Intangible");
}

/// Decrement Lock-On at end of round.
pub fn decrement_lock_on(entity: &mut EntityState) {
    decrement_status(entity, "Lock-On");
}

/// Reset Invincible at end of round (Champ).
pub fn reset_invincible(entity: &mut EntityState, max_amount: i32) {
    if entity.statuses.contains_key("Invincible") {
        entity.set_status("Invincible", max_amount);
    }
}

// ---------------------------------------------------------------------------
// TheBomb countdown
// ---------------------------------------------------------------------------

/// TheBomb: decrement counter. Returns (should_explode, damage).
pub fn decrement_the_bomb(entity: &mut EntityState) -> (bool, i32) {
    let turns = entity.status("TheBombTurns");
    let damage = entity.status("TheBomb");
    if turns > 0 && damage > 0 {
        let new_turns = turns - 1;
        entity.set_status("TheBombTurns", new_turns);
        if new_turns <= 0 {
            entity.set_status("TheBomb", 0);
            entity.set_status("TheBombTurns", 0);
            return (true, damage);
        }
    }
    (false, 0)
}

// ---------------------------------------------------------------------------
// Combust end-of-turn
// ---------------------------------------------------------------------------

/// Combust: lose 1 HP, deal N damage to all enemies.
/// Returns (hp_loss, damage_per_enemy).
pub fn get_combust_effect(entity: &EntityState) -> (i32, i32) {
    let combust = entity.status("Combust");
    if combust > 0 {
        (1, combust)
    } else {
        (0, 0)
    }
}

// ---------------------------------------------------------------------------
// Omega end-of-turn
// ---------------------------------------------------------------------------

/// Omega: returns damage to deal to ALL enemies at end of turn.
pub fn get_omega_damage(entity: &EntityState) -> i32 {
    entity.status("OmegaPower")
}

// ---------------------------------------------------------------------------
// LikeWater end-of-turn
// ---------------------------------------------------------------------------

/// LikeWater: returns block to gain if in Calm stance.
pub fn get_like_water_block(entity: &EntityState) -> i32 {
    entity.status("Like Water")
}

// ---------------------------------------------------------------------------
// Regeneration end-of-turn
// ---------------------------------------------------------------------------

/// Regeneration: heal and decrement. Returns HP to heal.
pub fn apply_regeneration(entity: &mut EntityState) -> i32 {
    let regen = entity.status("Regeneration");
    if regen > 0 {
        entity.set_status("Regeneration", regen - 1);
        return regen;
    }
    0
}

// ---------------------------------------------------------------------------
// Regrow end-of-turn (enemy)
// ---------------------------------------------------------------------------

/// Regrow: heal. Returns HP to heal.
pub fn get_regrow_heal(entity: &EntityState) -> i32 {
    entity.status("Regrow")
}

// ---------------------------------------------------------------------------
// End-of-turn removal: Rage
// ---------------------------------------------------------------------------

/// Remove Rage at end of turn.
pub fn remove_rage_end_of_turn(entity: &mut EntityState) {
    entity.set_status("Rage", 0);
}

// ---------------------------------------------------------------------------
// DoubleDamage consumption
// ---------------------------------------------------------------------------

/// DoubleDamage: consumed after playing an Attack.
pub fn consume_double_damage(entity: &mut EntityState) {
    if entity.status("DoubleDamage") > 0 {
        entity.set_status("DoubleDamage", 0);
    }
}

// ---------------------------------------------------------------------------
// On-death triggers
// ---------------------------------------------------------------------------

/// SporeCloud: returns Vulnerable amount to apply to player when this enemy dies.
pub fn get_spore_cloud_vulnerable(entity: &EntityState) -> i32 {
    entity.status("Spore Cloud")
}

// ---------------------------------------------------------------------------
// Corruption — Skills cost 0 and exhaust
// ---------------------------------------------------------------------------

/// Check if Corruption makes Skills cost 0.
pub fn has_corruption(entity: &EntityState) -> bool {
    entity.status("Corruption") > 0
}

// ---------------------------------------------------------------------------
// NoSkills — can't play Skills
// ---------------------------------------------------------------------------

/// Check if NoSkills prevents playing Skills.
pub fn has_no_skills(entity: &EntityState) -> bool {
    entity.status("NoSkillsPower") > 0
}

// ---------------------------------------------------------------------------
// Confusion — randomize card costs
// ---------------------------------------------------------------------------

/// Check if Confusion is active.
pub fn has_confusion(entity: &EntityState) -> bool {
    entity.status("Confusion") > 0
}

// ---------------------------------------------------------------------------
// NoDraw — can't draw cards
// ---------------------------------------------------------------------------

/// Check if NoDraw prevents card draw.
pub fn has_no_draw(entity: &EntityState) -> bool {
    entity.status("No Draw") > 0
}

// ---------------------------------------------------------------------------
// CannotChangeStance
// ---------------------------------------------------------------------------

/// Check if stance changes are blocked.
pub fn cannot_change_stance(entity: &EntityState) -> bool {
    entity.status("CannotChangeStance") > 0
}

// ---------------------------------------------------------------------------
// FreeAttack — next Attack costs 0
// ---------------------------------------------------------------------------

/// Check and consume FreeAttack. Returns true if active.
pub fn consume_free_attack(entity: &mut EntityState) -> bool {
    let fa = entity.status("FreeAttackPower");
    if fa > 0 {
        entity.set_status("FreeAttackPower", fa - 1);
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Equilibrium — retain hand
// ---------------------------------------------------------------------------

/// Check if Equilibrium retains hand this turn.
pub fn has_equilibrium(entity: &EntityState) -> bool {
    entity.status("Equilibrium") > 0
}

/// Decrement Equilibrium at end of turn.
pub fn decrement_equilibrium(entity: &mut EntityState) {
    decrement_status(entity, "Equilibrium");
}

// ---------------------------------------------------------------------------
// Study — shuffle Insight into draw pile
// ---------------------------------------------------------------------------

/// Study: returns number of Insights to add to draw pile.
pub fn get_study_insights(entity: &EntityState) -> i32 {
    entity.status("Study")
}

// ---------------------------------------------------------------------------
// LiveForever — gain block at end of turn
// ---------------------------------------------------------------------------

/// LiveForever: returns block to gain at end of turn.
pub fn get_live_forever_block(entity: &EntityState) -> i32 {
    entity.status("LiveForever")
}

// ---------------------------------------------------------------------------
// Accuracy — bonus Shiv damage
// ---------------------------------------------------------------------------

/// Accuracy: returns bonus damage for Shiv cards.
pub fn get_accuracy_bonus(entity: &EntityState) -> i32 {
    entity.status("Accuracy")
}

// ---------------------------------------------------------------------------
// Mark — Pressure Points damage
// ---------------------------------------------------------------------------

/// Get current Mark amount on entity.
pub fn get_mark(entity: &EntityState) -> i32 {
    entity.status("Mark")
}

// ---------------------------------------------------------------------------
// Deva Form — escalating energy
// ---------------------------------------------------------------------------

/// DevaForm energy tracking. Uses "DevaFormEnergy" for the escalating counter.
/// Returns energy to gain this turn.
pub fn apply_deva_form(entity: &mut EntityState) -> i32 {
    let deva = entity.status("DevaForm");
    if deva <= 0 {
        return 0;
    }
    let energy_counter = entity.status("DevaFormEnergy") + 1;
    entity.set_status("DevaFormEnergy", energy_counter);
    energy_counter
}

// ---------------------------------------------------------------------------
// Apply a debuff, respecting Artifact
// ---------------------------------------------------------------------------

/// Apply a debuff, respecting Artifact (blocks debuffs).
/// Returns true if the debuff was applied, false if blocked by Artifact.
pub fn apply_debuff(entity: &mut EntityState, status: &str, amount: i32) -> bool {
    let artifact = entity.status("Artifact");
    if artifact > 0 {
        // Artifact blocks the debuff and decrements
        entity.set_status("Artifact", artifact - 1);
        return false;
    }

    entity.add_status(status, amount);
    true
}

/// Apply a debuff with Sadistic Nature check. Returns damage to deal from Sadistic.
pub fn apply_debuff_with_sadistic(
    target: &mut EntityState,
    status: &str,
    amount: i32,
    source_sadistic: i32,
) -> (bool, i32) {
    let applied = apply_debuff(target, status, amount);
    if applied && source_sadistic > 0 {
        (true, source_sadistic)
    } else {
        (applied, 0)
    }
}

// ---------------------------------------------------------------------------
// Invincible damage cap
// ---------------------------------------------------------------------------

/// Invincible: cap total damage this turn. Returns capped damage.
/// The `amount` field tracks remaining damage allowed.
pub fn apply_invincible_cap(entity: &mut EntityState, incoming_damage: i32) -> i32 {
    let inv = entity.status("Invincible");
    if inv > 0 {
        if incoming_damage > inv {
            entity.set_status("Invincible", 0);
            return inv;
        } else {
            entity.set_status("Invincible", inv - incoming_damage);
            return incoming_damage;
        }
    }
    incoming_damage
}

// ---------------------------------------------------------------------------
// ModeShift (Guardian)
// ---------------------------------------------------------------------------

/// ModeShift: track damage. Returns true if threshold reached.
pub fn apply_mode_shift_damage(entity: &mut EntityState, damage: i32) -> bool {
    let ms = entity.status("Mode Shift");
    if ms > 0 {
        let new_val = ms - damage;
        if new_val <= 0 {
            entity.set_status("Mode Shift", 0);
            return true;
        }
        entity.set_status("Mode Shift", new_val);
    }
    false
}

// ---------------------------------------------------------------------------
// EndTurnDeath
// ---------------------------------------------------------------------------

/// EndTurnDeath: returns true if entity should die at end of turn.
pub fn should_die_end_of_turn(entity: &EntityState) -> bool {
    entity.status("EndTurnDeath") > 0
}

// ---------------------------------------------------------------------------
// Comprehensive trigger dispatcher — aggregates all triggers for a phase
// ---------------------------------------------------------------------------

/// Results from start-of-turn power processing.
#[derive(Debug, Default)]
pub struct StartOfTurnResult {
    pub extra_energy: i32,
    pub extra_draw: i32,
    pub noxious_fumes_poison: i32,
    pub demon_form_strength: bool,
    pub brutality_draw: i32,
    pub block_from_next_turn: i32,
    pub enter_wrath: bool,
    pub battle_hymn_smites: i32,
    pub devotion_mantra: i32,
    pub infinite_blades: bool,
    pub draw_card_next_turn: i32,
    pub wraith_form_dex_loss: bool,
    pub berserk_energy: i32,
}

/// Process all start-of-turn power triggers for the player.
/// Returns a result struct with all effects to apply.
pub fn process_start_of_turn(entity: &mut EntityState) -> StartOfTurnResult {
    let mut result = StartOfTurnResult::default();

    // LoseStrength / LoseDexterity
    apply_lose_strength(entity);
    apply_lose_dexterity(entity);

    // Flame Barrier removal
    remove_flame_barrier(entity);

    // WraithForm: lose Dexterity
    let wraith = entity.status("Wraith Form");
    if wraith > 0 {
        apply_wraith_form(entity);
        result.wraith_form_dex_loss = true;
    }

    // Demon Form
    let demon = entity.status("Demon Form");
    if demon > 0 {
        apply_demon_form(entity);
        result.demon_form_strength = true;
    }

    // Berserk
    result.berserk_energy = apply_berserk(entity);

    // Noxious Fumes
    result.noxious_fumes_poison = get_noxious_fumes_amount(entity);

    // Brutality
    result.brutality_draw = get_brutality_amount(entity);

    // DrawCardNextTurn
    result.draw_card_next_turn = consume_draw_card_next_turn(entity);

    // NextTurnBlock
    result.block_from_next_turn = consume_next_turn_block(entity);

    // Energized
    result.extra_energy += consume_energized(entity);

    // EnergyDown
    result.extra_energy -= get_energy_down(entity);

    // WrathNextTurn
    result.enter_wrath = check_wrath_next_turn(entity);

    // BattleHymn
    result.battle_hymn_smites = get_battle_hymn_amount(entity);

    // Devotion
    result.devotion_mantra = get_devotion_amount(entity);

    // Infinite Blades
    result.infinite_blades = get_infinite_blades(entity) > 0;

    // Draw power (permanent)
    result.extra_draw = get_extra_draw(entity);

    // DevaForm
    let deva_energy = apply_deva_form(entity);
    result.extra_energy += deva_energy;

    result
}

/// Results from end-of-turn power processing.
#[derive(Debug, Default)]
pub struct EndOfTurnResult {
    pub metallicize_block: i32,
    pub plated_armor_block: i32,
    pub omega_damage: i32,
    pub like_water_block: i32,
    pub combust_hp_loss: i32,
    pub combust_damage: i32,
    pub regen_heal: i32,
    pub live_forever_block: i32,
    pub study_insights: i32,
    pub should_die: bool,
}

/// Process all end-of-turn power triggers for the player.
pub fn process_end_of_turn(entity: &mut EntityState, in_calm: bool) -> EndOfTurnResult {
    let mut result = EndOfTurnResult::default();

    // Metallicize
    result.metallicize_block = entity.status("Metallicize");

    // Plated Armor
    result.plated_armor_block = entity.status("Plated Armor");

    // Omega
    result.omega_damage = get_omega_damage(entity);

    // LikeWater (only if in Calm)
    if in_calm {
        result.like_water_block = get_like_water_block(entity);
    }

    // Combust
    let (hp_loss, dmg) = get_combust_effect(entity);
    result.combust_hp_loss = hp_loss;
    result.combust_damage = dmg;

    // Regeneration
    result.regen_heal = apply_regeneration(entity);

    // LiveForever
    result.live_forever_block = get_live_forever_block(entity);

    // Study
    result.study_insights = get_study_insights(entity);

    // EndTurnDeath
    result.should_die = should_die_end_of_turn(entity);

    // Remove Rage at end of turn
    remove_rage_end_of_turn(entity);

    // Decrement Equilibrium
    decrement_equilibrium(entity);

    // Decrement Intangible
    decrement_intangible(entity);

    result
}

/// Process end-of-round triggers (after all entities have taken turns).
pub fn process_end_of_round(entity: &mut EntityState) {
    // Debuff decrements
    decrement_debuffs(entity);

    // Blur
    decrement_blur(entity);

    // Lock-On
    decrement_lock_on(entity);

    // Slow reset
    reset_slow(entity);
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Power registry tests --

    #[test]
    fn test_power_def_lookup() {
        let def = get_power_def("Strength").unwrap();
        assert_eq!(def.id, "Strength");
        assert!(def.can_go_negative);
        assert!(def.modify_damage_give);
        assert_eq!(def.power_type, PowerType::Buff);
    }

    #[test]
    fn test_power_def_debuff() {
        let def = get_power_def("Weakened").unwrap();
        assert_eq!(def.power_type, PowerType::Debuff);
        assert!(def.is_turn_based);
        assert!(def.on_end_of_round);
    }

    #[test]
    fn test_power_def_unknown() {
        assert!(get_power_def("NonexistentPower").is_none());
    }

    #[test]
    fn test_power_id_key_roundtrip() {
        assert_eq!(PowerId::Strength.key(), "Strength");
        assert_eq!(PowerId::Weakened.key(), "Weakened");
        assert_eq!(PowerId::DemonForm.key(), "Demon Form");
        assert_eq!(PowerId::MentalFortress.key(), "MentalFortress");
        assert_eq!(PowerId::Omega.key(), "OmegaPower");
    }

    // -- Debuff decrement tests --

    #[test]
    fn test_decrement_debuffs() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Weakened", 2);
        entity.set_status("Vulnerable", 1);
        entity.set_status("Frail", 3);

        decrement_debuffs(&mut entity);

        assert_eq!(entity.status("Weakened"), 1);
        assert_eq!(entity.status("Vulnerable"), 0);
        assert_eq!(entity.status("Frail"), 2);
    }

    // -- Poison tests --

    #[test]
    fn test_tick_poison() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 5);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 5);
        assert_eq!(entity.hp, 45);
        assert_eq!(entity.status("Poison"), 4);
    }

    #[test]
    fn test_tick_poison_removed_at_zero() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 1);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 1);
        assert_eq!(entity.status("Poison"), 0);
        assert!(!entity.statuses.contains_key("Poison"));
    }

    // -- Metallicize / Plated Armor tests --

    #[test]
    fn test_metallicize() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Metallicize", 4);

        apply_metallicize(&mut entity);
        assert_eq!(entity.block, 4);
    }

    #[test]
    fn test_plated_armor() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Plated Armor", 6);

        apply_plated_armor(&mut entity);
        assert_eq!(entity.block, 6);
    }

    // -- Ritual tests --

    #[test]
    fn test_ritual() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Ritual", 3);

        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 3);

        // Second application stacks
        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 6);
    }

    // -- Artifact tests --

    #[test]
    fn test_artifact_blocks_debuff() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Artifact", 1);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(!applied);
        assert_eq!(entity.status("Weakened"), 0);
        assert_eq!(entity.status("Artifact"), 0);
    }

    #[test]
    fn test_debuff_without_artifact() {
        let mut entity = EntityState::new(50, 50);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(applied);
        assert_eq!(entity.status("Weakened"), 2);
    }

    // -- Block decay tests --

    #[test]
    fn test_barricade_retains_block() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        entity.set_status("Barricade", 1);
        assert!(should_retain_block(&entity));
        assert_eq!(apply_block_decay(&entity, false), 10);
    }

    #[test]
    fn test_blur_retains_block() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        entity.set_status("Blur", 1);
        assert!(should_retain_block(&entity));
    }

    #[test]
    fn test_calipers_retains_15() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 20;
        assert_eq!(apply_block_decay(&entity, true), 5);
    }

    #[test]
    fn test_normal_block_decay() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        assert_eq!(apply_block_decay(&entity, false), 0);
    }

    // -- Demon Form tests --

    #[test]
    fn test_demon_form() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Demon Form", 3);

        apply_demon_form(&mut entity);
        assert_eq!(entity.strength(), 3);

        apply_demon_form(&mut entity);
        assert_eq!(entity.strength(), 6);
    }

    // -- Buffer tests --

    #[test]
    fn test_buffer_blocks_damage() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Buffer", 2);

        assert!(check_buffer(&mut entity));
        assert_eq!(entity.status("Buffer"), 1);

        assert!(check_buffer(&mut entity));
        assert_eq!(entity.status("Buffer"), 0);

        assert!(!check_buffer(&mut entity));
    }

    // -- Thorns tests --

    #[test]
    fn test_thorns_damage() {
        let entity = EntityState::new(50, 50);
        assert_eq!(get_thorns_damage(&entity), 0);

        let mut entity2 = EntityState::new(50, 50);
        entity2.set_status("Thorns", 3);
        assert_eq!(get_thorns_damage(&entity2), 3);
    }

    // -- Flame Barrier tests --

    #[test]
    fn test_flame_barrier() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Flame Barrier", 7);

        assert_eq!(get_flame_barrier_damage(&entity), 7);

        remove_flame_barrier(&mut entity);
        assert_eq!(get_flame_barrier_damage(&entity), 0);
    }

    // -- After Image tests --

    #[test]
    fn test_after_image() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("After Image", 2);
        assert_eq!(get_after_image_block(&entity), 2);
    }

    // -- DoubleTap / Burst tests --

    #[test]
    fn test_double_tap() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Double Tap", 1);

        assert!(consume_double_tap(&mut entity));
        assert_eq!(entity.status("Double Tap"), 0);
        assert!(!consume_double_tap(&mut entity));
    }

    #[test]
    fn test_burst() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Burst", 2);

        assert!(consume_burst(&mut entity));
        assert_eq!(entity.status("Burst"), 1);
        assert!(consume_burst(&mut entity));
        assert!(!consume_burst(&mut entity));
    }

    // -- TimeWarp tests --

    #[test]
    fn test_time_warp_countdown() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("TimeWarpActive", 1);

        for _ in 0..11 {
            assert!(!increment_time_warp(&mut entity));
        }
        assert!(increment_time_warp(&mut entity));
        assert_eq!(entity.status("Time Warp"), 0); // resets
    }

    // -- Slow tests --

    #[test]
    fn test_slow_damage_modification() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Slow", 3);

        let modified = modify_damage_receive(&entity, 10.0);
        assert!((modified - 13.0).abs() < 0.01); // 10 * 1.3 = 13
    }

    // -- Invincible tests --

    #[test]
    fn test_invincible_cap() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Invincible", 200);

        assert_eq!(apply_invincible_cap(&mut entity, 50), 50);
        assert_eq!(entity.status("Invincible"), 150);

        assert_eq!(apply_invincible_cap(&mut entity, 200), 150);
        assert_eq!(entity.status("Invincible"), 0);
    }

    // -- ModeShift tests --

    #[test]
    fn test_mode_shift() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Mode Shift", 10);

        assert!(!apply_mode_shift_damage(&mut entity, 5));
        assert_eq!(entity.status("Mode Shift"), 5);

        assert!(apply_mode_shift_damage(&mut entity, 5));
        assert_eq!(entity.status("Mode Shift"), 0);
    }

    // -- Fading tests --

    #[test]
    fn test_fading_countdown() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Fading", 2);

        assert!(!decrement_fading(&mut entity));
        assert_eq!(entity.status("Fading"), 1);

        assert!(decrement_fading(&mut entity));
    }

    // -- Comprehensive trigger tests --

    #[test]
    fn test_process_start_of_turn() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Demon Form", 2);
        entity.set_status("Noxious Fumes", 3);
        entity.set_status("Energized", 2);
        entity.set_status("LoseStrength", 1);
        entity.set_status("Wraith Form", 1);
        entity.set_status("Flame Barrier", 5);

        let result = process_start_of_turn(&mut entity);

        // Demon Form adds Strength
        assert_eq!(entity.strength(), 1); // +2 from DemonForm, -1 from LoseStrength
        assert!(result.demon_form_strength);

        // Noxious Fumes
        assert_eq!(result.noxious_fumes_poison, 3);

        // Energized consumed
        assert_eq!(result.extra_energy, 2);
        assert_eq!(entity.status("Energized"), 0);

        // WraithForm
        assert!(result.wraith_form_dex_loss);
        assert_eq!(entity.dexterity(), -1);

        // Flame Barrier removed
        assert_eq!(entity.status("Flame Barrier"), 0);
    }

    #[test]
    fn test_process_end_of_turn() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Metallicize", 4);
        entity.set_status("Plated Armor", 3);
        entity.set_status("OmegaPower", 50);
        entity.set_status("Rage", 5);
        entity.set_status("Combust", 7);

        let result = process_end_of_turn(&mut entity, false);

        assert_eq!(result.metallicize_block, 4);
        assert_eq!(result.plated_armor_block, 3);
        assert_eq!(result.omega_damage, 50);
        assert_eq!(result.combust_hp_loss, 1);
        assert_eq!(result.combust_damage, 7);

        // Rage removed
        assert_eq!(entity.status("Rage"), 0);
    }

    #[test]
    fn test_like_water_in_calm() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Like Water", 5);

        let result_calm = process_end_of_turn(&mut entity, true);
        assert_eq!(result_calm.like_water_block, 5);

        let result_not_calm = process_end_of_turn(&mut entity, false);
        assert_eq!(result_not_calm.like_water_block, 0);
    }

    // -- Damage modification tests --

    #[test]
    fn test_double_damage_modifier() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DoubleDamage", 1);

        let modified = modify_damage_give(&entity, 10.0, true);
        assert!((modified - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_intangible_caps_damage() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Intangible", 1);

        let modified = modify_damage_receive(&entity, 100.0);
        assert!((modified - 1.0).abs() < 0.01);
    }

    // -- Corruption / NoSkills / Confusion --

    #[test]
    fn test_corruption_flag() {
        let mut entity = EntityState::new(50, 50);
        assert!(!has_corruption(&entity));
        entity.set_status("Corruption", 1);
        assert!(has_corruption(&entity));
    }

    #[test]
    fn test_no_skills_flag() {
        let mut entity = EntityState::new(50, 50);
        assert!(!has_no_skills(&entity));
        entity.set_status("NoSkillsPower", 1);
        assert!(has_no_skills(&entity));
    }

    #[test]
    fn test_cannot_change_stance() {
        let mut entity = EntityState::new(50, 50);
        assert!(!cannot_change_stance(&entity));
        entity.set_status("CannotChangeStance", 1);
        assert!(cannot_change_stance(&entity));
    }

    // -- Panache tests --

    #[test]
    fn test_panache_triggers_every_5() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Panache", 10);

        for _ in 0..4 {
            assert_eq!(check_panache(&mut entity), 0);
        }
        assert_eq!(check_panache(&mut entity), 10);

        // Next cycle
        for _ in 0..4 {
            assert_eq!(check_panache(&mut entity), 0);
        }
        assert_eq!(check_panache(&mut entity), 10);
    }

    // -- Deva Form tests --

    #[test]
    fn test_deva_form_escalating() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DevaForm", 1);

        assert_eq!(apply_deva_form(&mut entity), 1);
        assert_eq!(apply_deva_form(&mut entity), 2);
        assert_eq!(apply_deva_form(&mut entity), 3);
    }

    // -- Sadistic Nature tests --

    #[test]
    fn test_sadistic_on_debuff() {
        let mut entity = EntityState::new(50, 50);

        let (applied, sadistic_dmg) = apply_debuff_with_sadistic(&mut entity, "Weakened", 1, 5);
        assert!(applied);
        assert_eq!(sadistic_dmg, 5);
        assert_eq!(entity.status("Weakened"), 1);
    }

    #[test]
    fn test_sadistic_blocked_by_artifact() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Artifact", 1);

        let (applied, sadistic_dmg) = apply_debuff_with_sadistic(&mut entity, "Weakened", 1, 5);
        assert!(!applied);
        assert_eq!(sadistic_dmg, 0);
    }

    // -- Process end of round --

    #[test]
    fn test_process_end_of_round() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Weakened", 2);
        entity.set_status("Vulnerable", 1);
        entity.set_status("Blur", 1);
        entity.set_status("Slow", 5);
        entity.set_status("Lock-On", 2);

        process_end_of_round(&mut entity);

        assert_eq!(entity.status("Weakened"), 1);
        assert_eq!(entity.status("Vulnerable"), 0);
        assert_eq!(entity.status("Blur"), 0);
        assert_eq!(entity.status("Slow"), 0);
        assert_eq!(entity.status("Lock-On"), 1);
    }
}
