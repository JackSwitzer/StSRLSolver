use crate::cards::prelude::*;

fn malaise_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let amount = ctx.x_value + ctx.card.base_magic.max(0);
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        crate::powers::apply_debuff(&mut engine.state.enemies[tidx].entity, sid::WEAKENED, amount);
        engine.state.enemies[tidx].entity.add_status(sid::STRENGTH, -amount);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Malaise", name: "Malaise", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effects: &["x_cost"], effect_data: &[], complex_hook: Some(malaise_hook),
            });
    insert(cards, CardDef {
                id: "Malaise+", name: "Malaise+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["x_cost"], effect_data: &[], complex_hook: Some(malaise_hook),
            });
}
