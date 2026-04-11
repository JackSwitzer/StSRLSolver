use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
        // TODO: Full effect requires choosing a card from draw pile and playing it twice
    insert(cards, CardDef {
                id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[], complex_hook: None,
            });
}
