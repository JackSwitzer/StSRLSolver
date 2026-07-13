use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java queues exactly two 4-damage hits (6 upgraded) and sets selfRetain.
    // It does not define magicNumber; the hit count is fixed card logic.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlyingSleeves.java
    insert(cards, CardDef {
        id: "FlyingSleeves", name: "Flying Sleeves", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::ExtraHits(A::Fixed(2)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "FlyingSleeves+", name: "Flying Sleeves+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::ExtraHits(A::Fixed(2)),
        ], complex_hook: None,
    });
}
