use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Auto Shields: 1 cost, 11 block only if no block
    insert(cards, CardDef {
                id: "Auto Shields", name: "Auto-Shields", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_if_no_block"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Auto Shields+", name: "Auto-Shields+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_if_no_block"], effect_data: &[], complex_hook: None,
            });
}
