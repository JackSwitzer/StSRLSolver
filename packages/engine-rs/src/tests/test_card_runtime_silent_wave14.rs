#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CalculatedGambleAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Concentrate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/BladeFuryAction.java

use crate::cards::global_registry;
use crate::actions::Action;
use crate::effects::declarative::{AmountSource as A, BulkAction, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE};
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
    assert_eq!(
        calculated_gamble_plus.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::DrawCards(A::HandSizeAtPlayPlus(1))),
        ],
    );
    assert!(calculated_gamble_plus.complex_hook.is_none());
    assert!(!calculated_gamble_plus.exhaust);

    let mut engine = engine_without_start(
        make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Calculated Gamble", "Strike_G", "Strike_G", "Strike_G"]);
    engine.state.draw_pile = make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]);

    assert!(play_self(&mut engine, "Calculated Gamble"));

    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(discard_prefix_count(&engine, "Strike_G"), 3);
}

#[test]
fn silent_wave14_concentrate_is_declarative_discard_for_effect() {
    let registry = global_registry();
    let concentrate = registry.get("Concentrate").expect("Concentrate should exist");
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

    let concentrate_plus = registry.get("Concentrate+").expect("Concentrate+ should exist");
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
        make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 3;
    engine.state.hand = make_deck(&["Concentrate", "Strike_G", "Strike_G", "Strike_G"]);

    assert!(play_self(&mut engine, "Concentrate"));
    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.energy, 5);
    assert_eq!(engine.state.hand.len(), 0);
    assert_eq!(discard_prefix_count(&engine, "Strike_G"), 3);
}

#[test]
fn silent_wave14_storm_of_steel_bulk_discards_hand_and_adds_shivs() {
    let registry = global_registry();
    let storm = registry.get("Storm of Steel").expect("Storm of Steel should exist");
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

    let storm_plus = registry.get("Storm of Steel+").expect("Storm of Steel+ should exist");
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
        make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Storm of Steel", "Strike_G", "Defend_G"]);

    assert!(play_self(&mut engine, "Storm of Steel"));

    assert_eq!(hand_count(&engine, "Shiv"), 2);
    assert_eq!(discard_prefix_count(&engine, "Strike_G"), 1);
    assert_eq!(discard_prefix_count(&engine, "Defend_G"), 1);
}
