use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Source: cards/green/BladeDance.java costs 1 and queues one
        // MakeTempCardInHandAction for 3 Shivs; the upgrade raises that to 4.
    insert(cards, CardDef {
                id: "Blade Dance", name: "Blade Dance", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blade Dance+", name: "Blade Dance+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
                ], complex_hook: None,
            });
}
