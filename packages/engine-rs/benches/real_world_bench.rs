//! Named combat slices for benchmarking the Rust engine on realistic early fights.
//!
//! These benches intentionally use small Watcher decks and Exordium enemies so we
//! can measure the fast path on the parts of the game we expect to solve cheaply.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sts_engine::actions::Action;
use sts_engine::engine::CombatEngine;
use sts_engine::state::{CombatState, EnemyCombatState};

#[derive(Clone)]
struct Scenario {
    name: &'static str,
    deck: Vec<String>,
    enemies: Vec<EnemyCombatState>,
    seed: u64,
}

fn starter_watcher_deck() -> Vec<String> {
    vec![
        "Strike_P".to_string(),
        "Strike_P".to_string(),
        "Strike_P".to_string(),
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Defend_P".to_string(),
        "Defend_P".to_string(),
        "Defend_P".to_string(),
        "Eruption".to_string(),
        "Vigilance".to_string(),
    ]
}

fn early_elite_deck() -> Vec<String> {
    vec![
        "Strike_P+".to_string(),
        "Strike_P".to_string(),
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Defend_P".to_string(),
        "Eruption".to_string(),
        "Vigilance".to_string(),
        "CutThroughFate".to_string(),
        "Tantrum".to_string(),
        "BowlingBash".to_string(),
    ]
}

fn enemy(id: &str, hp: i32, dmg: i32, hits: i32) -> EnemyCombatState {
    let mut enemy = EnemyCombatState::new(id, hp, hp);
    enemy.set_move(1, dmg, hits, 0);
    enemy
}

fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "jaw_worm_starter",
            deck: starter_watcher_deck(),
            enemies: vec![enemy("JawWorm", 42, 11, 1)],
            seed: 42,
        },
        Scenario {
            name: "cultist_starter",
            deck: starter_watcher_deck(),
            enemies: vec![enemy("Cultist", 50, 6, 1)],
            seed: 43,
        },
        Scenario {
            name: "two_louse_starter",
            deck: starter_watcher_deck(),
            enemies: vec![enemy("FuzzyLouseNormal", 13, 6, 1), enemy("FuzzyLouseDefensive", 15, 5, 1)],
            seed: 44,
        },
        Scenario {
            name: "gremlin_nob_early_deck",
            deck: early_elite_deck(),
            enemies: vec![enemy("GremlinNob", 90, 14, 1)],
            seed: 45,
        },
        Scenario {
            name: "lagavulin_early_deck",
            deck: early_elite_deck(),
            enemies: vec![enemy("Lagavulin", 116, 18, 1)],
            seed: 46,
        },
        Scenario {
            name: "three_sentries_early_deck",
            deck: early_elite_deck(),
            enemies: vec![
                enemy("Sentry", 39, 9, 1),
                enemy("Sentry", 39, 9, 1),
                enemy("Sentry", 39, 9, 1),
            ],
            seed: 47,
        },
    ]
}

fn make_engine(scenario: &Scenario) -> CombatEngine {
    let state = CombatState::new(72, 72, scenario.enemies.clone(), scenario.deck.clone(), 3);
    let mut engine = CombatEngine::new(state, scenario.seed);
    engine.start_combat();
    engine
}

fn play_three_turn_window(engine: &mut CombatEngine) {
    for _ in 0..3 {
        loop {
            let actions = engine.get_legal_actions();
            let next_action = actions
                .iter()
                .find(|action| matches!(action, Action::PlayCard { .. }))
                .cloned()
                .unwrap_or(Action::EndTurn);

            let is_end_turn = matches!(next_action, Action::EndTurn);
            engine.execute_action(&next_action);
            if is_end_turn || engine.is_combat_over() {
                break;
            }
        }

        if engine.is_combat_over() {
            break;
        }
    }
}

fn bench_real_world_turn_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_turn_windows");
    for scenario in scenarios() {
        group.bench_with_input(BenchmarkId::from_parameter(scenario.name), &scenario, |b, scenario| {
            b.iter(|| {
                let mut engine = make_engine(scenario);
                play_three_turn_window(&mut engine);
                black_box(&engine);
            });
        });
    }
    group.finish();
}

fn bench_real_world_clone_for_mcts(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_clone_for_mcts");
    for scenario in scenarios() {
        let engine = make_engine(&scenario);
        group.bench_with_input(BenchmarkId::from_parameter(scenario.name), &engine, |b, engine| {
            b.iter(|| {
                let cloned = engine.clone_state();
                black_box(cloned);
            });
        });
    }
    group.finish();
}

criterion_group!(real_world_benches, bench_real_world_turn_windows, bench_real_world_clone_for_mcts);
criterion_main!(real_world_benches);
