use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Glass Knife ---- (cost 1, 8 dmg x2, -2 dmg each play; +2 dmg)
    insert(cards, CardDef {
                id: "Glass Knife", name: "Glass Knife", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "glass_knife"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Glass Knife+", name: "Glass Knife+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "glass_knife"], effect_data: &[], complex_hook: None,
            });
}
