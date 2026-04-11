use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Blood for Blood ---- (cost 4, 18 dmg, -1 cost per HP loss; upgrade: cost 3, +4 dmg)
    insert(cards, CardDef {
                id: "Blood for Blood", name: "Blood for Blood", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 4, base_damage: 18, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["cost_reduce_on_hp_loss"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blood for Blood+", name: "Blood for Blood+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 3, base_damage: 22, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["cost_reduce_on_hp_loss"], effect_data: &[], complex_hook: None,
            });
}
