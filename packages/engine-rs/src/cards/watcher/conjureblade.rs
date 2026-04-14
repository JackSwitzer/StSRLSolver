use crate::cards::prelude::*;

fn conjure_blade_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let mut expunger_hits = ctx.x_value.max(0);
    if ctx.card.id.ends_with('+') {
        expunger_hits += 1;
    }
    if expunger_hits <= 0 {
        return;
    }

    if let Some(card) = engine.state.hand.last_mut() {
        let name = engine.card_registry.card_name(card.def_id);
        if (name == "Expunger" || name == "Expunger+") && card.misc < 0 {
            card.misc = expunger_hits as i16;
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
    insert(cards, CardDef {
                id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddCard("Expunger", P::Hand, A::Fixed(1))),
                ], complex_hook: Some(conjure_blade_hook),
            });
    insert(cards, CardDef {
                id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddCard("Expunger", P::Hand, A::Fixed(1))),
                ], complex_hook: Some(conjure_blade_hook),
            });
}
