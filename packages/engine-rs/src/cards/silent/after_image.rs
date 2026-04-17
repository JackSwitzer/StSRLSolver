use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: After Image ---- (cost 1, power, 1 block per card played; upgrade: cost 0)  [Note: ID is "After Image"]
    insert(cards, CardDef {
                id: "After Image", name: "After Image", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::AFTER_IMAGE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "After Image+", name: "After Image+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::AFTER_IMAGE, A::Magic)),
                ], complex_hook: None,
            });
}
