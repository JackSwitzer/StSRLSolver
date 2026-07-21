//! Canonical combat actions used by the simulator core.

use serde::{Deserialize, Serialize};

/// A canonical combat action.
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
