use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Recycle: 1 cost, exhaust a card, gain energy equal to its cost (upgrade: cost 0)
    insert(cards, CardDef {
                id: "Recycle", name: "Recycle", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["recycle"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Recycle+", name: "Recycle+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["recycle"], effect_data: &[], complex_hook: None,
            });
}
