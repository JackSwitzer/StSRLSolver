use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Demon Form ---- (cost 3, power, +2 str/turn; +1 magic)
    insert(cards, CardDef {
                id: "Demon Form", name: "Demon Form", card_type: CardType::Power,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DEMON_FORM, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Demon Form+", name: "Demon Form+", card_type: CardType::Power,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DEMON_FORM, A::Magic)),
                ], complex_hook: None,
            });
}
