#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P};
use crate::tests::support::*;

#[test]
fn watcher_wave21_deus_ex_machina_stays_engine_path_covered() {
    let engine = engine_with(make_deck(&["DeusExMachina"]), 50, 0);
    assert_eq!(hand_count(&engine, "Miracle"), 2);
    assert_eq!(hand_count(&engine, "DeusExMachina"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina"), 1);
}

#[test]
#[ignore = "Collect still needs upgraded X-cost miracle generation semantics from the watcher primitive wave; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java"]
fn watcher_wave21_collect_stays_explicit_blocker_until_x_cost_upgrade_semantics_exist() {}

#[test]
#[ignore = "Conjure Blade still needs draw-pile Expunger generation with X-hit transfer semantics from the watcher primitive wave; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java"]
fn watcher_wave21_conjure_blade_stays_explicit_blocker_until_expunger_x_transfer_exists() {}

#[test]
#[ignore = "Fasting still needs the max-energy reduction side effect from the watcher primitive wave; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java"]
fn watcher_wave21_fasting_stays_explicit_blocker_until_energy_down_effect_exists() {}

#[test]
fn watcher_wave21_omniscience_uses_the_typed_draw_pile_free_play_surface() {
    let omniscience = global_registry()
        .get("Omniscience")
        .expect("Omniscience should be registered");
    assert_eq!(
        omniscience.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::PlayForFree,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(omniscience.complex_hook.is_none());
}
