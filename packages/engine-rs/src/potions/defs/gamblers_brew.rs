use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Gambler's Brew: discard entire hand, then draw that many cards.
/// Irreducible -- hand size must be captured before discard.
fn gamblers_brew_hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let hand_size = engine.state.hand.len() as i32;
    engine.state.discard_pile.extend(engine.state.hand.drain(..));
    engine.state.player.set_status(sid::POTION_DRAW, hand_size);
}

pub static DEF: EntityDef = EntityDef {
    id: "GamblersBrew",
    name: "Gambler's Brew",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(gamblers_brew_hook),
};
