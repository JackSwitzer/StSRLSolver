//! Deterministic bounded search harnesses for combat and full-run planning.
//!
//! These helpers are intentionally simple and stable: they expand exact legal
//! actions from the engine, clone state for branching, and score leaves with a
//! progress-first heuristic. That keeps the RL surface deterministic while we
//! continue replacing adapter-backed gameplay systems with the universal runtime.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

use crate::actions::Action;
use crate::decision::DecisionAction;
use crate::engine::CombatEngine;
use crate::obs::encode_combat_state_v2;
use crate::run::RunEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBudget {
    pub max_nodes: usize,
    pub max_depth: usize,
}

impl SearchBudget {
    pub const fn new(max_nodes: usize, max_depth: usize) -> Self {
        Self {
            max_nodes,
            max_depth,
        }
    }

    fn normalized(self) -> Self {
        Self {
            max_nodes: self.max_nodes.max(1),
            max_depth: self.max_depth.max(1),
        }
    }
}

impl Default for SearchBudget {
    fn default() -> Self {
        Self {
            max_nodes: 256,
            max_depth: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSuite {
    pub seeds: Vec<u64>,
}

impl SeedSuite {
    pub fn new(seeds: Vec<u64>) -> Self {
        Self { seeds }
    }

    pub fn benchmark_defaults() -> Self {
        Self {
            seeds: vec![42, 1337, 2024],
        }
    }
}

impl Default for SeedSuite {
    fn default() -> Self {
        Self::benchmark_defaults()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSearchResult {
    pub explored_nodes: usize,
    pub depth_reached: usize,
    pub actions: Vec<Action>,
    pub terminal: bool,
    pub player_won: bool,
    pub score: i64,
    pub final_state_hash: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunSearchResult {
    pub explored_nodes: usize,
    pub depth_reached: usize,
    pub actions: Vec<DecisionAction>,
    pub terminal: bool,
    pub run_won: bool,
    pub score: i64,
    pub total_reward: f32,
    pub act: i32,
    pub deepest_floor: i32,
    pub final_state_hash: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeedSuiteEntryReport {
    pub seed: u64,
    pub result: RunSearchResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeedSuiteReport {
    pub ascension: i32,
    pub budget: SearchBudget,
    pub entries: Vec<SeedSuiteEntryReport>,
    pub total_nodes: usize,
}

pub fn search_combat(snapshot: &CombatEngine, budget: SearchBudget) -> CombatSearchResult {
    let budget = budget.normalized();
    let mut planner = CombatPlanner::new(budget);
    let mut path = Vec::new();
    planner.visit(snapshot.clone_state(), 0, &mut path);
    planner.finish()
}

pub fn search_run(snapshot: &RunEngine, budget: SearchBudget) -> RunSearchResult {
    let budget = budget.normalized();
    let mut planner = RunPlanner::new(budget);
    let mut path = Vec::new();
    planner.visit(snapshot.clone(), 0, 0.0, &mut path);
    planner.finish()
}

pub fn run_seed_suite(seed_suite: &SeedSuite, ascension: i32, budget: SearchBudget) -> SeedSuiteReport {
    let budget = budget.normalized();
    let mut total_nodes = 0;
    let entries = seed_suite
        .seeds
        .iter()
        .map(|seed| {
            let engine = RunEngine::new(*seed, ascension);
            let result = search_run(&engine, budget);
            total_nodes += result.explored_nodes;
            SeedSuiteEntryReport { seed: *seed, result }
        })
        .collect();

    SeedSuiteReport {
        ascension,
        budget,
        entries,
        total_nodes,
    }
}

#[derive(Debug, Clone)]
struct CombatCandidate {
    score: i64,
    depth_reached: usize,
    actions: Vec<Action>,
    terminal: bool,
    player_won: bool,
    final_state_hash: u64,
}

struct CombatPlanner {
    budget: SearchBudget,
    explored_nodes: usize,
    best: Option<CombatCandidate>,
}

impl CombatPlanner {
    fn new(budget: SearchBudget) -> Self {
        Self {
            budget,
            explored_nodes: 0,
            best: None,
        }
    }

    fn visit(&mut self, engine: CombatEngine, depth: usize, path: &mut Vec<Action>) {
        if self.explored_nodes >= self.budget.max_nodes {
            return;
        }
        self.explored_nodes += 1;

        let legal = stable_combat_actions(&engine);
        let terminal = engine.is_combat_over();
        let should_stop = terminal || depth >= self.budget.max_depth || legal.is_empty();
        if should_stop {
            self.consider(&engine, depth, path, terminal);
            return;
        }

        for action in legal.into_iter().rev() {
            if self.explored_nodes >= self.budget.max_nodes {
                break;
            }
            let mut next = engine.clone_state();
            next.execute_action(&action);
            path.push(action);
            self.visit(next, depth + 1, path);
            path.pop();
        }
    }

    fn consider(&mut self, engine: &CombatEngine, depth: usize, path: &[Action], terminal: bool) {
        let candidate = CombatCandidate {
            score: combat_score(engine),
            depth_reached: depth,
            actions: path.to_vec(),
            terminal,
            player_won: engine.state.player_won,
            final_state_hash: combat_state_hash(engine),
        };

        if self
            .best
            .as_ref()
            .map_or(true, |best| combat_candidate_key(&candidate) > combat_candidate_key(best))
        {
            self.best = Some(candidate);
        }
    }

    fn finish(self) -> CombatSearchResult {
        let best = self.best.unwrap_or(CombatCandidate {
            score: i64::MIN,
            depth_reached: 0,
            actions: Vec::new(),
            terminal: false,
            player_won: false,
            final_state_hash: 0,
        });

        CombatSearchResult {
            explored_nodes: self.explored_nodes,
            depth_reached: best.depth_reached,
            actions: best.actions,
            terminal: best.terminal,
            player_won: best.player_won,
            score: best.score,
            final_state_hash: best.final_state_hash,
        }
    }
}

#[derive(Debug, Clone)]
struct RunCandidate {
    score: i64,
    depth_reached: usize,
    total_reward: f32,
    actions: Vec<DecisionAction>,
    terminal: bool,
    run_won: bool,
    act: i32,
    deepest_floor: i32,
    final_state_hash: u64,
}

struct RunPlanner {
    budget: SearchBudget,
    explored_nodes: usize,
    best: Option<RunCandidate>,
}

impl RunPlanner {
    fn new(budget: SearchBudget) -> Self {
        Self {
            budget,
            explored_nodes: 0,
            best: None,
        }
    }

    fn visit(
        &mut self,
        engine: RunEngine,
        depth: usize,
        total_reward: f32,
        path: &mut Vec<DecisionAction>,
    ) {
        if self.explored_nodes >= self.budget.max_nodes {
            return;
        }
        self.explored_nodes += 1;

        let legal = stable_decision_actions(&engine);
        let terminal = engine.is_done();
        let should_stop = terminal || depth >= self.budget.max_depth || legal.is_empty();
        if should_stop {
            self.consider(&engine, depth, total_reward, path, terminal);
            return;
        }

        for action in legal.into_iter().rev() {
            if self.explored_nodes >= self.budget.max_nodes {
                break;
            }
            let mut next = engine.clone();
            let step = next.step_with_result(&action.to_run_action());
            path.push(action);
            self.visit(next, depth + 1, total_reward + step.reward, path);
            path.pop();
        }
    }

    fn consider(
        &mut self,
        engine: &RunEngine,
        depth: usize,
        total_reward: f32,
        path: &[DecisionAction],
        terminal: bool,
    ) {
        let candidate = RunCandidate {
            score: run_score(engine, total_reward),
            depth_reached: depth,
            total_reward,
            actions: path.to_vec(),
            terminal,
            run_won: engine.run_state.run_won,
            act: engine.run_state.act,
            deepest_floor: engine.run_state.floor,
            final_state_hash: run_state_hash(engine),
        };

        if self
            .best
            .as_ref()
            .map_or(true, |best| run_candidate_key(&candidate) > run_candidate_key(best))
        {
            self.best = Some(candidate);
        }
    }

    fn finish(self) -> RunSearchResult {
        let best = self.best.unwrap_or(RunCandidate {
            score: i64::MIN,
            depth_reached: 0,
            total_reward: 0.0,
            actions: Vec::new(),
            terminal: false,
            run_won: false,
            act: 0,
            deepest_floor: 0,
            final_state_hash: 0,
        });

        RunSearchResult {
            explored_nodes: self.explored_nodes,
            depth_reached: best.depth_reached,
            actions: best.actions,
            terminal: best.terminal,
            run_won: best.run_won,
            score: best.score,
            total_reward: best.total_reward,
            act: best.act,
            deepest_floor: best.deepest_floor,
            final_state_hash: best.final_state_hash,
        }
    }
}

fn stable_combat_actions(engine: &CombatEngine) -> Vec<Action> {
    let mut actions = engine.get_legal_actions();
    actions.sort_by_key(combat_action_sort_key);
    actions
}

fn stable_decision_actions(engine: &RunEngine) -> Vec<DecisionAction> {
    let mut actions = engine.get_legal_decision_actions();
    actions.sort_by_key(decision_action_sort_key);
    actions
}

fn combat_action_sort_key(action: &Action) -> (u8, i32, i32) {
    match action {
        Action::PlayCard {
            card_idx,
            target_idx,
        } => (0, *card_idx as i32, *target_idx),
        Action::UsePotion {
            potion_idx,
            target_idx,
        } => (1, *potion_idx as i32, *target_idx),
        Action::Choose(idx) => (2, *idx as i32, 0),
        Action::ConfirmSelection => (3, 0, 0),
        Action::EndTurn => (4, 0, 0),
    }
}

fn decision_action_sort_key(action: &DecisionAction) -> (u8, i32, i32, i32) {
    match action {
        DecisionAction::ChooseNeowOption(idx) => (0, *idx as i32, 0, 0),
        DecisionAction::Combat(action) => {
            let key = combat_action_sort_key(action);
            (1, key.0 as i32, key.1, key.2)
        }
        DecisionAction::ChooseMapPath(idx) => (2, *idx as i32, 0, 0),
        DecisionAction::ClaimRewardItem { item_index } => (3, *item_index as i32, 0, 0),
        DecisionAction::PickRewardChoice {
            item_index,
            choice_index,
        } => (4, *item_index as i32, *choice_index as i32, 0),
        DecisionAction::SkipRewardItem { item_index } => (5, *item_index as i32, 0, 0),
        DecisionAction::CampfireRest => (6, 0, 0, 0),
        DecisionAction::CampfireUpgrade(idx) => (7, *idx as i32, 0, 0),
        DecisionAction::ShopBuyCard(idx) => (8, *idx as i32, 0, 0),
        DecisionAction::ShopRemoveCard(idx) => (9, *idx as i32, 0, 0),
        DecisionAction::ShopLeave => (10, 0, 0, 0),
        DecisionAction::EventChoice(idx) => (11, *idx as i32, 0, 0),
    }
}

fn combat_score(engine: &CombatEngine) -> i64 {
    let living_enemies = engine.state.enemies.iter().filter(|enemy| enemy.is_alive()).count() as i64;
    let enemy_hp: i64 = engine
        .state
        .enemies
        .iter()
        .map(|enemy| i64::from(enemy.entity.hp.max(0)))
        .sum();
    let player_hp = i64::from(engine.state.player.hp.max(0));
    let turn_penalty = i64::from(engine.state.turn.max(0)) * 10;
    let terminal_bonus = if engine.state.player_won {
        1_000_000
    } else if engine.state.player.hp <= 0 {
        -1_000_000
    } else {
        0
    };

    terminal_bonus + player_hp * 100 - enemy_hp * 25 - living_enemies * 500 - turn_penalty
}

fn run_score(engine: &RunEngine, total_reward: f32) -> i64 {
    let terminal_bonus = if engine.run_state.run_won {
        10_000_000
    } else if engine.run_state.run_over {
        -10_000_000
    } else {
        0
    };

    terminal_bonus
        + i64::from(engine.run_state.act) * 1_000_000
        + i64::from(engine.run_state.floor) * 10_000
        + i64::from(engine.run_state.current_hp.max(0)) * 100
        + i64::from(engine.run_state.gold.max(0))
        + (total_reward * 100.0) as i64
}

fn combat_candidate_key(candidate: &CombatCandidate) -> (i64, usize, i64, u64) {
    (
        candidate.score,
        candidate.depth_reached,
        -(candidate.actions.len() as i64),
        candidate.final_state_hash,
    )
}

fn run_candidate_key(candidate: &RunCandidate) -> (i64, usize, i64, u64) {
    (
        candidate.score,
        candidate.depth_reached,
        -(candidate.actions.len() as i64),
        candidate.final_state_hash,
    )
}

pub(crate) fn combat_state_hash(engine: &CombatEngine) -> u64 {
    let mut hasher = DefaultHasher::new();
    mem::discriminant(&engine.phase).hash(&mut hasher);
    format!("{:?}", engine.export_persisted_effects()).hash(&mut hasher);
    format!("{:?}", engine.event_log).hash(&mut hasher);
    engine.state.player.hp.hash(&mut hasher);
    engine.state.player.max_hp.hash(&mut hasher);
    engine.state.player.block.hash(&mut hasher);
    engine.state.player.statuses.hash(&mut hasher);
    engine.state.energy.hash(&mut hasher);
    engine.state.max_energy.hash(&mut hasher);
    engine.state.turn.hash(&mut hasher);
    engine.state.combat_over.hash(&mut hasher);
    engine.state.player_won.hash(&mut hasher);
    engine.state.cards_played_this_turn.hash(&mut hasher);
    engine.state.attacks_played_this_turn.hash(&mut hasher);
    engine.state.total_damage_dealt.hash(&mut hasher);
    engine.state.total_damage_taken.hash(&mut hasher);
    engine.state.total_cards_played.hash(&mut hasher);
    engine.state.mantra.hash(&mut hasher);
    engine.state.mantra_gained.hash(&mut hasher);
    format!("{:?}", engine.state.last_card_type).hash(&mut hasher);
    engine.state.skip_enemy_turn.hash(&mut hasher);
    engine.state.blasphemy_active.hash(&mut hasher);
    engine.state.hand.hash(&mut hasher);
    engine.state.draw_pile.hash(&mut hasher);
    engine.state.discard_pile.hash(&mut hasher);
    engine.state.exhaust_pile.hash(&mut hasher);
    engine.state.potions.hash(&mut hasher);
    engine.state.relics.hash(&mut hasher);
    engine.state.relic_counters.hash(&mut hasher);
    engine.state.stance.hash(&mut hasher);
    format!("{:?}", engine.state.orb_slots).hash(&mut hasher);
    engine.runtime_played_card.hash(&mut hasher);
    engine.runtime_play_target_idx.hash(&mut hasher);
    engine.runtime_play_stack.hash(&mut hasher);
    engine.runtime_replay_window.hash(&mut hasher);
    format!("{:?}", engine.choice).hash(&mut hasher);
    for enemy in &engine.state.enemies {
        enemy.id.hash(&mut hasher);
        enemy.entity.hp.hash(&mut hasher);
        enemy.entity.max_hp.hash(&mut hasher);
        enemy.entity.block.hash(&mut hasher);
        enemy.entity.statuses.hash(&mut hasher);
        enemy.move_id.hash(&mut hasher);
        enemy.move_history.hash(&mut hasher);
        enemy.first_turn.hash(&mut hasher);
        enemy.is_escaping.hash(&mut hasher);
        format!("{:?}", enemy.intent).hash(&mut hasher);
    }
    hasher.finish()
}

pub(crate) fn run_state_hash(engine: &RunEngine) -> u64 {
    let mut hasher = DefaultHasher::new();
    mem::discriminant(&engine.current_phase()).hash(&mut hasher);
    engine.seed.hash(&mut hasher);
    engine.current_room_type().hash(&mut hasher);
    engine.run_state.current_hp.hash(&mut hasher);
    engine.run_state.max_hp.hash(&mut hasher);
    engine.run_state.gold.hash(&mut hasher);
    engine.run_state.ascension.hash(&mut hasher);
    engine.run_state.floor.hash(&mut hasher);
    engine.run_state.act.hash(&mut hasher);
    engine.run_state.max_potions.hash(&mut hasher);
    engine.run_state.map_x.hash(&mut hasher);
    engine.run_state.map_y.hash(&mut hasher);
    engine.run_state.has_ruby_key.hash(&mut hasher);
    engine.run_state.has_emerald_key.hash(&mut hasher);
    engine.run_state.has_sapphire_key.hash(&mut hasher);
    engine.run_state.run_won.hash(&mut hasher);
    engine.run_state.run_over.hash(&mut hasher);
    engine.run_state.deck.hash(&mut hasher);
    engine.run_state.relics.hash(&mut hasher);
    engine.run_state.potions.hash(&mut hasher);
    engine.run_state.combats_won.hash(&mut hasher);
    engine.run_state.elites_killed.hash(&mut hasher);
    engine.run_state.bosses_killed.hash(&mut hasher);
    format!("{:?}", engine.run_state.relic_flags).hash(&mut hasher);
    format!("{:?}", engine.run_state.persisted_effect_states).hash(&mut hasher);
    engine.get_card_rewards().hash(&mut hasher);
    format!("{:?}", engine.last_combat_events()).hash(&mut hasher);
    engine.total_reward.to_bits().hash(&mut hasher);
    engine.decision_stack_depth().hash(&mut hasher);
    engine.current_reward_choice_count().hash(&mut hasher);
    engine.pending_event_combat_summary().hash(&mut hasher);
    format!("{:?}", engine.current_decision_state()).hash(&mut hasher);
    format!("{:?}", engine.current_decision_context()).hash(&mut hasher);
    if engine.current_phase() == crate::run::RunPhase::Combat {
        for value in encode_combat_state_v2(engine) {
            value.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}
