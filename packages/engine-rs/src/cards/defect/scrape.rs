use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Scrape: 1 cost, 7 dmg, draw 4 then discard non-0-cost cards drawn
    insert(cards, CardDef {
                id: "Scrape", name: "Scrape", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["draw_discard_non_zero"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_scrape),
            });
    insert(cards, CardDef {
                id: "Scrape+", name: "Scrape+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["draw_discard_non_zero"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_scrape),
            });
}
