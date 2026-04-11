use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Thinking Ahead: 0 cost, draw 2, put 1 card from hand on top of draw, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Thinking Ahead", name: "Thinking Ahead", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["thinking_ahead"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Thinking Ahead+", name: "Thinking Ahead+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["thinking_ahead"], effect_data: &[], complex_hook: None,
            });
}
