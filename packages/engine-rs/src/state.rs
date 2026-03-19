//! Combat state types — mirrors packages/engine/state/combat.py.
//!
//! Design: all state is owned, Clone for MCTS tree copies. HashMap<String, i32>
//! for statuses matches the Python dict approach.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Stance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stance {
    Neutral,
    Wrath,
    Calm,
    Divinity,
}

impl Stance {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Wrath" => Stance::Wrath,
            "Calm" => Stance::Calm,
            "Divinity" => Stance::Divinity,
            _ => Stance::Neutral,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Stance::Neutral => "Neutral",
            Stance::Wrath => "Wrath",
            Stance::Calm => "Calm",
            Stance::Divinity => "Divinity",
        }
    }

    /// Outgoing damage multiplier for this stance.
    pub fn outgoing_mult(&self) -> f64 {
        match self {
            Stance::Wrath => 2.0,
            Stance::Divinity => 3.0,
            _ => 1.0,
        }
    }

    /// Incoming damage multiplier for this stance.
    /// Only Wrath doubles incoming damage; Divinity does NOT.
    pub fn incoming_mult(&self) -> f64 {
        match self {
            Stance::Wrath => 2.0,
            _ => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// EntityState — shared between player and enemies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    /// All statuses as a flat map: "Strength" -> 3, "Weakened" -> 2, etc.
    pub statuses: HashMap<String, i32>,
}

impl EntityState {
    pub fn new(hp: i32, max_hp: i32) -> Self {
        Self {
            hp,
            max_hp,
            block: 0,
            statuses: HashMap::new(),
        }
    }

    // -- Convenience accessors (match Python properties) --

    pub fn strength(&self) -> i32 {
        *self.statuses.get("Strength").unwrap_or(&0)
    }

    pub fn dexterity(&self) -> i32 {
        *self.statuses.get("Dexterity").unwrap_or(&0)
    }

    pub fn is_weak(&self) -> bool {
        self.statuses.get("Weakened").copied().unwrap_or(0) > 0
    }

    pub fn is_vulnerable(&self) -> bool {
        self.statuses.get("Vulnerable").copied().unwrap_or(0) > 0
    }

    pub fn is_frail(&self) -> bool {
        self.statuses.get("Frail").copied().unwrap_or(0) > 0
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// Get a status value, defaulting to 0.
    pub fn status(&self, key: &str) -> i32 {
        *self.statuses.get(key).unwrap_or(&0)
    }

    /// Set a status value. Removes the key if value is 0.
    pub fn set_status(&mut self, key: &str, value: i32) {
        if value == 0 {
            self.statuses.remove(key);
        } else {
            self.statuses.insert(key.to_string(), value);
        }
    }

    /// Add to a status value.
    pub fn add_status(&mut self, key: &str, amount: i32) {
        let current = self.status(key);
        self.set_status(key, current + amount);
    }
}

// ---------------------------------------------------------------------------
// EnemyCombatState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyCombatState {
    pub entity: EntityState,
    pub id: String,
    pub name: String,
    /// Current intended move
    pub move_id: i32,
    pub move_damage: i32,
    pub move_hits: i32,
    pub move_block: i32,
    /// Simple effects map: "weak" -> 2, "vulnerable" -> 1, etc.
    pub move_effects: HashMap<String, i32>,
    pub move_history: Vec<i32>,
    pub first_turn: bool,
    pub is_escaping: bool,
}

impl EnemyCombatState {
    pub fn new(id: &str, hp: i32, max_hp: i32) -> Self {
        Self {
            entity: EntityState::new(hp, max_hp),
            id: id.to_string(),
            name: id.to_string(),
            move_id: -1,
            move_damage: 0,
            move_hits: 1,
            move_block: 0,
            move_effects: HashMap::new(),
            move_history: Vec::new(),
            first_turn: true,
            is_escaping: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.entity.hp > 0 && !self.is_escaping
    }

    pub fn is_attacking(&self) -> bool {
        self.move_damage > 0
    }

    pub fn total_incoming_damage(&self) -> i32 {
        self.move_damage * self.move_hits
    }

    /// Set the enemy's next move.
    pub fn set_move(&mut self, move_id: i32, damage: i32, hits: i32, block: i32) {
        self.move_id = move_id;
        self.move_damage = damage;
        self.move_hits = hits;
        self.move_block = block;
        self.move_effects.clear();
    }
}

// ---------------------------------------------------------------------------
// CombatState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    // Player
    pub player: EntityState,
    pub energy: i32,
    pub max_energy: i32,
    pub stance: Stance,

    // Card piles (card IDs as strings, "Strike+" means upgraded)
    pub hand: Vec<String>,
    pub draw_pile: Vec<String>,
    pub discard_pile: Vec<String>,
    pub exhaust_pile: Vec<String>,

    // Enemies
    pub enemies: Vec<EnemyCombatState>,

    // Potions
    pub potions: Vec<String>,

    // Combat tracking
    pub turn: i32,
    pub cards_played_this_turn: i32,
    pub attacks_played_this_turn: i32,
    pub combat_over: bool,
    pub player_won: bool,

    // Watcher-specific
    pub mantra: i32,

    // Statistics
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub total_cards_played: i32,

    // Relics (just IDs for checking effects)
    pub relics: Vec<String>,

    // Card effect tracking
    /// Type of the last card played this turn (for Follow-Up, Sash Whip, etc.)
    pub last_card_type: Option<crate::cards::CardType>,
    /// Total mantra gained this combat (for Brilliance)
    pub total_mantra_gained: i32,
    /// Wreath of Flame bonus: extra damage on next attack
    pub wreath_of_flame_bonus: i32,
    /// Blasphemy: player dies at start of next turn
    pub die_next_turn: bool,
    /// Powers applied this combat (id -> amount)
    /// Used for Mental Fortress, Rushdown, Like Water, etc.
    pub powers_applied: std::collections::HashMap<String, i32>,
    /// Deva Form energy stacking counter
    pub deva_form_energy: i32,
}

impl CombatState {
    /// Create a new combat state with initial setup.
    pub fn new(
        player_hp: i32,
        player_max_hp: i32,
        enemies: Vec<EnemyCombatState>,
        deck: Vec<String>,
        energy: i32,
    ) -> Self {
        Self {
            player: EntityState::new(player_hp, player_max_hp),
            energy,
            max_energy: energy,
            stance: Stance::Neutral,
            hand: Vec::new(),
            draw_pile: deck,
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            enemies,
            potions: vec!["".to_string(); 3],
            turn: 0,
            cards_played_this_turn: 0,
            attacks_played_this_turn: 0,
            combat_over: false,
            player_won: false,
            mantra: 0,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            total_cards_played: 0,
            relics: Vec::new(),
            last_card_type: None,
            total_mantra_gained: 0,
            wreath_of_flame_bonus: 0,
            die_next_turn: false,
            powers_applied: HashMap::new(),
            deva_form_energy: 0,
        }
    }

    pub fn is_victory(&self) -> bool {
        self.enemies.iter().all(|e| e.entity.is_dead())
    }

    pub fn is_defeat(&self) -> bool {
        self.player.is_dead()
    }

    pub fn is_terminal(&self) -> bool {
        self.is_victory() || self.is_defeat()
    }

    pub fn living_enemy_indices(&self) -> Vec<usize> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_alive())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn has_relic(&self, relic_id: &str) -> bool {
        self.relics.iter().any(|r| r == relic_id)
    }
}

// ---------------------------------------------------------------------------
// PyO3 wrapper for CombatState — returned to Python as a dict
// ---------------------------------------------------------------------------

#[pyclass(name = "CombatState")]
#[derive(Clone)]
pub struct PyCombatState {
    pub inner: CombatState,
}

#[pymethods]
impl PyCombatState {
    /// Convert the state to a Python dict for inspection.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("player_hp", self.inner.player.hp)?;
        dict.set_item("player_max_hp", self.inner.player.max_hp)?;
        dict.set_item("player_block", self.inner.player.block)?;
        dict.set_item("energy", self.inner.energy)?;
        dict.set_item("max_energy", self.inner.max_energy)?;
        dict.set_item("stance", self.inner.stance.as_str())?;
        dict.set_item("turn", self.inner.turn)?;
        dict.set_item("combat_over", self.inner.combat_over)?;
        dict.set_item("player_won", self.inner.player_won)?;

        // Hand
        let hand: Vec<&str> = self.inner.hand.iter().map(|s| s.as_str()).collect();
        dict.set_item("hand", hand)?;

        // Draw/discard sizes
        dict.set_item("draw_pile_size", self.inner.draw_pile.len())?;
        dict.set_item("discard_pile_size", self.inner.discard_pile.len())?;
        dict.set_item("exhaust_pile_size", self.inner.exhaust_pile.len())?;

        // Enemies
        let enemies: Vec<_> = self
            .inner
            .enemies
            .iter()
            .map(|e| {
                format!(
                    "{}(hp={}/{}, move_dmg={}, move_hits={})",
                    e.id, e.entity.hp, e.entity.max_hp, e.move_damage, e.move_hits
                )
            })
            .collect();
        dict.set_item("enemies", enemies)?;

        // Player statuses
        let statuses = PyDict::new_bound(py);
        for (k, v) in &self.inner.player.statuses {
            statuses.set_item(k.as_str(), *v)?;
        }
        dict.set_item("player_statuses", statuses)?;

        // Stats
        dict.set_item("total_damage_dealt", self.inner.total_damage_dealt)?;
        dict.set_item("total_damage_taken", self.inner.total_damage_taken)?;
        dict.set_item("total_cards_played", self.inner.total_cards_played)?;

        Ok(dict)
    }

    fn __repr__(&self) -> String {
        format!(
            "CombatState(hp={}/{}, energy={}, turn={}, hand={}, enemies={})",
            self.inner.player.hp,
            self.inner.player.max_hp,
            self.inner.energy,
            self.inner.turn,
            self.inner.hand.len(),
            self.inner.enemies.len(),
        )
    }
}
