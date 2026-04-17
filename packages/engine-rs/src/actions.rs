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

    /// Pick an option during AwaitingChoice phase. Index into ChoiceContext.options.
    Choose(usize),

    /// Finalize a multi-select choice (Scry, Gambling Chip).
    ConfirmSelection,
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
            Action::Choose(idx) => format!("Choose({})", idx),
            Action::ConfirmSelection => "ConfirmSelection".to_string(),
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

    /// Create a Choose action.
    #[staticmethod]
    fn choose(idx: usize) -> Self {
        PyAction {
            inner: Action::Choose(idx),
        }
    }

    /// Create a ConfirmSelection action.
    #[staticmethod]
    fn confirm_selection() -> Self {
        PyAction {
            inner: Action::ConfirmSelection,
        }
    }

    /// Get the action type as a string.
    #[getter]
    fn action_type(&self) -> &str {
        match &self.inner {
            Action::PlayCard { .. } => "PlayCard",
            Action::UsePotion { .. } => "UsePotion",
            Action::EndTurn => "EndTurn",
            Action::Choose(_) => "Choose",
            Action::ConfirmSelection => "ConfirmSelection",
        }
    }

    /// For PlayCard/UsePotion: the card/potion index. For Choose: the choice index.
    #[getter]
    fn index(&self) -> Option<usize> {
        match &self.inner {
            Action::PlayCard { card_idx, .. } => Some(*card_idx),
            Action::UsePotion { potion_idx, .. } => Some(*potion_idx),
            Action::Choose(idx) => Some(*idx),
            _ => None,
        }
    }

    /// For PlayCard/UsePotion: the target index. Returns None for others.
    #[getter]
    fn target(&self) -> Option<i32> {
        match &self.inner {
            Action::PlayCard { target_idx, .. } => Some(*target_idx),
            Action::UsePotion { target_idx, .. } => Some(*target_idx),
            _ => None,
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
