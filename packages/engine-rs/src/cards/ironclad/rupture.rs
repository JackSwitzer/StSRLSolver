use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Rupture ---- (cost 1, power, +1 str when lose HP from card; +1 magic)
    insert(cards, CardDef {
                id: "Rupture", name: "Rupture", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rupture+", name: "Rupture+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::RUPTURE, A::Magic)),
                ], complex_hook: None,
            });
}
