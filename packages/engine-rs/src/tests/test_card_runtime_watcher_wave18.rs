#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java

use crate::cards::global_registry;

#[test]
fn test_card_runtime_watcher_wave18_registry_documents_the_remaining_hook_cleanup_tail() {
    let registry = global_registry();

    let collect = registry.get("Collect").expect("Collect should exist");
    assert_eq!(collect.effect_data.len(), 1);
    assert!(collect.complex_hook.is_some());

    let collect_plus = registry.get("Collect+").expect("Collect+ should exist");
    assert_eq!(collect_plus.effect_data.len(), 1);
    assert!(collect_plus.complex_hook.is_some());

    let conjure_blade = registry
        .get("ConjureBlade")
        .expect("Conjure Blade should exist");
    assert_eq!(conjure_blade.effect_data.len(), 1);
    assert!(conjure_blade.complex_hook.is_some());

    let fasting = registry.get("Fasting2").expect("Fasting should exist");
    assert_eq!(fasting.effect_data.len(), 2);
    assert!(fasting.complex_hook.is_some());

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should exist");
    assert_eq!(lesson_learned.effect_data.len(), 1);
    assert!(lesson_learned.complex_hook.is_some());

    let pressure_points = registry
        .get("PathToVictory")
        .expect("Pressure Points should exist");
    assert_eq!(pressure_points.effect_data.len(), 1);
    assert!(pressure_points.complex_hook.is_some());

    let wallop = registry.get("Wallop").expect("Wallop should exist");
    assert_eq!(wallop.effect_data.len(), 1);
    assert!(wallop.complex_hook.is_some());
}

#[test]
#[ignore = "Collect still needs an upgrade-aware X-count primitive that can add one extra Miracle only on the upgraded card; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java"]
fn test_card_runtime_watcher_wave18_collect_needs_upgrade_aware_xcount() {}

#[test]
#[ignore = "Conjure Blade still needs a generated-card payload primitive that can set Expunger.setX(energyOnUse + upgrade bonus); Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java"]
fn test_card_runtime_watcher_wave18_conjure_blade_needs_generated_card_payload_with_x_hits() {}

#[test]
#[ignore = "Fasting still needs a post-install max-energy reduction primitive rather than a side-effect hook; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java"]
fn test_card_runtime_watcher_wave18_fasting_needs_post_install_max_energy_reduction() {}

#[test]
#[ignore = "Lesson Learned still needs a kill-triggered random upgrade primitive over eligible draw/discard cards; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java"]
fn test_card_runtime_watcher_wave18_lesson_learned_needs_kill_triggered_random_upgrade() {}

#[test]
#[ignore = "Pressure Points still needs a mark-triggered HP-loss primitive that bypasses block after applying Mark; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java"]
fn test_card_runtime_watcher_wave18_pressure_points_needs_mark_triggered_hp_loss() {}

#[test]
#[ignore = "Wallop still needs a post-damage block-gain primitive keyed off unblocked damage from the attack context; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java"]
fn test_card_runtime_watcher_wave18_wallop_needs_post_damage_block_gain() {}
