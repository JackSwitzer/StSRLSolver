use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Pray", name: "Pray", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["mantra"], effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::AddCard("Insight", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Pray+", name: "Pray+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["mantra"], effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::AddCard("Insight", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
