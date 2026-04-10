//! Modular card effect dispatch system.
//!
//! Extends the `powers/registry.rs` pattern to all card effects. Each effect tag
//! is a registry entry with optional fn pointers per hook type (can_play, modify_cost,
//! modify_damage, on_play, on_retain, on_draw, on_discard, post_play_dest).
//!
//! The EffectFlags bitset provides O(1) tag checking in the MCTS hot path,
//! replacing the previous O(n) string scan per `card.effects.contains(&"tag")`.

pub mod flags;
pub mod types;
pub mod registry;

// Hook implementation files (Step 1)
pub mod hooks_can_play;
pub mod hooks_cost;
pub mod hooks_retain;
pub mod hooks_draw;
pub mod hooks_discard;
pub mod hooks_dest;

pub mod hooks_damage;
pub mod hooks_orb;
pub mod declarative;
pub mod interpreter;
pub mod fx;

// Future hook files (Step 3):
pub mod hooks_simple;
// pub mod hooks_debuff;
// pub mod hooks_generate;
// pub mod hooks_complex;
// pub mod hooks_power;
// pub mod hooks_scaling;

pub use flags::EffectFlags;
pub use types::*;
pub use registry::{
    build_effect_flags,
    dispatch_can_play,
    dispatch_modify_cost,
    dispatch_modify_damage,
    dispatch_on_play,
    dispatch_on_retain,
    dispatch_on_draw,
    dispatch_on_discard,
    dispatch_post_play_dest,
};
