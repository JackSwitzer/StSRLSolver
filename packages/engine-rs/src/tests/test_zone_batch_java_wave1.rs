#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Seek.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java

use crate::actions::Action;
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, CombatPhase};
use crate::tests::support::{
    combat_state_with, enemy_no_intent, force_player_turn, hand_count, make_deck, play_on_enemy,
    play_self, TEST_SEED,
};

fn engine_for(
    hand: &[&str],
    draw: &[&str],
    discard: &[&str],
    energy: i32,
) -> CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
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
fn seek_plus_moves_two_chosen_cards_from_draw_pile_to_hand() {
    let mut engine = engine_for(
        &["Seek+"],
        &["Zap", "Defend", "Strike"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Seek+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("seek choice");
    assert_eq!(choice.reason, ChoiceReason::SearchDrawPile);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 2);
    assert_eq!(choice.options.len(), 3);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_count(&engine, "Zap"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.draw_pile[0].def_id), "Defend");
}

#[test]
fn headbutt_moves_a_chosen_discard_card_to_the_top_of_draw() {
    let mut engine = engine_for(
        &["Headbutt"],
        &["Shrug It Off"],
        &["Strike", "Defend"],
        3,
    );

    assert!(play_on_enemy(&mut engine, "Headbutt", 0));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("headbutt choice");
    assert_eq!(choice.reason, ChoiceReason::PickFromDiscard);

    let selected_name = match choice.options[1] {
        ChoiceOption::DiscardCard(idx) => engine.card_registry.card_name(engine.state.discard_pile[idx].def_id).to_string(),
        _ => panic!("headbutt should expose discard-pile options"),
    };
    assert_eq!(selected_name, "Defend");

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine.card_registry.card_name(engine.state.draw_pile.last().expect("top draw").def_id),
        "Defend"
    );
    assert_eq!(hand_count(&engine, "Defend"), 0);
}

#[test]
fn dual_wield_only_offers_attack_and_power_cards_then_copies_the_selected_card() {
    let mut engine = engine_for(
        &["Dual Wield+", "Strike", "Defend", "Inflame"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Dual Wield+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("dual wield choice");
    assert_eq!(choice.reason, ChoiceReason::DualWield);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 1);
    assert_eq!(choice.options.len(), 2);

    let option_names: Vec<String> = choice
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::HandCard(idx) => engine.card_registry.card_name(engine.state.hand[*idx].def_id).to_string(),
            _ => panic!("dual wield should expose hand-card options"),
        })
        .collect();
    assert_eq!(option_names, vec!["Strike", "Inflame"]);

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_count(&engine, "Strike"), 3);
    assert_eq!(hand_count(&engine, "Inflame"), 1);
    assert_eq!(hand_count(&engine, "Defend"), 1);
}

#[test]
fn true_grit_plus_exhausts_the_selected_card_instead_of_a_random_one() {
    let mut engine = engine_for(
        &["True Grit+", "Strike", "Defend"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "True Grit+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("true grit choice");
    assert_eq!(choice.reason, ChoiceReason::ExhaustFromHand);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert_eq!(
        engine
            .state
            .exhaust_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Defend")
            .count(),
        1
    );
}

#[test]
fn burning_pact_exhausts_selected_card_then_draws_after_resolution() {
    let mut engine = engine_for(
        &["Burning Pact", "Strike"],
        &["Defend", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Burning Pact"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("burning pact choice");
    assert_eq!(choice.reason, ChoiceReason::ExhaustFromHand);
    assert_eq!(choice.options.len(), 1);

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Defend".to_string()));
    assert!(names.contains(&"Bash".to_string()));
    assert_eq!(
        engine
            .state
            .exhaust_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Strike")
            .count(),
        1
    );
}
