use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn steroid_potion_hook(
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
            crate::potions::effective_potency_runtime(
                &engine.state,
                &engine.state.potions[idx],
            )
        }
        _ => return,
    };

    // Java queues StrengthPower first, then the debuff-typed
    // LoseStrengthPower. Artifact can therefore block only the delayed loss.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SteroidPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
    engine.state.player.add_status(sid::STRENGTH, amount);
    crate::powers::apply_debuff(&mut engine.state.player, sid::LOSE_STRENGTH, amount);
}

pub static DEF: EntityDef = EntityDef {
    id: "SteroidPotion",
    name: "Flex Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(steroid_potion_hook),
    status_guard: None,
};
