use super::prelude::*;
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[E::Simple(SE::AddStatus(T::Player, sid::ORB_SLOTS, A::PotionPotency))],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "PotionOfCapacity",
    name: "Potion of Capacity",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
