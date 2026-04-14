//! Relic support for the owner-aware combat runtime.
//!
//! `defs` contains the canonical runtime schema for relic behavior. The old
//! helper-oracle test surfaces have been retired; production paths use
//! `effects::runtime`, while `run.rs` keeps only the live helper functions
//! still needed by engine code.

mod run;
pub mod defs;
