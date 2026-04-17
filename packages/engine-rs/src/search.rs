//! Deterministic bounded search harnesses for combat and full-run planning.
//!
//! These helpers are intentionally simple and stable: they expand exact legal
//! actions from the engine, clone state for branching, and score leaves with a
//! progress-first heuristic. That keeps the RL surface deterministic while we
//! continue replacing adapter-backed gameplay systems with the universal runtime.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::time::Instant;

use crate::actions::Action;
use crate::decision::DecisionAction;
use crate::engine::CombatEngine;
use crate::obs::encode_combat_state_v2;
use crate::run::RunEngine;
use crate::training_contract::{
    CombatOutcomeVectorV1, CombatPuctConfigV1, CombatPuctLineV1, CombatPuctResultV1,
    CombatSearchStopReasonV1, CombatTrainingStateV1,
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct CombatPuctLeafEvaluationV1 {
    pub candidate_priors: Vec<f32>,
    pub outcome: CombatOutcomeVectorV1,
}

#[derive(Debug, Clone, PartialEq)]
struct CombatOutcomeSums {
    solve_probability: f64,
    expected_hp_loss: f64,
    expected_turns: f64,
    potion_cost: f64,
    setup_value_delta: f64,
    persistent_scaling_delta: f64,
}

impl CombatOutcomeSums {
    fn add(&mut self, outcome: &CombatOutcomeVectorV1) {
        self.solve_probability += f64::from(outcome.solve_probability);
        self.expected_hp_loss += f64::from(outcome.expected_hp_loss);
        self.expected_turns += f64::from(outcome.expected_turns);
        self.potion_cost += f64::from(outcome.potion_cost);
        self.setup_value_delta += f64::from(outcome.setup_value_delta);
        self.persistent_scaling_delta += f64::from(outcome.persistent_scaling_delta);
    }

    fn average(&self, visits: u32) -> CombatOutcomeVectorV1 {
        if visits == 0 {
            return CombatOutcomeVectorV1::default();
        }
        let denom = visits as f64;
        CombatOutcomeVectorV1 {
            solve_probability: (self.solve_probability / denom) as f32,
            expected_hp_loss: (self.expected_hp_loss / denom) as f32,
            expected_turns: (self.expected_turns / denom) as f32,
            potion_cost: (self.potion_cost / denom) as f32,
            setup_value_delta: (self.setup_value_delta / denom) as f32,
            persistent_scaling_delta: (self.persistent_scaling_delta / denom) as f32,
        }
    }
}

#[derive(Clone)]
struct CombatPuctNode {
    action_id_from_parent: Option<i32>,
    dense_index_from_parent: Option<usize>,
    depth: u32,
    engine: CombatEngine,
    visits: u32,
    value_sum: f64,
    outcome_sums: CombatOutcomeSums,
    terminal: bool,
    expanded: bool,
    legal_candidates: Vec<crate::training_contract::LegalActionCandidateV1>,
    child_indices: Vec<usize>,
    prior_from_parent: f32,
}

impl CombatPuctNode {
    fn new_root(engine: CombatEngine) -> Self {
        Self {
            action_id_from_parent: None,
            dense_index_from_parent: None,
            depth: 0,
            engine,
            visits: 0,
            value_sum: 0.0,
            outcome_sums: CombatOutcomeSums {
                solve_probability: 0.0,
                expected_hp_loss: 0.0,
                expected_turns: 0.0,
                potion_cost: 0.0,
                setup_value_delta: 0.0,
                persistent_scaling_delta: 0.0,
            },
            terminal: false,
            expanded: false,
            legal_candidates: Vec::new(),
            child_indices: Vec::new(),
            prior_from_parent: 1.0,
        }
    }

    fn new_child(
        action_id: i32,
        dense_index: usize,
        depth: u32,
        engine: CombatEngine,
        prior: f32,
    ) -> Self {
        Self {
            action_id_from_parent: Some(action_id),
            dense_index_from_parent: Some(dense_index),
            depth,
            engine,
            visits: 0,
            value_sum: 0.0,
            outcome_sums: CombatOutcomeSums {
                solve_probability: 0.0,
                expected_hp_loss: 0.0,
                expected_turns: 0.0,
                potion_cost: 0.0,
                setup_value_delta: 0.0,
                persistent_scaling_delta: 0.0,
            },
            terminal: false,
            expanded: false,
            legal_candidates: Vec::new(),
            child_indices: Vec::new(),
            prior_from_parent: prior,
        }
    }

    fn mean_outcome(&self) -> CombatOutcomeVectorV1 {
        self.outcome_sums.average(self.visits)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ConvergenceSnapshot {
    best_dense_index: Option<usize>,
    top_dense_indices: Vec<usize>,
    best_visit_share_lead: f32,
    root_value: f32,
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

pub fn search_combat_puct<E, F, G>(
    snapshot: &CombatEngine,
    config: CombatPuctConfigV1,
    execution_id_for_action: G,
    mut evaluator: F,
) -> Result<CombatPuctResultV1, E>
where
    F: FnMut(&CombatTrainingStateV1) -> Result<CombatPuctLeafEvaluationV1, E>,
    G: Fn(&Action) -> i32 + Copy,
{
    let config = config.normalized();
    let start = Instant::now();
    let mut nodes = vec![CombatPuctNode::new_root(snapshot.clone_state())];
    let root_terminal = snapshot.is_combat_over();
    if root_terminal {
        let outcome = terminal_outcome(snapshot);
        backpropagate(&mut nodes, &[0], &outcome);
        return Ok(build_puct_result(
            &nodes,
            &config,
            CombatSearchStopReasonV1::TerminalRoot,
            0,
            0,
            0,
            start.elapsed().as_millis() as u64,
        ));
    }

    let root_actions = stable_combat_actions(snapshot);
    if root_actions.is_empty() {
        let outcome = terminal_outcome(snapshot);
        backpropagate(&mut nodes, &[0], &outcome);
        return Ok(build_puct_result(
            &nodes,
            &config,
            CombatSearchStopReasonV1::NoLegalActions,
            0,
            0,
            0,
            start.elapsed().as_millis() as u64,
        ));
    }

    let mut leaf_evaluations = 0_u32;
    let mut max_depth_reached = 0_u32;
    let mut previous_snapshot: Option<ConvergenceSnapshot> = None;
    let mut stable_windows = 0_u32;

    loop {
        let path = simulate_puct(
            &mut nodes,
            &config,
            execution_id_for_action,
            &mut evaluator,
            &mut leaf_evaluations,
            &mut max_depth_reached,
        )?;
        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms >= config.time_cap_ms {
            return Ok(build_puct_result(
                &nodes,
                &config,
                CombatSearchStopReasonV1::TimeCap,
                stable_windows,
                leaf_evaluations,
                max_depth_reached,
                elapsed_ms,
            ));
        }

        let root_visits = nodes[0].visits;
        if root_visits >= config.hard_visit_cap {
            return Ok(build_puct_result(
                &nodes,
                &config,
                CombatSearchStopReasonV1::HardVisitCap,
                stable_windows,
                leaf_evaluations,
                max_depth_reached,
                elapsed_ms,
            ));
        }

        if root_visits < config.min_visits {
            let _ = path;
            continue;
        }

        if root_visits % config.visit_window != 0 {
            let _ = path;
            continue;
        }

        let current_snapshot = convergence_snapshot(&nodes);
        let converged = previous_snapshot.as_ref().is_some_and(|previous| {
            let root_value_delta = (current_snapshot.root_value - previous.root_value).abs();
            current_snapshot.best_dense_index == previous.best_dense_index
                && current_snapshot.top_dense_indices == previous.top_dense_indices
                && current_snapshot.best_visit_share_lead >= config.best_visit_share_lead_threshold
                && root_value_delta <= config.root_value_delta_threshold
        });
        if converged {
            stable_windows += 1;
        } else {
            stable_windows = 0;
        }
        previous_snapshot = Some(current_snapshot);

        if stable_windows >= config.stable_windows_required {
            return Ok(build_puct_result(
                &nodes,
                &config,
                CombatSearchStopReasonV1::Converged,
                stable_windows,
                leaf_evaluations,
                max_depth_reached,
                elapsed_ms,
            ));
        }
    }
}

fn simulate_puct<E, F, G>(
    nodes: &mut Vec<CombatPuctNode>,
    config: &CombatPuctConfigV1,
    execution_id_for_action: G,
    evaluator: &mut F,
    leaf_evaluations: &mut u32,
    max_depth_reached: &mut u32,
) -> Result<Vec<usize>, E>
where
    F: FnMut(&CombatTrainingStateV1) -> Result<CombatPuctLeafEvaluationV1, E>,
    G: Fn(&Action) -> i32 + Copy,
{
    let mut path = vec![0usize];
    let mut current_idx = 0usize;
    loop {
        *max_depth_reached = (*max_depth_reached).max(nodes[current_idx].depth);
        if nodes[current_idx].engine.is_combat_over() {
            nodes[current_idx].terminal = true;
            let outcome = terminal_outcome(&nodes[current_idx].engine);
            backpropagate(nodes, &path, &outcome);
            return Ok(path);
        }

        if nodes[current_idx].depth >= config.max_rollout_depth {
            let training_state = crate::training_contract::combat_training_state_from_combat(
                &nodes[current_idx].engine,
                execution_id_for_action,
            );
            let evaluation = evaluator(&training_state)?;
            *leaf_evaluations += 1;
            backpropagate(nodes, &path, &evaluation.outcome);
            return Ok(path);
        }

        if !nodes[current_idx].expanded {
            let outcome = expand_puct_node(
                nodes,
                current_idx,
                execution_id_for_action,
                evaluator,
                leaf_evaluations,
            )?;
            backpropagate(nodes, &path, &outcome);
            return Ok(path);
        }

        if nodes[current_idx].child_indices.is_empty() {
            let outcome = terminal_outcome(&nodes[current_idx].engine);
            backpropagate(nodes, &path, &outcome);
            return Ok(path);
        }

        let next_idx = select_puct_child(nodes, current_idx, config.cpuct);
        path.push(next_idx);
        current_idx = next_idx;
    }
}

fn expand_puct_node<E, F, G>(
    nodes: &mut Vec<CombatPuctNode>,
    node_idx: usize,
    execution_id_for_action: G,
    evaluator: &mut F,
    leaf_evaluations: &mut u32,
) -> Result<CombatOutcomeVectorV1, E>
where
    F: FnMut(&CombatTrainingStateV1) -> Result<CombatPuctLeafEvaluationV1, E>,
    G: Fn(&Action) -> i32 + Copy,
{
    let training_state = crate::training_contract::combat_training_state_from_combat(
        &nodes[node_idx].engine,
        execution_id_for_action,
    );
    let legal_candidates = training_state.legal_candidates.clone();
    if legal_candidates.is_empty() {
        nodes[node_idx].expanded = true;
        nodes[node_idx].legal_candidates = legal_candidates;
        return Ok(terminal_outcome(&nodes[node_idx].engine));
    }

    let actions = stable_combat_actions(&nodes[node_idx].engine);
    let evaluation = evaluator(&training_state)?;
    *leaf_evaluations += 1;
    let priors = normalize_candidate_priors(&evaluation.candidate_priors, actions.len());
    let parent_depth = nodes[node_idx].depth;
    let parent_engine = nodes[node_idx].engine.clone_state();
    let mut child_indices = Vec::with_capacity(actions.len());
    for (dense_index, action) in actions.into_iter().enumerate() {
        let mut child_engine = parent_engine.clone_state();
        child_engine.execute_action(&action);
        let child_idx = nodes.len();
        nodes.push(CombatPuctNode::new_child(
            legal_candidates[dense_index].execution_id,
            dense_index,
            parent_depth + 1,
            child_engine,
            priors[dense_index],
        ));
        child_indices.push(child_idx);
    }

    let node = &mut nodes[node_idx];
    node.legal_candidates = legal_candidates;
    node.child_indices = child_indices;
    node.expanded = true;
    Ok(evaluation.outcome)
}

fn select_puct_child(nodes: &[CombatPuctNode], parent_idx: usize, cpuct: f32) -> usize {
    let parent = &nodes[parent_idx];
    let parent_visits = parent.visits.max(1) as f64;
    *parent
        .child_indices
        .iter()
        .max_by(|left_idx, right_idx| {
            let left = &nodes[**left_idx];
            let right = &nodes[**right_idx];
            let left_score = puct_score(left, parent_visits, cpuct);
            let right_score = puct_score(right, parent_visits, cpuct);
            left_score
                .partial_cmp(&right_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .dense_index_from_parent
                        .cmp(&left.dense_index_from_parent)
                })
        })
        .expect("expanded parent should contain children")
}

fn puct_score(node: &CombatPuctNode, parent_visits: f64, cpuct: f32) -> f64 {
    let q = if node.visits == 0 {
        0.0
    } else {
        node.value_sum / f64::from(node.visits)
    };
    let exploration = f64::from(cpuct)
        * f64::from(node.prior_from_parent)
        * parent_visits.sqrt()
        / (1.0 + f64::from(node.visits));
    q + exploration
}

fn backpropagate(nodes: &mut [CombatPuctNode], path: &[usize], outcome: &CombatOutcomeVectorV1) {
    let value = search_value(outcome);
    for node_idx in path.iter().copied() {
        let node = &mut nodes[node_idx];
        node.visits += 1;
        node.value_sum += f64::from(value);
        node.outcome_sums.add(outcome);
    }
}

fn normalize_candidate_priors(priors: &[f32], candidate_count: usize) -> Vec<f32> {
    if candidate_count == 0 {
        return Vec::new();
    }
    let mut normalized = vec![0.0_f32; candidate_count];
    for (idx, value) in priors.iter().copied().enumerate().take(candidate_count) {
        normalized[idx] = value.max(0.0);
    }
    let sum: f32 = normalized.iter().sum();
    if sum <= f32::EPSILON {
        let uniform = 1.0 / candidate_count as f32;
        normalized.fill(uniform);
    } else {
        for value in &mut normalized {
            *value /= sum;
        }
    }
    normalized
}

fn convergence_snapshot(nodes: &[CombatPuctNode]) -> ConvergenceSnapshot {
    let root = &nodes[0];
    let total_child_visits: u32 = root
        .child_indices
        .iter()
        .map(|idx| nodes[*idx].visits)
        .sum();
    let mut ranked_children = root.child_indices.clone();
    ranked_children.sort_by(|left_idx, right_idx| {
        let left = &nodes[*left_idx];
        let right = &nodes[*right_idx];
        right
            .visits
            .cmp(&left.visits)
            .then_with(|| left.dense_index_from_parent.cmp(&right.dense_index_from_parent))
    });
    let top_dense_indices = ranked_children
        .iter()
        .take(3)
        .filter_map(|idx| nodes[*idx].dense_index_from_parent)
        .collect::<Vec<_>>();
    let best_visit_share_lead = if ranked_children.len() >= 2 && total_child_visits > 0 {
        let best = nodes[ranked_children[0]].visits as f32 / total_child_visits as f32;
        let second = nodes[ranked_children[1]].visits as f32 / total_child_visits as f32;
        best - second
    } else if ranked_children.len() == 1 {
        1.0
    } else {
        0.0
    };
    let root_value = if root.visits == 0 {
        0.0
    } else {
        (root.value_sum / f64::from(root.visits)) as f32
    };
    ConvergenceSnapshot {
        best_dense_index: ranked_children
            .first()
            .and_then(|idx| nodes[*idx].dense_index_from_parent),
        top_dense_indices,
        best_visit_share_lead,
        root_value,
    }
}

fn build_puct_result(
    nodes: &[CombatPuctNode],
    config: &CombatPuctConfigV1,
    stop_reason: CombatSearchStopReasonV1,
    stable_windows: u32,
    leaf_evaluations: u32,
    max_depth_reached: u32,
    elapsed_ms: u64,
) -> CombatPuctResultV1 {
    let root = &nodes[0];
    let root_total_visits = root.visits;
    let root_action_ids = root
        .legal_candidates
        .iter()
        .map(|candidate| candidate.execution_id)
        .collect::<Vec<_>>();
    let mut root_visits = vec![0_u32; root_action_ids.len()];
    let mut root_priors = vec![0.0_f32; root_action_ids.len()];
    let total_child_visits: u32 = root
        .child_indices
        .iter()
        .map(|idx| nodes[*idx].visits)
        .sum();
    for child_idx in &root.child_indices {
        let child = &nodes[*child_idx];
        if let Some(dense_index) = child.dense_index_from_parent {
            root_visits[dense_index] = child.visits;
            root_priors[dense_index] = child.prior_from_parent;
        }
    }
    let root_visit_shares = root_visits
        .iter()
        .map(|visits| {
            if total_child_visits == 0 {
                0.0
            } else {
                *visits as f32 / total_child_visits as f32
            }
        })
        .collect::<Vec<_>>();
    let mut frontier_candidates = root.child_indices.clone();
    frontier_candidates.sort_by(|left_idx, right_idx| {
        let left = &nodes[*left_idx];
        let right = &nodes[*right_idx];
        right
            .visits
            .cmp(&left.visits)
            .then_with(|| left.dense_index_from_parent.cmp(&right.dense_index_from_parent))
    });
    let frontier = frontier_candidates
        .into_iter()
        .take(config.frontier_capacity)
        .enumerate()
        .map(|(line_index, node_idx)| {
            let node = &nodes[node_idx];
            CombatPuctLineV1 {
                line_index,
                action_prefix: greedy_action_prefix(nodes, node_idx),
                visits: node.visits,
                visit_share: if total_child_visits == 0 {
                    0.0
                } else {
                    node.visits as f32 / total_child_visits as f32
                },
                prior: node.prior_from_parent,
                expanded_nodes: count_subtree_nodes(nodes, node_idx) as u32,
                elapsed_ms,
                outcome: node.mean_outcome(),
            }
        })
        .collect::<Vec<_>>();
    CombatPuctResultV1 {
        chosen_action_id: frontier.first().and_then(|line| line.action_prefix.first().copied()),
        root_action_ids,
        root_visits,
        root_visit_shares,
        root_priors,
        frontier,
        root_outcome: root.mean_outcome(),
        root_total_visits,
        stable_windows,
        nodes_expanded: nodes.len() as u32,
        leaf_evaluations,
        max_depth_reached,
        elapsed_ms,
        stop_reason,
    }
}

fn greedy_action_prefix(nodes: &[CombatPuctNode], start_idx: usize) -> Vec<i32> {
    let mut prefix = Vec::new();
    let mut current_idx = start_idx;
    loop {
        let node = &nodes[current_idx];
        if let Some(action_id) = node.action_id_from_parent {
            prefix.push(action_id);
        }
        let Some(next_idx) = node
            .child_indices
            .iter()
            .copied()
            .max_by(|left_idx, right_idx| {
                let left = &nodes[*left_idx];
                let right = &nodes[*right_idx];
                left.visits
                    .cmp(&right.visits)
                    .then_with(|| right.dense_index_from_parent.cmp(&left.dense_index_from_parent))
            })
        else {
            break;
        };
        if nodes[next_idx].visits == 0 {
            break;
        }
        current_idx = next_idx;
    }
    prefix
}

fn count_subtree_nodes(nodes: &[CombatPuctNode], root_idx: usize) -> usize {
    let mut count = 0usize;
    let mut stack = vec![root_idx];
    while let Some(node_idx) = stack.pop() {
        count += 1;
        stack.extend(nodes[node_idx].child_indices.iter().copied());
    }
    count
}

fn search_value(outcome: &CombatOutcomeVectorV1) -> f32 {
    outcome.solve_probability
        - 0.02 * outcome.expected_hp_loss
        - 0.02 * outcome.potion_cost
        + 0.01 * outcome.setup_value_delta
        + 0.02 * outcome.persistent_scaling_delta
        - 0.01 * outcome.expected_turns
}

fn terminal_outcome(engine: &CombatEngine) -> CombatOutcomeVectorV1 {
    let lost_hp = (engine.state.player.max_hp - engine.state.player.hp).max(0) as f32;
    CombatOutcomeVectorV1 {
        solve_probability: if engine.state.player_won { 1.0 } else { 0.0 },
        expected_hp_loss: lost_hp,
        expected_turns: engine.state.turn.max(0) as f32,
        potion_cost: 0.0,
        setup_value_delta: 0.0,
        persistent_scaling_delta: 0.0,
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

pub(crate) fn stable_combat_actions(engine: &CombatEngine) -> Vec<Action> {
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
        for value in encode_combat_state_v2(engine).iter().copied() {
            value.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}
