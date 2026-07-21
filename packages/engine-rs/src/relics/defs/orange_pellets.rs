//! Orange Pellets: Playing ATK + SKL + POW in one turn clears debuffs.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    // Source: reference/extracted/methods/relic/OrangePellets.java
    // ATTACK, SKILL, and POWER flags reset at turn start and after firing.
    match event.kind {
        Trigger::CombatStart | Trigger::TurnStart => {
            state.set(0, 0);
            state.set(1, 0);
            state.set(2, 0);
            return;
        }
        Trigger::OnUseCard => match event.card_type {
            Some(crate::cards::CardType::Attack) => state.set(0, 1),
            Some(crate::cards::CardType::Skill) => state.set(1, 1),
            Some(crate::cards::CardType::Power) => state.set(2, 1),
            _ => return,
        },
        _ => return,
    }

    if state.get(0) <= 0 || state.get(1) <= 0 || state.get(2) <= 0 {
        return;
    }

    // OrangePellets.java queues RemoveDebuffsAction, which removes every
    // AbstractPower whose type is DEBUFF rather than a fixed hand-picked list.
    for idx in 0..engine.state.player.statuses.len() {
        let status = crate::ids::StatusId(idx as u16);
        if crate::powers::registry::status_is_debuff(status) {
            engine.state.player.set_status(status, 0);
        }
    }
    state.set(0, 0);
    state.set(1, 0);
    state.set(2, 0);
}

static TRIGGERS: [TriggeredEffect; 3] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
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
        trigger: Trigger::OnUseCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "OrangePellets",
    name: "Orange Pellets",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
