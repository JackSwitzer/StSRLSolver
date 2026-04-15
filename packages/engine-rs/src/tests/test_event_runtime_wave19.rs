use crate::events::{typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};
use std::collections::HashMap;

const CURSE_IDS: &[&str] = &[
    "AscendersBane",
    "Clumsy",
    "CurseOfTheBell",
    "Decay",
    "Doubt",
    "Injury",
    "Necronomicurse",
    "Normality",
    "Pain",
    "Parasite",
    "Pride",
    "Regret",
    "Shame",
    "Writhe",
];

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

fn enter_match_and_keep(engine: &mut RunEngine) {
    engine.debug_set_typed_event_state(typed_shrine_event("Match and Keep!"));

    let intro = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(intro.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);

    let rules = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(rules.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
}

#[test]
fn match_and_keep_is_supported_in_the_typed_catalog() {
    let event = typed_shrine_event("Match and Keep!");
    assert!(matches!(event.options[0].status, EventRuntimeStatus::Supported));
}

#[test]
fn match_and_keep_reveals_the_selected_card_by_index() {
    let mut engine = RunEngine::new(421, 20);
    enter_match_and_keep(&mut engine);

    let board = engine
        .debug_match_and_keep_board()
        .expect("match board should exist");
    assert_eq!(board.len(), 12);

    let reveal = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(reveal.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);

    let ctx = engine.current_decision_context();
    let event = ctx.event.expect("match event context");
    let expected_label = crate::gameplay::global_registry()
        .card(&board[0])
        .map(|def| def.name.clone())
        .unwrap_or_else(|| board[0].clone());
    assert!(event.options[0].label.contains("Revealed slot 1"));
    assert!(event.options[0].label.contains(&expected_label));
}

#[test]
fn match_and_keep_board_starts_with_six_pairs_and_five_attempts() {
    let mut engine = RunEngine::new(421, 10);
    enter_match_and_keep(&mut engine);

    let board = engine
        .debug_match_and_keep_board()
        .expect("match board should exist");
    let mut counts: HashMap<String, usize> = HashMap::new();
    for card_id in board {
        *counts.entry(card_id).or_insert(0) += 1;
    }

    assert_eq!(counts.len(), 6);
    assert!(counts.values().all(|count| *count == 2));
    assert_eq!(counts.get("Eruption"), Some(&2));
    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(5));
}

#[test]
fn match_and_keep_uses_the_current_run_class_for_the_starter_pair() {
    let mut engine = RunEngine::new(421, 10);
    engine.run_state.deck = vec![
        "Strike_G".to_string(),
        "Strike_G".to_string(),
        "Strike_G".to_string(),
        "Strike_G".to_string(),
        "Defend_G".to_string(),
        "Defend_G".to_string(),
        "Defend_G".to_string(),
        "Defend_G".to_string(),
        "Neutralize".to_string(),
        "Survivor".to_string(),
    ];
    engine.run_state.relics = vec!["Ring of the Snake".to_string()];
    enter_match_and_keep(&mut engine);

    let board = engine
        .debug_match_and_keep_board()
        .expect("match board should exist");
    let mut counts: HashMap<String, usize> = HashMap::new();
    for card_id in board {
        *counts.entry(card_id).or_insert(0) += 1;
    }

    assert_eq!(counts.get("Neutralize"), Some(&2));
    assert!(!counts.contains_key("Eruption"));
}

#[test]
fn match_and_keep_ascension_fifteen_upgrades_the_board_to_two_curse_pairs() {
    let mut low_asc_engine = RunEngine::new(421, 0);
    enter_match_and_keep(&mut low_asc_engine);
    let low_asc_curse_count = low_asc_engine
        .debug_match_and_keep_board()
        .expect("low ascension board should exist")
        .into_iter()
        .filter(|card_id| CURSE_IDS.contains(&card_id.as_str()))
        .count();

    let mut high_asc_engine = RunEngine::new(421, 20);
    enter_match_and_keep(&mut high_asc_engine);
    let high_asc_curse_count = high_asc_engine
        .debug_match_and_keep_board()
        .expect("high ascension board should exist")
        .into_iter()
        .filter(|card_id| CURSE_IDS.contains(&card_id.as_str()))
        .count();

    assert_eq!(low_asc_curse_count, 2);
    assert_eq!(high_asc_curse_count, 4);
}

#[test]
fn match_and_keep_mismatch_then_match_consumes_attempts_and_adds_only_the_matched_card() {
    let mut engine = RunEngine::new(421, 20);
    let deck_before = engine.run_state.deck.len();
    enter_match_and_keep(&mut engine);

    let board = engine
        .debug_match_and_keep_board()
        .expect("match board should exist");

    let mismatch_pair = (0..board.len())
        .find_map(|i| {
            (0..board.len())
                .find(|&j| i != j && board[i] != board[j])
                .map(|j| (i, j))
        })
        .expect("board should contain a mismatch pair");
    let match_pair = (0..board.len())
        .find_map(|i| {
            ((i + 1)..board.len())
                .find(|&j| board[i] == board[j])
                .map(|j| (i, j, board[i].clone()))
        })
        .expect("board should contain a match pair");

    let first = engine.step_with_result(&RunAction::EventChoice(mismatch_pair.0));
    assert!(first.action_accepted);
    let second = engine.step_with_result(&RunAction::EventChoice(mismatch_pair.1));
    assert!(second.action_accepted);
    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(4));
    assert_eq!(engine.run_state.deck.len(), deck_before);
    assert_eq!(
        engine.debug_match_and_keep_board().expect("board after mismatch").len(),
        12
    );

    let third = engine.step_with_result(&RunAction::EventChoice(match_pair.0));
    assert!(third.action_accepted);
    let fourth = engine.step_with_result(&RunAction::EventChoice(match_pair.1));
    assert!(fourth.action_accepted);
    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(3));
    assert_eq!(engine.run_state.deck.len(), deck_before + 1);
    assert_eq!(
        engine.debug_match_and_keep_board().expect("board after match").len(),
        10
    );
    assert_eq!(engine.run_state.deck.last(), Some(&match_pair.2));
}

#[test]
fn match_and_keep_exits_after_five_failed_attempts() {
    let mut engine = RunEngine::new(421, 20);
    enter_match_and_keep(&mut engine);

    let board = engine
        .debug_match_and_keep_board()
        .expect("match board should exist");
    let mismatch_pair = (0..board.len())
        .find_map(|i| {
            (0..board.len())
                .find(|&j| i != j && board[i] != board[j])
                .map(|j| (i, j))
        })
        .expect("board should contain a mismatch pair");

    for _ in 0..5 {
        let first = engine.step_with_result(&RunAction::EventChoice(mismatch_pair.0));
        assert!(first.action_accepted);
        let second = engine.step_with_result(&RunAction::EventChoice(mismatch_pair.1));
        assert!(second.action_accepted);
    }

    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(0));
    assert_eq!(engine.event_option_count(), 1);

    let leave = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(leave.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
}
