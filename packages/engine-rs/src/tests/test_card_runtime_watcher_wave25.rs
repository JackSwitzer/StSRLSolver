#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::tests::support::{engine_with, hand_count, exhaust_prefix_count};

#[test]
fn watcher_wave25_deus_ex_machina_stays_engine_path_covered_on_draw() {
    let engine = engine_with(crate::tests::support::make_deck(&["DeusExMachina"]), 50, 0);
    assert_eq!(hand_count(&engine, "Miracle"), 2);
    assert_eq!(hand_count(&engine, "DeusExMachina"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina"), 1);
}

#[test]
fn watcher_wave25_registry_exports_match_current_surface_for_blocked_cards() {
    let registry = global_registry();

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should be registered");
    assert_eq!(lesson_learned.effect_data.len(), 1);
    assert!(lesson_learned.complex_hook.is_some());

    let omniscience = registry
        .get("Omniscience")
        .expect("Omniscience should be registered");
    assert!(omniscience.effect_data.is_empty());
    assert!(omniscience.complex_hook.is_some());
}

#[test]
#[ignore = "Lesson Learned still needs a kill-triggered random-upgrade primitive; Java upgrades a random unupgraded card from draw pile or discard after the enemy dies."]
fn lesson_learned_still_needs_kill_triggered_random_upgrade_primitive() {}

#[test]
#[ignore = "Omniscience still needs a draw-pile card selection plus play-twice primitive; Java selects a card from the draw pile and uses it twice via UseCardAction semantics."]
fn omniscience_still_needs_draw_pile_play_twice_primitive() {}
