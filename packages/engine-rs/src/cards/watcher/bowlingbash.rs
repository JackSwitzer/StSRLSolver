use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common Watcher Cards ----
    insert(cards, CardDef {
                id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_per_enemy"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_per_enemy"], effect_data: &[], complex_hook: None,
            });
}
