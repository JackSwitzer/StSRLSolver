use crate::search::{
    combat_state_hash, run_seed_suite, run_state_hash, search_combat, search_run, SearchBudget,
    SeedSuite,
};
use crate::tests::support::{engine_with, run_engine};

#[test]
fn combat_search_is_deterministic_and_replayable() {
    let engine = engine_with(crate::tests::support::make_deck(&["Strike_R", "Defend_R"]), 20, 5);
    let budget = SearchBudget::new(32, 4);

    let first = search_combat(&engine, budget);
    let second = search_combat(&engine, budget);

    assert_eq!(first, second);
    assert!(first.explored_nodes > 0);

    let mut replay = engine.clone_state();
    for action in &first.actions {
        assert!(
            replay.get_legal_actions().contains(action),
            "search returned illegal combat action: {action:?}"
        );
        replay.execute_action(action);
    }
}

#[test]
fn run_search_is_deterministic_and_replayable() {
    let engine = run_engine(42, 20);
    let budget = SearchBudget::new(64, 5);

    let first = search_run(&engine, budget);
    let second = search_run(&engine, budget);

    assert_eq!(first, second);
    assert!(first.explored_nodes > 0);
    assert!(!first.actions.is_empty(), "bounded search should emit a candidate path");

    let mut replay = engine.clone();
    for action in &first.actions {
        assert!(
            replay.get_legal_decision_actions().contains(action),
            "search returned illegal run decision: {action:?}"
        );
        let step = replay.step_with_result(&action.to_run_action());
        assert!(step.action_accepted, "planner path should replay cleanly");
    }
}

#[test]
fn seed_suite_report_is_stable_for_identical_inputs() {
    let suite = SeedSuite::new(vec![42, 99, 12345]);
    let budget = SearchBudget::new(48, 4);

    let first = run_seed_suite(&suite, 20, budget);
    let second = run_seed_suite(&suite, 20, budget);

    assert_eq!(first, second);
    assert_eq!(first.entries.len(), 3);
    assert_eq!(
        first.total_nodes,
        first.entries.iter().map(|entry| entry.result.explored_nodes).sum::<usize>()
    );
}

#[test]
fn combat_state_hash_is_stable_for_identical_snapshots_and_changes_on_branch() {
    let engine = engine_with(crate::tests::support::make_deck(&["Strike_R", "Defend_R"]), 20, 5);
    let clone = engine.clone_state();

    assert_eq!(combat_state_hash(&engine), combat_state_hash(&clone));

    let mut branched = clone;
    let action = branched
        .get_legal_actions()
        .into_iter()
        .next()
        .expect("expected a legal combat action");
    branched.execute_action(&action);

    assert_ne!(combat_state_hash(&engine), combat_state_hash(&branched));
}

#[test]
fn run_state_hash_is_stable_for_identical_snapshots_and_changes_on_branch() {
    let engine = run_engine(42, 20);
    let clone = engine.clone();

    assert_eq!(run_state_hash(&engine), run_state_hash(&clone));

    let mut branched = clone;
    branched.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

    assert_ne!(run_state_hash(&engine), run_state_hash(&branched));
}
