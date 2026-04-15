use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Combust ---- (cost 1, power, lose 1 HP/turn, deal 5 dmg to all; +2 magic)
    insert(cards, CardDef {
                id: "Combust", name: "Combust", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Combust+", name: "Combust+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
                ], complex_hook: None,
            });
}
