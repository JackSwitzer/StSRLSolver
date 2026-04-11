use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Like Water ---- (cost 1, power, if in Calm at end of turn gain 5 block; +2 magic upgrade)
    insert(cards, CardDef {
                id: "LikeWater", name: "Like Water", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["like_water"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "LikeWater+", name: "Like Water+", card_type: CardType::Power,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effects: &["like_water"], effect_data: &[], complex_hook: None,
            });
}
