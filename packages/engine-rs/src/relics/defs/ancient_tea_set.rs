//! Ancient Tea Set: after entering a rest room, gain 2 energy on turn 1 of the
//! next combat.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/AncientTeaSet.java.

use crate::effects::declarative::Effect;
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

static EFFECTS: [Effect; 0] = [];
static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::FirstTurn,
    effects: &EFFECTS,
    counter: None,
}];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    let counter = crate::relic_flags::counter::ANCIENT_TEA_SET;
    if event.kind == Trigger::TurnStart
        && event.is_first_turn
        && engine.state.relic_counters[counter] > 0
    {
        engine.state.energy += 2;
        engine.state.relic_counters[counter] = 0;
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Ancient Tea Set",
    name: "Ancient Tea Set",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
