//! Duality (Yang): On Attack play, gain 1 temporary Dexterity.
//!
//! complex_hook needed: the Dex is temporary (add LOSE_DEXTERITY alongside).
//! Old dispatch: adds DEXTERITY +1 and LOSE_DEXTERITY +1.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::DEXTERITY, AmountSource::Fixed(1))),
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::LOSE_DEXTERITY, AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnAttackPlayed,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Yang",
    name: "Duality",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
