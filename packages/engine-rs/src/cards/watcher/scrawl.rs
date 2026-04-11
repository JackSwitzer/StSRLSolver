use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Scrawl ---- (cost 1, skill, exhaust, draw until you have 10 cards; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["draw_to_ten"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["draw_to_ten"], effect_data: &[], complex_hook: None,
            });
}
