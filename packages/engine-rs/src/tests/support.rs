#![cfg(test)]

use crate::actions::Action;
use crate::cards::CardRegistry;
use crate::combat_types::CardInstance;
use crate::engine::{CombatEngine, CombatPhase};
use crate::run::{RunAction, RunEngine};
use crate::state::{CombatState, EnemyCombatState, Stance};

pub(crate) const TEST_SEED: u64 = 42;

/// Create a deck of CardInstances from card name strings.
pub(crate) fn make_deck(names: &[&str]) -> Vec<CardInstance> {
    let reg = CardRegistry::new();
    names.iter().map(|n| reg.make_card(n)).collect()
}

/// Create N copies of the same card.
pub(crate) fn make_deck_n(name: &str, n: usize) -> Vec<CardInstance> {
    let reg = CardRegistry::new();
    vec![reg.make_card(name); n]
}

pub(crate) fn enemy(id: &str, hp: i32, max_hp: i32, move_id: i32, move_damage: i32, move_hits: i32) -> EnemyCombatState {
    let mut enemy = EnemyCombatState::new(id, hp, max_hp);
    enemy.set_move(move_id, move_damage, move_hits, 0);
    enemy
}

pub(crate) fn enemy_no_intent(id: &str, hp: i32, max_hp: i32) -> EnemyCombatState {
    EnemyCombatState::new(id, hp, max_hp)
}

pub(crate) fn combat_state_with(deck: Vec<CardInstance>, enemies: Vec<EnemyCombatState>, energy: i32) -> CombatState {
    CombatState::new(80, 80, enemies, deck, energy)
}

pub(crate) fn engine_with_state(state: CombatState) -> CombatEngine {
    let mut engine = CombatEngine::new(state, TEST_SEED);
    engine.start_combat();
    engine
}

pub(crate) fn engine_with(deck: Vec<CardInstance>, enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
    engine_with_state(combat_state_with(
        deck,
        vec![enemy("JawWorm", enemy_hp, enemy_hp, 1, enemy_dmg, 1)],
        3,
    ))
}

pub(crate) fn engine_with_enemy_id(deck: Vec<CardInstance>, enemy_id: &str, enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
    engine_with_state(combat_state_with(
        deck,
        vec![enemy(enemy_id, enemy_hp, enemy_hp, 1, enemy_dmg, 1)],
        3,
    ))
}

pub(crate) fn engine_with_enemies(deck: Vec<CardInstance>, enemies: Vec<EnemyCombatState>, energy: i32) -> CombatEngine {
    engine_with_state(combat_state_with(deck, enemies, energy))
}

pub(crate) fn engine_without_start(deck: Vec<CardInstance>, enemies: Vec<EnemyCombatState>, energy: i32) -> CombatEngine {
    CombatEngine::new(combat_state_with(deck, enemies, energy), TEST_SEED)
}

pub(crate) fn force_player_turn(engine: &mut CombatEngine) {
    engine.phase = CombatPhase::PlayerTurn;
    if engine.state.turn == 0 {
        engine.state.turn = 1;
    }
}

pub(crate) fn ensure_in_hand(engine: &mut CombatEngine, card_id: &str) {
    if !engine.state.hand.iter().any(|c| engine.card_registry.card_name(c.def_id) == card_id) {
        engine.state.hand.push(engine.card_registry.make_card(card_id));
    }
}

pub(crate) fn ensure_on_top_of_draw(engine: &mut CombatEngine, card_id: &str) {
    engine.state.draw_pile.push(engine.card_registry.make_card(card_id));
}

pub(crate) fn play_card(engine: &mut CombatEngine, card_id: &str, target_idx: i32) -> bool {
    if let Some(idx) = engine.state.hand.iter().position(|c| engine.card_registry.card_name(c.def_id) == card_id) {
        engine.execute_action(&Action::PlayCard { card_idx: idx, target_idx });
        true
    } else {
        false
    }
}

pub(crate) fn play_self(engine: &mut CombatEngine, card_id: &str) -> bool {
    play_card(engine, card_id, -1)
}

pub(crate) fn play_on_enemy(engine: &mut CombatEngine, card_id: &str, enemy_idx: usize) -> bool {
    play_card(engine, card_id, enemy_idx as i32)
}

pub(crate) fn end_turn(engine: &mut CombatEngine) {
    engine.execute_action(&Action::EndTurn);
}

pub(crate) fn hand_count(engine: &CombatEngine, exact_id: &str) -> usize {
    engine.state.hand.iter().filter(|c| engine.card_registry.card_name(c.def_id) == exact_id).count()
}

pub(crate) fn hand_prefix_count(engine: &CombatEngine, prefix: &str) -> usize {
    engine.state.hand.iter().filter(|c| engine.card_registry.card_name(c.def_id).starts_with(prefix)).count()
}

pub(crate) fn discard_prefix_count(engine: &CombatEngine, prefix: &str) -> usize {
    engine.state.discard_pile.iter().filter(|c| engine.card_registry.card_name(c.def_id).starts_with(prefix)).count()
}

pub(crate) fn exhaust_prefix_count(engine: &CombatEngine, prefix: &str) -> usize {
    engine.state.exhaust_pile.iter().filter(|c| engine.card_registry.card_name(c.def_id).starts_with(prefix)).count()
}

pub(crate) fn draw_prefix_count(engine: &CombatEngine, prefix: &str) -> usize {
    engine.state.draw_pile.iter().filter(|c| engine.card_registry.card_name(c.def_id).starts_with(prefix)).count()
}

pub(crate) fn set_stance(engine: &mut CombatEngine, stance: Stance) {
    engine.state.stance = stance;
}

pub(crate) fn run_engine(seed: u64, ascension: i32) -> RunEngine {
    RunEngine::new(seed, ascension)
}

pub(crate) fn choose_first_path(engine: &mut RunEngine) -> (f32, bool) {
    engine.step(&RunAction::ChoosePath(0))
}

pub(crate) fn step_until_phase(engine: &mut RunEngine, phase: crate::run::RunPhase, max_steps: usize) {
    for _ in 0..max_steps {
        if engine.current_phase() == phase || engine.is_done() {
            return;
        }
        let Some(action) = engine.get_legal_actions().into_iter().next() else {
            return;
        };
        engine.step(&action);
    }
}
