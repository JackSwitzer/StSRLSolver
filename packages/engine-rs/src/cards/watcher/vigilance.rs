use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Vigilance", name: "Vigilance", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Vigilance+", name: "Vigilance+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
                base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[], effect_data: &[], complex_hook: None,
            });
}
