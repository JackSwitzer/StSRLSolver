use crate::cards::prelude::*;

fn collect_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Declarative effect_data sets the base X count; the hook adds the upgrade bump.
    if ctx.card.id.ends_with('+') {
        let miracles = engine.state.player.status(sid::COLLECT_MIRACLES);
        engine.state.player.set_status(sid::COLLECT_MIRACLES, miracles + 1);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
    insert(cards, CardDef {
                id: "Collect", name: "Collect", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCost)),
                ], complex_hook: Some(collect_hook),
            });
    insert(cards, CardDef {
                id: "Collect+", name: "Collect+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCost)),
                ], complex_hook: Some(collect_hook),
            });
}
