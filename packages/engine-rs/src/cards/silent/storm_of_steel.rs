use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Storm of Steel ---- (cost 1, discard hand, add Shiv per card; upgrade: Shiv+)
    insert(cards, CardDef {
                id: "Storm of Steel", name: "Storm of Steel", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["storm_of_steel"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Storm of Steel+", name: "Storm of Steel+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["storm_of_steel"], effect_data: &[], complex_hook: None,  // handler checks card name for Shiv vs Shiv+
            });
}
