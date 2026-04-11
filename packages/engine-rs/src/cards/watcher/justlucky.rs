use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Just Lucky ---- (cost 0, 3 dmg, 2 block, scry 1; +1/+1/+1 upgrade)
    insert(cards, CardDef {
                id: "JustLucky", name: "Just Lucky", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: 2,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["scry"], effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "JustLucky+", name: "Just Lucky+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: 3,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["scry"], effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                ], complex_hook: None,
            });
}
