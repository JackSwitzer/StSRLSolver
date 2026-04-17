use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "CutThroughFate", name: "Cut Through Fate", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "CutThroughFate+", name: "Cut Through Fate+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
}
