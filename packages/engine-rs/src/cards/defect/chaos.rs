use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Chaos: 1 cost, channel 1 random orb (upgrade: 2)
    insert(cards, CardDef {
                id: "Chaos", name: "Chaos", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_random"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_chaos),
            });
    insert(cards, CardDef {
                id: "Chaos+", name: "Chaos+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["channel_random"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_chaos),
            });
}
