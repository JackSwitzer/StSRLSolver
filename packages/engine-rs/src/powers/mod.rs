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
            on_turn_start: true, // add Smite to hand (Java: atStartOfTurn)
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
            on_turn_end: true, // retained cards cost 1 less
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
            power_type: PowerType::Debuff,
            can_go_negative: true,
            on_turn_end: true, // lose 1 Dexterity each turn
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

mod buffs;
mod debuffs;
mod enemy_powers;

// Re-export all trigger functions from sub-modules
pub use buffs::should_retain_block;
pub use buffs::apply_block_decay;
pub use buffs::apply_metallicize;
pub use buffs::apply_plated_armor;
pub use buffs::remove_flame_barrier;
pub use buffs::check_wrath_next_turn;
pub use buffs::apply_demon_form;
pub use buffs::apply_berserk;
pub use buffs::get_noxious_fumes_amount;
pub use buffs::get_brutality_amount;
pub use buffs::consume_draw_card_next_turn;
pub use buffs::consume_next_turn_block;
pub use buffs::consume_energized;
pub use buffs::get_extra_draw;
pub use buffs::get_energy_down;
pub use buffs::get_battle_hymn_amount;
pub use buffs::get_devotion_amount;
pub use buffs::get_infinite_blades;
pub use buffs::get_after_image_block;
pub use buffs::get_thousand_cuts_damage;
pub use buffs::get_rage_block;
pub use buffs::check_panache;
pub use buffs::consume_double_tap;
pub use buffs::consume_burst;
pub use buffs::get_heatsink_draw;
pub use buffs::should_storm_channel;
pub use buffs::check_forcefield;
pub use buffs::get_skill_burn_damage;
pub use buffs::get_thorns_damage;
pub use buffs::get_flame_barrier_damage;
pub use buffs::decrement_plated_armor_on_hit;
pub use buffs::check_buffer;
pub use buffs::get_envenom_amount;
pub use buffs::apply_rupture;
pub use buffs::get_static_discharge;
pub use buffs::get_dark_embrace_draw;
pub use buffs::get_feel_no_pain_block;
pub use buffs::get_evolve_draw;
pub use buffs::get_fire_breathing_damage;
pub use buffs::get_mental_fortress_block;
pub use buffs::get_rushdown_draw;
pub use buffs::get_nirvana_block;
pub use buffs::get_juggernaut_damage;
pub use buffs::get_wave_of_the_hand_weak;
pub use buffs::modify_damage_give;
pub use buffs::modify_block;
pub use buffs::modify_heal;
pub use buffs::get_combust_effect;
pub use buffs::get_omega_damage;
pub use buffs::get_like_water_block;
pub use buffs::remove_rage_end_of_turn;
pub use buffs::consume_double_damage;
pub use buffs::has_corruption;
pub use buffs::has_no_skills;
pub use buffs::has_confusion;
pub use buffs::has_no_draw;
pub use buffs::cannot_change_stance;
pub use buffs::consume_free_attack;
pub use buffs::has_equilibrium;
pub use buffs::decrement_equilibrium;
pub use buffs::get_study_insights;
pub use buffs::get_live_forever_block;
pub use buffs::get_accuracy_bonus;
pub use buffs::get_mark;
pub use buffs::apply_deva_form;
pub use buffs::should_die_end_of_turn;
pub use buffs::process_start_of_turn;
pub use buffs::process_end_of_turn;
pub use buffs::process_end_of_round;

pub use debuffs::decrement_debuffs;
pub use debuffs::tick_poison;
pub use debuffs::apply_lose_strength;
pub use debuffs::apply_lose_dexterity;
pub use debuffs::apply_wraith_form;
pub use debuffs::modify_damage_receive;
pub use debuffs::decrement_fading;
pub use debuffs::decrement_blur;
pub use debuffs::decrement_intangible;
pub use debuffs::decrement_lock_on;
pub use debuffs::apply_debuff;
pub use debuffs::apply_debuff_with_sadistic;
pub use debuffs::apply_invincible_cap;
pub use debuffs::apply_mode_shift_damage;

pub use enemy_powers::apply_ritual;
pub use enemy_powers::apply_generic_strength_up;
pub use enemy_powers::get_beat_of_death_damage;
pub use enemy_powers::increment_slow;
pub use enemy_powers::increment_time_warp;
pub use enemy_powers::apply_angry_on_hit;
pub use enemy_powers::apply_curiosity;
pub use enemy_powers::reset_slow;
pub use enemy_powers::decrement_explosive;
pub use enemy_powers::apply_growth;
pub use enemy_powers::reset_invincible;
pub use enemy_powers::decrement_the_bomb;
pub use enemy_powers::apply_regeneration;
pub use enemy_powers::get_regrow_heal;
pub use enemy_powers::get_spore_cloud_vulnerable;
