use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Fiend Fire ---- (cost 2, exhaust, 7 dmg per card in hand exhausted; +3 dmg)
    insert(cards, CardDef {
                id: "Fiend Fire", name: "Fiend Fire", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["fiend_fire"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_fiend_fire),
            });
    insert(cards, CardDef {
                id: "Fiend Fire+", name: "Fiend Fire+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["fiend_fire"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_fiend_fire),
            });
}
