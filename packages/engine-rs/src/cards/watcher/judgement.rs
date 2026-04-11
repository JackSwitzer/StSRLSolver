use crate::cards::prelude::*;

fn judgement_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let threshold = ctx.card.base_magic;
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        if engine.state.enemies[tidx].entity.hp <= threshold && engine.state.enemies[tidx].is_alive() {
            engine.deal_damage_to_enemy(tidx, 9999);
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Judgement ---- (cost 1, skill, if enemy HP <= 30, kill it; +10 magic upgrade)
    insert(cards, CardDef {
                id: "Judgement", name: "Judgement", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 30, exhaust: false, enter_stance: None,
                effects: &["judgement"], effect_data: &[], complex_hook: Some(judgement_hook),
            });
    insert(cards, CardDef {
                id: "Judgement+", name: "Judgement+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 40, exhaust: false, enter_stance: None,
                effects: &["judgement"], effect_data: &[], complex_hook: Some(judgement_hook),
            });
}
