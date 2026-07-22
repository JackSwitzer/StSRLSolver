//! Card and entity effect runtime.
//!
//! Cards, relics, powers, and potions now all execute through typed runtime
//! metadata and owner-aware runtime instances from `runtime.rs`.

pub mod card_runtime;
pub mod types;

// Hook implementation files used by the canonical typed runtime.
pub mod hooks_discard;
pub mod hooks_draw;

pub mod declarative;
pub mod entity_def;
pub mod hooks_damage;
pub mod interpreter;
pub mod runtime;
pub mod trigger;

pub mod hooks_simple;

pub use types::*;
