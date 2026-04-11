use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare Watcher Cards ----
    insert(cards, CardDef {
                id: "Ragnarok", name: "Ragnarok", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 5, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Ragnarok+", name: "Ragnarok+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 6, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
            });
}
