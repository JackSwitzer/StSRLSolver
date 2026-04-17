use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Prostrate", name: "Prostrate", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Prostrate+", name: "Prostrate+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
}
