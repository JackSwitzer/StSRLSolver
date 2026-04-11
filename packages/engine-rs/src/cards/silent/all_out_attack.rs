use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: All-Out Attack ---- (cost 1, 10 AoE dmg, discard random; +4 dmg)
    insert(cards, CardDef {
                id: "All-Out Attack", name: "All-Out Attack", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discard_random"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "All-Out Attack+", name: "All-Out Attack+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discard_random"], effect_data: &[], complex_hook: None,
            });
}
