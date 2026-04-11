use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Unraveling ---- (cost 2, skill, exhaust, play all cards in hand for free; upgrade: cost 1)
    insert(cards, CardDef {
                id: "Unraveling", name: "Unraveling", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Unraveling+", name: "Unraveling+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
}
