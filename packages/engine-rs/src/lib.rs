//! Fast Rust combat engine for Slay the Spire RL.
//!
//! This crate provides a stripped-down combat engine optimized for MCTS simulations.
//! It handles the common path fast (basic cards, damage/block math, turn loop) while
//! the Python engine handles 100% of edge cases.
//!
//! PyO3 bindings expose the engine to Python as `sts_engine`.

pub mod actions;
pub mod cards;
pub mod damage;
pub mod enemies;
pub mod engine;
pub mod potions;
pub mod powers;
pub mod relics;
pub mod state;

#[cfg(test)]
mod tests;

use pyo3::prelude::*;

/// Python module entry point.
#[pymodule]
fn sts_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<engine::RustCombatEngine>()?;
    m.add_class::<state::PyCombatState>()?;
    m.add_class::<actions::PyAction>()?;
    Ok(())
}
