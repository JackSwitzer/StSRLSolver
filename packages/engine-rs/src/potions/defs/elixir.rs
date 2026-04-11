use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Elixir: exhaust entire hand. Irreducible -- needs engine access.
fn elixir_hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    engine.state.exhaust_pile.extend(engine.state.hand.drain(..));
}

pub static DEF: EntityDef = EntityDef {
    id: "Elixir",
    name: "Elixir",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(elixir_hook),
};
