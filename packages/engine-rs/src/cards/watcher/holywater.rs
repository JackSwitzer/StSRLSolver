use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Holy Water: 0 cost, 5 block, retain, exhaust (from HolyWater relic)
    insert(cards, CardDef {
                id: "HolyWater", name: "HolyWater", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "HolyWater+", name: "HolyWater+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
}
