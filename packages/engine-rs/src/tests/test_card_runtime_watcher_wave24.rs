#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_with, engine_without_start, force_player_turn, make_deck, play_self};

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

    let omniscience = registry.get("Omniscience").expect("Omniscience should exist");
    assert_eq!(
        omniscience.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::PlayForFree,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(omniscience.complex_hook.is_none());

    let omniscience_plus = registry.get("Omniscience+").expect("Omniscience+ should exist");
    assert_eq!(
        omniscience_plus.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::PlayForFree,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(omniscience_plus.complex_hook.is_none());
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
fn watcher_wave24_omniscience_uses_the_typed_draw_pile_free_play_surface() {
    let mut engine = engine_without_start(
        make_deck(&["Omniscience+", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        4,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 4;
    engine.state.hand = make_deck(&["Omniscience+"]);
    engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P"]);

    assert!(play_self(&mut engine, "Omniscience+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(
        engine.choice.as_ref().expect("Omniscience+ should open a choice").reason,
        ChoiceReason::PlayCardFreeFromDraw
    );

    engine.execute_action(&crate::actions::Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Strike_P");
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.draw_pile.len(), 1);
}
