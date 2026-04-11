use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "FlurryOfBlows", name: "Flurry of Blows", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "FlurryOfBlows+", name: "Flurry of Blows+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
}
