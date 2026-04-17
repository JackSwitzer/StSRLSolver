use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Cold Snap: 1 cost, 6 dmg, channel 1 Frost
    insert(cards, CardDef {
                id: "Cold Snap", name: "Cold Snap", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Cold Snap+", name: "Cold Snap+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Magic)),
                ], complex_hook: None,
            });
}
