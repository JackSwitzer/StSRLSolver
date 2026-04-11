use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Omega (from Beta chain): cost 3, power, deal 50 dmg at end of turn
    insert(cards, CardDef {
                id: "Omega", name: "Omega", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 50, exhaust: false, enter_stance: None,
                effects: &["omega"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Omega+", name: "Omega+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 60, exhaust: false, enter_stance: None,
                effects: &["omega"], effect_data: &[], complex_hook: None,
            });
}
