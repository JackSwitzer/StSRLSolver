use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Feel No Pain ---- (cost 1, power, 3 block on exhaust; +1 magic)
    insert(cards, CardDef {
                id: "Feel No Pain", name: "Feel No Pain", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FEEL_NO_PAIN, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Feel No Pain+", name: "Feel No Pain+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FEEL_NO_PAIN, A::Magic)),
                ], complex_hook: None,
            });
}
