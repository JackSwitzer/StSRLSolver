use std::thread;
use std::time::Duration;

use crate::actions::Action;
use crate::search::{search_combat_puct, stable_combat_actions, CombatPuctLeafEvaluationV1};
use crate::tests::support::{engine_with, make_deck};
use crate::training_contract::{
    CombatOutcomeVectorV1, CombatPuctConfigV1, CombatSearchStopReasonV1,
};

fn test_execution_id(action: &Action) -> i32 {
    match action {
        Action::EndTurn => 9_999,
        Action::ConfirmSelection => 8_888,
        Action::Choose(choice_idx) => 7_000 + *choice_idx as i32,
        Action::UsePotion {
            potion_idx,
            target_idx,
        } => 6_000 + (*potion_idx as i32 * 16) + (*target_idx + 1),
        Action::PlayCard {
            card_idx,
            target_idx,
        } => 1_000 + (*card_idx as i32 * 16) + (*target_idx + 1),
    }
}

fn biased_leaf_eval(
    solve_probability: f32,
) -> impl FnMut(&crate::training_contract::CombatTrainingStateV1) -> Result<CombatPuctLeafEvaluationV1, ()> {
    move |state| {
        let mut priors = vec![0.05; state.legal_candidates.len()];
        if let Some(first) = priors.first_mut() {
            *first = 0.85;
        }
        Ok(CombatPuctLeafEvaluationV1 {
            candidate_priors: priors,
            outcome: CombatOutcomeVectorV1 {
                solve_probability,
                expected_hp_loss: 2.0,
                expected_turns: 3.0,
                potion_cost: 0.0,
                setup_value_delta: 0.0,
                persistent_scaling_delta: 0.0,
            },
        })
    }
}

#[test]
fn combat_puct_is_deterministic_and_replayable() {
    let engine = engine_with(make_deck(&["Strike", "Defend", "Strike"]), 18, 5);
    let mut config = CombatPuctConfigV1::hallway_default();
    config.min_visits = 16;
    config.visit_window = 4;
    config.hard_visit_cap = 64;
    config.stable_windows_required = 2;
    let first = search_combat_puct(&engine, config.clone(), test_execution_id, biased_leaf_eval(0.8))
        .expect("puct should evaluate");
    let second = search_combat_puct(&engine, config, test_execution_id, biased_leaf_eval(0.8))
        .expect("puct should evaluate");

    assert_eq!(first, second);
    assert_eq!(first.stop_reason, CombatSearchStopReasonV1::Converged);
    assert!(first.root_total_visits >= 16);
    assert_eq!(
        first.root_action_ids,
        stable_combat_actions(&engine)
            .iter()
            .map(test_execution_id)
            .collect::<Vec<_>>()
    );

    let chosen_id = first.chosen_action_id.expect("expected chosen action");
    let action = stable_combat_actions(&engine)
        .into_iter()
        .find(|candidate| test_execution_id(candidate) == chosen_id)
        .expect("chosen action should map back to a legal action");
    let mut replay = engine.clone_state();
    replay.execute_action(&action);
}

#[test]
fn combat_puct_terminal_root_uses_terminal_override() {
    let mut engine = engine_with(make_deck(&["Strike"]), 1, 0);
    engine.state.combat_over = true;
    engine.state.player_won = true;
    let result = search_combat_puct(
        &engine,
        CombatPuctConfigV1::hallway_default(),
        test_execution_id,
        |_state| -> Result<CombatPuctLeafEvaluationV1, ()> {
            panic!("terminal roots should not call the evaluator");
        },
    )
    .expect("terminal roots should still return a result");

    assert_eq!(result.stop_reason, CombatSearchStopReasonV1::TerminalRoot);
    assert_eq!(result.leaf_evaluations, 0);
    assert_eq!(result.root_outcome.solve_probability, 1.0);
}

#[test]
fn combat_puct_converges_under_hallway_elite_and_boss_budgets() {
    for config in [
        CombatPuctConfigV1::hallway_default(),
        CombatPuctConfigV1::elite_default(),
        CombatPuctConfigV1::boss_default(),
    ] {
        let engine = engine_with(make_deck(&["Strike", "Defend"]), 20, 5);
        let result =
            search_combat_puct(&engine, config.clone(), test_execution_id, biased_leaf_eval(0.9))
                .expect("puct should evaluate");
        assert_eq!(result.stop_reason, CombatSearchStopReasonV1::Converged);
        assert!(result.root_total_visits >= config.min_visits);
        assert!(!result.frontier.is_empty());
    }
}

#[test]
fn combat_puct_reports_hard_cap_when_convergence_is_disabled() {
    let engine = engine_with(make_deck(&["Strike", "Defend", "Strike"]), 22, 5);
    let config = CombatPuctConfigV1 {
        min_visits: 64,
        visit_window: 8,
        hard_visit_cap: 12,
        stable_windows_required: 99,
        ..CombatPuctConfigV1::hallway_default()
    };
    let result =
        search_combat_puct(&engine, config, test_execution_id, biased_leaf_eval(0.7)).expect(
            "hard-cap configuration should still return a result",
        );

    assert_eq!(result.stop_reason, CombatSearchStopReasonV1::HardVisitCap);
    assert!(result.root_total_visits >= 12);
}

#[test]
fn combat_puct_reports_time_cap_when_leaf_eval_is_slow() {
    let engine = engine_with(make_deck(&["Strike", "Defend", "Strike"]), 24, 6);
    let config = CombatPuctConfigV1 {
        min_visits: 1_000,
        visit_window: 64,
        hard_visit_cap: 2_000,
        time_cap_ms: 1,
        stable_windows_required: 99,
        ..CombatPuctConfigV1::hallway_default()
    };
    let result = search_combat_puct(
        &engine,
        config,
        test_execution_id,
        |state| -> Result<CombatPuctLeafEvaluationV1, ()> {
            thread::sleep(Duration::from_millis(2));
            let mut priors = vec![0.1; state.legal_candidates.len()];
            if let Some(first) = priors.first_mut() {
                *first = 0.7;
            }
            Ok(CombatPuctLeafEvaluationV1 {
                candidate_priors: priors,
                outcome: CombatOutcomeVectorV1 {
                    solve_probability: 0.6,
                    expected_hp_loss: 3.0,
                    expected_turns: 4.0,
                    potion_cost: 0.0,
                    setup_value_delta: 0.0,
                    persistent_scaling_delta: 0.0,
                },
            })
        },
    )
    .expect("time-cap configuration should still return a result");

    assert_eq!(result.stop_reason, CombatSearchStopReasonV1::TimeCap);
    assert!(result.leaf_evaluations >= 1);
}
