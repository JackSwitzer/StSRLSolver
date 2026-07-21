use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Gain 6 Block, then MakeTempCardInHandAction creates 1 Shiv (2 upgraded).
        // Source: reference/extracted/methods/card/CloakAndDagger.java
    insert(cards, CardDef {
                id: "Cloak and Dagger", name: "Cloak and Dagger", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Cloak and Dagger+", name: "Cloak and Dagger+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
                ], complex_hook: None,
            });
}
