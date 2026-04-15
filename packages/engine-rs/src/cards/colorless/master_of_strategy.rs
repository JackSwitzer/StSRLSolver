use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Master of Strategy: 0 cost, draw 3, exhaust
    insert(cards, CardDef {
                id: "Master of Strategy", name: "Master of Strategy", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Master of Strategy+", name: "Master of Strategy+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
}
