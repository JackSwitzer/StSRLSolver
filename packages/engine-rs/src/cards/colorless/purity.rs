use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Purity: 0 cost, exhaust up to 3 cards from hand, exhaust
    insert(cards, CardDef {
                id: "Purity", name: "Purity", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["exhaust_from_hand"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_exhaust_from_hand),
            });
    insert(cards, CardDef {
                id: "Purity+", name: "Purity+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["exhaust_from_hand"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_exhaust_from_hand),
            });
}
