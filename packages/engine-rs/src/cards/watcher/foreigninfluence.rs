use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from other class; upgrade: upgraded choices)
    insert(cards, CardDef {
                id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["foreign_influence"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["foreign_influence"], effect_data: &[], complex_hook: None,
            });
}
