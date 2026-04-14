#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{engine_with, play_self};

#[test]
fn watcher_wave24_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let collect = registry.get("Collect").expect("Collect should exist");
    assert_eq!(
        collect.effect_data,
        &[E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCostPlus(0)))]
    );
    assert!(collect.complex_hook.is_none());

    let collect_plus = registry.get("Collect+").expect("Collect+ should exist");
    assert_eq!(
        collect_plus.effect_data,
        &[E::Simple(SE::SetStatus(T::SelfEntity, sid::COLLECT_MIRACLES, A::XCostPlus(1)))]
    );
    assert!(collect_plus.complex_hook.is_none());

    let conjure_blade = registry
        .get("ConjureBlade")
        .expect("Conjure Blade should exist");
    assert_eq!(
        conjure_blade.effect_data,
        &[E::Simple(SE::AddCardWithMisc(
            "Expunger",
            P::Draw,
            A::Fixed(1),
            A::XCostPlus(0),
        ))]
    );
    assert!(conjure_blade.complex_hook.is_none());

    let conjure_blade_plus = registry
        .get("ConjureBlade+")
        .expect("Conjure Blade+ should exist");
    assert_eq!(
        conjure_blade_plus.effect_data,
        &[E::Simple(SE::AddCardWithMisc(
            "Expunger",
            P::Draw,
            A::Fixed(1),
            A::XCostPlus(1),
        ))]
    );
    assert!(conjure_blade_plus.complex_hook.is_none());

    let fasting = registry.get("Fasting2").expect("Fasting should exist");
    assert_eq!(
        fasting.effect_data,
        &[
            E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
            E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
            E::Simple(SE::ModifyMaxEnergy(A::Fixed(-1))),
        ]
    );
    assert!(fasting.complex_hook.is_none());

    let fasting_plus = registry.get("Fasting2+").expect("Fasting+ should exist");
    assert_eq!(
        fasting_plus.effect_data,
        &[
            E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
            E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
            E::Simple(SE::ModifyMaxEnergy(A::Fixed(-1))),
        ]
    );
    assert!(fasting_plus.complex_hook.is_none());
}

#[test]
fn watcher_wave24_collect_and_fasting_follow_typed_effect_data() {
    let mut collect = engine_with(crate::tests::support::make_deck(&["Collect"]), 40, 0);
    assert!(play_self(&mut collect, "Collect"));
    assert_eq!(collect.state.player.status(sid::COLLECT_MIRACLES), 3);

    let mut collect_plus = engine_with(crate::tests::support::make_deck(&["Collect+"]), 40, 0);
    assert!(play_self(&mut collect_plus, "Collect+"));
    assert_eq!(collect_plus.state.player.status(sid::COLLECT_MIRACLES), 4);

    let mut fasting = engine_with(crate::tests::support::make_deck(&["Fasting2"]), 40, 0);
    assert!(play_self(&mut fasting, "Fasting2"));
    assert_eq!(fasting.state.player.status(sid::STRENGTH), 3);
    assert_eq!(fasting.state.player.status(sid::DEXTERITY), 3);
    assert_eq!(fasting.state.max_energy, 2);

    let mut fasting_plus = engine_with(crate::tests::support::make_deck(&["Fasting2+"]), 40, 0);
    assert!(play_self(&mut fasting_plus, "Fasting2+"));
    assert_eq!(fasting_plus.state.player.status(sid::STRENGTH), 4);
    assert_eq!(fasting_plus.state.player.status(sid::DEXTERITY), 4);
    assert_eq!(fasting_plus.state.max_energy, 2);
}

#[test]
fn watcher_wave24_conjure_blade_follow_the_typed_generated_card_surface() {
    let mut conjure_blade = engine_with(crate::tests::support::make_deck(&["ConjureBlade"]), 40, 0);
    assert!(play_self(&mut conjure_blade, "ConjureBlade"));
    let expunger = conjure_blade
        .state
        .draw_pile
        .iter()
        .find(|card| conjure_blade.card_registry.card_name(card.def_id) == "Expunger")
        .expect("Conjure Blade should add Expunger to draw pile");
    assert_eq!(expunger.misc, 3);

    let mut conjure_blade_plus = engine_with(crate::tests::support::make_deck(&["ConjureBlade+"]), 40, 0);
    assert!(play_self(&mut conjure_blade_plus, "ConjureBlade+"));
    let expunger_plus = conjure_blade_plus
        .state
        .draw_pile
        .iter()
        .find(|card| conjure_blade_plus.card_registry.card_name(card.def_id) == "Expunger")
        .expect("Conjure Blade+ should add Expunger to draw pile");
    assert_eq!(expunger_plus.misc, 4);
}

#[test]
#[ignore = "Omniscience still needs a typed draw-pile choice / play-twice primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java"]
fn watcher_wave24_omniscience_stays_queued_until_choice_primitive_exists() {}
