use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Dash ---- (cost 2, 10 dmg + 10 block; +3/+3)
    insert(cards, CardDef {
                id: "Dash", name: "Dash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: 10,
                base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dash+", name: "Dash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: 13,
                base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
}
