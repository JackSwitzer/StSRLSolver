use crate::cards::prelude::*;

fn pressure_points_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Apply Mark to target
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        engine.state.enemies[tidx].entity.add_status(sid::MARK, ctx.card.base_magic);
    }
    // Deal damage to ALL enemies equal to their Mark (bypasses block)
    let living = engine.state.living_enemy_indices();
    for idx in living {
        let mark = engine.state.enemies[idx].entity.status(sid::MARK);
        if mark > 0 {
            engine.state.enemies[idx].entity.hp -= mark;
            if engine.state.enemies[idx].entity.hp <= 0 {
                engine.state.enemies[idx].entity.hp = 0;
            }
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
    insert(cards, CardDef {
                id: "PressurePoints", name: "Pressure Points", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 8, exhaust: false, enter_stance: None,
                effects: &["pressure_points"], effect_data: &[], complex_hook: Some(pressure_points_hook),
            });
    insert(cards, CardDef {
                id: "PressurePoints+", name: "Pressure Points+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 11, exhaust: false, enter_stance: None,
                effects: &["pressure_points"], effect_data: &[], complex_hook: Some(pressure_points_hook),
            });
}
