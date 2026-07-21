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
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretWeapon.java
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

fn pile_names(engine: &CombatEngine, pile: &[crate::combat_types::CardInstance]) -> Vec<String> {
    pile.iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn headbutt_moves_the_selected_discard_card_to_the_top_of_draw() {
    let mut engine = engine_for(
        &["Headbutt"],
        &["Shrug It Off"],
        &["Strike", "Defend"],
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
        "Defend"
    );
}

#[test]
fn true_grit_plus_uses_the_choice_surface_to_exhaust_the_selected_card() {
    let mut engine = engine_for(
        &["True Grit+", "Strike", "Defend"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "True Grit+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::ExhaustFromHand);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert!(engine
        .state
        .exhaust_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend"));
}

#[test]
fn burning_pact_exhausts_selected_card_then_draws_after_resolution() {
    // ExhaustAction auto-exhausts a singleton hand; keep two cards here so
    // the source-defined manual-selection branch is exercised.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
    let mut engine = engine_for(
        &["Burning Pact", "Strike", "Anger"],
        &["Defend", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Burning Pact"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::ExhaustFromHand);

    engine.execute_action(&Action::Choose(0));

    let names = hand_names(&engine);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"Anger".to_string()));
    assert!(names.contains(&"Defend".to_string()));
    assert!(names.contains(&"Bash".to_string()));
}

#[test]
fn second_wind_exhausts_all_non_attacks_and_triggers_exhaust_hooks_per_card() {
    let mut engine = engine_for(
        &["Second Wind", "Defend", "Battle Trance", "Strike"],
        &[],
        &[],
        3,
    );
    engine.state.player.set_status(sid::FEEL_NO_PAIN, 3);

    assert!(play_self(&mut engine, "Second Wind"));

    assert_eq!(hand_names(&engine), vec!["Strike"]);
    assert_eq!(engine.state.exhaust_pile.len(), 2);
    assert_eq!(engine.state.player.block, 16);
}

#[test]
fn fiend_fire_exhausts_the_hand_and_fires_exhaust_triggers_for_each_card() {
    // FiendFireAction queues one random ExhaustAction per remaining hand card;
    // ExhaustAction.getRandomCard consumes cardRandomRng even at random(0).
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FiendFireAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
    let mut engine = engine_for(
        &["Fiend Fire", "Strike", "Defend"],
        &["Bash", "Shrug It Off"],
        &[],
        3,
    );
    engine.state.player.set_status(sid::DARK_EMBRACE, 1);
    let hp_before = engine.state.enemies[0].entity.hp;
    let rng_before = engine.card_random_rng.counter;
    let mut oracle = engine.card_random_rng.copy();
    let mut candidates = vec!["Strike", "Defend"];
    let mut expected_exhaust_order = Vec::new();
    while !candidates.is_empty() {
        let index = oracle.random_int((candidates.len() - 1) as i32) as usize;
        expected_exhaust_order.push(candidates.remove(index));
    }

    assert!(play_on_enemy(&mut engine, "Fiend Fire", 0));

    assert_eq!(engine.card_random_rng.counter - rng_before, 2);
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 14);
    assert_eq!(engine.state.exhaust_pile.len(), 3);
    let actual_exhaust_order: Vec<_> = engine.state.exhaust_pile[..2]
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(actual_exhaust_order, expected_exhaust_order);
    assert_eq!(engine.state.hand.len(), 2);
    assert!(hand_names(&engine).contains(&"Bash".to_string()));
    assert!(hand_names(&engine).contains(&"Shrug It Off".to_string()));
}

#[test]
fn storm_of_steel_discards_the_hand_and_adds_one_shiv_per_discarded_card() {
    let mut engine = engine_for(
        &["Storm of Steel", "Strike", "Defend"],
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
        .any(|card| engine.card_registry.card_name(card.def_id) == "Strike"));
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend"));
}

#[test]
fn purity_uses_zero_to_many_exhaust_selection_up_to_its_cap() {
    let mut engine = engine_for(
        &["Purity", "Strike", "Defend", "Bash"],
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
    assert_eq!(engine.state.exhaust_pile.len(), 3);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn secret_technique_auto_moves_the_only_skill_and_exhausts() {
    let mut engine = engine_for(
        &["Secret Technique"],
        &["Strike", "Shrug It Off", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Technique"));
    // Java SkillFromDeckToHandAction moves a singleton directly instead of
    // opening grid select.
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert!(engine.choice.is_none());
    assert_eq!(hand_names(&engine), vec!["Shrug It Off"]);
    assert_eq!(pile_names(&engine, &engine.state.draw_pile), vec!["Strike", "Bash"]);
    assert_eq!(pile_names(&engine, &engine.state.exhaust_pile), vec!["Secret Technique"]);
}

#[test]
fn upgraded_secret_technique_auto_moves_the_only_skill_without_exhausting() {
    let mut engine = engine_for(
        &["Secret Technique+"],
        &["Strike", "Shrug It Off", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Technique+"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_names(&engine), vec!["Shrug It Off"]);
    assert_eq!(pile_names(&engine, &engine.state.discard_pile), vec!["Secret Technique+"]);
    assert!(engine.state.exhaust_pile.is_empty());
}

#[test]
fn secret_weapon_auto_moves_the_only_attack_and_exhausts() {
    let mut engine = engine_for(
        &["Secret Weapon"],
        &["Defend", "Strike", "Shrug It Off"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Weapon"));
    // Java AttackFromDeckToHandAction moves a singleton directly instead of
    // opening grid select.
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert!(engine.choice.is_none());
    assert_eq!(hand_names(&engine), vec!["Strike"]);
    assert_eq!(
        pile_names(&engine, &engine.state.draw_pile),
        vec!["Defend", "Shrug It Off"]
    );
    assert_eq!(
        pile_names(&engine, &engine.state.exhaust_pile),
        vec!["Secret Weapon"]
    );
}

#[test]
fn upgraded_secret_weapon_auto_moves_the_only_attack_without_exhausting() {
    let mut engine = engine_for(
        &["Secret Weapon+"],
        &["Defend", "Strike", "Shrug It Off"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Weapon+"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_names(&engine), vec!["Strike"]);
    assert_eq!(
        pile_names(&engine, &engine.state.discard_pile),
        vec!["Secret Weapon+"]
    );
    assert!(engine.state.exhaust_pile.is_empty());
}

#[test]
fn secret_weapon_requires_an_attack_and_only_offers_attacks() {
    let no_attack = engine_for(
        &["Secret Weapon"],
        &["Defend", "Shrug It Off"],
        &[],
        3,
    );
    let secret_idx = no_attack
        .state
        .hand
        .iter()
        .position(|card| no_attack.card_registry.card_name(card.def_id) == "Secret Weapon")
        .expect("Secret Weapon should be in hand");
    assert!(!no_attack.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. } if *card_idx == secret_idx
    )));

    let mut multiple = engine_for(
        &["Secret Weapon"],
        &["Strike", "Defend", "Bash", "Shrug It Off"],
        &[],
        3,
    );
    assert!(play_self(&mut multiple, "Secret Weapon"));
    assert_eq!(multiple.phase, CombatPhase::AwaitingChoice);
    let choice = multiple.choice.as_ref().expect("attack search choice");
    assert_eq!(choice.reason, ChoiceReason::SearchDrawPile);
    let mut offered: Vec<_> = choice
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::DrawCard(index) => multiple
                .card_registry
                .card_name(multiple.state.draw_pile[*index].def_id)
                .to_string(),
            _ => panic!("Secret Weapon should expose only draw-pile cards"),
        })
        .collect();
    offered.sort();
    assert_eq!(offered, vec!["Bash", "Strike"]);
}

#[test]
fn secret_technique_should_be_unplayable_with_no_skill_in_draw_pile() {
    let mut engine = engine_for(
        &["Secret Technique"],
        &["Strike", "Bash"],
        &[],
        3,
    );

    let secret_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "Secret Technique")
        .expect("Secret Technique should be in hand");

    assert!(!engine.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. } if *card_idx == secret_idx
    )));

    engine.execute_action(&Action::PlayCard {
        card_idx: secret_idx,
        target_idx: -1,
    });
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_names(&engine), vec!["Secret Technique"]);
}

#[test]
fn violence_draws_only_attacks_from_draw_pile_up_to_its_cap() {
    let mut engine = engine_for(
        &["Violence"],
        &["Shrug It Off", "Strike", "Bash", "Defend"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Violence"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| name == "Strike" || name == "Bash"));
    assert!(engine
        .state
        .draw_pile
        .iter()
        .all(|card| engine.card_registry.card_def_by_id(card.def_id).card_type != crate::cards::CardType::Attack));
}
