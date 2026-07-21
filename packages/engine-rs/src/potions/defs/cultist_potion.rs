use super::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/potions/CultistPotion.java
// use() targets the player and applies player-controlled Ritual at potion
// potency; sound and talk actions are cosmetic.

static EFFECTS: [E; 1] = [
    E::Simple(SE::AddStatus(T::Player, sid::RITUAL, A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "CultistPotion",
    name: "Cultist Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
