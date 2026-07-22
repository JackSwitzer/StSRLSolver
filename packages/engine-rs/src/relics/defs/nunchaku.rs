//! Nunchaku: Every 10 Attacks played, gain 1 Energy.
//! Counter persists across combats.
//!
//! Source: `reference/extracted/methods/relic/Nunchaku.java` (`onUseCard`
//! counts only ATTACK cards, resets at ten, and queues GainEnergyAction(1)).

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::Fixed(1),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAttackPlayed,
    condition: TriggerCondition::CounterReached,
    effects: &EFFECTS,
    counter: Some((sid::NUNCHAKU_COUNTER, 10)),
}];

pub static DEF: EntityDef = EntityDef {
    id: "Nunchaku",
    name: "Nunchaku",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
