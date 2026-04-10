use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_status(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Universal Status/Curse Cards ----
        insert(cards, CardDef {
            id: "Slimed", name: "Slimed", card_type: CardType::Status,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wound", name: "Wound", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &["unplayable"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Daze", name: "Daze", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Burn", name: "Burn", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Burn+", name: "Burn+", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"], effect_data: &[], complex_hook: None,
        });

        // Burn+: unplayable, 4 end-of-turn damage (upgraded from 2)
        insert(cards, CardDef {
            id: "Burn+", name: "Burn+", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"], effect_data: &[], complex_hook: None,
        });
        // Void: unplayable, ethereal, lose 1 energy on draw
        insert(cards, CardDef {
            id: "Void", name: "Void", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal", "lose_energy_on_draw"], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
