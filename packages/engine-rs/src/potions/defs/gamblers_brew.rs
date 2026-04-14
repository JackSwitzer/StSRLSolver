use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Gambler's Brew: discard entire hand, then draw that many cards.
/// Irreducible -- hand size must be captured before discard.
fn gamblers_brew_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let hand_size = engine.state.hand.len() as i32;
    engine.state.discard_pile.extend(engine.state.hand.drain(..));
    engine.state.player.set_status(sid::POTION_DRAW, hand_size);
}

pub static DEF: EntityDef = EntityDef {
    id: "GamblersBrew",
    name: "Gambler's Brew",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(gamblers_brew_hook),
    status_guard: None,
};
