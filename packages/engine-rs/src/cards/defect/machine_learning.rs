use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Machine Learning: 1 cost, power, draw 1 extra card each turn (upgrade: innate)
    insert(cards, CardDef {
        id: "Machine Learning", name: "Machine Learning", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
        effects: &[],
        effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::DRAW, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Machine Learning+", name: "Machine Learning+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
        effects: &["innate"],
        effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::DRAW, A::Magic))],
        complex_hook: None,
    });
}
