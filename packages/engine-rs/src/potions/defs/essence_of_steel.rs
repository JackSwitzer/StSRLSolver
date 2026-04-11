use super::prelude::*;

static EFFECTS: [E; 1] = [
    E::Simple(SE::AddStatus(T::Player, sid::PLATED_ARMOR, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "EssenceOfSteel",
    name: "Essence of Steel",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
};
