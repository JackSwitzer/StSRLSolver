use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Shrug It Off ---- (cost 1, 8 block, draw 1; +3 block)
    insert(cards, CardDef {
                id: "Shrug It Off", name: "Shrug It Off", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Shrug It Off+", name: "Shrug It Off+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Fixed(1))),
                ], complex_hook: None,
            });
}
