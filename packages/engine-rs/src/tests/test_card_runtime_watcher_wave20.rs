#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java

use crate::cards::global_registry;
use crate::effects::declarative::{Effect as E, SimpleEffect as SE, AmountSource as A};
use crate::tests::support::*;

#[test]
fn test_card_runtime_watcher_wave20_registry_promotes_judgement_to_typed_primary_surface() {
    let judgement = global_registry().get("Judgement").expect("Judgement should exist");
    assert_eq!(judgement.effect_data, &[E::Simple(SE::Judgement(A::Magic))]);
    assert!(judgement.complex_hook.is_none());

    let judgement_plus = global_registry().get("Judgement+").expect("Judgement+ should exist");
    assert_eq!(judgement_plus.effect_data, &[E::Simple(SE::Judgement(A::Magic))]);
    assert!(judgement_plus.complex_hook.is_none());
}

#[test]
fn test_card_runtime_watcher_wave20_judgement_kills_when_target_is_below_threshold() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 20, 20)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Judgement"]);
    engine.state.enemies[0].entity.block = 4;

    assert!(play_on_enemy(&mut engine, "Judgement", 0));
    assert!(engine.state.enemies[0].entity.is_dead());
}

#[test]
#[ignore = "Blocked on Java mark-triggered damage primitive for Pressure Points; the current runtime can apply Mark but cannot yet resolve enemy-specific Mark damage that bypasses block. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java"]
fn test_card_runtime_watcher_wave20_pressure_points_needs_mark_damage_primitive() {}
