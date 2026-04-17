use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Envenom ---- (cost 2, power, apply 1 poison on attack dmg; upgrade: cost 1)
    insert(cards, CardDef {
                id: "Envenom", name: "Envenom", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENVENOM, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Envenom+", name: "Envenom+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENVENOM, A::Magic)),
                ], complex_hook: None,
            });
}
