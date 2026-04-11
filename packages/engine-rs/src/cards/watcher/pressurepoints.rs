use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
    insert(cards, CardDef {
                id: "PressurePoints", name: "Pressure Points", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 8, exhaust: false, enter_stance: None,
                effects: &["pressure_points"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "PressurePoints+", name: "Pressure Points+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 11, exhaust: false, enter_stance: None,
                effects: &["pressure_points"], effect_data: &[], complex_hook: None,
            });
}
