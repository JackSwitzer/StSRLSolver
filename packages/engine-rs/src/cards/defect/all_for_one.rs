use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Defect Rare Cards ----
        // All For One: 2 cost, 10 dmg, return all 0-cost cards from discard to hand
    insert(cards, CardDef {
                id: "All For One", name: "All For One", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_zero_cost_from_discard"], effect_data: &[
                    E::ForEachInPile { pile: P::Discard, filter: CardFilter::ZeroCost, action: BulkAction::MoveToHand },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "All For One+", name: "All For One+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_zero_cost_from_discard"], effect_data: &[
                    E::ForEachInPile { pile: P::Discard, filter: CardFilter::ZeroCost, action: BulkAction::MoveToHand },
                ], complex_hook: None,
            });
}
