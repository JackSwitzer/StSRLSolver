use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn fruit_juice_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let amount = match owner {
        crate::effects::runtime::EffectOwner::PotionSlot { slot } => {
            let idx = slot as usize;
            if idx >= engine.state.potions.len() {
                return;
            }
            crate::potions::effective_potency_runtime(&engine.state, &engine.state.potions[idx])
        }
        _ => return,
    };

    // AbstractCreature.increaseMaxHp first raises maxHealth, then calls heal,
    // so Magic Flower and Mark of the Bloom modify only the current-HP gain.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    engine.state.player.max_hp += amount;
    engine.heal_player(amount);
}

pub static DEF: EntityDef = EntityDef {
    id: "FruitJuice",
    name: "Fruit Juice",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(fruit_juice_hook),
    status_guard: None,
};
