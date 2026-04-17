use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Sweeping Beam: 1 cost, 6 dmg AoE, draw 1
    insert(cards, CardDef {
                id: "Sweeping Beam", name: "Sweeping Beam", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sweeping Beam+", name: "Sweeping Beam+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
