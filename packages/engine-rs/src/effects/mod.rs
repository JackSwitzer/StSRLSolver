//! Card and entity effect runtime.
//!
//! Cards, relics, powers, and potions now all execute through typed runtime
//! metadata and owner-aware runtime instances from `runtime.rs`.

pub mod types;
pub mod card_runtime;

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
pub mod trigger;
pub mod entity_def;
pub mod runtime;

// Future hook files (Step 3):
pub mod hooks_simple;
// pub mod hooks_debuff;
// pub mod hooks_generate;
pub mod hooks_complex;
// pub mod hooks_power;
// pub mod hooks_scaling;

pub use types::*;
