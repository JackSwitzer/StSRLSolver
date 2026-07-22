#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CalculatedGambleAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Concentrate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/BladeFuryAction.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{
    AmountSource as A, BulkAction, CardFilter, ChoiceAction, Effect as E, Pile as P,
    SimpleEffect as SE,
};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn silent_wave14_calculated_gamble_is_declarative_and_uses_hand_size_at_play() {
    let registry = global_registry();
    let calculated_gamble = registry
        .get("Calculated Gamble")
        .expect("Calculated Gamble should exist");
    assert_eq!(
        calculated_gamble.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlay)),
        ],
    );
    assert!(calculated_gamble.complex_hook.is_none());
    assert!(calculated_gamble.exhaust);

    let calculated_gamble_plus = registry
        .get("Calculated Gamble+")
        .expect("Calculated Gamble+ should exist");
    // CalculatedGamble.use constructs CalculatedGambleAction(false) even after
    // upgrade, so the otherwise-present count+1 branch is never selected.
    // Java: cards/green/CalculatedGamble.java and
    // actions/unique/CalculatedGambleAction.java.
    assert_eq!(
        calculated_gamble_plus.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlay)),
        ],
    );
    assert!(calculated_gamble_plus.complex_hook.is_none());
    assert!(!calculated_gamble_plus.exhaust);

    let mut engine = engine_without_start(
        make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Calculated Gamble", "Strike", "Strike", "Strike"]);
    engine.state.draw_pile =
        make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Calculated Gamble"));

    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(discard_prefix_count(&engine, "Strike"), 3);
}

#[test]
fn silent_wave14_concentrate_is_declarative_discard_for_effect() {
    let registry = global_registry();
    let concentrate = registry
        .get("Concentrate")
        .expect("Concentrate should exist");
    assert_eq!(
        concentrate.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::DiscardForEffect,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }],
    );
    assert!(concentrate.complex_hook.is_none());

    let concentrate_plus = registry
        .get("Concentrate+")
        .expect("Concentrate+ should exist");
    assert_eq!(
        concentrate_plus.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::DiscardForEffect,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }],
    );
    assert!(concentrate_plus.complex_hook.is_none());

    let mut engine = engine_without_start(
        make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 3;
    engine.state.hand = make_deck(&["Concentrate", "Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Concentrate"));
    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.energy, 5);
    assert_eq!(engine.state.hand.len(), 0);
    assert_eq!(discard_prefix_count(&engine, "Strike"), 3);
}

#[test]
fn silent_wave14_storm_of_steel_bulk_discards_hand_and_adds_shivs() {
    // BladeFuryAction snapshots the hand after Storm of Steel has left it,
    // queues DiscardAction above MakeTempCardInHandAction, and only afterward
    // resolves effects (such as Reflex draws) queued to the bottom by manual
    // discards. This ordering preserves all nine generated Shivs at hand cap.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/BladeFuryAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Reflex.java
    let registry = global_registry();
    let storm = registry
        .get("Storm of Steel")
        .expect("Storm of Steel should exist");
    assert_eq!(
        storm.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::AddCard("Shiv", P::Hand, A::HandSizeAtPlay)),
        ],
    );
    assert!(storm.complex_hook.is_none());

    let storm_plus = registry
        .get("Storm of Steel+")
        .expect("Storm of Steel+ should exist");
    assert_eq!(
        storm_plus.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::AddCard("Shiv+", P::Hand, A::HandSizeAtPlay)),
        ],
    );
    assert!(storm_plus.complex_hook.is_none());

    let mut engine = engine_without_start(
        make_deck(&["Strike", "Strike", "Strike", "Strike"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Storm of Steel", "Strike", "Defend"]);

    assert!(play_self(&mut engine, "Storm of Steel"));

    assert_eq!(hand_count(&engine, "Shiv"), 2);
    assert_eq!(discard_prefix_count(&engine, "Strike"), 1);
    assert_eq!(discard_prefix_count(&engine, "Defend"), 1);

    let mut capped = engine_without_start(
        make_deck(&["Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut capped);
    capped.state.hand = make_deck(&[
        "Storm of Steel+",
        "Reflex",
        "Strike",
        "Strike",
        "Strike",
        "Defend",
        "Defend",
        "Defend",
        "Neutralize",
        "Survivor",
    ]);
    capped.state.draw_pile = make_deck(&["Strike", "Defend"]);
    assert!(play_self(&mut capped, "Storm of Steel+"));
    assert_eq!(hand_count(&capped, "Shiv+"), 9);
    assert_eq!(capped.state.hand.len(), 10);
    assert_eq!(capped.state.draw_pile.len(), 1);
    assert_eq!(capped.state.player.status(sid::DISCARDED_THIS_TURN), 9);
}
