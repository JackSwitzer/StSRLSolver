use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Pummel queues magicNumber separate damage actions at two damage each;
    // upgrading adds one hit without changing cost or damage, and both exhaust.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Pummel.java
    insert(cards, CardDef {
                id: "Pummel", name: "Pummel", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Pummel+", name: "Pummel+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
}
