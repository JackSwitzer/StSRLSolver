use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Basic: Survivor ---- (cost 1, 8 block, discard 1; +3 block)
    insert(cards, CardDef {
                id: "Survivor", name: "Survivor", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discard"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Survivor+", name: "Survivor+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discard"], effect_data: &[], complex_hook: None,
            });
}
