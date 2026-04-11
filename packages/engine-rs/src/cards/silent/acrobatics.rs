use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Acrobatics ---- (cost 1, draw 3, discard 1; +1 draw)
    insert(cards, CardDef {
                id: "Acrobatics", name: "Acrobatics", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Acrobatics+", name: "Acrobatics+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
            });
}
