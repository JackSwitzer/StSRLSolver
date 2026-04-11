use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Brilliance ---- (cost 1, 12 dmg + mantra gained this combat; +4 dmg upgrade)
    insert(cards, CardDef {
                id: "Brilliance", name: "Brilliance", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &["damage_plus_mantra"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Brilliance+", name: "Brilliance+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &["damage_plus_mantra"], effect_data: &[], complex_hook: None,
            });
}
