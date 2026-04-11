use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Reboot: 0 cost, shuffle hand+discard into draw, draw 4, exhaust
    insert(cards, CardDef {
                id: "Reboot", name: "Reboot", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["reboot"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_reboot),
            });
    insert(cards, CardDef {
                id: "Reboot+", name: "Reboot+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: true, enter_stance: None,
                effects: &["reboot"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_reboot),
            });
}
