use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Mayhem: 2 cost, power, auto-play top card of draw pile each turn (upgrade: cost 1)
    insert(cards, CardDef {
                id: "Mayhem", name: "Mayhem", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["mayhem"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Mayhem+", name: "Mayhem+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["mayhem"], effect_data: &[], complex_hook: None,
            });
}
