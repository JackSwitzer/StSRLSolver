use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Tranquility ---- (cost 1, enter Calm, exhaust, retain; upgrade: cost 0)
        // Java ID: ClearTheMind, run.rs uses Tranquility
    insert(cards, CardDef {
                id: "ClearTheMind", name: "Tranquility", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ClearTheMind+", name: "Tranquility+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
}
