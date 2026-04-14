use super::prelude::*;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[E::Simple(SE::AddCard(
        "Miracle",
        crate::effects::declarative::Pile::Hand,
        A::PotionPotency,
    ))],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "BottledMiracle",
    name: "Bottled Miracle",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
