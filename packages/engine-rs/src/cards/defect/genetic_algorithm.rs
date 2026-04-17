use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Genetic Algorithm", name: "Genetic Algorithm", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 1,
        base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Genetic Algorithm+", name: "Genetic Algorithm+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
        base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ],
        complex_hook: None,
    });
}
