//! Faithful, deterministic Slay the Spire simulation core.
//!
//! Consumer-specific observations, search policies, training rewards, and
//! language bindings intentionally live outside this crate's supported API.

pub mod actions;
pub mod card_effects;
pub mod cards;
pub mod checkpoint;
pub mod combat_hooks;
pub mod combat_types;
pub mod damage;
pub mod decision;
pub mod effects;
pub mod enemies;
pub mod engine;
pub mod events;
pub mod gameplay;
pub mod ids;
pub mod map;
pub mod orbs;
pub mod potions;
pub mod powers;
pub mod relic_flags;
pub mod relics;
pub mod run;
pub mod seed;
pub mod state;
pub mod status_effects;
pub mod status_ids;
pub mod trace;

#[cfg(test)]
mod tests;
