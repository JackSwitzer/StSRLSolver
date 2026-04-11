use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
    insert(cards, CardDef {
                id: "Collect", name: "Collect", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Collect+", name: "Collect+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
            });
}
