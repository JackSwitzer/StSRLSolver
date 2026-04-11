use super::prelude::*;

static EFFECTS: [E; 1] = [
    E::Simple(SE::HealHp(T::Player, A::PercentMaxHp(20))),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "BloodPotion",
    name: "Blood Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
};
