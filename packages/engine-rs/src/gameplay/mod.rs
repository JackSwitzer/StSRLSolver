//! Universal gameplay architecture layer.
//!
//! This module provides a single definition envelope and registry surface for
//! cards, relics, powers, potions, enemies, events, and future run effects.
//! Current gameplay execution still routes through the existing engines during
//! the migration, but this module establishes the canonical shared type system.

pub mod registry;
pub mod runtime;
pub mod session;
pub mod types;

pub use registry::{global_registry, GameplayRegistry};
pub use runtime::*;
pub use session::*;
pub use types::*;
