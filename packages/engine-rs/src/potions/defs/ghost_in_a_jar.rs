use super::prelude::*;

static EFFECTS: [E; 1] = [
    E::Simple(SE::AddStatus(T::Player, sid::INTANGIBLE, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "GhostInAJar",
    name: "Ghost In A Jar",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
