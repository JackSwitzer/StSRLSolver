use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Pommel Strike ---- (cost 1, 9 dmg, draw 1; +1/+1)
    insert(cards, CardDef {
                id: "Pommel Strike", name: "Pommel Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["draw"], effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Pommel Strike+", name: "Pommel Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["draw"], effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
