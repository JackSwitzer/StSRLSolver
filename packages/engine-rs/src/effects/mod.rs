//! Card and entity effect runtime.
//!
//! Cards still use the fast static hook tables in this module, while relics,
//! powers, and potions now install owner-aware runtime instances from
//! `runtime.rs`. The legacy trigger scanner remains only as a parity oracle for
//! internal tests while the engine emits typed events through the runtime.

pub mod flags;
pub mod types;
pub mod registry;
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

pub use flags::EffectFlags;
pub use types::*;
pub use registry::{
    build_effect_flags,
    dispatch_modify_damage,
    dispatch_on_play,
    dispatch_on_retain,
    dispatch_on_draw,
    dispatch_on_discard,
    dispatch_post_play_dest,
};
