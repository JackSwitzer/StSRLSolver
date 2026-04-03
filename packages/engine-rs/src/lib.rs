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
pub mod combat_types;
pub mod combat_verbs;
pub mod card_effects;
pub mod cards;
pub mod combat_hooks;
pub mod ids;
pub mod status_ids;
pub mod damage;
pub mod enemies;
pub mod engine;
pub mod events;
pub mod map;
pub mod obs;
pub mod orbs;
pub mod potions;
pub mod powers;
pub mod relics;
pub mod run;
pub mod seed;
pub mod state;
pub mod status_effects;
pub mod status_keys;

#[cfg(test)]
mod tests;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

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
    m.add_class::<StSEngine>()?;
    m.add_class::<ActionInfo>()?;
    m.add_class::<CombatSolver>()?;
    Ok(())
}

// ===========================================================================
// ActionInfo -- describes a legal action with rich metadata
// ===========================================================================

#[pyclass]
#[derive(Clone)]
pub struct ActionInfo {
    #[pyo3(get)]
    pub id: i32,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub action_type: String,
    /// Card name (if card play), potion name (if potion use), else empty.
    #[pyo3(get)]
    pub card_name: String,
    /// Target index (-1 = no target, 0+ = enemy index).
    #[pyo3(get)]
    pub target: i32,
    /// Human-readable description of what this action does.
    #[pyo3(get)]
    pub description: String,
}

#[pymethods]
impl ActionInfo {
    fn __repr__(&self) -> String {
        format!(
            "ActionInfo(id={}, name='{}', type='{}', card='{}', target={}, desc='{}')",
            self.id, self.name, self.action_type, self.card_name, self.target, self.description
        )
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new_bound(py);
        d.set_item("id", self.id)?;
        d.set_item("name", &self.name)?;
        d.set_item("action_type", &self.action_type)?;
        d.set_item("card_name", &self.card_name)?;
        d.set_item("target", self.target)?;
        d.set_item("description", &self.description)?;
        Ok(d)
    }
}

// ===========================================================================
// CombatSolver -- cloned combat state for MCTS lookahead
// ===========================================================================

#[pyclass]
#[derive(Clone)]
pub struct CombatSolver {
    engine: engine::CombatEngine,
}

#[pymethods]
impl CombatSolver {
    fn step(&mut self, action_id: i32) -> PyResult<(f32, bool)> {
        let action = decode_combat_action_id(action_id)?;
        self.engine.execute_action(&action);
        let done = self.engine.is_combat_over();
        Ok((0.0, done))
    }

    fn get_legal_actions(&self) -> Vec<i32> {
        self.engine
            .get_legal_actions()
            .iter()
            .map(encode_combat_action)
            .collect()
    }

    fn get_legal_action_infos(&self) -> Vec<ActionInfo> {
        self.engine
            .get_legal_actions()
            .iter()
            .map(|a| {
                let id = encode_combat_action(a);
                describe_combat_action(a, id, &self.engine.state)
            })
            .collect()
    }

    fn is_done(&self) -> bool {
        self.engine.is_combat_over()
    }

    fn is_won(&self) -> bool {
        self.engine.state.player_won
    }

    fn get_hp(&self) -> (i32, i32) {
        (self.engine.state.player.hp, self.engine.state.player.max_hp)
    }

    fn get_energy(&self) -> i32 {
        self.engine.state.energy
    }

    fn get_turn(&self) -> i32 {
        self.engine.state.turn
    }

    /// Deep copy for MCTS tree branching.
    fn clone_solver(&self) -> Self {
        Self {
            engine: self.engine.clone(),
        }
    }

    /// Alias for clone_solver (backward compat).
    fn copy(&self) -> Self {
        self.clone_solver()
    }

    fn __repr__(&self) -> String {
        format!(
            "CombatSolver(hp={}/{}, energy={}, turn={}, done={})",
            self.engine.state.player.hp,
            self.engine.state.player.max_hp,
            self.engine.state.energy,
            self.engine.state.turn,
            self.engine.is_combat_over(),
        )
    }
}

// ===========================================================================
// Combat action encoding helpers
// ===========================================================================

fn encode_combat_action(a: &crate::actions::Action) -> i32 {
    match a {
        crate::actions::Action::EndTurn => 0,
        crate::actions::Action::PlayCard {
            card_idx,
            target_idx,
        } => 1 + (*card_idx as i32 * 6) + (*target_idx + 1),
        crate::actions::Action::UsePotion {
            potion_idx,
            target_idx,
        } => 100 + (*potion_idx as i32 * 6) + (*target_idx + 1),
    }
}

fn decode_combat_action_id(action_id: i32) -> PyResult<crate::actions::Action> {
    match action_id {
        0 => Ok(crate::actions::Action::EndTurn),
        id if id >= 1 && id < 100 => {
            let c = id - 1;
            Ok(crate::actions::Action::PlayCard {
                card_idx: (c / 6) as usize,
                target_idx: (c % 6) as i32 - 1,
            })
        }
        id if id >= 100 => {
            let p = id - 100;
            Ok(crate::actions::Action::UsePotion {
                potion_idx: (p / 6) as usize,
                target_idx: (p % 6) as i32 - 1,
            })
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err("Invalid action id")),
    }
}

fn describe_combat_action(
    action: &crate::actions::Action,
    id: i32,
    state: &crate::state::CombatState,
) -> ActionInfo {
    match action {
        crate::actions::Action::EndTurn => ActionInfo {
            id,
            name: "end_turn".to_string(),
            action_type: "combat".to_string(),
            card_name: String::new(),
            target: -1,
            description: "End your turn".to_string(),
        },
        crate::actions::Action::PlayCard {
            card_idx,
            target_idx,
        } => {
            let registry = crate::cards::CardRegistry::new();
            let card_name = state
                .hand
                .get(*card_idx)
                .map(|c| registry.card_name(c.def_id).to_string())
                .unwrap_or_else(|| format!("card_{}", card_idx));
            let target_desc = if *target_idx >= 0 {
                let enemy_name = state
                    .enemies
                    .get(*target_idx as usize)
                    .map(|e| e.name.as_str())
                    .unwrap_or("?");
                format!(" -> {}", enemy_name)
            } else {
                String::new()
            };
            ActionInfo {
                id,
                name: format!("play_{}_{}", card_idx, target_idx),
                action_type: "card".to_string(),
                card_name: card_name.clone(),
                target: *target_idx,
                description: format!("Play {}{}", card_name, target_desc),
            }
        }
        crate::actions::Action::UsePotion {
            potion_idx,
            target_idx,
        } => {
            let potion_name = state
                .potions
                .get(*potion_idx)
                .cloned()
                .unwrap_or_else(|| format!("potion_{}", potion_idx));
            let target_desc = if *target_idx >= 0 {
                let enemy_name = state
                    .enemies
                    .get(*target_idx as usize)
                    .map(|e| e.name.as_str())
                    .unwrap_or("?");
                format!(" -> {}", enemy_name)
            } else {
                String::new()
            };
            ActionInfo {
                id,
                name: format!("use_potion_{}_{}", potion_idx, target_idx),
                action_type: "potion".to_string(),
                card_name: potion_name.clone(),
                target: *target_idx,
                description: format!("Use {}{}", potion_name, target_desc),
            }
        }
    }
}

// ===========================================================================
// StSEngine -- Gym-style API wrapping RunEngine
// ===========================================================================

#[pyclass]
pub struct StSEngine {
    inner: run::RunEngine,
    run_engine_py: PyRunEngine,
}

#[pymethods]
impl StSEngine {
    #[new]
    #[pyo3(signature = (seed, ascension=20, character="watcher"))]
    fn new(seed: &str, ascension: i32, character: &str) -> Self {
        let _ = character;
        let seed_val = seed::seed_from_string(seed);
        let engine = run::RunEngine::new(seed_val, ascension);
        let run_py = PyRunEngine {
            inner: engine.clone(),
        };
        Self {
            inner: engine,
            run_engine_py: run_py,
        }
    }

    /// Gym-style step: action -> (state_dict, reward, done, info)
    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: i32,
    ) -> PyResult<(Bound<'py, PyDict>, f32, bool, Bound<'py, PyDict>)> {
        self.run_engine_py.inner = self.inner.clone();
        let (reward, done) = self.run_engine_py.step(action);
        self.inner = self.run_engine_py.inner.clone();

        let state_dict = self.build_state_dict(py)?;
        let info = self.build_info_dict(py, reward)?;

        Ok((state_dict, reward, done, info))
    }

    fn reset(&mut self, seed: &str) {
        let seed_val = seed::seed_from_string(seed);
        self.inner.reset(seed_val);
    }

    /// Rich state dict with ALL game state: run + combat + phase-specific data.
    fn get_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.build_state_dict(py)
    }

    /// Observation vector for neural network input.
    fn get_obs(&self) -> Vec<f32> {
        obs::get_observation(&self.inner).to_vec()
    }

    fn get_legal_actions(&self) -> Vec<ActionInfo> {
        let py_re = PyRunEngine {
            inner: self.inner.clone(),
        };
        let action_ids = py_re.get_legal_actions();
        action_ids
            .into_iter()
            .map(|id| self.describe_action(id))
            .collect()
    }

    /// Return just the action IDs (faster than full ActionInfo for hot loops).
    fn get_legal_action_ids(&self) -> Vec<i32> {
        let py_re = PyRunEngine {
            inner: self.inner.clone(),
        };
        py_re.get_legal_actions()
    }

    /// Clone the current combat state for MCTS lookahead.
    /// Returns None if not in combat phase.
    fn clone_combat(&self) -> Option<CombatSolver> {
        if self.inner.current_phase() != run::RunPhase::Combat {
            return None;
        }
        self.inner
            .get_combat_engine()
            .map(|ce| CombatSolver {
                engine: ce.clone(),
            })
    }

    fn get_seed(&self) -> String {
        seed::seed_to_string(self.inner.seed)
    }

    fn get_seed_int(&self) -> u64 {
        self.inner.seed
    }

    #[getter]
    fn ascension(&self) -> i32 {
        self.inner.run_state.ascension
    }

    #[getter]
    fn floor(&self) -> i32 {
        self.inner.run_state.floor
    }

    #[getter]
    fn hp(&self) -> i32 {
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
    fn phase(&self) -> &str {
        phase_str(self.inner.current_phase())
    }

    #[getter]
    fn done(&self) -> bool {
        self.inner.is_done()
    }

    #[getter]
    fn won(&self) -> bool {
        self.inner.run_state.run_won
    }

    fn __repr__(&self) -> String {
        format!(
            "StSEngine(seed='{}', floor={}, hp={}/{}, phase={:?}, A{})",
            self.get_seed(),
            self.inner.run_state.floor,
            self.inner.run_state.current_hp,
            self.inner.run_state.max_hp,
            self.inner.current_phase(),
            self.inner.run_state.ascension,
        )
    }
}

impl StSEngine {
    fn build_state_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new_bound(py);
        let rs = &self.inner.run_state;
        let phase = self.inner.current_phase();

        d.set_item("floor", rs.floor)?;
        d.set_item("hp", rs.current_hp)?;
        d.set_item("max_hp", rs.max_hp)?;
        d.set_item("gold", rs.gold)?;
        d.set_item("ascension", rs.ascension)?;
        d.set_item("act", rs.act)?;
        d.set_item("phase", phase_str(phase))?;
        d.set_item("done", rs.run_over)?;
        d.set_item("run_won", rs.run_won)?;
        d.set_item("seed", seed::seed_to_string(self.inner.seed))?;

        let deck_list = PyList::new_bound(py, &rs.deck);
        d.set_item("deck", deck_list)?;
        d.set_item("deck_size", rs.deck.len())?;
        let relic_list = PyList::new_bound(py, &rs.relics);
        d.set_item("relics", relic_list)?;
        let potion_list = PyList::new_bound(py, &rs.potions);
        d.set_item("potions", potion_list)?;

        d.set_item("combats_won", rs.combats_won)?;
        d.set_item("elites_killed", rs.elites_killed)?;
        d.set_item("bosses_killed", rs.bosses_killed)?;
        d.set_item("total_reward", self.inner.total_reward)?;
        d.set_item("boss", self.inner.boss_name())?;

        d.set_item("has_ruby_key", rs.has_ruby_key)?;
        d.set_item("has_emerald_key", rs.has_emerald_key)?;
        d.set_item("has_sapphire_key", rs.has_sapphire_key)?;

        match phase {
            run::RunPhase::Combat => {
                if let Some(ce) = self.inner.get_combat_engine() {
                    let cs = &ce.state;
                    let combat = PyDict::new_bound(py);
                    combat.set_item("energy", cs.energy)?;
                    combat.set_item("max_energy", cs.max_energy)?;
                    combat.set_item("turn", cs.turn)?;
                    combat.set_item("stance", cs.stance.as_str())?;
                    combat.set_item("block", cs.player.block)?;
                    combat.set_item("mantra", cs.mantra)?;
                    combat.set_item("cards_played_this_turn", cs.cards_played_this_turn)?;

                    let hand_names: Vec<String> = cs.hand.iter()
                        .map(|c| ce.card_registry.card_name(c.def_id).to_string())
                        .collect();
                    let hand = PyList::new_bound(py, &hand_names);
                    combat.set_item("hand", hand)?;
                    combat.set_item("draw_pile_size", cs.draw_pile.len())?;
                    combat.set_item("discard_pile_size", cs.discard_pile.len())?;
                    combat.set_item("exhaust_pile_size", cs.exhaust_pile.len())?;

                    let statuses = PyDict::new_bound(py);
                    for (i, &val) in cs.player.statuses.iter().enumerate() {
                        if val != 0 {
                            let name = crate::status_ids::status_name(crate::ids::StatusId(i as u16));
                            statuses.set_item(name, val as i32)?;
                        }
                    }
                    combat.set_item("player_statuses", statuses)?;

                    let enemies_list = PyList::empty_bound(py);
                    for e in &cs.enemies {
                        let ed = PyDict::new_bound(py);
                        ed.set_item("id", &e.id)?;
                        ed.set_item("name", &e.name)?;
                        ed.set_item("hp", e.entity.hp)?;
                        ed.set_item("max_hp", e.entity.max_hp)?;
                        ed.set_item("block", e.entity.block)?;
                        ed.set_item("alive", e.is_alive())?;
                        ed.set_item("move_damage", e.move_damage())?;
                        ed.set_item("move_hits", e.move_hits())?;
                        ed.set_item("move_block", e.move_block())?;
                        ed.set_item("intent_damage", e.total_incoming_damage())?;
                        let es = PyDict::new_bound(py);
                        for (i, &val) in e.entity.statuses.iter().enumerate() {
                            if val != 0 {
                                let name = crate::status_ids::status_name(crate::ids::StatusId(i as u16));
                                es.set_item(name, val as i32)?;
                            }
                        }
                        ed.set_item("statuses", es)?;
                        enemies_list.append(ed)?;
                    }
                    combat.set_item("enemies", enemies_list)?;

                    combat.set_item("total_damage_dealt", cs.total_damage_dealt)?;
                    combat.set_item("total_damage_taken", cs.total_damage_taken)?;
                    combat.set_item("total_cards_played", cs.total_cards_played)?;

                    d.set_item("combat", combat)?;
                }
            }
            run::RunPhase::CardReward => {
                let rewards = self.inner.get_card_rewards();
                let reward_list = PyList::new_bound(py, rewards);
                d.set_item("card_rewards", reward_list)?;
            }
            run::RunPhase::Shop => {
                if let Some(shop) = self.inner.get_shop() {
                    let shop_dict = PyDict::new_bound(py);
                    let items = PyList::empty_bound(py);
                    for (card, price) in &shop.cards {
                        let item = PyDict::new_bound(py);
                        item.set_item("card", card.as_str())?;
                        item.set_item("price", *price)?;
                        items.append(item)?;
                    }
                    shop_dict.set_item("cards", items)?;
                    shop_dict.set_item("remove_price", shop.remove_price)?;
                    shop_dict.set_item("removal_used", shop.removal_used)?;
                    d.set_item("shop", shop_dict)?;
                }
            }
            run::RunPhase::Event => {
                d.set_item("event_options", self.inner.event_option_count())?;
            }
            _ => {}
        }

        Ok(d)
    }

    fn build_info_dict<'py>(
        &self,
        py: Python<'py>,
        reward: f32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let info = PyDict::new_bound(py);
        info.set_item("floor", self.inner.run_state.floor)?;
        info.set_item("hp", self.inner.run_state.current_hp)?;
        info.set_item("phase", phase_str(self.inner.current_phase()))?;
        info.set_item("run_won", self.inner.run_state.run_won)?;
        info.set_item("step_reward", reward)?;
        info.set_item("total_reward", self.inner.total_reward)?;
        Ok(info)
    }

    fn describe_action(&self, id: i32) -> ActionInfo {
        if id >= COMBAT_BASE {
            let combat_id = id - COMBAT_BASE;
            let action = if combat_id == 0 {
                crate::actions::Action::EndTurn
            } else if combat_id >= 100 {
                let p = combat_id - 100;
                crate::actions::Action::UsePotion {
                    potion_idx: (p / 6) as usize,
                    target_idx: (p % 6) as i32 - 1,
                }
            } else {
                let c = combat_id - 1;
                crate::actions::Action::PlayCard {
                    card_idx: (c / 6) as usize,
                    target_idx: (c % 6) as i32 - 1,
                }
            };

            if let Some(ce) = self.inner.get_combat_engine() {
                return describe_combat_action(&action, id, &ce.state);
            }
            return ActionInfo {
                id,
                name: format!("combat_{}", combat_id),
                action_type: "combat".to_string(),
                card_name: String::new(),
                target: -1,
                description: format!("Combat action {}", combat_id),
            };
        }

        let (name, atype, desc) = if id >= EVENT_BASE {
            let idx = id - EVENT_BASE;
            (
                format!("event_choice_{}", idx),
                "event".to_string(),
                format!("Choose event option {}", idx),
            )
        } else if id == SHOP_LEAVE {
            (
                "shop_leave".to_string(),
                "shop".to_string(),
                "Leave the shop".to_string(),
            )
        } else if id >= SHOP_REMOVE_BASE {
            let idx = id - SHOP_REMOVE_BASE;
            let card = self
                .inner
                .run_state
                .deck
                .get(idx as usize)
                .cloned()
                .unwrap_or_else(|| format!("card_{}", idx));
            (
                format!("shop_remove_{}", idx),
                "shop".to_string(),
                format!("Remove {} from deck", card),
            )
        } else if id >= SHOP_BUY_BASE {
            let idx = id - SHOP_BUY_BASE;
            let card_info = self
                .inner
                .get_shop()
                .and_then(|s| s.cards.get(idx as usize))
                .map(|(c, p)| format!("{} ({}g)", c, p))
                .unwrap_or_else(|| format!("item_{}", idx));
            (
                format!("shop_buy_{}", idx),
                "shop".to_string(),
                format!("Buy {}", card_info),
            )
        } else if id >= CAMP_UPGRADE_BASE {
            let idx = id - CAMP_UPGRADE_BASE;
            let card = self
                .inner
                .run_state
                .deck
                .get(idx as usize)
                .cloned()
                .unwrap_or_else(|| format!("card_{}", idx));
            (
                format!("camp_upgrade_{}", idx),
                "campfire".to_string(),
                format!("Upgrade {}", card),
            )
        } else if id == CAMP_REST {
            (
                "camp_rest".to_string(),
                "campfire".to_string(),
                "Rest and heal".to_string(),
            )
        } else if id == CARD_SKIP {
            (
                "card_skip".to_string(),
                "card_reward".to_string(),
                "Skip card reward".to_string(),
            )
        } else if id >= CARD_PICK_BASE {
            let idx = (id - CARD_PICK_BASE) as usize;
            let card = self
                .inner
                .get_card_rewards()
                .get(idx)
                .cloned()
                .unwrap_or_else(|| format!("card_{}", idx));
            (
                format!("card_pick_{}", idx),
                "card_reward".to_string(),
                format!("Pick {}", card),
            )
        } else {
            (
                format!("choose_path_{}", id),
                "map".to_string(),
                format!("Choose map path {}", id),
            )
        };

        ActionInfo {
            id,
            name,
            action_type: atype,
            card_name: String::new(),
            target: -1,
            description: desc,
        }
    }
}

fn phase_str(phase: run::RunPhase) -> &'static str {
    match phase {
        run::RunPhase::MapChoice => "map",
        run::RunPhase::Combat => "combat",
        run::RunPhase::CardReward => "card_reward",
        run::RunPhase::Campfire => "campfire",
        run::RunPhase::Shop => "shop",
        run::RunPhase::Event => "event",
        run::RunPhase::GameOver => "game_over",
    }
}

// ===========================================================================
// PyO3 RustRunEngine -- full run simulation exposed to Python
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
        phase_str(self.inner.current_phase())
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
            run::RunAction::CombatAction(a) => match a {
                crate::actions::Action::EndTurn => COMBAT_BASE,
                crate::actions::Action::PlayCard {
                    card_idx,
                    target_idx,
                } => COMBAT_BASE + 1 + (*card_idx as i32 * 6) + (*target_idx + 1),
                crate::actions::Action::UsePotion {
                    potion_idx,
                    target_idx,
                } => COMBAT_BASE + 100 + (*potion_idx as i32 * 6) + (*target_idx + 1),
            },
        }
    }

    pub(crate) fn decode_action(&self, action_id: i32) -> Option<run::RunAction> {
        if action_id >= COMBAT_BASE {
            let combat_id = action_id - COMBAT_BASE;
            if combat_id == 0 {
                return Some(run::RunAction::CombatAction(
                    crate::actions::Action::EndTurn,
                ));
            } else if combat_id >= 100 {
                let p = combat_id - 100;
                return Some(run::RunAction::CombatAction(
                    crate::actions::Action::UsePotion {
                        potion_idx: (p / 6) as usize,
                        target_idx: (p % 6) as i32 - 1,
                    },
                ));
            } else {
                let c = combat_id - 1;
                return Some(run::RunAction::CombatAction(
                    crate::actions::Action::PlayCard {
                        card_idx: (c / 6) as usize,
                        target_idx: (c % 6) as i32 - 1,
                    },
                ));
            }
        } else if action_id >= EVENT_BASE {
            return Some(run::RunAction::EventChoice(
                (action_id - EVENT_BASE) as usize,
            ));
        } else if action_id == SHOP_LEAVE {
            return Some(run::RunAction::ShopLeave);
        } else if action_id >= SHOP_REMOVE_BASE {
            return Some(run::RunAction::ShopRemoveCard(
                (action_id - SHOP_REMOVE_BASE) as usize,
            ));
        } else if action_id >= SHOP_BUY_BASE {
            return Some(run::RunAction::ShopBuyCard(
                (action_id - SHOP_BUY_BASE) as usize,
            ));
        } else if action_id >= CAMP_UPGRADE_BASE {
            return Some(run::RunAction::CampfireUpgrade(
                (action_id - CAMP_UPGRADE_BASE) as usize,
            ));
        } else if action_id == CAMP_REST {
            return Some(run::RunAction::CampfireRest);
        } else if action_id == CARD_SKIP {
            return Some(run::RunAction::SkipCardReward);
        } else if action_id >= CARD_PICK_BASE {
            return Some(run::RunAction::PickCard(
                (action_id - CARD_PICK_BASE) as usize,
            ));
        } else if action_id >= PATH_BASE {
            return Some(run::RunAction::ChoosePath(action_id as usize));
        }

        None
    }
}
