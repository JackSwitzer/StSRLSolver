//! Teardrop Locket: Start combat in Calm stance.
//!
//! Runtime-backed via a combat-start stance change effect.

use crate::effects::declarative::{Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::state::Stance;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::ChangeStance(Stance::Calm)),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "TeardropLocket",
    name: "Teardrop Locket",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
