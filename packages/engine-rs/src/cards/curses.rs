use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_curses(cards: &mut HashMap<&'static str, CardDef>) {
        insert(cards, CardDef {
            id: "Decay", name: "Decay", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"],
        });
        insert(cards, CardDef {
            id: "Regret", name: "Regret", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_regret"],
        });
        insert(cards, CardDef {
            id: "Doubt", name: "Doubt", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_weak"],
        });
        insert(cards, CardDef {
            id: "Shame", name: "Shame", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_frail"],
        });
        insert(cards, CardDef {
            id: "AscendersBane", name: "Ascender's Bane", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"],
        });

        // AscendersBane already registered above

        // Clumsy: unplayable, ethereal
        insert(cards, CardDef {
            id: "Clumsy", name: "Clumsy", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"],
        });
        // CurseOfTheBell: unplayable, cannot be removed
        insert(cards, CardDef {
            id: "CurseOfTheBell", name: "Curse of the Bell", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable"],
        });
        // Decay: unplayable, deal 2 dmg to player at end of turn
        insert(cards, CardDef {
            id: "Decay", name: "Decay", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"],
        });
        // Doubt: unplayable, apply 1 Weak at end of turn
        insert(cards, CardDef {
            id: "Doubt", name: "Doubt", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_weak"],
        });
        // Injury: unplayable
        insert(cards, CardDef {
            id: "Injury", name: "Injury", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable"],
        });
        // Necronomicurse: unplayable, cannot be removed
        insert(cards, CardDef {
            id: "Necronomicurse", name: "Necronomicurse", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "unremovable"],
        });
        // Normality: unplayable, can only play 3 cards per turn
        insert(cards, CardDef {
            id: "Normality", name: "Normality", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "limit_cards_per_turn"],
        });
        // Pain: unplayable, lose 1 HP when played from hand
        insert(cards, CardDef {
            id: "Pain", name: "Pain", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "damage_on_draw"],
        });
        // Parasite: unplayable, lose 3 max HP if removed
        insert(cards, CardDef {
            id: "Parasite", name: "Parasite", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "lose_max_hp_on_remove"],
        });
        // Pride: 1 cost, exhaust, innate, add copy to draw pile at end of turn
        insert(cards, CardDef {
            id: "Pride", name: "Pride", card_type: CardType::Curse,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate", "add_copy_end_turn"],
        });
        // Regret: unplayable, lose HP equal to cards in hand at end of turn
        insert(cards, CardDef {
            id: "Regret", name: "Regret", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_hp_loss_per_card"],
        });
        // Shame: unplayable, apply 1 Frail at end of turn
        insert(cards, CardDef {
            id: "Shame", name: "Shame", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_frail"],
        });
        // Writhe: unplayable, innate
        insert(cards, CardDef {
            id: "Writhe", name: "Writhe", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "innate"],
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
