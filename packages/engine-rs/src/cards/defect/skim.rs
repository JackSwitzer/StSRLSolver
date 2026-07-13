use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Skim.java draws 3 cards for one energy; upgradeMagicNumber(1) is its
        // only upgrade change.
        // Java: reference/extracted/methods/card/Skim.java
    insert(cards, CardDef {
                id: "Skim", name: "Skim", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Skim+", name: "Skim+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
