use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Scrape.java costs 1, deals 7 damage, then draws 4; its follow-up
    // manually discards each directly drawn card unless costForTurn is 0 or
    // freeToPlayOnce. The upgrade adds 3 damage and 1 draw.
    // Java: reference/extracted/methods/card/Scrape.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/
    // ScrapeFollowUpAction.java
    insert(
        cards,
        CardDef {
            id: "Scrape",
            name: "Scrape",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 7,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCardsThenDiscardDrawnNonZeroCost(
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Scrape+",
            name: "Scrape+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCardsThenDiscardDrawnNonZeroCost(
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
