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
pub mod card_effects;
pub mod cards;
pub mod combat_hooks;
pub mod effects;
pub mod ids;
pub mod status_ids;
pub mod damage;
pub mod decision;
pub mod enemies;
pub mod engine;
pub mod events;
pub mod gameplay;
pub mod map;
pub mod obs;
pub mod orbs;
pub mod potions;
pub mod powers;
pub mod relic_flags;
pub mod relics;
pub mod run;
pub mod search;
pub mod seed;
pub mod state;
pub mod status_effects;

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
        if !self.engine.get_legal_actions().contains(&action) {
            return Err(pyo3::exceptions::PyValueError::new_err("Illegal combat action"));
        }
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

const COMBAT_PLAY_BASE_ID: i32 = 0x10_000;
const COMBAT_POTION_BASE_ID: i32 = 0x20_000;
const COMBAT_CONFIRM_SELECTION_ID: i32 = 0x30_000;
const COMBAT_CHOOSE_BASE_ID: i32 = 0x40_000;
const COMBAT_INDEX_SHIFT: i32 = 8;
const COMBAT_TARGET_MASK: i32 = 0xff;

fn encode_target_slot(target_idx: i32) -> i32 {
    target_idx + 1
}

fn decode_target_slot(encoded: i32) -> PyResult<i32> {
    if !(0..=COMBAT_TARGET_MASK).contains(&encoded) {
        return Err(pyo3::exceptions::PyValueError::new_err("Invalid combat target encoding"));
    }
    Ok(encoded - 1)
}

fn encode_indexed_combat_id(base: i32, idx: usize, target_idx: i32) -> i32 {
    base + ((idx as i32) << COMBAT_INDEX_SHIFT) + encode_target_slot(target_idx)
}

fn decode_indexed_combat_id(action_id: i32, base: i32) -> PyResult<(usize, i32)> {
    let encoded = action_id - base;
    if encoded < 0 {
        return Err(pyo3::exceptions::PyValueError::new_err("Invalid combat action id"));
    }
    let idx = (encoded >> COMBAT_INDEX_SHIFT) as usize;
    let target = decode_target_slot(encoded & COMBAT_TARGET_MASK)?;
    Ok((idx, target))
}

fn encode_combat_action(a: &crate::actions::Action) -> i32 {
    match a {
        crate::actions::Action::EndTurn => 0,
        crate::actions::Action::PlayCard {
            card_idx,
            target_idx,
        } => encode_indexed_combat_id(COMBAT_PLAY_BASE_ID, *card_idx, *target_idx),
        crate::actions::Action::UsePotion {
            potion_idx,
            target_idx,
        } => encode_indexed_combat_id(COMBAT_POTION_BASE_ID, *potion_idx, *target_idx),
        crate::actions::Action::ConfirmSelection => COMBAT_CONFIRM_SELECTION_ID,
        crate::actions::Action::Choose(idx) => COMBAT_CHOOSE_BASE_ID + *idx as i32,
    }
}

fn decode_combat_action_id(action_id: i32) -> PyResult<crate::actions::Action> {
    match action_id {
        0 => Ok(crate::actions::Action::EndTurn),
        id if (COMBAT_PLAY_BASE_ID..COMBAT_POTION_BASE_ID).contains(&id) => {
            let (card_idx, target_idx) = decode_indexed_combat_id(id, COMBAT_PLAY_BASE_ID)?;
            Ok(crate::actions::Action::PlayCard { card_idx, target_idx })
        }
        id if (COMBAT_POTION_BASE_ID..COMBAT_CONFIRM_SELECTION_ID).contains(&id) => {
            let (potion_idx, target_idx) = decode_indexed_combat_id(id, COMBAT_POTION_BASE_ID)?;
            Ok(crate::actions::Action::UsePotion {
                potion_idx,
                target_idx,
            })
        }
        COMBAT_CONFIRM_SELECTION_ID => Ok(crate::actions::Action::ConfirmSelection),
        id if id >= COMBAT_CHOOSE_BASE_ID => Ok(crate::actions::Action::Choose(
            (id - COMBAT_CHOOSE_BASE_ID) as usize,
        )),
        _ => Err(pyo3::exceptions::PyValueError::new_err("Invalid action id")),
    }
}

fn decode_combat_action_id_in_run(action_id: i32) -> Option<crate::actions::Action> {
    decode_combat_action_id(action_id).ok()
}

fn combat_choice_reason_name(reason: &crate::engine::ChoiceReason) -> &'static str {
    match reason {
        crate::engine::ChoiceReason::Scry => "scry",
        crate::engine::ChoiceReason::DiscardFromHand => "discard_from_hand",
        crate::engine::ChoiceReason::ExhaustFromHand => "exhaust_from_hand",
        crate::engine::ChoiceReason::PutOnTopFromHand => "put_on_top_from_hand",
        crate::engine::ChoiceReason::PickFromDiscard => "pick_from_discard",
        crate::engine::ChoiceReason::PickFromDrawPile => "pick_from_draw_pile",
        crate::engine::ChoiceReason::DiscoverCard => "discover_card",
        crate::engine::ChoiceReason::PickOption => "pick_option",
        crate::engine::ChoiceReason::PlayCardFree => "play_card_free",
        crate::engine::ChoiceReason::DualWield => "dual_wield",
        crate::engine::ChoiceReason::UpgradeCard => "upgrade_card",
        crate::engine::ChoiceReason::PickFromExhaust => "pick_from_exhaust",
        crate::engine::ChoiceReason::SearchDrawPile => "search_draw_pile",
        crate::engine::ChoiceReason::ReturnFromDiscard => "return_from_discard",
        crate::engine::ChoiceReason::ForethoughtPick => "forethought_pick",
        crate::engine::ChoiceReason::RecycleCard => "recycle_card",
        crate::engine::ChoiceReason::DiscardForEffect => "discard_for_effect",
        crate::engine::ChoiceReason::SetupPick => "setup_pick",
        crate::engine::ChoiceReason::PlayCardFreeFromDraw => "play_card_free_from_draw",
    }
}

fn choice_option_payload(
    option: &crate::engine::ChoiceOption,
    combat: &crate::engine::CombatEngine,
) -> (String, i32, String) {
    match option {
        crate::engine::ChoiceOption::HandCard(idx) => {
            let name = combat
                .state
                .hand
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("hand_{}", idx));
            ("hand_card".to_string(), *idx as i32, name)
        }
        crate::engine::ChoiceOption::DrawCard(idx) => {
            let name = combat
                .state
                .draw_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("draw_{}", idx));
            ("draw_card".to_string(), *idx as i32, name)
        }
        crate::engine::ChoiceOption::DiscardCard(idx) => {
            let name = combat
                .state
                .discard_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("discard_{}", idx));
            ("discard_card".to_string(), *idx as i32, name)
        }
        crate::engine::ChoiceOption::RevealedCard(card) => (
            "revealed_card".to_string(),
            -1,
            combat.card_registry.card_name(card.def_id).to_string(),
        ),
        crate::engine::ChoiceOption::GeneratedCard(card) => (
            "generated_card".to_string(),
            -1,
            combat.card_registry.card_name(card.def_id).to_string(),
        ),
        crate::engine::ChoiceOption::Named(name) => ("named".to_string(), -1, (*name).to_string()),
        crate::engine::ChoiceOption::ExhaustCard(idx) => {
            let name = combat
                .state
                .exhaust_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("exhaust_{}", idx));
            ("exhaust_card".to_string(), *idx as i32, name)
        }
    }
}

fn build_combat_choice_dict<'py>(
    py: Python<'py>,
    combat: &crate::engine::CombatEngine,
) -> PyResult<Bound<'py, PyDict>> {
    let choice = PyDict::new_bound(py);
    if let Some(ctx) = &combat.choice {
        choice.set_item("active", true)?;
        choice.set_item("reason", combat_choice_reason_name(&ctx.reason))?;
        choice.set_item("option_count", ctx.options.len())?;
        choice.set_item("min_picks", ctx.min_picks)?;
        choice.set_item("max_picks", ctx.max_picks)?;
        choice.set_item("selected", PyList::new_bound(py, &ctx.selected))?;

        let options = PyList::empty_bound(py);
        for (option_idx, option) in ctx.options.iter().enumerate() {
            let option_dict = PyDict::new_bound(py);
            let (kind, source_idx, label) = choice_option_payload(option, combat);
            option_dict.set_item("index", option_idx)?;
            option_dict.set_item("kind", kind)?;
            option_dict.set_item("source_index", source_idx)?;
            option_dict.set_item("label", label)?;
            option_dict.set_item("selected", ctx.selected.contains(&option_idx))?;
            options.append(option_dict)?;
        }
        choice.set_item("options", options)?;
    } else {
        choice.set_item("active", false)?;
        choice.set_item("reason", py.None())?;
        choice.set_item("option_count", 0)?;
        choice.set_item("min_picks", 0)?;
        choice.set_item("max_picks", 0)?;
        choice.set_item("selected", PyList::empty_bound(py))?;
        choice.set_item("options", PyList::empty_bound(py))?;
    }

    Ok(choice)
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
            let registry = crate::cards::global_registry();
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
        crate::actions::Action::Choose(idx) => ActionInfo {
            id,
            name: format!("choose_{}", idx),
            action_type: "choice".to_string(),
            card_name: String::new(),
            target: -1,
            description: format!("Choose option {}", idx),
        },
        crate::actions::Action::ConfirmSelection => ActionInfo {
            id,
            name: "confirm_selection".to_string(),
            action_type: "choice".to_string(),
            card_name: String::new(),
            target: -1,
            description: "Confirm selection".to_string(),
        },
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
        let (reward, done) = self.run_engine_py.step(action)?;
        self.inner = self.run_engine_py.inner.clone();

        let state_dict = self.build_state_dict(py)?;
        let info = self.build_info_dict(py, reward)?;

        Ok((state_dict, reward, done, info))
    }

    fn step_with_result<'py>(
        &mut self,
        py: Python<'py>,
        action: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        self.run_engine_py.inner = self.inner.clone();
        let result = self.run_engine_py.step_with_result(py, action)?;
        self.inner = self.run_engine_py.inner.clone();
        Ok(result)
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
                    combat.set_item("potions", PyList::new_bound(py, &cs.potions))?;
                    combat.set_item("choice", build_combat_choice_dict(py, ce)?)?;

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
                d.set_item("card_rewards", PyList::new_bound(py, rewards))?;
                if let Some(screen) = self.inner.current_reward_screen() {
                    let screen_dict = PyDict::new_bound(py);
                    screen_dict.set_item("ordered", screen.ordered)?;
                    screen_dict.set_item("active_item", screen.active_item)?;
                    let items = PyList::empty_bound(py);
                    for item in screen.items {
                        let item_dict = PyDict::new_bound(py);
                        item_dict.set_item("index", item.index)?;
                        item_dict.set_item("kind", format!("{:?}", item.kind))?;
                        item_dict.set_item("state", format!("{:?}", item.state))?;
                        item_dict.set_item("label", item.label)?;
                        item_dict.set_item("claimable", item.claimable)?;
                        item_dict.set_item("active", item.active)?;
                        item_dict.set_item("skip_allowed", item.skip_allowed)?;
                        item_dict.set_item("skip_label", item.skip_label)?;
                        let choices = PyList::empty_bound(py);
                        for choice in item.choices {
                            let choice_dict = PyDict::new_bound(py);
                            match choice {
                                crate::decision::RewardChoice::Card { index, card_id } => {
                                    choice_dict.set_item("index", index)?;
                                    choice_dict.set_item("kind", "card")?;
                                    choice_dict.set_item("label", card_id)?;
                                }
                                crate::decision::RewardChoice::Named { index, label } => {
                                    choice_dict.set_item("index", index)?;
                                    choice_dict.set_item("kind", "named")?;
                                    choice_dict.set_item("label", label)?;
                                }
                            }
                            choices.append(choice_dict)?;
                        }
                        item_dict.set_item("choices", choices)?;
                        items.append(item_dict)?;
                    }
                    screen_dict.set_item("items", items)?;
                    d.set_item("reward_screen", screen_dict)?;
                }
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
        if id >= NEOW_BASE {
            let idx = (id - NEOW_BASE) as usize;
            let label = self
                .inner
                .current_decision_context()
                .neow
                .and_then(|neow| {
                    neow.options
                        .get(idx)
                        .map(|option| option.label.clone())
                })
                .unwrap_or_else(|| format!("neow_{}", idx));
            return ActionInfo {
                id,
                name: format!("neow_{}", idx),
                action_type: "neow".to_string(),
                card_name: String::new(),
                target: -1,
                description: label,
            };
        }

        if id >= COMBAT_BASE {
            let combat_id = id - COMBAT_BASE;
            let Some(action) = decode_combat_action_id_in_run(combat_id) else {
                return ActionInfo {
                    id,
                    name: format!("combat_{}", combat_id),
                    action_type: "combat".to_string(),
                    card_name: String::new(),
                    target: -1,
                    description: format!("Unknown combat action {}", combat_id),
                };
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
        } else if id >= REWARD_SKIP_BASE {
            let item_index = (id - REWARD_SKIP_BASE) as usize;
            (
                format!("reward_skip_{}", item_index),
                "card_reward".to_string(),
                format!("Skip reward item {}", item_index),
            )
        } else if id >= REWARD_CHOICE_BASE {
            let encoded = id - REWARD_CHOICE_BASE;
            let item_index = (encoded >> REWARD_ITEM_SHIFT) as usize;
            let choice_index = (encoded & REWARD_INDEX_MASK) as usize;
            let label = self
                .inner
                .current_reward_screen()
                .and_then(|screen| {
                    screen.items.get(item_index).and_then(|item| {
                        item.choices.get(choice_index).map(|choice| match choice {
                            crate::decision::RewardChoice::Card { card_id, .. } => card_id.clone(),
                            crate::decision::RewardChoice::Named { label, .. } => label.clone(),
                        })
                    })
                })
                .unwrap_or_else(|| format!("choice_{}", choice_index));
            (
                format!("reward_choice_{}_{}", item_index, choice_index),
                "card_reward".to_string(),
                format!("Choose {} from reward item {}", label, item_index),
            )
        } else if id >= REWARD_SELECT_BASE {
            let item_index = (id - REWARD_SELECT_BASE) as usize;
            let label = self
                .inner
                .current_reward_screen()
                .and_then(|screen| screen.items.get(item_index).map(|item| item.label.clone()))
                .unwrap_or_else(|| format!("reward_{}", item_index));
            (
                format!("reward_select_{}", item_index),
                "card_reward".to_string(),
                format!("Open or claim {}", label),
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
        run::RunPhase::Neow => "neow",
        run::RunPhase::MapChoice => "map",
        run::RunPhase::Combat => "combat",
        run::RunPhase::CardReward => "card_reward",
        run::RunPhase::Campfire => "campfire",
        run::RunPhase::Shop => "shop",
        run::RunPhase::Event => "event",
        run::RunPhase::GameOver => "game_over",
    }
}

fn decision_kind_str(kind: crate::decision::DecisionKind) -> &'static str {
    match kind {
        crate::decision::DecisionKind::NeowChoice => "neow_choice",
        crate::decision::DecisionKind::CombatAction => "combat_action",
        crate::decision::DecisionKind::CombatChoice => "combat_choice",
        crate::decision::DecisionKind::RewardScreen => "reward_screen",
        crate::decision::DecisionKind::MapPath => "map_path",
        crate::decision::DecisionKind::EventOption => "event_option",
        crate::decision::DecisionKind::ShopAction => "shop_action",
        crate::decision::DecisionKind::CampfireAction => "campfire_action",
        crate::decision::DecisionKind::GameOver => "game_over",
    }
}

fn reward_screen_source_str(source: crate::decision::RewardScreenSource) -> &'static str {
    match source {
        crate::decision::RewardScreenSource::Combat => "combat",
        crate::decision::RewardScreenSource::BossCombat => "boss_combat",
        crate::decision::RewardScreenSource::Event => "event",
        crate::decision::RewardScreenSource::Treasure => "treasure",
        crate::decision::RewardScreenSource::Unknown => "unknown",
    }
}

fn reward_item_state_str(state: crate::decision::RewardItemState) -> &'static str {
    match state {
        crate::decision::RewardItemState::Available => "available",
        crate::decision::RewardItemState::Claimed => "claimed",
        crate::decision::RewardItemState::Skipped => "skipped",
        crate::decision::RewardItemState::Disabled => "disabled",
    }
}

fn reward_item_kind_str(kind: crate::decision::RewardItemKind) -> &'static str {
    match kind {
        crate::decision::RewardItemKind::CardChoice => "card_choice",
        crate::decision::RewardItemKind::Relic => "relic",
        crate::decision::RewardItemKind::Gold => "gold",
        crate::decision::RewardItemKind::Potion => "potion",
        crate::decision::RewardItemKind::Key => "key",
        crate::decision::RewardItemKind::Unknown => "unknown",
    }
}

fn build_reward_choice_dict<'py>(
    py: Python<'py>,
    choice: &crate::decision::RewardChoice,
) -> PyResult<Bound<'py, PyDict>> {
    let choice_dict = PyDict::new_bound(py);
    match choice {
        crate::decision::RewardChoice::Card { index, card_id } => {
            choice_dict.set_item("index", *index)?;
            choice_dict.set_item("kind", "card")?;
            choice_dict.set_item("label", card_id)?;
        }
        crate::decision::RewardChoice::Named { index, label } => {
            choice_dict.set_item("index", *index)?;
            choice_dict.set_item("kind", "named")?;
            choice_dict.set_item("label", label)?;
        }
    }
    Ok(choice_dict)
}

fn build_reward_screen_dict<'py>(
    py: Python<'py>,
    screen: &crate::decision::RewardScreen,
) -> PyResult<Bound<'py, PyDict>> {
    let screen_dict = PyDict::new_bound(py);
    screen_dict.set_item("source", reward_screen_source_str(screen.source.clone()))?;
    screen_dict.set_item("ordered", screen.ordered)?;
    screen_dict.set_item("active_item", screen.active_item)?;
    let items = PyList::empty_bound(py);
    for item in &screen.items {
        let item_dict = PyDict::new_bound(py);
        item_dict.set_item("index", item.index)?;
        item_dict.set_item("kind", reward_item_kind_str(item.kind))?;
        item_dict.set_item("state", reward_item_state_str(item.state))?;
        item_dict.set_item("label", &item.label)?;
        item_dict.set_item("claimable", item.claimable)?;
        item_dict.set_item("active", item.active)?;
        item_dict.set_item("skip_allowed", item.skip_allowed)?;
        item_dict.set_item("skip_label", &item.skip_label)?;
        let choices = PyList::empty_bound(py);
        for choice in &item.choices {
            choices.append(build_reward_choice_dict(py, choice)?)?;
        }
        item_dict.set_item("choices", choices)?;
        items.append(item_dict)?;
    }
    screen_dict.set_item("items", items)?;
    Ok(screen_dict)
}

fn build_decision_state_dict<'py>(
    py: Python<'py>,
    state: &crate::decision::DecisionState,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("kind", decision_kind_str(state.kind))?;
    dict.set_item("phase", phase_str(state.phase))?;
    dict.set_item("terminal", state.terminal)?;
    dict.set_item("room_type", &state.room_type)?;
    Ok(dict)
}

fn build_decision_context_dict<'py>(
    py: Python<'py>,
    context: &crate::decision::DecisionContext,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("kind", decision_kind_str(context.kind))?;

    if let Some(combat) = &context.combat {
        let combat_dict = PyDict::new_bound(py);
        let potions = PyList::empty_bound(py);
        for slot in &combat.potions {
            let slot_dict = PyDict::new_bound(py);
            slot_dict.set_item("slot", slot.slot)?;
            slot_dict.set_item("occupied", slot.occupied)?;
            slot_dict.set_item("potion_id", &slot.potion_id)?;
            slot_dict.set_item("requires_target", slot.requires_target)?;
            potions.append(slot_dict)?;
        }
        combat_dict.set_item("potions", potions)?;

        let choice = PyDict::new_bound(py);
        choice.set_item("active", combat.choice.active)?;
        choice.set_item("reason", &combat.choice.reason)?;
        choice.set_item("option_count", combat.choice.option_count)?;
        choice.set_item("min_picks", combat.choice.min_picks)?;
        choice.set_item("max_picks", combat.choice.max_picks)?;
        choice.set_item("selected", PyList::new_bound(py, &combat.choice.selected))?;
        let options = PyList::empty_bound(py);
        for option in &combat.choice.options {
            let option_dict = PyDict::new_bound(py);
            option_dict.set_item("index", option.index)?;
            option_dict.set_item("kind", &option.kind)?;
            option_dict.set_item("source_index", option.source_index)?;
            option_dict.set_item("label", &option.label)?;
            option_dict.set_item("selected", option.selected)?;
            options.append(option_dict)?;
        }
        choice.set_item("options", options)?;
        combat_dict.set_item("choice", choice)?;
        dict.set_item("combat", combat_dict)?;
    } else {
        dict.set_item("combat", py.None())?;
    }

    if let Some(neow) = &context.neow {
        let neow_dict = PyDict::new_bound(py);
        let options = PyList::empty_bound(py);
        for option in &neow.options {
            let option_dict = PyDict::new_bound(py);
            option_dict.set_item("index", option.index)?;
            option_dict.set_item("label", &option.label)?;
            options.append(option_dict)?;
        }
        neow_dict.set_item("options", options)?;
        dict.set_item("neow", neow_dict)?;
    } else {
        dict.set_item("neow", py.None())?;
    }

    if let Some(screen) = &context.reward_screen {
        dict.set_item("reward_screen", build_reward_screen_dict(py, screen)?)?;
    } else {
        dict.set_item("reward_screen", py.None())?;
    }

    if let Some(map) = &context.map {
        let map_dict = PyDict::new_bound(py);
        map_dict.set_item("available_paths", map.available_paths)?;
        dict.set_item("map", map_dict)?;
    } else {
        dict.set_item("map", py.None())?;
    }

    if let Some(event) = &context.event {
        let event_dict = PyDict::new_bound(py);
        event_dict.set_item("name", &event.name)?;
        let options = PyList::empty_bound(py);
        for option in &event.options {
            let option_dict = PyDict::new_bound(py);
            option_dict.set_item("index", option.index)?;
            option_dict.set_item("label", &option.label)?;
            options.append(option_dict)?;
        }
        event_dict.set_item("options", options)?;
        dict.set_item("event", event_dict)?;
    } else {
        dict.set_item("event", py.None())?;
    }

    if let Some(shop) = &context.shop {
        let shop_dict = PyDict::new_bound(py);
        let offers = PyList::empty_bound(py);
        for offer in &shop.offers {
            let offer_dict = PyDict::new_bound(py);
            offer_dict.set_item("index", offer.index)?;
            offer_dict.set_item("card_id", &offer.card_id)?;
            offer_dict.set_item("price", offer.price)?;
            offer_dict.set_item("affordable", offer.affordable)?;
            offers.append(offer_dict)?;
        }
        shop_dict.set_item("offers", offers)?;
        shop_dict.set_item("remove_price", shop.remove_price)?;
        shop_dict.set_item("removal_used", shop.removal_used)?;
        shop_dict.set_item("removable_cards", shop.removable_cards)?;
        dict.set_item("shop", shop_dict)?;
    } else {
        dict.set_item("shop", py.None())?;
    }

    if let Some(campfire) = &context.campfire {
        let campfire_dict = PyDict::new_bound(py);
        campfire_dict.set_item("can_rest", campfire.can_rest)?;
        campfire_dict.set_item(
            "upgradable_cards",
            PyList::new_bound(py, &campfire.upgradable_cards),
        )?;
        dict.set_item("campfire", campfire_dict)?;
    } else {
        dict.set_item("campfire", py.None())?;
    }

    Ok(dict)
}

// ===========================================================================
// PyO3 RustRunEngine -- full run simulation exposed to Python
// ===========================================================================

/// Run-level action IDs for the flat action space.
const PATH_BASE: i32 = 0;
const REWARD_SELECT_BASE: i32 = 100;
const REWARD_CHOICE_BASE: i32 = 120;
const REWARD_SKIP_BASE: i32 = 180;
const REWARD_ITEM_SHIFT: i32 = 4;
const REWARD_INDEX_MASK: i32 = 0x0f;
const CAMP_REST: i32 = 200;
const CAMP_UPGRADE_BASE: i32 = 201;
const NEOW_BASE: i32 = 1_000_000;
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

    fn step(&mut self, action_id: i32) -> PyResult<(f32, bool)> {
        let action = self.decode_action(action_id).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Unknown action id {action_id}"))
        })?;
        if !self.inner.get_legal_actions().contains(&action) {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Illegal action id {action_id}"
            )));
        }
        Ok(self.inner.step(&action))
    }

    fn step_with_result<'py>(
        &mut self,
        py: Python<'py>,
        action_id: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let action = self.decode_action(action_id).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Unknown action id {action_id}"))
        })?;
        let result = self.inner.step_with_result(&action);
        let dict = PyDict::new_bound(py);
        dict.set_item("action_id", action_id)?;
        dict.set_item("action_accepted", result.action_accepted)?;
        dict.set_item("reward", result.reward)?;
        dict.set_item("reward_delta", result.reward)?;
        dict.set_item("done", result.done)?;
        dict.set_item("terminal", result.done)?;
        dict.set_item("phase", phase_str(result.phase))?;
        dict.set_item(
            "legal_actions",
            result
                .legal_actions
                .iter()
                .map(|action| self.encode_action(action))
                .collect::<Vec<_>>(),
        )?;
        dict.set_item("decision_kind", decision_kind_str(result.decision_state.kind))?;
        dict.set_item("decision_state", build_decision_state_dict(py, &result.decision_state)?)?;
        dict.set_item(
            "decision_context",
            build_decision_context_dict(py, &result.decision_context)?,
        )?;
        dict.set_item(
            "legal_decision_actions",
            result
                .legal_decision_actions
                .iter()
                .map(|action| self.encode_action(&action.to_run_action()))
                .collect::<Vec<_>>(),
        )?;
        Ok(dict)
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

    fn get_combat_obs_v2(&self) -> Vec<f32> {
        obs::encode_combat_state_v2(&self.inner).to_vec()
    }

    fn get_combat_context<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let context = PyDict::new_bound(py);
        if let Some(combat) = self.inner.get_combat_engine() {
            context.set_item("potions", PyList::new_bound(py, &combat.state.potions))?;
            context.set_item("choice", build_combat_choice_dict(py, combat)?)?;
        } else {
            context.set_item("potions", PyList::empty_bound(py))?;
            let empty_choice = PyDict::new_bound(py);
            empty_choice.set_item("active", false)?;
            context.set_item("choice", empty_choice)?;
        }
        Ok(context)
    }

    fn get_last_combat_events(&self) -> Vec<(String, String, Option<String>, Option<String>)> {
        self.inner
            .last_combat_events()
            .iter()
            .map(|record| {
                (
                    format!("{:?}", record.phase),
                    format!("{:?}", record.event),
                    record.owner.map(|owner| format!("{owner:?}")),
                    record.def_id.map(str::to_string),
                )
            })
            .collect()
    }

    fn get_last_combat_event_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let records = PyList::empty_bound(py);
        for record in self.inner.last_combat_events() {
            let item = PyDict::new_bound(py);
            item.set_item("phase", format!("{:?}", record.phase))?;
            item.set_item("event", format!("{:?}", record.event))?;
            item.set_item("owner", record.owner.map(|owner| format!("{owner:?}")))?;
            item.set_item("def_id", record.def_id.map(str::to_string))?;
            item.set_item(
                "execution",
                record.execution.map(|execution| format!("{execution:?}")),
            )?;
            item.set_item("card_type", record.card_type.map(|card_type| format!("{card_type:?}")))?;
            item.set_item("is_first_turn", record.is_first_turn)?;
            item.set_item("target_idx", record.target_idx)?;
            item.set_item("enemy_idx", record.enemy_idx)?;
            item.set_item("potion_slot", record.potion_slot)?;
            item.set_item("status_id", record.status_id.map(|status_id| status_id.0))?;
            item.set_item("amount", record.amount)?;
            records.append(item)?;
        }
        Ok(records)
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

    fn get_decision_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let state = self.inner.current_decision_state();
        build_decision_state_dict(py, &state)
    }

    fn get_decision_context<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let context = self.inner.current_decision_context();
        build_decision_context_dict(py, &context)
    }

    fn get_legal_decision_actions(&self) -> Vec<i32> {
        self.inner
            .get_legal_decision_actions()
            .iter()
            .map(|action| self.encode_action(&action.to_run_action()))
            .collect()
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
            run::RunAction::ChooseNeowOption(i) => NEOW_BASE + *i as i32,
            run::RunAction::ChoosePath(i) => PATH_BASE + *i as i32,
            run::RunAction::SelectRewardItem(i) => REWARD_SELECT_BASE + *i as i32,
            run::RunAction::ChooseRewardOption {
                item_index,
                choice_index,
            } => {
                REWARD_CHOICE_BASE
                    + ((*item_index as i32) << REWARD_ITEM_SHIFT)
                    + (*choice_index as i32)
            }
            run::RunAction::SkipRewardItem(i) => REWARD_SKIP_BASE + *i as i32,
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
                } => COMBAT_BASE
                    + encode_indexed_combat_id(COMBAT_PLAY_BASE_ID, *card_idx, *target_idx),
                crate::actions::Action::UsePotion {
                    potion_idx,
                    target_idx,
                } => COMBAT_BASE
                    + encode_indexed_combat_id(COMBAT_POTION_BASE_ID, *potion_idx, *target_idx),
                crate::actions::Action::ConfirmSelection => COMBAT_BASE + COMBAT_CONFIRM_SELECTION_ID,
                crate::actions::Action::Choose(idx) => COMBAT_BASE + COMBAT_CHOOSE_BASE_ID + *idx as i32,
            },
        }
    }

    pub(crate) fn decode_action(&self, action_id: i32) -> Option<run::RunAction> {
        if action_id >= NEOW_BASE {
            return Some(run::RunAction::ChooseNeowOption(
                (action_id - NEOW_BASE) as usize,
            ));
        }
        if action_id >= COMBAT_BASE {
            let combat_id = action_id - COMBAT_BASE;
            return decode_combat_action_id_in_run(combat_id).map(run::RunAction::CombatAction);
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
        } else if action_id >= REWARD_SKIP_BASE {
            return Some(run::RunAction::SkipRewardItem(
                (action_id - REWARD_SKIP_BASE) as usize,
            ));
        } else if action_id >= REWARD_CHOICE_BASE {
            let encoded = action_id - REWARD_CHOICE_BASE;
            return Some(run::RunAction::ChooseRewardOption {
                item_index: (encoded >> REWARD_ITEM_SHIFT) as usize,
                choice_index: (encoded & REWARD_INDEX_MASK) as usize,
            });
        } else if action_id >= REWARD_SELECT_BASE {
            return Some(run::RunAction::SelectRewardItem(
                (action_id - REWARD_SELECT_BASE) as usize,
            ));
        } else if action_id >= PATH_BASE {
            return Some(run::RunAction::ChoosePath(action_id as usize));
        }

        None
    }
}
