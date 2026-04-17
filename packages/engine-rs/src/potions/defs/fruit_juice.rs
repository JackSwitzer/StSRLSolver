use super::prelude::*;

static EFFECTS: [E; 1] = [
    E::Simple(SE::ModifyMaxHp(A::Fixed(5))),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "FruitJuice",
    name: "Fruit Juice",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
