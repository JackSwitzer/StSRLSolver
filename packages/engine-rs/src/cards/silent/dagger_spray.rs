use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Dagger Spray ---- (cost 1, 4 dmg x2 AoE; +2 dmg)
    insert(cards, CardDef {
                id: "Dagger Spray", name: "Dagger Spray", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 4, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dagger Spray+", name: "Dagger Spray+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[], complex_hook: None,
            });
}
