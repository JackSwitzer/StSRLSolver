use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Scrape: 1 cost, 7 dmg, draw N then discard newly drawn cards costing > 0.
    insert(cards, CardDef {
                id: "Scrape", name: "Scrape", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCardsThenDiscardDrawnNonZeroCost(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Scrape+", name: "Scrape+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCardsThenDiscardDrawnNonZeroCost(A::Magic))], complex_hook: None,
            });
}
