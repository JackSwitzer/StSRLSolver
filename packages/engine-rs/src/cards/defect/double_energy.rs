use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Double Energy: 1 cost, double your energy, exhaust (upgrade: cost 0)
    insert(cards, CardDef {
                id: "Double Energy", name: "Double Energy", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["double_energy"], effect_data: &[], complex_hook: Some(crate::effects::hooks_simple::hook_double_energy),
            });
    insert(cards, CardDef {
                id: "Double Energy+", name: "Double Energy+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["double_energy"], effect_data: &[], complex_hook: Some(crate::effects::hooks_simple::hook_double_energy),
            });
}
