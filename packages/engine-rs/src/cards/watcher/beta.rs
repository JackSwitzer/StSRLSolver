use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Special Generated Cards ----
        // Beta (from Alpha chain): cost 2, skill, exhaust, add Omega to draw
    insert(cards, CardDef {
                id: "Beta", name: "Beta", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Beta+", name: "Beta+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
