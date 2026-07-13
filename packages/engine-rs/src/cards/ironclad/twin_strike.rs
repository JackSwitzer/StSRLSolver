use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // TwinStrike.java queues two separate DamageActions for 5 each, carries
        // STRIKE, and upgradeDamage(2) raises both hits to 7.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/TwinStrike.java
    insert(cards, CardDef {
                id: "Twin Strike", name: "Twin Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Twin Strike+", name: "Twin Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
}
