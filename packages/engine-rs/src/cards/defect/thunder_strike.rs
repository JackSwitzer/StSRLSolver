use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Thunder Strike: 3 cost, deal 7 dmg for each Lightning channeled this combat
    insert(cards, CardDef {
                id: "Thunder Strike", name: "Thunder Strike", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 7, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &["damage_per_lightning_channeled"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_thunder_strike),
            });
    insert(cards, CardDef {
                id: "Thunder Strike+", name: "Thunder Strike+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 9, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &["damage_per_lightning_channeled"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_thunder_strike),
            });
}
