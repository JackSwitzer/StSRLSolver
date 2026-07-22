//! Power/status support for the owner-aware combat runtime.
//!
//! Visible power stacks live on `EntityState.statuses`; `defs/` and
//! `effects::runtime` own production trigger dispatch. This module exposes
//! only the small helper surface still called by the production engine.

mod buffs;
pub(crate) mod debuffs;
pub mod defs;
mod enemy_powers;
pub(crate) mod registry;

#[cfg(test)]
pub use buffs::process_end_of_round;
pub use buffs::{consume_next_turn_block, decrement_equilibrium};

pub use debuffs::{
    apply_debuff, apply_debuff_from_enemy, apply_invincible_cap_tracked, decrement_debuffs,
    reset_invincible_damage_taken, slow_damage_multiplier, tick_poison,
};

pub use enemy_powers::{apply_generic_strength_up, increment_time_warp, reset_slow};
