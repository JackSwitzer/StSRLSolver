use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Streamline: 2 cost, 15 dmg, costs 1 less each play
    insert(cards, CardDef {
                id: "Streamline", name: "Streamline", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["reduce_cost_each_play"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_streamline),
            });
    insert(cards, CardDef {
                id: "Streamline+", name: "Streamline+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["reduce_cost_each_play"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_streamline),
            });
}
