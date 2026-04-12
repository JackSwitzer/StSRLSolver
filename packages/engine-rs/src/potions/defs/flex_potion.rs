use super::prelude::*;

static EFFECTS: [E; 2] = [
    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::PotionPotency)),
    E::Simple(SE::AddStatus(T::Player, sid::LOSE_STRENGTH, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "SteroidPotion",
    name: "Flex Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
