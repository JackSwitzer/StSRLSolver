use super::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/potions/AncientPotion.java
// use() replaces the supplied target with the player and applies Artifact equal
// to potion potency, which Sacred Bark doubles.
static EFFECTS: [E; 1] = [
    E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "AncientPotion",
    name: "Ancient Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
