use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn speed_potion_hook(
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

    // Java queues DexterityPower first, then the debuff-typed
    // LoseDexterityPower. Artifact can therefore block only the delayed loss.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/SpeedPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseDexterityPower.java
    engine.state.player.add_status(sid::DEXTERITY, amount);
    crate::powers::apply_debuff(&mut engine.state.player, sid::LOSE_DEXTERITY, amount);
}

pub static DEF: EntityDef = EntityDef {
    id: "SpeedPotion",
    name: "Speed Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(speed_potion_hook),
    status_guard: None,
};
