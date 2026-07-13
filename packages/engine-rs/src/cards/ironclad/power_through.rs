use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // PowerThrough.java queues two Wounds into hand, then gains 15 Block;
    // upgrading changes only Block by +5.
    // Source: reference/extracted/methods/card/PowerThrough.java
    insert(cards, CardDef {
                id: "Power Through", name: "Power Through", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Power Through+", name: "Power Through+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 20,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
                ], complex_hook: None,
            });
}
