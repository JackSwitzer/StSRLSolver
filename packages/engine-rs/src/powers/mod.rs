//! Power/status effect system for Slay the Spire.
//!
//! Design:
//! - Powers stored as status IDs + amounts on `EntityState.statuses`
//! - `PowerRegistryEntry` in `registry.rs` is the single source of truth
//! - Registry dispatch functions fire hooks at the appropriate trigger points
//! - Inline helpers in `buffs.rs`, `debuffs.rs`, `enemy_powers.rs` handle
//!   powers that need engine context or don't fit the registry pattern

use crate::state::EntityState;

// ---------------------------------------------------------------------------
// PowerType — buff vs debuff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerType {
    Buff,
    Debuff,
}

pub mod hooks;
pub mod registry;
pub mod defs;
mod buffs;
mod debuffs;
mod enemy_powers;

// ---------------------------------------------------------------------------
// Re-exports — only functions still called from engine.rs / combat_hooks.rs
// or from test files that exercise live production logic.
// ---------------------------------------------------------------------------

// -- buffs --
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
pub use buffs::check_buffer;
pub use buffs::get_envenom_amount;
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

// -- debuffs --
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
pub use debuffs::apply_invincible_cap_tracked;
pub use debuffs::reset_invincible_damage_taken;
pub use debuffs::slow_damage_multiplier;
pub use debuffs::apply_mode_shift_damage;

// -- enemy powers --
pub use enemy_powers::apply_ritual;
pub use enemy_powers::apply_generic_strength_up;
pub use enemy_powers::get_beat_of_death_damage;
pub use enemy_powers::increment_slow;
pub use enemy_powers::increment_time_warp;
pub use enemy_powers::reset_slow;
pub use enemy_powers::apply_growth;
pub use enemy_powers::decrement_the_bomb;
pub use enemy_powers::apply_regeneration;
pub use enemy_powers::get_regrow_heal;
pub use enemy_powers::get_spore_cloud_vulnerable;
