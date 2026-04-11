use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Expertise ---- (cost 1, draw to 6 cards; upgrade: draw to 7)
    insert(cards, CardDef {
                id: "Expertise", name: "Expertise", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effects: &["draw_to_n"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Expertise+", name: "Expertise+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effects: &["draw_to_n"], effect_data: &[], complex_hook: None,
            });
}
