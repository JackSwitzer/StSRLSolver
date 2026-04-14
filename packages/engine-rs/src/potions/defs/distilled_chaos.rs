use super::prelude::*;
use crate::actions::Action;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Distilled Chaos: play top N cards from draw pile for free.
/// complex_hook because it must drive repeated top-of-draw plays through the
/// real card action pipeline.
fn distilled_chaos_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    // Distilled Chaos is a fixed 3-card top-of-draw play in Java, doubled by
    // Sacred Bark. The shared runtime potency helper still falls back to 1
    // for potions not listed in the legacy potency table, so keep the exact
    // action-path behavior local here until that shared table is normalized.
    let potency = if engine.state.has_relic("SacredBark") { 6 } else { 3 };
    for _ in 0..potency {
        if engine.state.draw_pile.is_empty() {
            break;
        }

        let card = engine
            .state
            .draw_pile
            .pop()
            .expect("checked non-empty draw pile");
        let free_card = card.set_free(true);
        engine.state.hand.push(free_card);
        let hand_idx = engine.state.hand.len() - 1;

        let target_idx = {
            let def = engine.card_registry.card_def_by_id(free_card.def_id);
            match def.target {
                crate::cards::CardTarget::Enemy => engine
                    .state
                    .living_enemy_indices()
                    .first()
                    .map(|idx| *idx as i32)
                    .unwrap_or(-1),
                _ => -1,
            }
        };

        engine.execute_action(&Action::PlayCard { card_idx: hand_idx, target_idx });
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "DistilledChaos",
    name: "Distilled Chaos",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(distilled_chaos_hook),
    status_guard: None,
};
