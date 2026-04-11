use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Rip and Tear: 1 cost, deal 7 dmg twice to random enemies
    insert(cards, CardDef {
                id: "Rip and Tear", name: "Rip and Tear", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_damage_random_hits),
            });
    insert(cards, CardDef {
                id: "Rip and Tear+", name: "Rip and Tear+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_damage_random_hits),
            });
}
