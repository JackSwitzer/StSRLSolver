use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Masterful Stab ---- (cost 0, 12 dmg, costs 1 more per HP lost; +4 dmg)
    insert(cards, CardDef {
                id: "Masterful Stab", name: "Masterful Stab", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["cost_increase_on_hp_loss"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Masterful Stab+", name: "Masterful Stab+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["cost_increase_on_hp_loss"], effect_data: &[], complex_hook: None,
            });
}
