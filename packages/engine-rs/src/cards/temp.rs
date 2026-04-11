use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_temp(cards: &mut HashMap<&'static str, CardDef>) {
        // Beta: 2 cost, shuffle Omega into draw pile, exhaust (upgrade: cost 1)
        insert(cards, CardDef {
            id: "Beta", name: "Beta", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Beta+", name: "Beta+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &[], effect_data: &[], complex_hook: None,
        });
        // Omega: 3 cost, power, deal 50 dmg to all enemies at end of each turn
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
        // Expunger: 1 cost, 9 dmg x magic (from Conjure Blade)
        insert(cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        // Insight: 0 cost, draw 2, retain, exhaust
        insert(cards, CardDef {
            id: "Insight", name: "Insight", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Insight+", name: "Insight+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"], effect_data: &[], complex_hook: None,
        });
        // Safety: 1 cost, 12 block, retain, exhaust
        insert(cards, CardDef {
            id: "Safety", name: "Safety", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Safety+", name: "Safety+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        // Through Violence: 0 cost, 20 dmg, retain, exhaust
        insert(cards, CardDef {
            id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        // Shiv: 0 cost, 4 dmg, exhaust
        insert(cards, CardDef {
            id: "Shiv", name: "Shiv", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
