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
    assert_eq!(
        collect.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::SetStatus(
                crate::effects::declarative::Target::SelfEntity,
                crate::status_ids::sid::COLLECT_MIRACLES,
                crate::effects::declarative::AmountSource::XCostPlus(0),
            )
        )]
    );
    assert!(collect.complex_hook.is_none());

    let collect_plus = registry.get("Collect+").expect("Collect+ should exist");
    assert_eq!(
        collect_plus.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::SetStatus(
                crate::effects::declarative::Target::SelfEntity,
                crate::status_ids::sid::COLLECT_MIRACLES,
                crate::effects::declarative::AmountSource::XCostPlus(1),
            )
        )]
    );
    assert!(collect_plus.complex_hook.is_none());

    let conjure_blade = registry
        .get("ConjureBlade")
        .expect("Conjure Blade should exist");
    assert_eq!(
        conjure_blade.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::AddCardWithMisc(
                "Expunger",
                crate::effects::declarative::Pile::Draw,
                crate::effects::declarative::AmountSource::Fixed(1),
                crate::effects::declarative::AmountSource::XCostPlus(0),
            )
        )]
    );
    assert!(conjure_blade.complex_hook.is_none());

    let fasting = registry.get("Fasting2").expect("Fasting should exist");
    assert_eq!(fasting.effect_data.len(), 3);
    assert!(fasting.complex_hook.is_none());

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should exist");
    assert_eq!(lesson_learned.effect_data.len(), 2);
    assert!(lesson_learned.complex_hook.is_none());

    let pressure_points = registry
        .get("PathToVictory")
        .expect("Pressure Points should exist");
    assert_eq!(pressure_points.effect_data.len(), 2);
    assert!(pressure_points.complex_hook.is_none());

    let wallop = registry.get("Wallop").expect("Wallop should exist");
    assert_eq!(wallop.effect_data.len(), 2);
    assert!(wallop.complex_hook.is_none());
}

#[test]
#[ignore = "Pressure Points still needs a mark-triggered HP-loss primitive that bypasses block after applying Mark; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java"]
fn test_card_runtime_watcher_wave18_pressure_points_needs_mark_triggered_hp_loss() {}
