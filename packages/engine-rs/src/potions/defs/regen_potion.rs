use super::prelude::*;

static EFFECTS: [E; 1] = [
    E::Simple(SE::AddStatus(T::Player, sid::REGENERATION, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "RegenPotion",
    name: "Regen Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
};
