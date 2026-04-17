//! Versioned combat-training contract for the rebuilt training stack.
//!
//! The goal of this module is to expose one Rust-canonical surface for
//! combat-first training work:
//! - token-first combat observations
//! - dense legal-candidate lists with stable opaque execution ids
//! - restriction policies layered above engine legality
//! - manifest/report/log structs shared with the training runtime

use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::decision::{DecisionAction, DecisionKind, RewardItemKind};
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine};
use crate::gameplay::session::GameplaySession;
use crate::orbs::OrbType;
use crate::relic_flags;
use crate::run::RunEngine;
use crate::state::CombatState;

pub const TRAINING_SESSION_SCHEMA_VERSION: u32 = 1;
pub const COMBAT_OBSERVATION_SCHEMA_VERSION: u32 = 1;
pub const ACTION_CANDIDATE_SCHEMA_VERSION: u32 = 1;
pub const GAMEPLAY_EXPORT_SCHEMA_VERSION: u32 = 1;
pub const REPLAY_EVENT_TRACE_SCHEMA_VERSION: u32 = 1;
pub const COMBAT_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const COMBAT_FRONTIER_CAPACITY: usize = 8;
pub const COMBAT_PUCT_STABLE_WINDOWS: u32 = 3;

const HAND_TOKEN_CAP: usize = 10;
const ENEMY_TOKEN_CAP: usize = 5;
const PLAYER_EFFECT_TOKEN_CAP: usize = 32;
const ENEMY_EFFECT_TOKEN_CAP: usize = 16;
const ORB_TOKEN_CAP: usize = 10;
const RELIC_COUNTER_TOKEN_CAP: usize = 8;
const CHOICE_TOKEN_CAP: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingSchemaVersionsV1 {
    pub training_session_schema_version: u32,
    pub combat_observation_schema_version: u32,
    pub action_candidate_schema_version: u32,
    pub gameplay_export_schema_version: u32,
    pub replay_event_trace_schema_version: u32,
}

impl Default for TrainingSchemaVersionsV1 {
    fn default() -> Self {
        Self {
            training_session_schema_version: TRAINING_SESSION_SCHEMA_VERSION,
            combat_observation_schema_version: COMBAT_OBSERVATION_SCHEMA_VERSION,
            action_candidate_schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
            gameplay_export_schema_version: GAMEPLAY_EXPORT_SCHEMA_VERSION,
            replay_event_trace_schema_version: REPLAY_EVENT_TRACE_SCHEMA_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatObservationCapsV1 {
    pub hand: usize,
    pub enemies: usize,
    pub player_effects: usize,
    pub enemy_effects_per_enemy: usize,
    pub orbs: usize,
    pub relic_counters: usize,
    pub choice_options: usize,
}

impl Default for CombatObservationCapsV1 {
    fn default() -> Self {
        Self {
            hand: HAND_TOKEN_CAP,
            enemies: ENEMY_TOKEN_CAP,
            player_effects: PLAYER_EFFECT_TOKEN_CAP,
            enemy_effects_per_enemy: ENEMY_EFFECT_TOKEN_CAP,
            orbs: ORB_TOKEN_CAP,
            relic_counters: RELIC_COUNTER_TOKEN_CAP,
            choice_options: CHOICE_TOKEN_CAP,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatGlobalTokenV1 {
    pub turn: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub cards_played_this_turn: i32,
    pub attacks_played_this_turn: i32,
    pub hand_size: usize,
    pub draw_pile_size: usize,
    pub discard_pile_size: usize,
    pub exhaust_pile_size: usize,
    pub potion_slots: usize,
    pub orb_slot_count: usize,
    pub occupied_orb_slots: usize,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_block: i32,
    pub stance: String,
    pub mantra: i32,
    pub mantra_gained: i32,
    pub skip_enemy_turn: bool,
    pub blasphemy_active: bool,
    pub combat_over: bool,
    pub player_won: bool,
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub total_cards_played: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTokenV1 {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub stance: String,
    pub strength: i32,
    pub dexterity: i32,
    pub focus: i32,
    pub weak: i32,
    pub vulnerable: i32,
    pub frail: i32,
    pub relics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardTokenV1 {
    pub hand_index: usize,
    pub card_id: String,
    pub card_name: String,
    pub card_type: String,
    pub target: String,
    pub cost_for_turn: i32,
    pub base_cost: i32,
    pub misc: i32,
    pub upgraded: bool,
    pub free_to_play: bool,
    pub retained: bool,
    pub ethereal: bool,
    pub runtime_only: bool,
    pub x_cost: bool,
    pub multi_hit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyTokenV1 {
    pub enemy_index: usize,
    pub enemy_id: String,
    pub enemy_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub alive: bool,
    pub targetable: bool,
    pub back_attack: bool,
    pub intent: String,
    pub intent_damage: i32,
    pub intent_hits: i32,
    pub intent_block: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusTokenV1 {
    pub status_id: u16,
    pub status_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyStatusTokenV1 {
    pub enemy_index: usize,
    pub status_id: u16,
    pub status_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrbTokenV1 {
    pub slot_index: usize,
    pub orb_type: String,
    pub base_passive: i32,
    pub base_evoke: i32,
    pub evoke_amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelicCounterTokenV1 {
    pub counter_name: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatChoiceOptionV1 {
    pub choice_index: usize,
    pub kind: String,
    pub source_index: i32,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatChoiceContextV1 {
    pub active: bool,
    pub reason: Option<String>,
    pub min_picks: usize,
    pub max_picks: usize,
    pub selected: Vec<usize>,
    pub options: Vec<CombatChoiceOptionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatObservationSchemaV1 {
    pub schema_version: u32,
    pub caps: CombatObservationCapsV1,
    pub global: CombatGlobalTokenV1,
    pub player: PlayerTokenV1,
    pub hand: Vec<CardTokenV1>,
    pub enemies: Vec<EnemyTokenV1>,
    pub player_effects: Vec<StatusTokenV1>,
    pub enemy_effects: Vec<EnemyStatusTokenV1>,
    pub orbs: Vec<OrbTokenV1>,
    pub relic_counters: Vec<RelicCounterTokenV1>,
    pub choice: CombatChoiceContextV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateCardFeaturesV1 {
    pub hand_index: usize,
    pub card_id: String,
    pub card_name: String,
    pub card_type: String,
    pub cost_for_turn: i32,
    pub base_cost: i32,
    pub upgraded: bool,
    pub x_cost: bool,
    pub multi_hit: bool,
    pub free_to_play: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateTargetFeaturesV1 {
    pub enemy_index: usize,
    pub enemy_name: String,
    pub hp: i32,
    pub block: i32,
    pub targetable: bool,
    pub back_attack: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePotionFeaturesV1 {
    pub slot: usize,
    pub potion_id: String,
    pub target_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateChoiceFeaturesV1 {
    pub choice_index: usize,
    pub label: String,
    pub kind: String,
    pub source_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalActionCandidateV1 {
    pub schema_version: u32,
    pub dense_index: usize,
    pub execution_id: i32,
    pub action_kind: String,
    pub description: String,
    pub card: Option<CandidateCardFeaturesV1>,
    pub target: Option<CandidateTargetFeaturesV1>,
    pub potion: Option<CandidatePotionFeaturesV1>,
    pub choice: Option<CandidateChoiceFeaturesV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatOutcomeVectorV1 {
    pub solve_probability: f32,
    pub expected_hp_loss: f32,
    pub expected_turns: f32,
    pub potion_cost: f32,
    pub setup_value_delta: f32,
    pub persistent_scaling_delta: f32,
}

impl Default for CombatOutcomeVectorV1 {
    fn default() -> Self {
        Self {
            solve_probability: 0.0,
            expected_hp_loss: 0.0,
            expected_turns: 0.0,
            potion_cost: 0.0,
            setup_value_delta: 0.0,
            persistent_scaling_delta: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatFrontierLineV1 {
    pub line_index: usize,
    pub action_prefix: Vec<i32>,
    pub visits: u32,
    pub expanded_nodes: u32,
    pub elapsed_ms: u64,
    pub outcome: CombatOutcomeVectorV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatFrontierSummaryV1 {
    pub capacity: usize,
    pub lines: Vec<CombatFrontierLineV1>,
}

impl Default for CombatFrontierSummaryV1 {
    fn default() -> Self {
        Self {
            capacity: COMBAT_FRONTIER_CAPACITY,
            lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSearchStopReasonV1 {
    Converged,
    HardVisitCap,
    TimeCap,
    TerminalRoot,
    NoLegalActions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatPuctConfigV1 {
    pub cpuct: f32,
    pub frontier_capacity: usize,
    pub min_visits: u32,
    pub visit_window: u32,
    pub hard_visit_cap: u32,
    pub time_cap_ms: u64,
    pub max_rollout_depth: u32,
    pub stable_windows_required: u32,
    pub best_visit_share_lead_threshold: f32,
    pub root_value_delta_threshold: f32,
}

impl CombatPuctConfigV1 {
    pub const fn hallway_default() -> Self {
        Self {
            cpuct: 1.35,
            frontier_capacity: COMBAT_FRONTIER_CAPACITY,
            min_visits: 1_024,
            visit_window: 256,
            hard_visit_cap: 4_096,
            time_cap_ms: 1_500,
            max_rollout_depth: 48,
            stable_windows_required: COMBAT_PUCT_STABLE_WINDOWS,
            best_visit_share_lead_threshold: 0.08,
            root_value_delta_threshold: 0.01,
        }
    }

    pub const fn elite_default() -> Self {
        Self {
            cpuct: 1.35,
            frontier_capacity: COMBAT_FRONTIER_CAPACITY,
            min_visits: 2_048,
            visit_window: 512,
            hard_visit_cap: 8_192,
            time_cap_ms: 4_000,
            max_rollout_depth: 64,
            stable_windows_required: COMBAT_PUCT_STABLE_WINDOWS,
            best_visit_share_lead_threshold: 0.08,
            root_value_delta_threshold: 0.01,
        }
    }

    pub const fn boss_default() -> Self {
        Self {
            cpuct: 1.35,
            frontier_capacity: COMBAT_FRONTIER_CAPACITY,
            min_visits: 4_096,
            visit_window: 1_024,
            hard_visit_cap: 16_384,
            time_cap_ms: 10_000,
            max_rollout_depth: 96,
            stable_windows_required: COMBAT_PUCT_STABLE_WINDOWS,
            best_visit_share_lead_threshold: 0.08,
            root_value_delta_threshold: 0.01,
        }
    }

    pub fn normalized(&self) -> Self {
        let frontier_capacity = self.frontier_capacity.max(1);
        let min_visits = self.min_visits.max(1);
        let visit_window = self.visit_window.max(1);
        let hard_visit_cap = self.hard_visit_cap.max(min_visits);
        let stable_windows_required = self.stable_windows_required.max(1);
        let max_rollout_depth = self.max_rollout_depth.max(1);
        Self {
            cpuct: self.cpuct.max(0.01),
            frontier_capacity,
            min_visits,
            visit_window,
            hard_visit_cap,
            time_cap_ms: self.time_cap_ms.max(1),
            max_rollout_depth,
            stable_windows_required,
            best_visit_share_lead_threshold: self.best_visit_share_lead_threshold.clamp(0.0, 1.0),
            root_value_delta_threshold: self.root_value_delta_threshold.max(0.0),
        }
    }
}

impl Default for CombatPuctConfigV1 {
    fn default() -> Self {
        Self::hallway_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatPuctLineV1 {
    pub line_index: usize,
    pub action_prefix: Vec<i32>,
    pub visits: u32,
    pub visit_share: f32,
    pub prior: f32,
    pub expanded_nodes: u32,
    pub elapsed_ms: u64,
    pub outcome: CombatOutcomeVectorV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatPuctResultV1 {
    pub chosen_action_id: Option<i32>,
    pub root_action_ids: Vec<i32>,
    pub root_visits: Vec<u32>,
    pub root_visit_shares: Vec<f32>,
    pub root_priors: Vec<f32>,
    pub frontier: Vec<CombatPuctLineV1>,
    pub root_outcome: CombatOutcomeVectorV1,
    pub root_total_visits: u32,
    pub stable_windows: u32,
    pub nodes_expanded: u32,
    pub leaf_evaluations: u32,
    pub max_depth_reached: u32,
    pub elapsed_ms: u64,
    pub stop_reason: CombatSearchStopReasonV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestrictionBuiltinV1 {
    NoCardRewards,
    NoCardAdds,
    UpgradeRemoveOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RestrictionPolicyV1 {
    #[serde(default)]
    pub builtins: Vec<RestrictionBuiltinV1>,
}

impl RestrictionPolicyV1 {
    pub fn has_builtin(&self, builtin: RestrictionBuiltinV1) -> bool {
        self.builtins.iter().any(|candidate| *candidate == builtin)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTrainingContextV1 {
    pub runtime_scope: String,
    pub decision_kind: String,
    pub phase_label: String,
    pub terminal: bool,
    pub floor: Option<i32>,
    pub ascension: Option<i32>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTrainingStateV1 {
    pub schema_versions: TrainingSchemaVersionsV1,
    pub context: CombatTrainingContextV1,
    pub observation: CombatObservationSchemaV1,
    pub legal_candidates: Vec<LegalActionCandidateV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardSnapshotV1 {
    pub card_id: String,
    pub cost_for_turn: i32,
    pub base_cost: i32,
    pub misc: i32,
    pub upgraded: bool,
    pub free_to_play: bool,
    pub retained: bool,
    pub ethereal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemySnapshotV1 {
    pub enemy_index: usize,
    pub enemy_id: String,
    pub enemy_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub back_attack: bool,
    pub move_id: i32,
    pub intent_damage: i32,
    pub intent_hits: i32,
    pub intent_block: i32,
    pub first_turn: bool,
    pub is_escaping: bool,
    pub statuses: Vec<StatusTokenV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatSnapshotV1 {
    pub schema_version: u32,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_block: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub turn: i32,
    pub cards_played_this_turn: i32,
    pub attacks_played_this_turn: i32,
    pub stance: String,
    pub mantra: i32,
    pub mantra_gained: i32,
    pub skip_enemy_turn: bool,
    pub blasphemy_active: bool,
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub total_cards_played: i32,
    pub player_effects: Vec<StatusTokenV1>,
    pub hand: Vec<CardSnapshotV1>,
    pub draw_pile: Vec<CardSnapshotV1>,
    pub discard_pile: Vec<CardSnapshotV1>,
    pub exhaust_pile: Vec<CardSnapshotV1>,
    pub enemies: Vec<EnemySnapshotV1>,
    pub potions: Vec<String>,
    pub relics: Vec<String>,
    pub relic_counters: Vec<RelicCounterTokenV1>,
    pub orb_slots: usize,
    pub rng_seed0: u64,
    pub rng_seed1: u64,
    pub rng_counter: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunManifestV1 {
    pub git_sha: String,
    pub git_dirty: bool,
    pub combat_observation_schema_version: u32,
    pub action_candidate_schema_version: u32,
    pub gameplay_export_schema_version: u32,
    pub replay_event_trace_schema_version: u32,
    pub model_version: String,
    pub benchmark_config: String,
    pub seed: u64,
    pub restriction_policy: RestrictionPolicyV1,
    pub hardware: String,
    pub runtime: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeStepV1 {
    pub step_index: usize,
    pub action_id: i32,
    pub reward_delta: f32,
    pub done: bool,
    pub search_frontier: Option<CombatFrontierSummaryV1>,
    pub value: Option<CombatOutcomeVectorV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EpisodeLogV1 {
    pub manifest: Option<RunManifestV1>,
    pub steps: Vec<EpisodeStepV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BenchmarkSliceResultV1 {
    pub slice_name: String,
    pub cases: usize,
    pub solve_rate: f32,
    pub expected_hp_loss: f32,
    pub expected_turns: f32,
    pub oracle_top_k_agreement: f32,
    pub p95_elapsed_ms: f32,
    pub p95_rss_gb: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BenchmarkReportV1 {
    pub manifest: Option<RunManifestV1>,
    pub slices: Vec<BenchmarkSliceResultV1>,
}

pub fn combat_training_state_from_combat(
    engine: &CombatEngine,
    execution_id_for_action: impl Fn(&Action) -> i32,
) -> CombatTrainingStateV1 {
    let decision_state = engine.gameplay_decision_state();
    CombatTrainingStateV1 {
        schema_versions: TrainingSchemaVersionsV1::default(),
        context: CombatTrainingContextV1 {
            runtime_scope: "combat".to_string(),
            decision_kind: format!("{:?}", decision_state.kind),
            phase_label: decision_state.phase_label,
            terminal: decision_state.terminal,
            floor: None,
            ascension: None,
            seed: None,
        },
        observation: build_combat_observation(engine),
        legal_candidates: build_legal_action_candidates(engine, execution_id_for_action),
    }
}

pub fn combat_training_state_from_run(
    run: &RunEngine,
    policy: &RestrictionPolicyV1,
    execution_id_for_action: impl Fn(&Action) -> i32,
) -> Option<CombatTrainingStateV1> {
    let combat = run.get_combat_engine()?;
    let decision_state = run.current_decision_state();
    let all_candidates = build_legal_action_candidates(combat, execution_id_for_action);
    let allowed_execution_ids = combat_execution_ids_for_allowed_actions(run, policy);
    let legal_candidates = all_candidates
        .into_iter()
        .filter(|candidate| allowed_execution_ids.contains(&candidate.execution_id))
        .collect();

    Some(CombatTrainingStateV1 {
        schema_versions: TrainingSchemaVersionsV1::default(),
        context: CombatTrainingContextV1 {
            runtime_scope: "run".to_string(),
            decision_kind: format!("{:?}", decision_state.kind),
            phase_label: format!("Run::{:?}", run.current_phase()),
            terminal: decision_state.terminal,
            floor: Some(run.run_state.floor),
            ascension: Some(run.run_state.ascension),
            seed: Some(run.seed),
        },
        observation: build_combat_observation(combat),
        legal_candidates,
    })
}

pub fn combat_snapshot_from_combat(engine: &CombatEngine) -> CombatSnapshotV1 {
    let (rng_seed0, rng_seed1, rng_counter) = engine.rng.state_tuple();
    let state = &engine.state;
    CombatSnapshotV1 {
        schema_version: COMBAT_SNAPSHOT_SCHEMA_VERSION,
        player_hp: state.player.hp,
        player_max_hp: state.player.max_hp,
        player_block: state.player.block,
        energy: state.energy,
        max_energy: state.max_energy,
        turn: state.turn,
        cards_played_this_turn: state.cards_played_this_turn,
        attacks_played_this_turn: state.attacks_played_this_turn,
        stance: state.stance.as_str().to_string(),
        mantra: state.mantra,
        mantra_gained: state.mantra_gained,
        skip_enemy_turn: state.skip_enemy_turn,
        blasphemy_active: state.blasphemy_active,
        total_damage_dealt: state.total_damage_dealt,
        total_damage_taken: state.total_damage_taken,
        total_cards_played: state.total_cards_played,
        player_effects: collect_status_tokens(&state.player.statuses),
        hand: state
            .hand
            .iter()
            .map(|card| build_card_snapshot(engine, card))
            .collect(),
        draw_pile: state
            .draw_pile
            .iter()
            .map(|card| build_card_snapshot(engine, card))
            .collect(),
        discard_pile: state
            .discard_pile
            .iter()
            .map(|card| build_card_snapshot(engine, card))
            .collect(),
        exhaust_pile: state
            .exhaust_pile
            .iter()
            .map(|card| build_card_snapshot(engine, card))
            .collect(),
        enemies: state
            .enemies
            .iter()
            .enumerate()
            .map(|(enemy_index, enemy)| EnemySnapshotV1 {
                enemy_index,
                enemy_id: enemy.id.clone(),
                enemy_name: enemy.name.clone(),
                hp: enemy.entity.hp,
                max_hp: enemy.entity.max_hp,
                block: enemy.entity.block,
                back_attack: enemy.back_attack,
                move_id: enemy.move_id,
                intent_damage: enemy.move_damage(),
                intent_hits: enemy.move_hits(),
                intent_block: enemy.move_block(),
                first_turn: enemy.first_turn,
                is_escaping: enemy.is_escaping,
                statuses: collect_status_tokens(&enemy.entity.statuses),
            })
            .collect(),
        potions: state.potions.clone(),
        relics: state.relics.clone(),
        relic_counters: collect_relic_counter_tokens(&state.relic_counters),
        orb_slots: state.orb_slots.max_slots,
        rng_seed0,
        rng_seed1,
        rng_counter,
    }
}

pub fn combat_snapshot_from_run(run: &RunEngine) -> Option<CombatSnapshotV1> {
    run.get_combat_engine().map(combat_snapshot_from_combat)
}

pub fn combat_engine_from_snapshot(snapshot: &CombatSnapshotV1) -> CombatEngine {
    let mut enemies = Vec::with_capacity(snapshot.enemies.len());
    for enemy in &snapshot.enemies {
        let mut state =
            crate::state::EnemyCombatState::new(&enemy.enemy_id, enemy.hp, enemy.max_hp);
        state.name = enemy.enemy_name.clone();
        state.entity.block = enemy.block;
        apply_status_tokens(&mut state.entity.statuses, &enemy.statuses);
        state.back_attack = enemy.back_attack;
        state.set_move(
            enemy.move_id,
            enemy.intent_damage,
            enemy.intent_hits,
            enemy.intent_block,
        );
        state.first_turn = enemy.first_turn;
        state.is_escaping = enemy.is_escaping;
        enemies.push(state);
    }

    let registry = crate::cards::global_registry();
    let mut state = CombatState::new(
        snapshot.player_hp,
        snapshot.player_max_hp,
        enemies,
        restore_card_snapshots(&registry, &snapshot.draw_pile),
        snapshot.max_energy,
    );
    state.player.block = snapshot.player_block;
    apply_status_tokens(&mut state.player.statuses, &snapshot.player_effects);
    state.energy = snapshot.energy;
    state.max_energy = snapshot.max_energy;
    state.turn = snapshot.turn;
    state.cards_played_this_turn = snapshot.cards_played_this_turn;
    state.attacks_played_this_turn = snapshot.attacks_played_this_turn;
    state.stance = crate::state::Stance::from_str(&snapshot.stance);
    state.mantra = snapshot.mantra;
    state.mantra_gained = snapshot.mantra_gained;
    state.skip_enemy_turn = snapshot.skip_enemy_turn;
    state.blasphemy_active = snapshot.blasphemy_active;
    state.total_damage_dealt = snapshot.total_damage_dealt;
    state.total_damage_taken = snapshot.total_damage_taken;
    state.total_cards_played = snapshot.total_cards_played;
    state.hand = restore_card_snapshots(&registry, &snapshot.hand);
    state.draw_pile = restore_card_snapshots(&registry, &snapshot.draw_pile);
    state.discard_pile = restore_card_snapshots(&registry, &snapshot.discard_pile);
    state.exhaust_pile = restore_card_snapshots(&registry, &snapshot.exhaust_pile);
    state.potions = snapshot.potions.clone();
    state.relics = snapshot.relics.clone();
    state.orb_slots = crate::orbs::OrbSlots::new(snapshot.orb_slots);
    for counter in &snapshot.relic_counters {
        if let Some(idx) = counter_index(&counter.counter_name) {
            state.relic_counters[idx] = counter.value as i16;
        }
    }

    let mut engine = CombatEngine::new(state, 0);
    engine.rng = crate::seed::StsRandom::from_state(
        snapshot.rng_seed0,
        snapshot.rng_seed1,
        snapshot.rng_counter,
    );
    engine.phase = if snapshot.turn <= 0 {
        crate::engine::CombatPhase::NotStarted
    } else {
        crate::engine::CombatPhase::PlayerTurn
    };
    engine
}

pub fn restricted_legal_decision_actions(
    run: &RunEngine,
    policy: &RestrictionPolicyV1,
) -> Vec<DecisionAction> {
    let context = run.current_decision_context();
    run.get_legal_decision_actions()
        .into_iter()
        .filter(|action| action_allowed(action, &context, policy))
        .collect()
}

fn combat_execution_ids_for_allowed_actions(
    run: &RunEngine,
    policy: &RestrictionPolicyV1,
) -> Vec<i32> {
    restricted_legal_decision_actions(run, policy)
        .into_iter()
        .filter_map(|action| match action {
            DecisionAction::Combat(combat_action) => {
                Some(crate::encode_combat_action(&combat_action))
            }
            _ => None,
        })
        .collect()
}

fn build_combat_observation(engine: &CombatEngine) -> CombatObservationSchemaV1 {
    let state = &engine.state;
    CombatObservationSchemaV1 {
        schema_version: COMBAT_OBSERVATION_SCHEMA_VERSION,
        caps: CombatObservationCapsV1::default(),
        global: CombatGlobalTokenV1 {
            turn: state.turn,
            energy: state.energy,
            max_energy: state.max_energy,
            cards_played_this_turn: state.cards_played_this_turn,
            attacks_played_this_turn: state.attacks_played_this_turn,
            hand_size: state.hand.len(),
            draw_pile_size: state.draw_pile.len(),
            discard_pile_size: state.discard_pile.len(),
            exhaust_pile_size: state.exhaust_pile.len(),
            potion_slots: state.potions.len(),
            orb_slot_count: state.orb_slots.max_slots,
            occupied_orb_slots: state.orb_slots.occupied_count(),
            player_hp: state.player.hp,
            player_max_hp: state.player.max_hp,
            player_block: state.player.block,
            stance: state.stance.as_str().to_string(),
            mantra: state.mantra,
            mantra_gained: state.mantra_gained,
            skip_enemy_turn: state.skip_enemy_turn,
            blasphemy_active: state.blasphemy_active,
            combat_over: state.combat_over,
            player_won: state.player_won,
            total_damage_dealt: state.total_damage_dealt,
            total_damage_taken: state.total_damage_taken,
            total_cards_played: state.total_cards_played,
        },
        player: PlayerTokenV1 {
            hp: state.player.hp,
            max_hp: state.player.max_hp,
            block: state.player.block,
            stance: state.stance.as_str().to_string(),
            strength: state.player.strength(),
            dexterity: state.player.dexterity(),
            focus: state.player.focus(),
            weak: state.player.status(crate::status_ids::sid::WEAKENED),
            vulnerable: state.player.status(crate::status_ids::sid::VULNERABLE),
            frail: state.player.status(crate::status_ids::sid::FRAIL),
            relics: state.relics.clone(),
        },
        hand: state
            .hand
            .iter()
            .enumerate()
            .map(|(hand_index, card)| build_card_token(engine, hand_index, card))
            .collect(),
        enemies: state
            .enemies
            .iter()
            .enumerate()
            .map(|(enemy_index, enemy)| EnemyTokenV1 {
                enemy_index,
                enemy_id: enemy.id.clone(),
                enemy_name: enemy.name.clone(),
                hp: enemy.entity.hp,
                max_hp: enemy.entity.max_hp,
                block: enemy.entity.block,
                alive: enemy.is_alive(),
                targetable: enemy.is_targetable(),
                back_attack: enemy.has_back_attack(),
                intent: intent_name(enemy.intent),
                intent_damage: enemy.move_damage(),
                intent_hits: enemy.move_hits(),
                intent_block: enemy.move_block(),
            })
            .collect(),
        player_effects: collect_status_tokens(&state.player.statuses),
        enemy_effects: collect_enemy_status_tokens(state),
        orbs: state
            .orb_slots
            .slots
            .iter()
            .enumerate()
            .filter(|(_, orb)| orb.orb_type != OrbType::Empty)
            .map(|(slot_index, orb)| OrbTokenV1 {
                slot_index,
                orb_type: orb.orb_type.as_str().to_string(),
                base_passive: orb.base_passive,
                base_evoke: orb.base_evoke,
                evoke_amount: orb.evoke_amount,
            })
            .collect(),
        relic_counters: collect_relic_counter_tokens(&state.relic_counters),
        choice: build_choice_context(engine),
    }
}

fn build_card_token(
    engine: &CombatEngine,
    hand_index: usize,
    card: &crate::combat_types::CardInstance,
) -> CardTokenV1 {
    let def = engine.card_registry.card_def_by_id(card.def_id);
    CardTokenV1 {
        hand_index,
        card_id: def.id.to_string(),
        card_name: def.name.to_string(),
        card_type: format!("{:?}", def.card_type),
        target: format!("{:?}", def.target),
        cost_for_turn: card.cost.max(-1) as i32,
        base_cost: card.base_cost.max(-1) as i32,
        misc: card.misc as i32,
        upgraded: card.is_upgraded(),
        free_to_play: card.is_free(),
        retained: card.is_retained(),
        ethereal: card.is_ethereal(),
        runtime_only: def.is_runtime_only(),
        x_cost: def.uses_x_cost(),
        multi_hit: def.uses_multi_hit_hint(),
    }
}

fn build_card_snapshot(
    engine: &CombatEngine,
    card: &crate::combat_types::CardInstance,
) -> CardSnapshotV1 {
    let def = engine.card_registry.card_def_by_id(card.def_id);
    CardSnapshotV1 {
        card_id: def.id.to_string(),
        cost_for_turn: card.cost.max(-1) as i32,
        base_cost: card.base_cost.max(-1) as i32,
        misc: card.misc as i32,
        upgraded: card.is_upgraded(),
        free_to_play: card.is_free(),
        retained: card.is_retained(),
        ethereal: card.is_ethereal(),
    }
}

fn restore_card_snapshots(
    registry: &crate::cards::CardRegistry,
    cards: &[CardSnapshotV1],
) -> Vec<crate::combat_types::CardInstance> {
    cards
        .iter()
        .map(|card| {
            let mut restored = registry.make_card(&card.card_id);
            restored.cost = card.cost_for_turn as i8;
            restored.base_cost = card.base_cost as i8;
            restored.misc = card.misc as i16;
            restored.set_retained(card.retained);
            if card.upgraded {
                restored.flags |= crate::combat_types::CardInstance::FLAG_UPGRADED;
            }
            if card.free_to_play {
                restored.flags |= crate::combat_types::CardInstance::FLAG_FREE;
            }
            if card.ethereal {
                restored.flags |= crate::combat_types::CardInstance::FLAG_ETHEREAL;
            }
            restored
        })
        .collect()
}

fn collect_status_tokens(statuses: &[i16; 256]) -> Vec<StatusTokenV1> {
    statuses
        .iter()
        .enumerate()
        .filter_map(|(idx, amount)| {
            if *amount == 0 {
                None
            } else {
                Some(StatusTokenV1 {
                    status_id: idx as u16,
                    status_name: crate::status_ids::status_name(crate::ids::StatusId(idx as u16))
                        .to_string(),
                    amount: *amount as i32,
                })
            }
        })
        .collect()
}

fn apply_status_tokens(statuses: &mut [i16; 256], tokens: &[StatusTokenV1]) {
    *statuses = [0; 256];
    for token in tokens {
        let idx = token.status_id as usize;
        if idx < statuses.len() {
            statuses[idx] = token.amount as i16;
        }
    }
}

fn collect_enemy_status_tokens(state: &CombatState) -> Vec<EnemyStatusTokenV1> {
    let mut tokens = Vec::new();
    for (enemy_index, enemy) in state.enemies.iter().enumerate() {
        for (status_idx, amount) in enemy.entity.statuses.iter().enumerate() {
            if *amount == 0 {
                continue;
            }
            tokens.push(EnemyStatusTokenV1 {
                enemy_index,
                status_id: status_idx as u16,
                status_name: crate::status_ids::status_name(crate::ids::StatusId(
                    status_idx as u16,
                ))
                .to_string(),
                amount: *amount as i32,
            });
        }
    }
    tokens
}

fn collect_relic_counter_tokens(
    counters: &[i16; relic_flags::counter::NUM_COUNTERS],
) -> Vec<RelicCounterTokenV1> {
    counter_names()
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            let value = counters[idx];
            if value == 0 {
                None
            } else {
                Some(RelicCounterTokenV1 {
                    counter_name: (*name).to_string(),
                    value: value as i32,
                })
            }
        })
        .collect()
}

fn counter_index(name: &str) -> Option<usize> {
    counter_names()
        .iter()
        .position(|candidate| *candidate == name)
}

fn build_choice_context(engine: &CombatEngine) -> CombatChoiceContextV1 {
    let Some(choice) = &engine.choice else {
        return CombatChoiceContextV1 {
            active: false,
            reason: None,
            min_picks: 0,
            max_picks: 0,
            selected: Vec::new(),
            options: Vec::new(),
        };
    };

    CombatChoiceContextV1 {
        active: true,
        reason: Some(choice_reason_name(choice.reason.clone()).to_string()),
        min_picks: choice.min_picks,
        max_picks: choice.max_picks,
        selected: choice.selected.clone(),
        options: choice
            .options
            .iter()
            .enumerate()
            .map(|(choice_index, option)| {
                let (kind, source_index, label) = choice_option_details(option, engine);
                CombatChoiceOptionV1 {
                    choice_index,
                    kind,
                    source_index,
                    label,
                    selected: choice.selected.contains(&choice_index),
                }
            })
            .collect(),
    }
}

fn build_legal_action_candidates(
    engine: &CombatEngine,
    execution_id_for_action: impl Fn(&Action) -> i32,
) -> Vec<LegalActionCandidateV1> {
    crate::search::stable_combat_actions(engine)
        .iter()
        .enumerate()
        .map(|(dense_index, action)| {
            build_candidate(engine, dense_index, action, &execution_id_for_action)
        })
        .collect()
}

fn build_candidate(
    engine: &CombatEngine,
    dense_index: usize,
    action: &Action,
    execution_id_for_action: &impl Fn(&Action) -> i32,
) -> LegalActionCandidateV1 {
    let execution_id = execution_id_for_action(action);
    match action {
        Action::EndTurn => LegalActionCandidateV1 {
            schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
            dense_index,
            execution_id,
            action_kind: "end_turn".to_string(),
            description: "End the current turn".to_string(),
            card: None,
            target: None,
            potion: None,
            choice: None,
        },
        Action::PlayCard {
            card_idx,
            target_idx,
        } => {
            let card_inst = engine.state.hand.get(*card_idx).expect("legal hand index");
            let def = engine.card_registry.card_def_by_id(card_inst.def_id);
            LegalActionCandidateV1 {
                schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
                dense_index,
                execution_id,
                action_kind: "play_card".to_string(),
                description: format!("Play {}", def.name),
                card: Some(CandidateCardFeaturesV1 {
                    hand_index: *card_idx,
                    card_id: def.id.to_string(),
                    card_name: def.name.to_string(),
                    card_type: format!("{:?}", def.card_type),
                    cost_for_turn: card_inst.cost.max(-1) as i32,
                    base_cost: card_inst.base_cost.max(-1) as i32,
                    upgraded: card_inst.is_upgraded(),
                    x_cost: def.uses_x_cost(),
                    multi_hit: def.uses_multi_hit_hint(),
                    free_to_play: card_inst.is_free(),
                }),
                target: candidate_target(engine, *target_idx),
                potion: None,
                choice: None,
            }
        }
        Action::UsePotion {
            potion_idx,
            target_idx,
        } => {
            let potion_id = engine
                .state
                .potions
                .get(*potion_idx)
                .cloned()
                .unwrap_or_default();
            LegalActionCandidateV1 {
                schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
                dense_index,
                execution_id,
                action_kind: "use_potion".to_string(),
                description: format!("Use {}", potion_id),
                card: None,
                target: candidate_target(engine, *target_idx),
                potion: Some(CandidatePotionFeaturesV1 {
                    slot: *potion_idx,
                    potion_id,
                    target_required: *target_idx >= 0,
                }),
                choice: None,
            }
        }
        Action::ConfirmSelection => LegalActionCandidateV1 {
            schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
            dense_index,
            execution_id,
            action_kind: "confirm_selection".to_string(),
            description: "Confirm the current multi-pick selection".to_string(),
            card: None,
            target: None,
            potion: None,
            choice: None,
        },
        Action::Choose(choice_idx) => {
            let (kind, source_index, label) = engine
                .choice
                .as_ref()
                .and_then(|ctx| ctx.options.get(*choice_idx))
                .map(|option| choice_option_details(option, engine))
                .unwrap_or_else(|| ("unknown".to_string(), -1, format!("choice_{}", choice_idx)));
            LegalActionCandidateV1 {
                schema_version: ACTION_CANDIDATE_SCHEMA_VERSION,
                dense_index,
                execution_id,
                action_kind: "choose_option".to_string(),
                description: format!("Choose {}", label),
                card: None,
                target: None,
                potion: None,
                choice: Some(CandidateChoiceFeaturesV1 {
                    choice_index: *choice_idx,
                    label,
                    kind,
                    source_index,
                }),
            }
        }
    }
}

fn candidate_target(engine: &CombatEngine, target_idx: i32) -> Option<CandidateTargetFeaturesV1> {
    if target_idx < 0 {
        return None;
    }
    let enemy_idx = target_idx as usize;
    let enemy = engine.state.enemies.get(enemy_idx)?;
    Some(CandidateTargetFeaturesV1 {
        enemy_index: enemy_idx,
        enemy_name: enemy.name.clone(),
        hp: enemy.entity.hp,
        block: enemy.entity.block,
        targetable: enemy.is_targetable(),
        back_attack: enemy.back_attack,
    })
}

fn action_allowed(
    action: &DecisionAction,
    context: &crate::decision::DecisionContext,
    policy: &RestrictionPolicyV1,
) -> bool {
    if policy.has_builtin(RestrictionBuiltinV1::NoCardRewards)
        && reward_action_adds_card(action, context)
    {
        return false;
    }

    if policy.has_builtin(RestrictionBuiltinV1::NoCardAdds) && action_adds_card(action, context) {
        return false;
    }

    if policy.has_builtin(RestrictionBuiltinV1::UpgradeRemoveOnly) {
        if action_adds_card(action, context) {
            return false;
        }
        if matches!(context.kind, DecisionKind::CampfireAction)
            && !matches!(action, DecisionAction::CampfireUpgrade(_))
        {
            return false;
        }
        if matches!(context.kind, DecisionKind::ShopAction)
            && !matches!(
                action,
                DecisionAction::ShopRemoveCard(_) | DecisionAction::ShopLeave
            )
        {
            return false;
        }
    }

    true
}

fn reward_action_adds_card(
    action: &DecisionAction,
    context: &crate::decision::DecisionContext,
) -> bool {
    let Some(screen) = context.reward_screen.as_ref() else {
        return false;
    };
    match action {
        DecisionAction::ClaimRewardItem { item_index } => screen
            .items
            .iter()
            .find(|item| item.index == *item_index)
            .is_some_and(|item| item.kind == RewardItemKind::CardChoice),
        DecisionAction::PickRewardChoice {
            item_index,
            choice_index: _,
        } => screen
            .items
            .iter()
            .find(|item| item.index == *item_index)
            .is_some_and(|item| item.kind == RewardItemKind::CardChoice),
        _ => false,
    }
}

fn action_adds_card(action: &DecisionAction, context: &crate::decision::DecisionContext) -> bool {
    if reward_action_adds_card(action, context) {
        return true;
    }
    matches!(action, DecisionAction::ShopBuyCard(_))
}

fn intent_name(intent: crate::combat_types::Intent) -> String {
    format!("{intent:?}")
}

fn counter_names() -> [&'static str; relic_flags::counter::NUM_COUNTERS] {
    [
        "NunchakuCounter",
        "IncenseBurnerCounter",
        "InkBottleCounter",
        "HappyFlowerCounter",
        "MawBankGold",
        "OmamoriUses",
        "MatryoshkaUses",
        "UnusedCounter7",
    ]
}

fn choice_reason_name(reason: ChoiceReason) -> &'static str {
    match reason {
        ChoiceReason::Scry => "scry",
        ChoiceReason::DiscardFromHand => "discard_from_hand",
        ChoiceReason::ExhaustFromHand => "exhaust_from_hand",
        ChoiceReason::PutOnTopFromHand => "put_on_top_from_hand",
        ChoiceReason::PickFromDiscard => "pick_from_discard",
        ChoiceReason::PickFromDrawPile => "pick_from_draw_pile",
        ChoiceReason::DiscoverCard => "discover_card",
        ChoiceReason::PickOption => "pick_option",
        ChoiceReason::PlayCardFree => "play_card_free",
        ChoiceReason::DualWield => "dual_wield",
        ChoiceReason::UpgradeCard => "upgrade_card",
        ChoiceReason::PickFromExhaust => "pick_from_exhaust",
        ChoiceReason::SearchDrawPile => "search_draw_pile",
        ChoiceReason::ReturnFromDiscard => "return_from_discard",
        ChoiceReason::ForethoughtPick => "forethought_pick",
        ChoiceReason::RecycleCard => "recycle_card",
        ChoiceReason::DiscardForEffect => "discard_for_effect",
        ChoiceReason::SetupPick => "setup_pick",
        ChoiceReason::PlayCardFreeFromDraw => "play_card_free_from_draw",
    }
}

fn choice_option_details(option: &ChoiceOption, engine: &CombatEngine) -> (String, i32, String) {
    match option {
        ChoiceOption::HandCard(idx) => {
            let label = engine
                .state
                .hand
                .get(*idx)
                .map(|card| {
                    engine
                        .card_registry
                        .card_def_by_id(card.def_id)
                        .name
                        .to_string()
                })
                .unwrap_or_else(|| format!("hand_{}", idx));
            ("hand_card".to_string(), *idx as i32, label)
        }
        ChoiceOption::DrawCard(idx) => {
            let label = engine
                .state
                .draw_pile
                .get(*idx)
                .map(|card| {
                    engine
                        .card_registry
                        .card_def_by_id(card.def_id)
                        .name
                        .to_string()
                })
                .unwrap_or_else(|| format!("draw_{}", idx));
            ("draw_card".to_string(), *idx as i32, label)
        }
        ChoiceOption::DiscardCard(idx) => {
            let label = engine
                .state
                .discard_pile
                .get(*idx)
                .map(|card| {
                    engine
                        .card_registry
                        .card_def_by_id(card.def_id)
                        .name
                        .to_string()
                })
                .unwrap_or_else(|| format!("discard_{}", idx));
            ("discard_card".to_string(), *idx as i32, label)
        }
        ChoiceOption::RevealedCard(card) => (
            "revealed_card".to_string(),
            -1,
            engine
                .card_registry
                .card_def_by_id(card.def_id)
                .name
                .to_string(),
        ),
        ChoiceOption::GeneratedCard(card) => (
            "generated_card".to_string(),
            -1,
            engine
                .card_registry
                .card_def_by_id(card.def_id)
                .name
                .to_string(),
        ),
        ChoiceOption::Named(name) => ("named".to_string(), -1, (*name).to_string()),
        ChoiceOption::ExhaustCard(idx) => {
            let label = engine
                .state
                .exhaust_pile
                .get(*idx)
                .map(|card| {
                    engine
                        .card_registry
                        .card_def_by_id(card.def_id)
                        .name
                        .to_string()
                })
                .unwrap_or_else(|| format!("exhaust_{}", idx));
            ("exhaust_card".to_string(), *idx as i32, label)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::ShopState;
    use crate::tests::support::{engine_with, make_deck, run_engine};

    #[test]
    fn combat_training_state_exposes_tokens_and_candidates() {
        let engine = engine_with(make_deck(&["Strike_R", "Defend_R"]), 20, 5);
        let state = combat_training_state_from_combat(&engine, crate::encode_combat_action);

        assert_eq!(state.schema_versions.combat_observation_schema_version, 1);
        assert_eq!(state.observation.schema_version, 1);
        assert!(!state.observation.hand.is_empty());
        assert_eq!(state.observation.enemies.len(), 1);
        assert!(state
            .legal_candidates
            .iter()
            .any(|candidate| candidate.action_kind == "end_turn"));
        assert!(state
            .legal_candidates
            .iter()
            .any(|candidate| candidate.action_kind == "play_card"
                && candidate
                    .card
                    .as_ref()
                    .is_some_and(|card| card.card_id == "Strike_R")));
    }

    #[test]
    fn no_card_rewards_policy_filters_card_reward_claims() {
        let mut engine = run_engine(42, 20);
        engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

        let baseline = restricted_legal_decision_actions(&engine, &RestrictionPolicyV1::default());
        assert!(baseline
            .iter()
            .any(|action| matches!(action, DecisionAction::ClaimRewardItem { .. })));

        let filtered = restricted_legal_decision_actions(
            &engine,
            &RestrictionPolicyV1 {
                builtins: vec![RestrictionBuiltinV1::NoCardRewards],
            },
        );
        assert!(!filtered
            .iter()
            .any(|action| matches!(action, DecisionAction::ClaimRewardItem { .. })));
    }

    #[test]
    fn upgrade_remove_only_filters_shop_and_campfire_actions() {
        let mut shop_engine = run_engine(42, 20);
        shop_engine.debug_set_shop_state(ShopState {
            cards: vec![("Wallop".to_string(), 50)],
            remove_price: 75,
            removal_used: false,
        });
        shop_engine.debug_enter_shop();

        let filtered_shop = restricted_legal_decision_actions(
            &shop_engine,
            &RestrictionPolicyV1 {
                builtins: vec![RestrictionBuiltinV1::UpgradeRemoveOnly],
            },
        );
        assert!(filtered_shop.iter().all(|action| matches!(
            action,
            DecisionAction::ShopRemoveCard(_) | DecisionAction::ShopLeave
        )));

        let mut campfire_engine = run_engine(7, 20);
        campfire_engine.debug_set_campfire_phase();
        let filtered_campfire = restricted_legal_decision_actions(
            &campfire_engine,
            &RestrictionPolicyV1 {
                builtins: vec![RestrictionBuiltinV1::UpgradeRemoveOnly],
            },
        );
        assert!(filtered_campfire
            .iter()
            .all(|action| matches!(action, DecisionAction::CampfireUpgrade(_))));
    }

    #[test]
    fn combat_snapshot_roundtrip_preserves_training_surface() {
        let mut engine = engine_with(make_deck(&["Strike_P", "Defend_P", "Eruption"]), 30, 12);
        engine.state.potions = vec!["FlexPotion".to_string(), "".to_string(), "".to_string()];
        engine.state.relics = vec!["PureWater".to_string()];
        engine.state.relic_counters[crate::relic_flags::counter::INK_BOTTLE] = 6;

        let snapshot = combat_snapshot_from_combat(&engine);
        let restored = combat_engine_from_snapshot(&snapshot);
        let restored_state =
            combat_training_state_from_combat(&restored, crate::encode_combat_action);

        assert_eq!(snapshot.player_hp, restored.state.player.hp);
        assert_eq!(snapshot.potions[0], "FlexPotion");
        assert_eq!(restored_state.observation.relic_counters.len(), 1);
        assert_eq!(
            restored_state.observation.hand.len(),
            engine.state.hand.len()
        );
        assert!(restored_state
            .legal_candidates
            .iter()
            .any(|candidate| candidate.action_kind == "end_turn"));
    }
}
