use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Reinforced Body: X cost, gain 7 block X times
    insert(cards, CardDef {
                id: "Reinforced Body", name: "Reinforced Body", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_x_times"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reinforced Body+", name: "Reinforced Body+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 9,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_x_times"], effect_data: &[], complex_hook: None,
            });
}
