use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Core Surge: 1 cost, 11 dmg, gain 1 Artifact, exhaust
    insert(cards, CardDef {
                id: "Core Surge", name: "Core Surge", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Core Surge+", name: "Core Surge+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic)),
                ], complex_hook: None,
            });
}
