use crate::cards::prelude::*;

fn escape_plan_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    engine.draw_cards(1);
    if !engine.state.hand.is_empty() {
        let last = engine.state.hand.last().unwrap();
        let last_type = engine.card_registry.card_def_by_id(last.def_id).card_type;
        if last_type == crate::cards::CardType::Skill {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = crate::damage::calculate_block(ctx.card.base_block.max(0), dex, frail);
            engine.gain_block_player(block);
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Escape Plan", name: "Escape Plan", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_if_skill"], effect_data: &[], complex_hook: Some(escape_plan_hook),
            });
    insert(cards, CardDef {
                id: "Escape Plan+", name: "Escape Plan+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_if_skill"], effect_data: &[], complex_hook: Some(escape_plan_hook),
            });
}
