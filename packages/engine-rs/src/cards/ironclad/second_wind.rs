use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Second Wind ---- (cost 1, exhaust all non-attack, gain block per; +2 block)
    insert(cards, CardDef {
                id: "Second Wind", name: "Second Wind", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["second_wind"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Second Wind+", name: "Second Wind+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["second_wind"], effect_data: &[], complex_hook: None,
            });
}
