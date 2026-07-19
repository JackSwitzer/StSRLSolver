use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn snecko_oil_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let amount = match owner {
        crate::effects::runtime::EffectOwner::PotionSlot { slot } => {
            let idx = slot as usize;
            if idx >= engine.state.potions.len() {
                return;
            }
            crate::potions::effective_potency_runtime(
                &engine.state,
                &engine.state.potions[idx],
            )
        }
        _ => return,
    };

    engine.draw_cards(amount);

    // RandomizeHandCostAction consumes one cardRandomRng.random(3) for every
    // non-X, playable-cost card now in hand. It permanently changes the combat
    // cost only when that roll differs; Snecko Oil does not apply Confusion.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SneckoOil.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RandomizeHandCostAction.java
    for idx in 0..engine.state.hand.len() {
        let card = engine.state.hand[idx];
        let printed_cost = engine.card_registry.card_def_by_id(card.def_id).cost;
        let permanent_cost = if card.base_cost >= 0 {
            card.base_cost as i32
        } else {
            printed_cost
        };
        if permanent_cost < 0 {
            continue;
        }
        let new_cost = engine.card_random_rng.random_int(3) as i8;
        if permanent_cost != new_cost as i32 {
            engine.state.hand[idx].set_permanent_cost(new_cost);
        }
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "SneckoOil",
    name: "Snecko Oil",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(snecko_oil_hook),
    status_guard: None,
};
