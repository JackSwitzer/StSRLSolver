use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DoomAndGloom.java deals its multiDamage to every enemy, then channels
    // one Dark. The upgrade changes only damage, from 10 to 14.
    // Java: reference/extracted/methods/card/DoomAndGloom.java
    insert(cards, CardDef {
                id: "Doom and Gloom", name: "Doom and Gloom", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Doom and Gloom+", name: "Doom and Gloom+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: None,
            });
}
