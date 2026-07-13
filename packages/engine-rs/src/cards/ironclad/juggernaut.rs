use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Juggernaut.java applies magicNumber 5 JuggernautPower for two energy;
    // upgradeMagicNumber(2) raises the trigger damage to 7.
    insert(cards, CardDef {
                id: "Juggernaut", name: "Juggernaut", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Juggernaut+", name: "Juggernaut+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::JUGGERNAUT, A::Magic)),
                ], complex_hook: None,
            });
}
