use crate::cards::prelude::*;

fn conjure_blade_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Create Expunger with X hits (X+1 for upgrade)
    let hits = if ctx.card.id.ends_with('+') { ctx.x_value + 1 } else { ctx.x_value };
    engine.state.player.set_status(sid::EXPUNGER_HITS, hits);
    let expunger = engine.temp_card("Expunger");
    if engine.state.hand.len() < 10 {
        engine.state.hand.push(expunger);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
    insert(cards, CardDef {
                id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["conjure_blade"], effect_data: &[], complex_hook: Some(conjure_blade_hook),
            });
    insert(cards, CardDef {
                id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["conjure_blade"], effect_data: &[], complex_hook: Some(conjure_blade_hook),
            });
}
