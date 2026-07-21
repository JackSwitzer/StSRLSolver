use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Metallicize.java applies magicNumber 3 as MetallicizePower for one energy;
    // upgradeMagicNumber(1) is the only upgrade change.
    insert(cards, CardDef {
                id: "Metallicize", name: "Metallicize", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::METALLICIZE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Metallicize+", name: "Metallicize+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::METALLICIZE, A::Magic)),
                ], complex_hook: None,
            });
}
