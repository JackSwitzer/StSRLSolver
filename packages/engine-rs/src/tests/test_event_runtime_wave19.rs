use crate::events::{typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::checkpoint::CoreCheckpoint;
use crate::run::{GameAction, RunEngine, RunPhase};
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

    let intro = engine.step_game(&GameAction::EventChoice(0));
    assert!(intro.accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);

    let rules = engine.step_game(&GameAction::EventChoice(0));
    assert!(rules.accepted());
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

    let reveal = engine.step_game(&GameAction::EventChoice(0));
    assert!(reveal.accepted());
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
        "Strike".to_string(),
        "Strike".to_string(),
        "Strike".to_string(),
        "Strike".to_string(),
        "Defend".to_string(),
        "Defend".to_string(),
        "Defend".to_string(),
        "Defend".to_string(),
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
fn match_and_keep_checkpoint_preserves_reveal_board_pools_and_rng_continuation() {
    // GremlinMatchGame is a multi-action event. A causal checkpoint after the
    // first reveal must preserve the mutable colorless-pool shuffle, the board,
    // attempts, pending pick, and all RNG stream states.
    let mut original = RunEngine::new(811, 0);
    enter_match_and_keep(&mut original);
    let board = original.debug_match_and_keep_board().expect("match board");
    let first = 0;
    let mate = board
        .iter()
        .enumerate()
        .find_map(|(index, card)| (index != first && card == &board[first]).then_some(index))
        .expect("paired card");
    assert!(original.step_game(&GameAction::EventChoice(first)).accepted());

    let checkpoint = CoreCheckpoint::capture(&original).expect("quiescent event decision");
    let json = checkpoint.to_json().expect("serialize match checkpoint");
    let mut restored = CoreCheckpoint::from_json(&json)
        .expect("deserialize match checkpoint")
        .restore()
        .expect("restore match checkpoint");
    assert_eq!(restored.get_legal_actions(), original.get_legal_actions());
    assert_eq!(restored.debug_match_and_keep_board(), original.debug_match_and_keep_board());
    assert_eq!(restored.rng_counters(), original.rng_counters());

    assert!(original.step_game(&GameAction::EventChoice(mate)).accepted());
    assert!(restored.step_game(&GameAction::EventChoice(mate)).accepted());
    assert_eq!(
        CoreCheckpoint::capture(&restored).unwrap(),
        CoreCheckpoint::capture(&original).unwrap()
    );
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

    let first = engine.step_game(&GameAction::EventChoice(mismatch_pair.0));
    assert!(first.accepted());
    let second = engine.step_game(&GameAction::EventChoice(mismatch_pair.1));
    assert!(second.accepted());
    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(4));
    assert_eq!(engine.run_state.deck.len(), deck_before);
    assert_eq!(
        engine.debug_match_and_keep_board().expect("board after mismatch").len(),
        12
    );

    let third = engine.step_game(&GameAction::EventChoice(match_pair.0));
    assert!(third.accepted());
    let fourth = engine.step_game(&GameAction::EventChoice(match_pair.1));
    assert!(fourth.accepted());
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
        let first = engine.step_game(&GameAction::EventChoice(mismatch_pair.0));
        assert!(first.accepted());
        let second = engine.step_game(&GameAction::EventChoice(mismatch_pair.1));
        assert!(second.accepted());
    }

    assert_eq!(engine.debug_match_and_keep_attempts_left(), Some(0));
    assert_eq!(engine.event_option_count(), 1);

    let leave = engine.step_game(&GameAction::EventChoice(0));
    assert!(leave.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
}
