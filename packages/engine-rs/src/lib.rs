//! Fast Rust engine for Slay the Spire RL.
//!
//! This crate provides:
//! - A combat engine optimized for MCTS simulations (CombatEngine)
//! - A full run simulation engine for Act 1 (RunEngine)
//! - Map generation, room types, events, shop, campfire
//! - 480-dim observation encoding matching Python's state_encoders.py
//!
//! PyO3 bindings expose both engines to Python as `sts_engine`.

pub mod actions;
pub mod cards;
pub mod damage;
pub mod enemies;
pub mod engine;
pub mod map;
pub mod obs;
pub mod potions;
pub mod powers;
pub mod relics;
pub mod run;
pub mod state;

#[cfg(test)]
mod tests;

use pyo3::prelude::*;
use pyo3::types::PyDict;

// ===========================================================================
// PyO3 module
// ===========================================================================

/// Python module entry point.
#[pymodule]
fn sts_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<engine::RustCombatEngine>()?;
    m.add_class::<state::PyCombatState>()?;
    m.add_class::<actions::PyAction>()?;
    m.add_class::<PyRunEngine>()?;
    Ok(())
}

// ===========================================================================
// PyO3 RustRunEngine — full run simulation exposed to Python
// ===========================================================================

/// Run-level action IDs for the flat action space.
const PATH_BASE: i32 = 0;
const CARD_PICK_BASE: i32 = 100;
const CARD_SKIP: i32 = 103;
const CAMP_REST: i32 = 200;
const CAMP_UPGRADE_BASE: i32 = 201;
const SHOP_BUY_BASE: i32 = 300;
const SHOP_REMOVE_BASE: i32 = 350;
const SHOP_LEAVE: i32 = 399;
const EVENT_BASE: i32 = 400;
const COMBAT_BASE: i32 = 500;

#[pyclass(name = "RustRunEngine")]
pub struct PyRunEngine {
    inner: run::RunEngine,
}

#[pymethods]
impl PyRunEngine {
    #[new]
    #[pyo3(signature = (seed=42, ascension=20))]
    fn new_py(seed: u64, ascension: i32) -> Self {
        PyRunEngine {
            inner: run::RunEngine::new(seed, ascension),
        }
    }

    fn reset(&mut self, seed: u64) {
        self.inner.reset(seed);
    }

    fn step(&mut self, action_id: i32) -> (f32, bool) {
        let action = self.decode_action(action_id);
        match action {
            Some(a) => self.inner.step(&a),
            None => (0.0, self.inner.is_done()),
        }
    }

    fn get_legal_actions(&self) -> Vec<i32> {
        let actions = self.inner.get_legal_actions();
        actions.iter().map(|a| self.encode_action(a)).collect()
    }

    fn get_obs(&self) -> Vec<f32> {
        obs::get_observation(&self.inner).to_vec()
    }

    fn get_combat_obs(&self) -> Vec<f32> {
        obs::encode_combat_state(&self.inner).to_vec()
    }

    fn is_done(&self) -> bool {
        self.inner.is_done()
    }

    fn is_won(&self) -> bool {
        self.inner.run_state.run_won
    }

    #[getter]
    fn floor(&self) -> i32 {
        self.inner.run_state.floor
    }

    #[getter]
    fn current_hp(&self) -> i32 {
        self.inner.run_state.current_hp
    }

    #[getter]
    fn max_hp(&self) -> i32 {
        self.inner.run_state.max_hp
    }

    #[getter]
    fn gold(&self) -> i32 {
        self.inner.run_state.gold
    }

    #[getter]
    fn deck(&self) -> Vec<String> {
        self.inner.run_state.deck.clone()
    }

    #[getter]
    fn relics(&self) -> Vec<String> {
        self.inner.run_state.relics.clone()
    }

    #[getter]
    fn potions(&self) -> Vec<String> {
        self.inner.run_state.potions.clone()
    }

    #[getter]
    fn phase(&self) -> &str {
        match self.inner.current_phase() {
            run::RunPhase::MapChoice => "map",
            run::RunPhase::Combat => "combat",
            run::RunPhase::CardReward => "card_reward",
            run::RunPhase::Campfire => "campfire",
            run::RunPhase::Shop => "shop",
            run::RunPhase::Event => "event",
            run::RunPhase::GameOver => "game_over",
        }
    }

    #[getter]
    fn total_reward(&self) -> f32 {
        self.inner.total_reward
    }

    #[getter]
    fn boss_name(&self) -> String {
        self.inner.boss_name().to_string()
    }

    fn get_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("floor", self.inner.run_state.floor)?;
        dict.set_item("hp", self.inner.run_state.current_hp)?;
        dict.set_item("max_hp", self.inner.run_state.max_hp)?;
        dict.set_item("gold", self.inner.run_state.gold)?;
        dict.set_item("phase", self.phase())?;
        dict.set_item("combats_won", self.inner.run_state.combats_won)?;
        dict.set_item("elites_killed", self.inner.run_state.elites_killed)?;
        dict.set_item("bosses_killed", self.inner.run_state.bosses_killed)?;
        dict.set_item("run_won", self.inner.run_state.run_won)?;
        dict.set_item("total_reward", self.inner.total_reward)?;
        dict.set_item("deck_size", self.inner.run_state.deck.len())?;
        dict.set_item("boss", self.inner.boss_name())?;
        Ok(dict)
    }

    fn copy(&self) -> Self {
        PyRunEngine {
            inner: self.inner.clone(),
        }
    }

    #[getter]
    fn seed(&self) -> u64 {
        self.inner.seed
    }

    fn __repr__(&self) -> String {
        format!(
            "RustRunEngine(floor={}, hp={}/{}, gold={}, phase={}, deck={}, done={})",
            self.inner.run_state.floor,
            self.inner.run_state.current_hp,
            self.inner.run_state.max_hp,
            self.inner.run_state.gold,
            self.phase(),
            self.inner.run_state.deck.len(),
            self.inner.is_done(),
        )
    }
}

impl PyRunEngine {
    pub(crate) fn encode_action(&self, action: &run::RunAction) -> i32 {
        match action {
            run::RunAction::ChoosePath(i) => PATH_BASE + *i as i32,
            run::RunAction::PickCard(i) => CARD_PICK_BASE + *i as i32,
            run::RunAction::SkipCardReward => CARD_SKIP,
            run::RunAction::CampfireRest => CAMP_REST,
            run::RunAction::CampfireUpgrade(i) => CAMP_UPGRADE_BASE + *i as i32,
            run::RunAction::ShopBuyCard(i) => SHOP_BUY_BASE + *i as i32,
            run::RunAction::ShopRemoveCard(i) => SHOP_REMOVE_BASE + *i as i32,
            run::RunAction::ShopLeave => SHOP_LEAVE,
            run::RunAction::EventChoice(i) => EVENT_BASE + *i as i32,
            run::RunAction::CombatAction(a) => {
                match a {
                    crate::actions::Action::EndTurn => COMBAT_BASE,
                    crate::actions::Action::PlayCard { card_idx, target_idx } => {
                        // target_idx: -1 (no target) => +0, 0 => +1, 1 => +2, etc.
                        COMBAT_BASE + 1 + (*card_idx as i32 * 6) + (*target_idx + 1)
                    }
                    crate::actions::Action::UsePotion { potion_idx, target_idx } => {
                        // Same encoding: -1 => +0, 0 => +1, 1 => +2
                        COMBAT_BASE + 100 + (*potion_idx as i32 * 6) + (*target_idx + 1)
                    }
                }
            }
        }
    }

    pub(crate) fn decode_action(&self, action_id: i32) -> Option<run::RunAction> {
        if action_id >= COMBAT_BASE {
            let combat_id = action_id - COMBAT_BASE;
            if combat_id == 0 {
                return Some(run::RunAction::CombatAction(crate::actions::Action::EndTurn));
            } else if combat_id >= 100 {
                let p = combat_id - 100;
                let potion_idx = (p / 6) as usize;
                let target_raw = p % 6;
                // Symmetric with encode: 0 => -1, 1 => 0, 2 => 1, etc.
                return Some(run::RunAction::CombatAction(
                    crate::actions::Action::UsePotion {
                        potion_idx,
                        target_idx: target_raw as i32 - 1,
                    },
                ));
            } else {
                let c = combat_id - 1;
                let card_idx = (c / 6) as usize;
                let target_raw = c % 6;
                // Symmetric with encode: 0 => -1, 1 => 0, 2 => 1, etc.
                return Some(run::RunAction::CombatAction(
                    crate::actions::Action::PlayCard {
                        card_idx,
                        target_idx: target_raw as i32 - 1,
                    },
                ));
            }
        } else if action_id >= EVENT_BASE {
            return Some(run::RunAction::EventChoice((action_id - EVENT_BASE) as usize));
        } else if action_id == SHOP_LEAVE {
            return Some(run::RunAction::ShopLeave);
        } else if action_id >= SHOP_REMOVE_BASE {
            return Some(run::RunAction::ShopRemoveCard((action_id - SHOP_REMOVE_BASE) as usize));
        } else if action_id >= SHOP_BUY_BASE {
            return Some(run::RunAction::ShopBuyCard((action_id - SHOP_BUY_BASE) as usize));
        } else if action_id >= CAMP_UPGRADE_BASE {
            return Some(run::RunAction::CampfireUpgrade((action_id - CAMP_UPGRADE_BASE) as usize));
        } else if action_id == CAMP_REST {
            return Some(run::RunAction::CampfireRest);
        } else if action_id == CARD_SKIP {
            return Some(run::RunAction::SkipCardReward);
        } else if action_id >= CARD_PICK_BASE {
            return Some(run::RunAction::PickCard((action_id - CARD_PICK_BASE) as usize));
        } else if action_id >= PATH_BASE {
            return Some(run::RunAction::ChoosePath(action_id as usize));
        }

        None
    }
}
