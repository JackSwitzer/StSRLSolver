use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Escape Plan", name: "Escape Plan", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
        base_magic: -1, exhaust: false, enter_stance: None,
        effects: &["block_if_skill"],
        effect_data: &[
            E::Simple(SE::DrawCards(A::Fixed(1))),
            E::Simple(SE::GainBlockIfLastHandCardType(CardType::Skill, A::Block)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Escape Plan+", name: "Escape Plan+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
        base_magic: -1, exhaust: false, enter_stance: None,
        effects: &["block_if_skill"],
        effect_data: &[
            E::Simple(SE::DrawCards(A::Fixed(1))),
            E::Simple(SE::GainBlockIfLastHandCardType(CardType::Skill, A::Block)),
        ],
        complex_hook: None,
    });
}
