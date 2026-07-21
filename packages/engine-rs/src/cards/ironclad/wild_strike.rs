use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // WildStrike.use deals 12 damage, then queues one Wound into a random
    // non-top draw-pile position. The upgrade adds five damage only.
    // Java: reference/extracted/methods/card/WildStrike.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    insert(cards, CardDef {
                id: "Wild Strike", name: "Wild Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCardToRandomDrawSpot("Wound", A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wild Strike+", name: "Wild Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 17, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCardToRandomDrawSpot("Wound", A::Fixed(1))),
                ], complex_hook: None,
            });
}
