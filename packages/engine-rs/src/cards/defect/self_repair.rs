use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Self Repair: 1 cost, power, heal 7 HP at end of combat
    insert(cards, CardDef {
                id: "Self Repair", name: "Self Repair", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effects: &["heal_end_of_combat"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Self Repair+", name: "Self Repair+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 10, exhaust: false, enter_stance: None,
                effects: &["heal_end_of_combat"], effect_data: &[], complex_hook: None,
            });
}
