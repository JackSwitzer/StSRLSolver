use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // FTL: 0 cost, 5 dmg, draw 1 if <3 cards played this turn
    insert(cards, CardDef {
                id: "FTL", name: "FTL", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["draw_if_few_cards_played"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "FTL+", name: "FTL+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["draw_if_few_cards_played"], effect_data: &[], complex_hook: None,
            });
}
