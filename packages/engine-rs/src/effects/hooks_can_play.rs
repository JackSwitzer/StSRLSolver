//! can_play hooks — determine whether a card can be played.

use crate::cards::{CardDef, CardType, CardRegistry};
use crate::combat_types::CardInstance;
use crate::state::CombatState;

/// Unplayable cards (Reflex, Tactician, etc.) — unless relic override.
/// Medical Kit makes Status cards playable; Blue Candle makes Curse cards playable.
pub fn hook_unplayable(state: &CombatState, card: &CardDef, _card_inst: CardInstance, _registry: &CardRegistry) -> bool {
    if card.card_type == CardType::Status
        && (state.has_relic("Medical Kit") || state.has_relic("MedicalKit"))
    {
        return true;
    }
    if card.card_type == CardType::Curse
        && (state.has_relic("Blue Candle") || state.has_relic("BlueCandle"))
    {
        return true;
    }
    false
}

/// Signature Move: only playable if no other Attack cards in hand.
pub fn hook_only_attack_in_hand(state: &CombatState, _card: &CardDef, card_inst: CardInstance, registry: &CardRegistry) -> bool {
    !state.hand.iter().any(|c| {
        let other_card = registry.card_def_by_id(c.def_id);
        other_card.card_type == CardType::Attack && c.def_id != card_inst.def_id
    })
}

/// Clash: only playable if all cards in hand are Attacks.
pub fn hook_only_attacks_in_hand(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, registry: &CardRegistry) -> bool {
    !state.hand.iter().any(|c| {
        let other_card = registry.card_def_by_id(c.def_id);
        other_card.card_type != CardType::Attack
    })
}

/// Grand Finale: only playable if draw pile is empty.
pub fn hook_only_empty_draw(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, _registry: &CardRegistry) -> bool {
    state.draw_pile.is_empty()
}
