use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Biased Cognition: 1 cost, power, gain 4 focus, lose 1 focus each turn
    insert(cards, CardDef {
                id: "Biased Cognition", name: "Biased Cognition", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["gain_focus", "lose_focus_each_turn"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Biased Cognition+", name: "Biased Cognition+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["gain_focus", "lose_focus_each_turn"], effect_data: &[], complex_hook: None,
            });
}
