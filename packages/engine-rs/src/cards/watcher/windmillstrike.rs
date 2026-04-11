use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Windmill Strike ---- (cost 2, 7 dmg, retain, +4 dmg each retain; +3 dmg +1 magic upgrade)
    insert(cards, CardDef {
                id: "WindmillStrike", name: "Windmill Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["retain", "grow_damage_on_retain"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "WindmillStrike+", name: "Windmill Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["retain", "grow_damage_on_retain"], effect_data: &[], complex_hook: None,
            });
}
