use super::prelude::*;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[E::Simple(SE::AddCard(
        "Shiv",
        crate::effects::declarative::Pile::Hand,
        A::PotionPotency,
    ))],
    counter: None,
}];

/// Cunning Potion: Add Shiv+ cards to hand (3 base, 6 with Sacred Bark).
pub static DEF: EntityDef = EntityDef {
    id: "CunningPotion",
    name: "Cunning Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
