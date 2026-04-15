use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Leap: 1 cost, 9 block
    insert(cards, CardDef {
        id: "Leap", name: "Leap", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Leap+", name: "Leap+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
    });
}
