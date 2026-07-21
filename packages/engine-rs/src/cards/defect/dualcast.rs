use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Dualcast.java evokes the front orb once without removing it, then evokes
    // that same orb normally. Upgrade changes only cost from 1 to 0.
    // Java: reference/extracted/methods/card/Dualcast.java
    insert(cards, CardDef {
                id: "Dualcast", name: "Dualcast", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeOrbWithoutRemoving),
                    E::Simple(SE::EvokeOrb(A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dualcast+", name: "Dualcast+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeOrbWithoutRemoving),
                    E::Simple(SE::EvokeOrb(A::Fixed(1))),
                ], complex_hook: None,
            });
}
