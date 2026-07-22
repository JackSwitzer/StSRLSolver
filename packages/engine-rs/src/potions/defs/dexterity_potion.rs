use super::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/potions/DexterityPotion.java
// use() replaces the supplied target with the player and applies permanent
// Dexterity equal to potency; getPotency always returns 2.

static EFFECTS: [E; 1] = [E::Simple(SE::AddStatus(
    T::Player,
    sid::DEXTERITY,
    A::PotionPotency,
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "DexterityPotion",
    name: "Dexterity Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
