use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Impatience: 0 cost, draw 2 if no attacks in hand
    insert(cards, CardDef {
                id: "Impatience", name: "Impatience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["draw_if_no_attacks"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Impatience+", name: "Impatience+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["draw_if_no_attacks"], effect_data: &[], complex_hook: None,
            });
}
