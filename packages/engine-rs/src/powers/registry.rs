//! Unified Power Registry — single source of truth for all powers.
//!
//! Each power is defined once as a `PowerRegistryEntry` with:
//! - Card effect tag (for `install_power()` lookup)
//! - StatusId (runtime status key)
//! - Power metadata (type, stackable, turn-based)
//! - Hook function pointers (None = doesn't fire on this trigger)
//!
//! ## Power Dispatch Directory
//!
//! ### Registry-dispatched powers (via hook tables):
//!
//! | Power            | Turn Start | Turn End | Card Pre | Card Post | Exhaust | Stance | Enemy Start |
//! |------------------|:----------:|:--------:|:--------:|:---------:|:-------:|:------:|:-----------:|
//! | Demon Form       |     X      |          |          |           |         |        |             |
//! | Noxious Fumes    |     X      |          |          |           |         |        |             |
//! | Brutality        |     X      |          |          |           |         |        |             |
//! | Berserk          |     X      |          |          |           |         |        |             |
//! | Infinite Blades  |     X      |          |          |           |         |        |             |
//! | Hello World      |     X      |          |          |           |         |        |             |
//! | Battle Hymn      |     X      |          |          |           |         |        |             |
//! | Wraith Form      |     X      |          |          |           |         |        |             |
//! | Creative AI      |     X      |          |          |           |         |        |             |
//! | Deva Form        |     X      |          |          |           |         |        |             |
//! | Magnetism        |     X      |          |          |           |         |        |             |
//! | Doppelganger Drw |     X      |          |          |           |         |        |             |
//! | Doppelganger Nrg |     X      |          |          |           |         |        |             |
//! | Enter Divinity   |     X      |          |          |           |         |        |             |
//! | Mayhem           |     X      |          |          |           |         |        |             |
//! | Tools/Trade      |     X      |          |          |           |         |        |             |
//! | Devotion         |     X      |          |          |           |         |        |             |
//! | Metallicize      |            |    X     |          |           |         |        |      X      |
//! | Plated Armor     |            |    X     |          |           |         |        |             |
//! | Like Water       |            |    X     |          |           |         |        |             |
//! | Study            |            |    X     |          |           |         |        |             |
//! | Omega            |            |    X     |          |           |         |        |             |
//! | Combust          |            |    X     |          |           |         |        |             |
//! | Rage             |            |    X     |          |     X     |         |        |             |
//! | After Image      |            |          |    X     |           |         |        |             |
//! | Feel No Pain     |            |          |          |           |    X    |        |             |
//! | Dark Embrace     |            |          |          |           |    X    |        |             |
//! | Mental Fortress  |            |          |          |           |         |   X    |             |
//! | Rushdown         |            |          |          |           |         |   X    |             |
//! | Regeneration     |            |          |          |           |         |        |      X      |
//! | Growth           |            |          |          |           |         |        |      X      |
//! | Fading           |            |          |          |           |         |        |      X      |
//! | The Bomb         |            |          |          |           |         |        |      X      |
//! | Ritual           |            |          |          |           |         |        |      X      |
//!
//! ### Inline-dispatched powers (in engine.rs / combat_hooks.rs):
//!
//! - **Card play**: Envenom, Sadistic, Electrodynamics, Thousand Cuts, Panache
//! - **Card play (enemy)**: Beat of Death, Slow, Time Warp, Curiosity, SkillBurn, Forcefield
//! - **Card replay**: Echo Form, Double Tap, Burst, Necronomicon
//! - **Card draw**: Evolve, Fire Breathing
//! - **Block gain**: Juggernaut, Wave of the Hand
//! - **HP loss**: Rupture, Plated Armor (decrement), Static Discharge
//! - **On attacked**: Thorns, Flame Barrier, Curl-Up, Malleable, Sharp Hide, Shifting
//! - **On shuffle**: Sundial, Abacus (relics)
//! - **On enemy death**: Spore Cloud, Gremlin Horn
//! - **Damage modify**: Slow, Intangible, Invincible, Flight
//! - **State flags**: Barricade, Blur, Corruption, Confusion, Entangled, NoAttack, NoDraw

use crate::ids::StatusId;
use crate::state::EntityState;
use crate::status_ids::sid;

use super::hooks::{
    TurnStartEffect, TurnEndEffect, OnCardPlayedEffect,
    OnExhaustEffect, OnStanceChangeEffect, EnemyTurnStartEffect,
};
use super::PowerType;

// ===========================================================================
// Hook function type aliases
// ===========================================================================

pub type TurnStartHookFn = fn(i32, &mut EntityState) -> TurnStartEffect;
pub type TurnEndHookFn = fn(i32, &mut EntityState) -> TurnEndEffect;
pub type OnCardPlayedHookFn = fn(i32, &EntityState) -> OnCardPlayedEffect;
pub type OnExhaustHookFn = fn(i32, &EntityState) -> OnExhaustEffect;
pub type OnStanceChangeHookFn = fn(i32, &EntityState, bool) -> OnStanceChangeEffect;
pub type EnemyTurnStartHookFn = fn(i32, &mut EntityState) -> EnemyTurnStartEffect;

// ===========================================================================
// Registry Entry
// ===========================================================================

pub struct PowerRegistryEntry {
    /// Card effect tag string (e.g., "demon_form"). Empty for non-card powers.
    pub tag: &'static str,
    /// Runtime status key.
    pub status_id: StatusId,
    /// Buff or Debuff (used for enemy debuff clearing).
    pub power_type: PowerType,
    /// Whether the power stacks additively.
    pub stackable: bool,
    /// Whether the power decrements each turn.
    pub is_turn_based: bool,

    // Hook pointers — None means this power doesn't fire on that trigger.
    pub on_turn_start: Option<TurnStartHookFn>,
    pub on_turn_end: Option<TurnEndHookFn>,
    pub on_card_played_pre: Option<OnCardPlayedHookFn>,
    pub on_card_played_post: Option<OnCardPlayedHookFn>,
    pub on_exhaust: Option<OnExhaustHookFn>,
    pub on_stance_change: Option<OnStanceChangeHookFn>,
    pub on_enemy_turn_start: Option<EnemyTurnStartHookFn>,
}

impl PowerRegistryEntry {
    pub const NONE: Self = Self {
        tag: "",
        status_id: StatusId(0),
        power_type: PowerType::Buff,
        stackable: true,
        is_turn_based: false,
        on_turn_start: None,
        on_turn_end: None,
        on_card_played_pre: None,
        on_card_played_post: None,
        on_exhaust: None,
        on_stance_change: None,
        on_enemy_turn_start: None,
    };
}

// ===========================================================================
// The Registry — ONE static table for all powers
// ===========================================================================

pub static POWER_REGISTRY: &[PowerRegistryEntry] = &[
    // ---- Turn Start powers ----
    PowerRegistryEntry {
        tag: "demon_form", status_id: sid::DEMON_FORM,
        on_turn_start: Some(super::hooks::hook_demon_form),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "noxious_fumes", status_id: sid::NOXIOUS_FUMES,
        on_turn_start: Some(super::hooks::hook_noxious_fumes),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "brutality", status_id: sid::BRUTALITY,
        on_turn_start: Some(super::hooks::hook_brutality),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "berserk", status_id: sid::BERSERK,
        on_turn_start: Some(super::hooks::hook_berserk),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "infinite_blades", status_id: sid::INFINITE_BLADES,
        on_turn_start: Some(super::hooks::hook_infinite_blades),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "hello_world", status_id: sid::HELLO_WORLD,
        on_turn_start: Some(super::hooks::hook_hello_world),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "battle_hymn", status_id: sid::BATTLE_HYMN,
        on_turn_start: Some(super::hooks::hook_battle_hymn),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::WRAITH_FORM,
        on_turn_start: Some(super::hooks::hook_wraith_form),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::CREATIVE_AI,
        on_turn_start: Some(super::hooks::hook_creative_ai),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "deva_form", status_id: sid::DEVA_FORM,
        on_turn_start: Some(super::hooks::hook_deva_form),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "magnetism", status_id: sid::MAGNETISM,
        on_turn_start: Some(super::hooks::hook_magnetism),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::DOPPELGANGER_DRAW,
        on_turn_start: Some(super::hooks::hook_doppelganger_draw),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::DOPPELGANGER_ENERGY,
        on_turn_start: Some(super::hooks::hook_doppelganger_energy),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::ENTER_DIVINITY,
        on_turn_start: Some(super::hooks::hook_enter_divinity),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "mayhem", status_id: sid::MAYHEM,
        on_turn_start: Some(super::hooks::hook_mayhem),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "tools_of_the_trade", status_id: sid::TOOLS_OF_THE_TRADE,
        on_turn_start: Some(super::hooks::hook_tools_of_the_trade),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "devotion", status_id: sid::DEVOTION,
        on_turn_start: Some(super::hooks::hook_devotion),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "foresight", status_id: sid::FORESIGHT,
        on_turn_start: Some(super::hooks::hook_foresight),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::COLLECT_MIRACLES,
        on_turn_start: Some(super::hooks::hook_collect_miracles),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::SIMMERING_FURY,
        on_turn_start: Some(super::hooks::hook_simmering_fury),
        ..PowerRegistryEntry::NONE
    },

    // ---- Turn End powers ----
    PowerRegistryEntry {
        tag: "metallicize", status_id: sid::METALLICIZE,
        on_turn_end: Some(super::hooks::hook_end_metallicize),
        on_enemy_turn_start: Some(super::hooks::hook_enemy_metallicize),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::PLATED_ARMOR,
        on_turn_end: Some(super::hooks::hook_end_plated_armor),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "like_water", status_id: sid::LIKE_WATER,
        on_turn_end: Some(super::hooks::hook_end_like_water),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "study", status_id: sid::STUDY,
        on_turn_end: Some(super::hooks::hook_end_study),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "omega", status_id: sid::OMEGA,
        on_turn_end: Some(super::hooks::hook_end_omega),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "combust", status_id: sid::COMBUST,
        on_turn_end: Some(super::hooks::hook_end_combust),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::RAGE,
        on_turn_end: Some(super::hooks::hook_end_rage),
        on_card_played_post: Some(super::hooks::hook_play_rage),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::TEMP_STRENGTH,
        on_turn_end: Some(super::hooks::hook_end_temp_strength),
        ..PowerRegistryEntry::NONE
    },

    // ---- On Card Played powers ----
    PowerRegistryEntry {
        tag: "after_image", status_id: sid::AFTER_IMAGE,
        on_card_played_pre: Some(super::hooks::hook_play_after_image),
        ..PowerRegistryEntry::NONE
    },

    // ---- On Exhaust powers ----
    PowerRegistryEntry {
        tag: "feel_no_pain", status_id: sid::FEEL_NO_PAIN,
        on_exhaust: Some(super::hooks::hook_exhaust_feel_no_pain),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "dark_embrace", status_id: sid::DARK_EMBRACE,
        on_exhaust: Some(super::hooks::hook_exhaust_dark_embrace),
        ..PowerRegistryEntry::NONE
    },

    // ---- On Stance Change powers ----
    PowerRegistryEntry {
        tag: "on_stance_change_block", status_id: sid::MENTAL_FORTRESS,
        on_stance_change: Some(super::hooks::hook_stance_mental_fortress),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "on_wrath_draw", status_id: sid::RUSHDOWN,
        on_stance_change: Some(super::hooks::hook_stance_rushdown),
        ..PowerRegistryEntry::NONE
    },

    // ---- Enemy Turn Start powers (no card tags) ----
    PowerRegistryEntry {
        tag: "", status_id: sid::REGENERATION,
        on_enemy_turn_start: Some(super::hooks::hook_enemy_regeneration),
        is_turn_based: true,
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::GROWTH,
        on_enemy_turn_start: Some(super::hooks::hook_enemy_growth),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::FADING,
        power_type: PowerType::Debuff,
        on_enemy_turn_start: Some(super::hooks::hook_enemy_fading),
        is_turn_based: true,
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::THE_BOMB,
        on_enemy_turn_start: Some(super::hooks::hook_enemy_the_bomb),
        ..PowerRegistryEntry::NONE
    },
    PowerRegistryEntry {
        tag: "", status_id: sid::RITUAL,
        on_enemy_turn_start: Some(super::hooks::hook_enemy_ritual),
        ..PowerRegistryEntry::NONE
    },

    // ---- Powers with card tags but no hooks (install-only, dispatched inline) ----
    PowerRegistryEntry { tag: "barricade", status_id: sid::BARRICADE, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "evolve", status_id: sid::EVOLVE, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "fire_breathing", status_id: sid::FIRE_BREATHING, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "juggernaut", status_id: sid::JUGGERNAUT, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "rupture", status_id: sid::RUPTURE, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "thousand_cuts", status_id: sid::THOUSAND_CUTS, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "envenom", status_id: sid::ENVENOM, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "accuracy", status_id: sid::ACCURACY, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "thorns", status_id: sid::THORNS, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "well_laid_plans", status_id: sid::WELL_LAID_PLANS, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "loop_orb", status_id: sid::LOOP, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "lightning_hits_all", status_id: sid::ELECTRODYNAMICS, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "sadistic_nature", status_id: sid::SADISTIC, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "panache", status_id: sid::PANACHE, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "on_scry_block", status_id: sid::NIRVANA, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "draw_on_power_play", status_id: sid::HEATSINK, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "channel_lightning_on_power", status_id: sid::STORM, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "buffer", status_id: sid::BUFFER, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "establishment", status_id: sid::ESTABLISHMENT, ..PowerRegistryEntry::NONE },
    PowerRegistryEntry { tag: "extra_draw_each_turn", status_id: sid::DRAW, ..PowerRegistryEntry::NONE },
];

// ===========================================================================
// Lookup helpers
// ===========================================================================

/// Find a registry entry by its card effect tag. Returns None for unknown tags.
pub fn lookup_by_tag(tag: &str) -> Option<&'static PowerRegistryEntry> {
    POWER_REGISTRY.iter().find(|e| !e.tag.is_empty() && e.tag == tag)
}

/// Check if a power name corresponds to a debuff (for enemy debuff clearing).
pub fn is_debuff(name: &str) -> bool {
    matches!(name,
        "Weakened" | "Vulnerable" | "Frail" | "Poison" | "Constricted" |
        "Hex" | "Confused" | "Entangled" | "NoDraw" | "DrawReduction" |
        "Slow" | "LockOn" | "Fading" | "NoAttack" | "Choked" |
        "NoBlock" | "Surrounded"
    )
}

// ===========================================================================
// Dispatch functions — replace manual hook tables
// ===========================================================================

/// Dispatch all turn-start power hooks for the player.
pub fn dispatch_turn_start(entity: &mut EntityState) -> TurnStartEffect {
    let mut out = TurnStartEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_turn_start {
            let amt = entity.status(entry.status_id);
            if amt != 0 {
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}

/// Dispatch all turn-end power hooks for the player.
pub fn dispatch_turn_end(entity: &mut EntityState, in_calm: bool) -> TurnEndEffect {
    let mut out = TurnEndEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_turn_end {
            let amt = entity.status(entry.status_id);
            if amt != 0 {
                // LikeWater needs stance context — skip if not in Calm
                if entry.status_id == sid::LIKE_WATER && !in_calm {
                    continue;
                }
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}

/// Dispatch pre-effects card-played hooks (AfterImage).
pub fn dispatch_on_card_played_pre(entity: &EntityState) -> OnCardPlayedEffect {
    let mut out = OnCardPlayedEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_card_played_pre {
            let amt = entity.status(entry.status_id);
            if amt > 0 {
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}

/// Dispatch post-effects card-played hooks (Rage on Attacks).
pub fn dispatch_on_card_played_post(entity: &EntityState, is_attack: bool) -> OnCardPlayedEffect {
    let mut out = OnCardPlayedEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_card_played_post {
            let amt = entity.status(entry.status_id);
            if amt > 0 {
                // Rage only fires on Attacks
                if entry.status_id == sid::RAGE && !is_attack {
                    continue;
                }
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}

/// Dispatch on-exhaust hooks (Feel No Pain, Dark Embrace).
pub fn dispatch_on_exhaust(entity: &EntityState) -> OnExhaustEffect {
    let mut out = OnExhaustEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_exhaust {
            let amt = entity.status(entry.status_id);
            if amt > 0 {
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}

/// Dispatch on-stance-change hooks.
pub fn dispatch_on_stance_change(entity: &EntityState, entering_wrath: bool) -> OnStanceChangeEffect {
    let mut out = OnStanceChangeEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_stance_change {
            let amt = entity.status(entry.status_id);
            if amt > 0 {
                out.merge(hook_fn(amt, entity, entering_wrath));
            }
        }
    }
    out
}

/// Count how many registered powers are active (value > 0) on an entity.
/// Used by Force Field to reduce cost by number of active powers.
pub fn count_active_powers(entity: &EntityState) -> i32 {
    let mut count = 0;
    for entry in POWER_REGISTRY.iter() {
        if entity.status(entry.status_id) > 0 {
            count += 1;
        }
    }
    count
}

/// Dispatch enemy-turn-start hooks.
pub fn dispatch_enemy_turn_start(entity: &mut EntityState, is_first_turn: bool) -> EnemyTurnStartEffect {
    let mut out = EnemyTurnStartEffect::default();
    for entry in POWER_REGISTRY.iter() {
        if let Some(hook_fn) = entry.on_enemy_turn_start {
            let amt = entity.status(entry.status_id);
            if amt != 0 {
                // Ritual skips first turn
                if entry.status_id == sid::RITUAL && is_first_turn {
                    continue;
                }
                out.merge(hook_fn(amt, entity));
            }
        }
    }
    out
}
