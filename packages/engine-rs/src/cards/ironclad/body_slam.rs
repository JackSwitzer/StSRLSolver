use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Body Slam ---- (cost 1, dmg = current block; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Body Slam", name: "Body Slam", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_equals_block"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Body Slam+", name: "Body Slam+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 0, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_equals_block"], effect_data: &[], complex_hook: None,
            });
}
