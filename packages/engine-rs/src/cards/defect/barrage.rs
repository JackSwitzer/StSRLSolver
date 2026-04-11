use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Barrage: 1 cost, 4 dmg x orbs
    insert(cards, CardDef {
                id: "Barrage", name: "Barrage", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[], complex_hook: Some(crate::effects::hooks_orb::hook_damage_per_orb),
            });
    insert(cards, CardDef {
                id: "Barrage+", name: "Barrage+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[], complex_hook: Some(crate::effects::hooks_orb::hook_damage_per_orb),
            });
}
