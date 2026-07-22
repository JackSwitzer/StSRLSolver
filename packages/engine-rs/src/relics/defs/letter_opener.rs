//! Letter Opener: Every 3 Skills played, deal 5 damage to ALL enemies.
//!
//! Source: `reference/extracted/methods/relic/LetterOpener.java`
//! (`onUseCard` queues a pure 5-damage matrix with `DamageType.THORNS`).

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnSkillPlayed {
        return;
    }
    for idx in engine.state.living_enemy_indices() {
        engine.deal_thorns_damage_to_enemy(idx, 5);
    }
}

static RESET_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::SetStatus(
    Target::Player,
    sid::LETTER_OPENER_COUNTER,
    AmountSource::Fixed(0),
))];

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::OnSkillPlayed,
        condition: TriggerCondition::CounterReached,
        effects: &[],
        counter: Some((sid::LETTER_OPENER_COUNTER, 3)),
    },
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &RESET_EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Letter Opener",
    name: "Letter Opener",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
