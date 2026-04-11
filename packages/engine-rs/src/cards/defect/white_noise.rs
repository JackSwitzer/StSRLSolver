use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // White Noise: 1 cost, add random Power to hand, exhaust (upgrade: cost 0)
    insert(cards, CardDef {
                id: "White Noise", name: "White Noise", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["add_random_power"], effect_data: &[
                    E::Simple(SE::AddCard("Defragment", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "White Noise+", name: "White Noise+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["add_random_power"], effect_data: &[
                    E::Simple(SE::AddCard("Defragment", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
}
