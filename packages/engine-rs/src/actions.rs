//! Action types — mirrors packages/engine/state/combat.py Action union.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// A combat action. Mirrors the Python Union[PlayCard, UsePotion, EndTurn].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a card from hand. card_idx is index into hand, target_idx is enemy
    /// index or -1 for self/no-target.
    PlayCard { card_idx: usize, target_idx: i32 },

    /// Use a potion. potion_idx is slot index, target_idx is enemy index or -1.
    UsePotion { potion_idx: usize, target_idx: i32 },

    /// End the player's turn.
    EndTurn,
}

impl Action {
    /// Human-readable description for debugging.
    pub fn describe(&self) -> String {
        match self {
            Action::PlayCard {
                card_idx,
                target_idx,
            } => {
                if *target_idx >= 0 {
                    format!("PlayCard(hand[{}] -> enemy[{}])", card_idx, target_idx)
                } else {
                    format!("PlayCard(hand[{}])", card_idx)
                }
            }
            Action::UsePotion {
                potion_idx,
                target_idx,
            } => {
                if *target_idx >= 0 {
                    format!("UsePotion(slot[{}] -> enemy[{}])", potion_idx, target_idx)
                } else {
                    format!("UsePotion(slot[{}])", potion_idx)
                }
            }
            Action::EndTurn => "EndTurn".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// PyO3 wrapper — so Python can receive and inspect actions
// ---------------------------------------------------------------------------

#[pyclass(name = "Action")]
#[derive(Clone)]
pub struct PyAction {
    pub inner: Action,
}

#[pymethods]
impl PyAction {
    /// Create a PlayCard action.
    #[staticmethod]
    fn play_card(card_idx: usize, target_idx: i32) -> Self {
        PyAction {
            inner: Action::PlayCard {
                card_idx,
                target_idx,
            },
        }
    }

    /// Create a UsePotion action.
    #[staticmethod]
    fn use_potion(potion_idx: usize, target_idx: i32) -> Self {
        PyAction {
            inner: Action::UsePotion {
                potion_idx,
                target_idx,
            },
        }
    }

    /// Create an EndTurn action.
    #[staticmethod]
    fn end_turn() -> Self {
        PyAction {
            inner: Action::EndTurn,
        }
    }

    /// Get the action type as a string: "PlayCard", "UsePotion", or "EndTurn".
    #[getter]
    fn action_type(&self) -> &str {
        match &self.inner {
            Action::PlayCard { .. } => "PlayCard",
            Action::UsePotion { .. } => "UsePotion",
            Action::EndTurn => "EndTurn",
        }
    }

    /// For PlayCard/UsePotion: the card/potion index. Returns None for EndTurn.
    #[getter]
    fn index(&self) -> Option<usize> {
        match &self.inner {
            Action::PlayCard { card_idx, .. } => Some(*card_idx),
            Action::UsePotion { potion_idx, .. } => Some(*potion_idx),
            Action::EndTurn => None,
        }
    }

    /// For PlayCard/UsePotion: the target index. Returns None for EndTurn.
    #[getter]
    fn target(&self) -> Option<i32> {
        match &self.inner {
            Action::PlayCard { target_idx, .. } => Some(*target_idx),
            Action::UsePotion { target_idx, .. } => Some(*target_idx),
            Action::EndTurn => None,
        }
    }

    fn __repr__(&self) -> String {
        self.inner.describe()
    }

    fn __eq__(&self, other: &PyAction) -> bool {
        self.inner == other.inner
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }
}
