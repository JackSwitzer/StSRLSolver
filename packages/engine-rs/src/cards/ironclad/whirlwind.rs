use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // WhirlwindAction queues one all-enemy 5-damage action per Energy spent;
    // Chemical X adds two spins and the upgrade raises each hit to eight.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Whirlwind.java
    // and actions/unique/WhirlwindAction.java.
    insert(cards, CardDef {
                id: "Whirlwind", name: "Whirlwind", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: -1, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Whirlwind+", name: "Whirlwind+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: -1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::XCost),
                ], complex_hook: None,
            });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave3.rs"]
mod test_card_runtime_ironclad_wave3;
