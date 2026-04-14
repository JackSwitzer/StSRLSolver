use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Inflame ---- (cost 1, power, +2 str; +1)
    insert(cards, CardDef {
                id: "Inflame", name: "Inflame", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Inflame+", name: "Inflame+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
}
