use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
    insert(cards, CardDef {
                id: "PathToVictory", name: "Pressure Points", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 8, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::MARK, A::Magic)),
                    E::Simple(SE::TriggerMarks),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "PathToVictory+", name: "Pressure Points+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 11, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::MARK, A::Magic)),
                    E::Simple(SE::TriggerMarks),
                ], complex_hook: None,
            });
}
