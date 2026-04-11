use crate::cards::prelude::*;

fn collect_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Set COLLECT_MIRACLES status = x_value (or x_value+1 for upgraded).
    // The turn-start hook will create that many Miracles next turn.
    let miracles = if ctx.card.id.ends_with('+') { ctx.x_value + 1 } else { ctx.x_value };
    if miracles > 0 {
        engine.state.player.set_status(sid::COLLECT_MIRACLES, miracles);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
    insert(cards, CardDef {
                id: "Collect", name: "Collect", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[], complex_hook: Some(collect_hook),
            });
    insert(cards, CardDef {
                id: "Collect+", name: "Collect+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[], complex_hook: Some(collect_hook),
            });
}
