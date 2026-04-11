use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Endless Agony ---- (cost 0, 4 dmg, exhaust, copy to hand on draw; +2 dmg)
    insert(cards, CardDef {
                id: "Endless Agony", name: "Endless Agony", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["copy_on_draw"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Endless Agony+", name: "Endless Agony+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["copy_on_draw"], effect_data: &[], complex_hook: None,
            });
}
