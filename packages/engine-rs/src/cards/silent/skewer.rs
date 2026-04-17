use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Skewer ---- (cost X, 7 dmg x X times; +3 dmg)
    insert(cards, CardDef {
                id: "Skewer", name: "Skewer", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: -1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Skewer+", name: "Skewer+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: -1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
}
