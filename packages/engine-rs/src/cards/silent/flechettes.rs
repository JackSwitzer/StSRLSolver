use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Flechettes ---- (cost 1, 4 dmg per skill in hand; +2 dmg)
    insert(cards, CardDef {
                id: "Flechettes", name: "Flechettes", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["flechettes"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_flechettes),
            });
    insert(cards, CardDef {
                id: "Flechettes+", name: "Flechettes+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["flechettes"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_flechettes),
            });
}
