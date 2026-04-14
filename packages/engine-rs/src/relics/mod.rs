//! Relic support for the owner-aware combat runtime.
//!
//! `defs` contains the canonical runtime schema for relic behavior. The old
//! helper-oracle test surfaces have been retired; production paths use
//! `effects::runtime`, while `run.rs` keeps only the live helper functions
//! still needed by engine code.

mod run;
pub mod defs;

pub(crate) use run::champion_belt_on_vulnerable;
pub(crate) use run::strike_dummy_bonus;
pub(crate) use run::wrist_blade_bonus;
pub(crate) use run::snecko_skull_bonus;
pub(crate) use run::chemical_x_bonus;
pub(crate) use run::violet_lotus_calm_exit_bonus;
pub(crate) use run::unceasing_top_should_draw;
pub(crate) use run::has_runic_pyramid;
pub(crate) use run::calipers_block_retention;
pub(crate) use run::has_ice_cream;
pub(crate) use run::necronomicon_should_trigger;
pub(crate) use run::necronomicon_mark_used;
pub(crate) use run::necronomicon_reset;
