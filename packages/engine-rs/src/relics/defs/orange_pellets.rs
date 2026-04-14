//! Orange Pellets: Playing ATK + SKL + POW in one turn clears debuffs.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
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

    engine.state.player.set_status(sid::WEAKENED, 0);
    engine.state.player.set_status(sid::VULNERABLE, 0);
    engine.state.player.set_status(sid::FRAIL, 0);
    engine.state.player.set_status(sid::ENTANGLED, 0);
    engine.state.player.set_status(sid::NO_DRAW, 0);
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
