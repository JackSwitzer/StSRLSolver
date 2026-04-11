use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Master Reality ---- (cost 1, power, created cards are upgraded; upgrade: cost 0)
    insert(cards, CardDef {
                id: "MasterReality", name: "Master Reality", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["master_reality"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "MasterReality+", name: "Master Reality+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["master_reality"], effect_data: &[], complex_hook: None,
            });
}
