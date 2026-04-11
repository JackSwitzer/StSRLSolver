use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Malaise ---- (cost X, -X str + X weak to enemy, exhaust; +1/+1)
    insert(cards, CardDef {
                id: "Malaise", name: "Malaise", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effects: &["x_cost", "malaise"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Malaise+", name: "Malaise+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["x_cost", "malaise"], effect_data: &[], complex_hook: None,
            });
}
