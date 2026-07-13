use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Caltrops.java
        // applies ThornsPower(3); upgradeMagicNumber(2) raises it to 5.
    insert(cards, CardDef {
                id: "Caltrops", name: "Caltrops", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::THORNS, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Caltrops+", name: "Caltrops+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::THORNS, A::Magic)),
                ], complex_hook: None,
            });
}
