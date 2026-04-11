use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Conclude", name: "Conclude", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["end_turn"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Conclude+", name: "Conclude+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["end_turn"], effect_data: &[], complex_hook: None,
            });
}
