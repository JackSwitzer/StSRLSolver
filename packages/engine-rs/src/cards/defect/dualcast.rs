use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Dualcast", name: "Dualcast", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb", "evoke_orb"], effect_data: &[
                    E::Simple(SE::EvokeOrb(A::Fixed(2))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dualcast+", name: "Dualcast+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb", "evoke_orb"], effect_data: &[
                    E::Simple(SE::EvokeOrb(A::Fixed(2))),
                ], complex_hook: None,
            });
}
