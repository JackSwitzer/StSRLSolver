//! Ring of the Serpent: +1 draw per turn at combat start (Silent starter upgrade).

use crate::effects::declarative::{Effect, SimpleEffect as SE, AmountSource as A, Target as T};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SE::SetStatus(T::Player, sid::RING_OF_SERPENT_DRAW, A::Fixed(1))),
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
    id: "Ring of the Serpent",
    name: "Ring of the Serpent",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
