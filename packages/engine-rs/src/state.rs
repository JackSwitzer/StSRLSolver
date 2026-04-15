//! Combat state types — mirrors packages/engine/state/combat.py.
//!
//! Design: all state is owned, Clone for MCTS tree copies. Statuses use a flat
//! [i16; 256] array indexed by StatusId for O(1) access and fast cloning.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::cards::CardType;
use crate::combat_types::{CardInstance, Intent};
use crate::ids::StatusId;
use crate::orbs::OrbSlots;
use crate::status_ids::sid;

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

#[derive(Debug, Clone)]
pub struct EntityState {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    /// All statuses as a flat array indexed by StatusId. Zero means absent.
    pub statuses: [i16; 256],
}

impl EntityState {
    pub fn new(hp: i32, max_hp: i32) -> Self {
        Self {
            hp,
            max_hp,
            block: 0,
            statuses: [0; 256],
        }
    }

    // -- Convenience accessors (match Python properties) --

    pub fn strength(&self) -> i32 {
        self.statuses[sid::STRENGTH.0 as usize] as i32
    }

    pub fn dexterity(&self) -> i32 {
        self.statuses[sid::DEXTERITY.0 as usize] as i32
    }

    pub fn focus(&self) -> i32 {
        self.statuses[sid::FOCUS.0 as usize] as i32
    }

    pub fn is_weak(&self) -> bool {
        self.statuses[sid::WEAKENED.0 as usize] > 0
    }

    pub fn is_vulnerable(&self) -> bool {
        self.statuses[sid::VULNERABLE.0 as usize] > 0
    }

    pub fn is_frail(&self) -> bool {
        self.statuses[sid::FRAIL.0 as usize] > 0
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// Get a status value, defaulting to 0.
    pub fn status(&self, id: StatusId) -> i32 {
        self.statuses[id.0 as usize] as i32
    }

    /// Set a status value.
    pub fn set_status(&mut self, id: StatusId, value: i32) {
        self.statuses[id.0 as usize] = value as i16;
    }

    /// Add to a status value.
    pub fn add_status(&mut self, id: StatusId, amount: i32) {
        let idx = id.0 as usize;
        self.statuses[idx] = (self.statuses[idx] as i32 + amount) as i16;
    }
}

// ---------------------------------------------------------------------------
// EnemyCombatState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EnemyCombatState {
    pub entity: EntityState,
    pub id: String,
    pub name: String,
    /// Java BackAttack legality bit for Smoke Bomb and similar checks.
    pub back_attack: bool,
    /// Current intended move
    pub move_id: i32,
    pub intent: Intent,
    /// Compact move effects: (MoveEffectId, amount)
    pub move_effects: SmallVec<[(u8, i16); 4]>,
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
            back_attack: false,
            move_id: -1,
            intent: Intent::Unknown,
            move_effects: SmallVec::new(),
            move_history: Vec::new(),
            first_turn: true,
            is_escaping: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        // RebirthPending enemies are "alive" for enemy turn processing
        (self.entity.hp > 0 || self.entity.status(sid::REBIRTH_PENDING) > 0) && !self.is_escaping
    }

    /// Returns true if this enemy can be targeted by the player (alive and not mid-rebirth).
    pub fn is_targetable(&self) -> bool {
        self.entity.hp > 0 && !self.is_escaping && self.entity.status(sid::REBIRTH_PENDING) == 0
    }

    pub fn has_back_attack(&self) -> bool {
        self.back_attack
    }

    pub fn is_attacking(&self) -> bool {
        matches!(self.intent,
            Intent::Attack { .. } | Intent::AttackBlock { .. } |
            Intent::AttackBuff { .. } | Intent::AttackDebuff { .. })
    }

    pub fn total_incoming_damage(&self) -> i32 {
        match self.intent {
            Intent::Attack { damage, hits, .. } |
            Intent::AttackBlock { damage, hits, .. } |
            Intent::AttackBuff { damage, hits, .. } |
            Intent::AttackDebuff { damage, hits, .. } => {
                damage as i32 * hits as i32
            }
            _ => 0,
        }
    }

    pub fn move_damage(&self) -> i32 {
        match self.intent {
            Intent::Attack { damage, .. } |
            Intent::AttackBlock { damage, .. } |
            Intent::AttackBuff { damage, .. } |
            Intent::AttackDebuff { damage, hits: _, .. } => damage as i32,
            _ => 0,
        }
    }

    pub fn move_hits(&self) -> i32 {
        match self.intent {
            Intent::Attack { hits, .. } |
            Intent::AttackBlock { hits, .. } |
            Intent::AttackBuff { hits, .. } |
            Intent::AttackDebuff { hits, .. } => hits as i32,
            _ => 0,
        }
    }

    pub fn move_block(&self) -> i32 {
        match self.intent {
            Intent::Block { amount, .. } |
            Intent::AttackBlock { block: amount, .. } |
            Intent::DefendBuff { block: amount, .. } => amount as i32,
            _ => 0,
        }
    }

    /// Set the enemy's next move (clears effects).
    pub fn set_move(&mut self, move_id: i32, damage: i32, hits: i32, block: i32) {
        self.move_id = move_id;
        self.move_effects.clear();
        // Convert to Intent based on damage/block
        if damage > 0 && block > 0 {
            self.intent = Intent::AttackBlock {
                damage: damage as i16, hits: hits as u8, block: block as i16, effects: 0
            };
        } else if damage > 0 {
            self.intent = Intent::Attack {
                damage: damage as i16, hits: hits as u8, effects: 0
            };
        } else if block > 0 {
            self.intent = Intent::Block { amount: block as i16, effects: 0 };
        } else {
            self.intent = Intent::Buff { effects: 0 };
        }
    }

    /// Add a move effect (replaces HashMap insert).
    pub fn add_effect(&mut self, effect_id: u8, amount: i16) {
        for entry in self.move_effects.iter_mut() {
            if entry.0 == effect_id {
                entry.1 = amount;
                return;
            }
        }
        self.move_effects.push((effect_id, amount));
    }

    /// Get a move effect amount (replaces HashMap get).
    pub fn effect(&self, effect_id: u8) -> Option<i16> {
        self.move_effects.iter()
            .find(|e| e.0 == effect_id)
            .map(|e| e.1)
    }
}

// ---------------------------------------------------------------------------
// CombatState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CombatState {
    // Player
    pub player: EntityState,
    pub energy: i32,
    pub max_energy: i32,
    pub stance: Stance,

    // Card piles (compact CardInstance, 4 bytes each)
    pub hand: Vec<CardInstance>,
    pub draw_pile: Vec<CardInstance>,
    pub discard_pile: Vec<CardInstance>,
    pub exhaust_pile: Vec<CardInstance>,

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
    /// Total mantra gained this combat (for Brilliance)
    pub mantra_gained: i32,
    /// Last card type played this turn (for CrushJoints/FollowUp checks)
    pub last_card_type: Option<CardType>,
    /// Skip enemy turn flag (Vault)
    pub skip_enemy_turn: bool,
    /// Blasphemy: die at start of next turn
    pub blasphemy_active: bool,

    // Statistics
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub total_cards_played: i32,
    /// Gold earned during combat that should be synced back to RunState on resolution.
    pub pending_run_gold: i32,

    // Relics (just IDs for checking effects)
    pub relics: Vec<String>,

    /// Cards explicitly retained this turn (e.g. by Meditate).
    /// Now tracked via FLAG_RETAINED on CardInstance; this Vec is kept for Establishment cost tracking.
    pub retained_cards: Vec<CardInstance>,

    /// Orb slots (Defect mechanic, also available for cross-character mods).
    pub orb_slots: OrbSlots,

    /// Cross-combat relic counters (Nunchaku, Incense Burner, Ink Bottle, Happy Flower, etc.)
    /// Indexed by relic_flags::counter::* constants. Synced from/to RunState.relic_flags.
    pub relic_counters: [i16; 8],

}

impl CombatState {
    /// Create a new combat state with initial setup.
    pub fn new(
        player_hp: i32,
        player_max_hp: i32,
        enemies: Vec<EnemyCombatState>,
        deck: Vec<CardInstance>,
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
            mantra_gained: 0,
            last_card_type: None,
            skip_enemy_turn: false,
            blasphemy_active: false,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            total_cards_played: 0,
            pending_run_gold: 0,
            relics: Vec::new(),
            retained_cards: Vec::new(),
            orb_slots: OrbSlots::new(0), // 0 slots by default (Watcher has no orbs)
            relic_counters: [0i16; 8],
        }
    }

    pub fn is_victory(&self) -> bool {
        self.enemies.iter().all(|e| e.entity.is_dead() && e.entity.status(sid::REBIRTH_PENDING) == 0)
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

    /// Indices of enemies that can be targeted by the player (alive and not mid-rebirth).
    pub fn targetable_enemy_indices(&self) -> Vec<usize> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_targetable())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn has_relic(&self, relic_id: &str) -> bool {
        self.relics.iter().any(|r| r == relic_id)
    }

    /// Centralized healing: checks Mark of the Bloom (blocks) and Magic Flower (1.5x).
    pub fn heal_player(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        if self.player.status(crate::status_ids::sid::HAS_MARK_OF_BLOOM) > 0 {
            return;
        }
        let mut heal = amount;
        if self.player.status(crate::status_ids::sid::HAS_MAGIC_FLOWER) > 0 {
            heal = (heal as f64 * 1.5) as i32;
        }
        self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
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
        let registry = crate::cards::global_registry();
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
        let hand: Vec<String> = self.inner.hand.iter()
            .map(|c| registry.card_name(c.def_id).to_string())
            .collect();
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
                    e.id, e.entity.hp, e.entity.max_hp, e.move_damage(), e.move_hits()
                )
            })
            .collect();
        dict.set_item("enemies", enemies)?;

        // Player statuses
        let statuses = PyDict::new_bound(py);
        for (i, &val) in self.inner.player.statuses.iter().enumerate() {
            if val != 0 {
                let name = crate::status_ids::status_name(crate::ids::StatusId(i as u16));
                statuses.set_item(name, val as i32)?;
            }
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
