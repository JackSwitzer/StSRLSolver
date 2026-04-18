#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CalculatedGambleAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/NightmareAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/powers/NightmarePower.java

use crate::actions::Action;
use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
use crate::tests::support::{
    combat_state_with, enemy_no_intent, force_player_turn, hand_count, make_deck, play_self,
    TEST_SEED,
};

fn engine_for(
    hand: &[&str],
    draw: &[&str],
    enemies: Vec<crate::state::EnemyCombatState>,
    energy: i32,
) -> CombatEngine {
    let mut state = combat_state_with(make_deck(draw), enemies, energy);
    state.hand = make_deck(hand);
    let mut engine = CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

fn hand_names(engine: &CombatEngine) -> Vec<String> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn calculated_gamble_discards_the_remaining_hand_then_draws_the_same_count() {
    let mut engine = engine_for(
        &["Calculated Gamble", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Calculated Gamble"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| matches!(
        name.as_str(),
        "Neutralize" | "Survivor" | "Deflect"
    )));
    assert_eq!(hand_count(&engine, "Calculated Gamble"), 0);
    assert_eq!(hand_count(&engine, "Strike"), 0);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Strike"));
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend"));
    assert!(engine
        .state
        .exhaust_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Calculated Gamble"));
}

#[test]
fn calculated_gamble_plus_draws_one_extra_card_after_discarding_the_remaining_hand() {
    let mut engine = engine_for(
        &["Calculated Gamble+", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Calculated Gamble+"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 3);
    assert!(names.iter().all(|name| matches!(
        name.as_str(),
        "Neutralize" | "Survivor" | "Deflect"
    )));
    assert_eq!(hand_count(&engine, "Strike"), 0);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert_eq!(engine.state.discard_pile.len(), 3);
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Calculated Gamble+"));
}

#[test]
fn nightmare_opens_a_single_card_choice_but_delayed_next_turn_copies_need_a_runtime_primitive() {
    let mut engine = engine_for(
        &["Nightmare", "Strike", "Defend"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Nightmare"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("nightmare choice");
    assert_eq!(choice.reason, ChoiceReason::DualWield);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 1);
    assert_eq!(choice.options.len(), 2);
}

#[test]
fn nightmare_delayed_copies_should_appear_next_turn_not_immediately() {
    let mut engine = engine_for(
        &["Nightmare", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Nightmare"));
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Strike+"), 0);

    engine.execute_action(&Action::EndTurn);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine.state.hand.len(),
        8,
        "Java Nightmare would add the copies at start of turn before the normal draw"
    );
}
