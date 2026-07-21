use super::prelude::*;
use crate::actions::Action;
use crate::engine::CombatEngine;

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
    let potency = if engine.state.has_relic("SacredBark") { 6 } else { 3 };

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

    let mut shuffles = 0;
    for target_idx in targets {
        // PlayTopCardAction retries after EmptyDeckShuffleAction whenever the
        // draw pile is empty but the discard pile still contains cards.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
        if engine.state.draw_pile.is_empty() && !engine.state.discard_pile.is_empty() {
            engine.state.draw_pile = std::mem::take(&mut engine.state.discard_pile);
            engine.shuffle_draw_pile();
            shuffles += 1;
        }

        let Some(card) = engine.state.draw_pile.pop() else {
            continue;
        };

        let free_card = card.set_free(true);
        engine.state.hand.push(free_card);
        let hand_idx = engine.state.hand.len() - 1;
        engine.execute_action(&Action::PlayCard { card_idx: hand_idx, target_idx });
    }

    // EmptyDeckShuffleAction constructs relic onShuffle actions behind the
    // already queued PlayTopCardActions, so dispatch after this potion's plays.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/EmptyDeckShuffleAction.java
    for _ in 0..shuffles {
        let ctx = crate::effects::trigger::TriggerContext::empty();
        engine.emit_event(crate::effects::runtime::GameEvent::from_trigger(
            crate::effects::trigger::Trigger::OnShuffle,
            &ctx,
        ));
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
