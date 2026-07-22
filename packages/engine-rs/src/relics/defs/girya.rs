//! Girya: +Strength based on lift count at combat start.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Girya.java
//! RunEngine persists campfire lifts and transfers them through GIRYA_COUNTER.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    state: &mut crate::effects::runtime::EffectState,
) {
    let lift_count = state.get(0);
    if lift_count > 0 {
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_top(TurnStartQueuedAction::AddPlayerStatus(
                sid::STRENGTH,
                lift_count,
            ));
        } else {
            engine.state.player.add_status(sid::STRENGTH, lift_count);
        }
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStartTop,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Girya",
    name: "Girya",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
