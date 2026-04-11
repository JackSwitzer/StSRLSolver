use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Discovery: 1 cost, choose 1 of 3 cards to add to hand, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Discovery", name: "Discovery", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["discovery"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Discovery+", name: "Discovery+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discovery"], effect_data: &[], complex_hook: None,
            });
}
