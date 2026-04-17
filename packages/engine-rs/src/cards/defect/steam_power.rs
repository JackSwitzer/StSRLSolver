use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Overclock (Java ID: Steam Power): 0 cost, draw 2, add Burn to discard
    insert(cards, CardDef {
                id: "Steam Power", name: "Overclock", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Steam Power+", name: "Overclock+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
}
