use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Insight (from Evaluate / Study): cost 0, draw 2, retain, exhaust
    insert(cards, CardDef {
                id: "Insight", name: "Insight", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Insight+", name: "Insight+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
