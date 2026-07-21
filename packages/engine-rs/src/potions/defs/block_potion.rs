use super::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/potions/BlockPotion.java
// GainBlockAction grants raw potion potency to the player; getPotency always
// returns 12, independent of ascension.

static EFFECTS: [E; 1] = [
    E::Simple(SE::GainBlock(A::PotionPotency)),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "BlockPotion",
    name: "Block Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
