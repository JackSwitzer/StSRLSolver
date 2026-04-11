use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Infinite Blades ---- (cost 1, power, add Shiv to hand at turn start; upgrade: cost 0)  [Note: ID is actually "Infinite Blades" not "InfiniteBlades"]
    insert(cards, CardDef {
                id: "Infinite Blades", name: "Infinite Blades", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["infinite_blades"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INFINITE_BLADES, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Infinite Blades+", name: "Infinite Blades+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["infinite_blades", "innate"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INFINITE_BLADES, A::Magic)),
                ], complex_hook: None,
            });
}
