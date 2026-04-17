use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Hyperbeam: 2 cost, 26 dmg AoE, lose 3 focus
    insert(cards, CardDef {
                id: "Hyperbeam", name: "Hyperbeam", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 26, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-3))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Hyperbeam+", name: "Hyperbeam+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 34, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-3))),
                ], complex_hook: None,
            });
}
