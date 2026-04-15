use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Finesse: 0 cost, 2 block, draw 1
    insert(cards, CardDef {
                id: "Finesse", name: "Finesse", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 2,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Finesse+", name: "Finesse+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
}
