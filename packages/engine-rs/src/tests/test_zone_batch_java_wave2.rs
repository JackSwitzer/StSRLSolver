#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Purity.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretTechnique.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java

use crate::actions::Action;
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, force_player_turn, hand_count, make_deck, play_on_enemy,
    play_self, TEST_SEED,
};

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
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
fn headbutt_moves_the_selected_discard_card_to_the_top_of_draw() {
    let mut engine = engine_for(
        &["Headbutt"],
        &["Shrug It Off"],
        &["Strike_R", "Defend_R"],
        3,
    );

    assert!(play_on_enemy(&mut engine, "Headbutt", 0));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::PickFromDiscard);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile.last().expect("top draw").def_id),
        "Defend_R"
    );
}

#[test]
fn true_grit_plus_uses_the_choice_surface_to_exhaust_the_selected_card() {
    let mut engine = engine_for(
        &["True Grit+", "Strike_R", "Defend_R"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "True Grit+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::ExhaustFromHand);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_count(&engine, "Strike_R"), 1);
    assert_eq!(hand_count(&engine, "Defend_R"), 0);
    assert!(engine
        .state
        .exhaust_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend_R"));
}

#[test]
fn burning_pact_exhausts_selected_card_then_draws_after_resolution() {
    let mut engine = engine_for(
        &["Burning Pact", "Strike_R"],
        &["Defend_R", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Burning Pact"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::ExhaustFromHand);

    engine.execute_action(&Action::Choose(0));

    let names = hand_names(&engine);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Defend_R".to_string()));
    assert!(names.contains(&"Bash".to_string()));
}

#[test]
fn second_wind_exhausts_all_non_attacks_and_triggers_exhaust_hooks_per_card() {
    let mut engine = engine_for(
        &["Second Wind", "Defend_R", "Battle Trance", "Strike_R"],
        &[],
        &[],
        3,
    );
    engine.state.player.set_status(sid::FEEL_NO_PAIN, 3);

    assert!(play_self(&mut engine, "Second Wind"));

    assert_eq!(hand_names(&engine), vec!["Strike_R"]);
    assert_eq!(engine.state.exhaust_pile.len(), 2);
    assert_eq!(engine.state.player.block, 16);
}

#[test]
fn fiend_fire_exhausts_the_hand_and_fires_exhaust_triggers_for_each_card() {
    let mut engine = engine_for(
        &["Fiend Fire", "Strike_R", "Defend_R"],
        &["Bash", "Shrug It Off"],
        &[],
        3,
    );
    engine.state.player.set_status(sid::DARK_EMBRACE, 1);
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Fiend Fire", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 14);
    assert_eq!(engine.state.exhaust_pile.len(), 3);
    assert_eq!(engine.state.hand.len(), 2);
    assert!(hand_names(&engine).contains(&"Bash".to_string()));
    assert!(hand_names(&engine).contains(&"Shrug It Off".to_string()));
}

#[test]
fn storm_of_steel_discards_the_hand_and_adds_one_shiv_per_discarded_card() {
    let mut engine = engine_for(
        &["Storm of Steel", "Strike_G", "Defend_G"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Storm of Steel"));

    assert_eq!(hand_count(&engine, "Shiv"), 2);
    assert_eq!(engine.state.discard_pile.len(), 3);
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Strike_G"));
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend_G"));
}

#[test]
fn purity_uses_zero_to_many_exhaust_selection_up_to_its_cap() {
    let mut engine = engine_for(
        &["Purity", "Strike_R", "Defend_R", "Bash"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Purity"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("purity choice");
    assert_eq!(choice.reason, ChoiceReason::ExhaustFromHand);
    assert_eq!(choice.min_picks, 0);
    assert_eq!(choice.max_picks, 3);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.exhaust_pile.len(), 2);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn secret_technique_opens_a_skill_only_draw_pile_search_choice() {
    let mut engine = engine_for(
        &["Secret Technique"],
        &["Strike_R", "Shrug It Off", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Technique"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("secret technique choice");
    assert_eq!(choice.reason, ChoiceReason::SearchDrawPile);
    assert_eq!(choice.options.len(), 1);

    let name = match choice.options[0] {
        ChoiceOption::DrawCard(idx) => engine.card_registry.card_name(engine.state.draw_pile[idx].def_id).to_string(),
        _ => panic!("secret technique should expose draw-pile card options"),
    };
    assert_eq!(name, "Shrug It Off");
}

#[test]
#[ignore = "Secret Technique can_use legality still lives outside this slice; add a Java-backed engine-path illegal-play test once the shared can_play surface is migrated"]
fn secret_technique_should_be_unplayable_with_no_skill_in_draw_pile() {
    let mut engine = engine_for(
        &["Secret Technique"],
        &["Strike_R", "Bash"],
        &[],
        3,
    );

    assert!(!play_self(&mut engine, "Secret Technique"));
}

#[test]
fn violence_draws_only_attacks_from_draw_pile_up_to_its_cap() {
    let mut engine = engine_for(
        &["Violence"],
        &["Shrug It Off", "Strike_R", "Bash", "Defend_R"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Violence"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| name == "Strike_R" || name == "Bash"));
    assert!(engine
        .state
        .draw_pile
        .iter()
        .all(|card| engine.card_registry.card_def_by_id(card.def_id).card_type != crate::cards::CardType::Attack));
}
