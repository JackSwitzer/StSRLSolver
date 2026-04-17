//! Benchmarks for the combat engine — measures throughput for MCTS.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sts_engine::actions::Action;
use sts_engine::engine::CombatEngine;
use sts_engine::state::{CombatState, EnemyCombatState};

fn make_bench_state() -> CombatState {
    let deck: Vec<String> = (0..5)
        .map(|_| "Strike_P".to_string())
        .chain((0..4).map(|_| "Defend_P".to_string()))
        .chain(std::iter::once("Eruption".to_string()))
        .collect();

    let mut enemy = EnemyCombatState::new("JawWorm", 44, 44);
    enemy.set_move(1, 11, 1, 0);

    CombatState::new(80, 80, vec![enemy], deck, 3)
}

fn bench_start_combat(c: &mut Criterion) {
    c.bench_function("start_combat", |b| {
        b.iter(|| {
            let state = make_bench_state();
            let mut engine = CombatEngine::new(state, 42);
            engine.start_combat();
            black_box(&engine);
        });
    });
}

fn bench_full_turn(c: &mut Criterion) {
    c.bench_function("full_turn_cycle", |b| {
        b.iter(|| {
            let state = make_bench_state();
            let mut engine = CombatEngine::new(state, 42);
            engine.start_combat();

            // Play all affordable cards then end turn
            loop {
                let actions = engine.get_legal_actions();
                if actions.len() <= 1 {
                    // Only EndTurn left
                    break;
                }
                // Play first card action
                if let Some(card_action) = actions
                    .iter()
                    .find(|a| matches!(a, Action::PlayCard { .. }))
                {
                    engine.execute_action(card_action);
                } else {
                    break;
                }
            }
            engine.execute_action(&Action::EndTurn);
            black_box(&engine);
        });
    });
}

fn bench_clone_state(c: &mut Criterion) {
    let state = make_bench_state();
    let mut engine = CombatEngine::new(state, 42);
    engine.start_combat();

    c.bench_function("clone_for_mcts", |b| {
        b.iter(|| {
            let cloned = engine.clone_state();
            black_box(cloned);
        });
    });
}

fn bench_get_legal_actions(c: &mut Criterion) {
    let state = make_bench_state();
    let mut engine = CombatEngine::new(state, 42);
    engine.start_combat();

    c.bench_function("get_legal_actions", |b| {
        b.iter(|| {
            let actions = engine.get_legal_actions();
            black_box(actions);
        });
    });
}

criterion_group!(
    benches,
    bench_start_combat,
    bench_full_turn,
    bench_clone_state,
    bench_get_legal_actions,
);
criterion_main!(benches);
