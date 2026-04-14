use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Whirlwind ---- (cost X, 5 dmg AoE per X; +3 dmg)
    insert(cards, CardDef {
                id: "Whirlwind", name: "Whirlwind", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: -1, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["x_cost"], effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Whirlwind+", name: "Whirlwind+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: -1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["x_cost"], effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave3.rs"]
mod test_card_runtime_ironclad_wave3;
