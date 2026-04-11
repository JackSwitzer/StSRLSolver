use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Foresight ---- (Java ID: Wireheading, cost 1, power, scry 3 at start of turn; +1 magic upgrade)
    insert(cards, CardDef {
                id: "Wireheading", name: "Foresight", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wireheading+", name: "Foresight+", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
}
