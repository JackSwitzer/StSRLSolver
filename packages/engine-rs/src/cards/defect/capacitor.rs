use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Capacitor: 1 cost, power, gain 2 orb slots
    insert(cards, CardDef {
                id: "Capacitor", name: "Capacitor", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["gain_orb_slots"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Capacitor+", name: "Capacitor+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["gain_orb_slots"], effect_data: &[], complex_hook: None,
            });
}
