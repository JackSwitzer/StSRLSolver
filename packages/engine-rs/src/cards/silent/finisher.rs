use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Finisher ---- (cost 1, 6 dmg per attack played this turn; +2 dmg)
    insert(cards, CardDef {
                id: "Finisher", name: "Finisher", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["finisher"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Finisher+", name: "Finisher+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["finisher"], effect_data: &[], complex_hook: None,
            });
}
