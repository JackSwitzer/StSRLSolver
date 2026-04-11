use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Escape Plan ---- (cost 0, draw 1, if skill gain 3 block; +2 block)
    insert(cards, CardDef {
                id: "Escape Plan", name: "Escape Plan", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["draw", "block_if_skill"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Escape Plan+", name: "Escape Plan+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["draw", "block_if_skill"], effect_data: &[], complex_hook: None,
            });
}
