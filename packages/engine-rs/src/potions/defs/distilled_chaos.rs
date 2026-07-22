use super::prelude::*;
use crate::engine::{CombatEngine, DeferredCombatOp};

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Distilled Chaos: play top N cards from draw pile for free.
/// complex_hook because it must drive repeated top-of-draw plays through the
/// real card action pipeline.
fn distilled_chaos_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    // Distilled Chaos is a fixed 3-card top-of-draw play in Java, doubled by
    // Sacred Bark, so its exact action-path behavior stays local to this
    // hook instead of going through the generic potency table.
    let potency = if engine.state.has_relic("SacredBark") {
        6
    } else {
        3
    };

    // DistilledChaosPotion.java chooses every target while `use` is building
    // its PlayTopCardAction queue, so all cardRandomRng ticks happen before a
    // top card can kill or otherwise change the monster group. MonsterGroup's
    // inclusive random(0, size - 1) also consumes a tick with one living enemy.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/DistilledChaosPotion.java
    // and decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    let living = engine.state.living_enemy_indices();
    let targets: Vec<i32> = if living.is_empty() {
        vec![-1; potency]
    } else {
        (0..potency)
            .map(|_| {
                let pick = engine
                    .card_random_rng
                    .random_int_range(0, (living.len() - 1) as i32)
                    as usize;
                living[pick] as i32
            })
            .collect()
    };

    for target_idx in targets {
        // Java's potion use queues these actions; it does not execute cards
        // reentrantly inside AbstractPotion.use. Run them after the enclosing
        // ManualActivation dispatch returns.
        engine
            .deferred_combat_ops
            .push(DeferredCombatOp::PlayTopCard { target_idx });
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "DistilledChaos",
    name: "Distilled Chaos",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(distilled_chaos_hook),
    status_guard: None,
};
