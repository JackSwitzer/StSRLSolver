use super::prelude::*;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[E::Simple(SE::AddCard(
        // Source: reference/extracted/methods/potion/CunningPotion.java. The
        // potion upgrades its template before making stat-equivalent copies.
        "Shiv+",
        crate::effects::declarative::Pile::Hand,
        A::PotionPotency,
    ))],
    counter: None,
}];

/// Cunning Potion: add three Shiv+ cards (six with Sacred Bark).
pub static DEF: EntityDef = EntityDef {
    id: "CunningPotion",
    name: "Cunning Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
