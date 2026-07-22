//! Stone Calendar: deal 52 damage to all enemies at the end of turn 7.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/StoneCalendar.java

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, EndTurnQueuedAction};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    match event.kind {
        Trigger::CombatSetup => state.set(0, 0),
        Trigger::TurnStart => state.add(0, 1),
        Trigger::TurnEndPreCard if state.get(0) == 7 => {
            engine.queue_end_turn_action_bottom(EndTurnQueuedAction::DamageAllEnemies(52));
        }
        Trigger::CombatVictory => state.set(0, -1),
        _ => {}
    }
}

static TRIGGERS: [TriggeredEffect; 4] = [
    TriggeredEffect {
        trigger: Trigger::CombatSetup,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnEndPreCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::CombatVictory,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "StoneCalendar",
    name: "Stone Calendar",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
