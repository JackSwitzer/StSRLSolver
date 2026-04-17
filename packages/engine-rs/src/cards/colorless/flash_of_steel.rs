use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Flash of Steel: 0 cost, 3 dmg, draw 1
    insert(cards, CardDef {
                id: "Flash of Steel", name: "Flash of Steel", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flash of Steel+", name: "Flash of Steel+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Magic))], complex_hook: None,
            });
}
