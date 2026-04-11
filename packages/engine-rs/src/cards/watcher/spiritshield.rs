use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
    insert(cards, CardDef {
                id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: None,
            });
}
