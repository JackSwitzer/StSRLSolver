//! Offline intake and comparison for human-recorded run bundles.
//!
//! The recorder dialect is intentionally adapted here instead of weakening
//! the canonical `GameAction` or language-neutral oracle-state contracts.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::actions::Action;
use crate::decision::{RewardChoice, RewardItemKind, RewardKeyColor};
use crate::run::{ActionStatus, GameAction, RunEngine, RunPhase};

use super::oracle_v2::{diff_partial_oracle_state, project_oracle_state, OracleStateFieldDiff};
use super::TraceProfileSnapshot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingMeta {
    pub v: u32,
    pub run_id: String,
    pub seed_long: i64,
    pub seed_display: String,
    pub character: String,
    pub ascension: i32,
    pub game_version: String,
    pub status: String,
    pub records: usize,
    #[serde(default)]
    pub sittings: Vec<String>,
    /// Authoritative recorder-captured profile inputs. Legacy bundles omit
    /// this additive field and remain replayable but initialization-uncertified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<TraceProfileSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecorderAction {
    pub idx: usize,
    pub action_type: String,
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedTraceRecord {
    pub idx: usize,
    pub floor: i32,
    pub act: i32,
    pub turn: i32,
    pub phase: String,
    pub screen: String,
    pub map: Value,
    pub action: Value,
    pub post: Value,
    pub deck: Value,
}

impl RecordedTraceRecord {
    /// Project only fields that the current Java recorder actually emits.
    pub fn partial_oracle_state(&self, compare_phase: bool) -> Result<Value, String> {
        let post = self
            .post
            .as_object()
            .ok_or_else(|| format!("trace action {} has non-object post state", self.idx))?;
        let mut state = Map::new();
        state.insert("floor".to_string(), json!(self.floor));
        state.insert("act".to_string(), json!(self.act));
        state.insert("turn".to_string(), json!(self.turn));
        state.insert("map".to_string(), self.map.clone());
        for key in ["player", "enemies", "piles", "relics", "potions", "rng"] {
            state.insert(
                key.to_string(),
                post.get(key)
                    .cloned()
                    .ok_or_else(|| format!("trace action {} omitted post.{key}", self.idx))?,
            );
        }
        // TraceWriter samples EnergyPanel.totalCount even outside combat,
        // where it is stale UI state rather than spendable run state.
        let path_combat_entry = self.phase == "COMBAT"
            && self.action.get("type").and_then(Value::as_str) == Some("PATH");
        if self.phase != "COMBAT" || path_combat_entry {
            state
                .get_mut("player")
                .and_then(Value::as_object_mut)
                .and_then(|player| player.remove("energy"));
        }
        // The map-click recorder hook fires before AbstractMonster's first
        // createIntent update, so Java still exposes transient DEBUG names.
        // All causal move fields remain compared on this record.
        if path_combat_entry {
            state.remove("piles");
        }
        if let Some(enemies) = state.get_mut("enemies").and_then(Value::as_array_mut) {
            for enemy in enemies {
                let Some(intent) = enemy.get_mut("intent").and_then(Value::as_object_mut) else {
                    continue;
                };
                let is_attack = intent
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.starts_with("ATTACK"));
                let is_transient_debug =
                    intent.get("name").and_then(Value::as_str) == Some("DEBUG");
                if !is_attack && !is_transient_debug {
                    // AbstractMonster.createIntent leaves intentDmg stale when
                    // a non-attack follows an attack. Canonical v2 uses -1.
                    intent.remove("dmg");
                }
                if path_combat_entry && is_transient_debug {
                    intent.remove("name");
                    intent.remove("dmg");
                    intent.remove("hits");
                }
            }
        }
        if matches!(self.screen.as_str(), "COMBAT_REWARD" | "CARD_REWARD") {
            strip_recorded_combat_transients(&mut state);
        }
        state.insert("deck".to_string(), self.deck.clone());
        if compare_phase {
            if let Some(phase) = canonical_recorder_phase(&self.phase, &self.screen, &self.action) {
                state.insert("phase".to_string(), json!(phase));
            }
        }
        Ok(Value::Object(state))
    }
}

#[derive(Debug, Clone)]
pub struct RecordingBundle {
    pub path: PathBuf,
    pub meta: RecordingMeta,
    pub actions: Vec<RecorderAction>,
    pub records: Vec<RecordedTraceRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleComparisonStatus {
    Match,
    Uncertified,
    Diverged,
    NoActions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleFirstDivergence {
    pub idx: usize,
    pub kind: String,
    pub path: String,
    pub expected: Value,
    pub actual: Value,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub differences: Vec<OracleStateFieldDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleInferredAction {
    /// Recorder action that could not be reached without this omitted action.
    pub before_idx: usize,
    pub action: GameAction,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleIgnoredRecorderAction {
    pub idx: usize,
    pub action: Value,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleUnverifiedAction {
    pub idx: usize,
    pub action: Value,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleInitializationQuarantine {
    pub kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleComparisonReport {
    pub status: BundleComparisonStatus,
    pub bundle: String,
    pub seed: i64,
    pub ascension: i32,
    pub outcome: String,
    /// Present when the replay had to use compatibility defaults for missing
    /// authoritative run-start inputs. Such a report is never certifiable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_quarantine: Option<BundleInitializationQuarantine>,
    /// Recorder actions with their own directly compared checkpoint.
    pub matched_actions: usize,
    /// Longest comparable action prefix, including coupled callbacks whose
    /// shared settled state was checked by the next direct checkpoint.
    pub comparable_actions: usize,
    /// Checkpoints that directly compared at least one recorded state field.
    pub matched_checkpoints: usize,
    /// Actions whose recorder state is coupled to the following checkpoint.
    pub coupled_actions: usize,
    /// Legacy actions whose effect was compared but semantic identity was absent.
    pub unverified_action_semantics: usize,
    /// Recorder actions whose resulting state can be checked but whose omitted
    /// identity prevents full semantic certification.
    pub unverified_actions: Vec<BundleUnverifiedAction>,
    /// Canonical actions absent from the recorder but uniquely forced by the
    /// following recorded transition. These are replay aids, not certification.
    pub inferred_actions: Vec<BundleInferredAction>,
    /// UI callbacks that Java recorded although its settled state proves the
    /// attempted action had no gameplay effect.
    pub ignored_recorder_actions: Vec<BundleIgnoredRecorderAction>,
    pub total_actions: usize,
    pub replayed_actions: usize,
    pub skipped_fields_total: u64,
    pub skipped_fields_by_path: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_divergence: Option<BundleFirstDivergence>,
}

impl BundleComparisonReport {
    fn mark_uncertified(&mut self) {
        if self.status == BundleComparisonStatus::Match {
            self.status = BundleComparisonStatus::Uncertified;
        }
    }
}

pub fn load_recording_meta(path: impl AsRef<Path>) -> Result<RecordingMeta, String> {
    let path = path.as_ref();
    let meta_path = path.join("meta.json");
    let file = File::open(&meta_path)
        .map_err(|error| format!("failed to open {}: {error}", meta_path.display()))?;
    let meta: RecordingMeta = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| format!("failed to parse {}: {error}", meta_path.display()))?;
    if meta.v != 1 {
        return Err(format!(
            "unsupported recording metadata version {} in {}",
            meta.v,
            meta_path.display()
        ));
    }
    if meta.character != "WATCHER" {
        return Err(format!(
            "recording {} uses unsupported character {}",
            meta.run_id, meta.character
        ));
    }
    if let Some(profile) = &meta.profile {
        profile.check_version().map_err(|error| {
            format!(
                "recording {} has invalid profile snapshot: {error}",
                meta.run_id
            )
        })?;
    }
    Ok(meta)
}

pub fn load_recording_bundle(path: impl AsRef<Path>) -> Result<RecordingBundle, String> {
    let path = path.as_ref();
    let meta = load_recording_meta(path)?;
    let actions = load_script_actions(&path.join("script.jsonl"))?;
    let records = load_trace_records(&path.join("trace.jsonl.gz"))?;
    validate_bundle_alignment(&meta, &actions, &records)?;
    Ok(RecordingBundle {
        path: path.to_path_buf(),
        meta,
        actions,
        records,
    })
}

fn load_script_actions(path: &Path) -> Result<Vec<RecorderAction>, String> {
    let file =
        File::open(path).map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    let mut actions = Vec::new();
    for (line_index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            format!(
                "failed to parse {} line {}: {error}",
                path.display(),
                line_index + 1
            )
        })?;
        if value.get("lifecycle").is_some() {
            continue;
        }
        let idx = required_usize(&value, "idx", path, line_index + 1)?;
        let action_type = required_str(&value, "type", path, line_index + 1)?.to_string();
        actions.push(RecorderAction {
            idx,
            action_type,
            raw: value,
        });
    }
    Ok(actions)
}

fn load_trace_records(path: &Path) -> Result<Vec<RecordedTraceRecord>, String> {
    let file =
        File::open(path).map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    parse_trace_records(MultiGzDecoder::new(file), path)
}

fn parse_trace_records(reader: impl Read, path: &Path) -> Result<Vec<RecordedTraceRecord>, String> {
    let mut reader = BufReader::new(reader);
    let mut records = Vec::new();
    let mut line = String::new();
    let mut line_index = 0usize;
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|error| format!("failed to decompress {}: {error}", path.display()))?;
        if bytes == 0 {
            break;
        }
        line_index += 1;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            format!(
                "failed to parse {} line {}: {error}",
                path.display(),
                line_index
            )
        })?;
        if value.get("kind").is_some() {
            continue;
        }
        let v = value
            .get("v")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("{} line {} omitted v", path.display(), line_index))?;
        if v != 1 {
            return Err(format!(
                "unsupported trace record version {v} in {} line {}",
                path.display(),
                line_index
            ));
        }
        records.push(RecordedTraceRecord {
            idx: required_usize(&value, "idx", path, line_index)?,
            floor: required_i32(&value, "floor", path, line_index)?,
            act: required_i32(&value, "act", path, line_index)?,
            turn: required_i32(&value, "turn", path, line_index)?,
            phase: required_str(&value, "phase", path, line_index)?.to_string(),
            screen: required_str(&value, "screen", path, line_index)?.to_string(),
            map: required_value(&value, "map", path, line_index)?.clone(),
            action: required_value(&value, "action", path, line_index)?.clone(),
            post: required_value(&value, "post", path, line_index)?.clone(),
            deck: required_value(&value, "deck", path, line_index)?.clone(),
        });
    }
    Ok(records)
}

fn validate_bundle_alignment(
    meta: &RecordingMeta,
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
) -> Result<(), String> {
    if meta.records != records.len() {
        return Err(format!(
            "recording {} declares {} records but trace contains {}",
            meta.run_id,
            meta.records,
            records.len()
        ));
    }
    if actions.len() != records.len() {
        return Err(format!(
            "recording {} has {} script actions but {} trace records",
            meta.run_id,
            actions.len(),
            records.len()
        ));
    }
    for (position, (action, record)) in actions.iter().zip(records).enumerate() {
        if action.idx != position || record.idx != position {
            return Err(format!(
                "recording {} is non-contiguous at position {} (script {}, trace {})",
                meta.run_id, position, action.idx, record.idx
            ));
        }
        let trace_type = record
            .action
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("trace action {} omitted type", record.idx))?;
        if action.action_type != trace_type {
            return Err(format!(
                "recording {} action {} type mismatch: script {}, trace {}",
                meta.run_id, position, action.action_type, trace_type
            ));
        }
        let mut script_payload = action.raw.clone();
        let script_object = script_payload.as_object_mut().ok_or_else(|| {
            format!(
                "recording {} action {} is not an object",
                meta.run_id, position
            )
        })?;
        script_object.remove("idx");
        if script_payload != record.action {
            return Err(format!(
                "recording {} action {} payload mismatch: script {}, trace {}",
                meta.run_id, position, script_payload, record.action
            ));
        }
    }
    Ok(())
}

pub fn compare_recording_bundle(
    bundle: &RecordingBundle,
) -> Result<BundleComparisonReport, String> {
    let mut report = BundleComparisonReport {
        status: if bundle.records.is_empty() {
            BundleComparisonStatus::NoActions
        } else if bundle.meta.profile.is_none() {
            BundleComparisonStatus::Uncertified
        } else {
            BundleComparisonStatus::Match
        },
        bundle: bundle.meta.run_id.clone(),
        seed: bundle.meta.seed_long,
        ascension: bundle.meta.ascension,
        outcome: bundle.meta.status.clone(),
        initialization_quarantine: bundle.meta.profile.is_none().then(|| {
            BundleInitializationQuarantine {
                kind: "missing_profile_snapshot".to_string(),
                reason: "legacy recording omitted authoritative profile/unlock state; replay uses the ordinary API's all-unlocked compatibility default and cannot certify relic-pool initialization"
                    .to_string(),
            }
        }),
        matched_actions: 0,
        comparable_actions: 0,
        matched_checkpoints: 0,
        coupled_actions: 0,
        unverified_action_semantics: 0,
        unverified_actions: Vec::new(),
        inferred_actions: Vec::new(),
        ignored_recorder_actions: Vec::new(),
        total_actions: bundle.records.len(),
        replayed_actions: 0,
        skipped_fields_total: 0,
        skipped_fields_by_path: BTreeMap::new(),
        first_divergence: None,
    };
    if bundle.records.is_empty() {
        return Ok(report);
    }

    let mut engine = match &bundle.meta.profile {
        Some(profile) => {
            profile.check_version()?;
            RunEngine::new_with_profile(
                bundle.meta.seed_long as u64,
                bundle.meta.ascension,
                profile.to_engine_profile()?,
            )
        }
        None => RunEngine::new(bundle.meta.seed_long as u64, bundle.meta.ascension),
    };
    let mut adapter = RecorderActionAdapter::default();
    let mut pending_coupled_actions = 0usize;
    for (position, (action, record)) in bundle.actions.iter().zip(&bundle.records).enumerate() {
        match infer_missing_prerequisites(&mut engine, action) {
            Ok(inferred) => {
                if !inferred.is_empty() {
                    report.mark_uncertified();
                }
                report.inferred_actions.extend(inferred);
            }
            Err(detail) => {
                report.status = BundleComparisonStatus::Diverged;
                report.first_divergence = Some(BundleFirstDivergence {
                    idx: action.idx,
                    kind: "action_mapping".to_string(),
                    path: "action".to_string(),
                    expected: action.raw.clone(),
                    actual: serde_json::to_value(engine.get_legal_actions())
                        .unwrap_or_else(|_| json!("unserializable legal actions")),
                    detail,
                    differences: Vec::new(),
                });
                return Ok(report);
            }
        }
        let recorder_only_distilled_chaos_callback =
            is_distilled_chaos_recorder_only_callback(&bundle.actions, &bundle.records, position);
        let mapped = if recorder_only_distilled_chaos_callback {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "recorder emitted a Distilled Chaos internal PLAY_CARD callback after the potion's settled checkpoint; hand_idx is -1 and the entire callback chain has identical recorded state"
                        .to_string(),
                });
            None
        } else {
            match adapter.map(&engine, action, record, bundle.records.get(position + 1)) {
                Ok(mapped) => mapped,
                Err(detail) => {
                    report.status = BundleComparisonStatus::Diverged;
                    report.first_divergence = Some(BundleFirstDivergence {
                        idx: action.idx,
                        kind: "action_mapping".to_string(),
                        path: "action".to_string(),
                        expected: action.raw.clone(),
                        actual: serde_json::to_value(engine.get_legal_actions())
                            .unwrap_or_else(|_| json!("unserializable legal actions")),
                        detail,
                        differences: Vec::new(),
                    });
                    return Ok(report);
                }
            }
        };
        let semantic_neow_selection = matches!(&mapped, Some(GameAction::ChooseNeowOption(_)));
        let mapping_unverified = adapter.take_last_mapping_unverified();
        let mapping_coupled_to_following = adapter.take_last_mapping_coupled_to_following();
        if semantic_neow_selection {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report.unverified_actions.push(BundleUnverifiedAction {
                idx: action.idx,
                action: action.raw.clone(),
                reason:
                    "recorder stores only the selected Neow index, not the generated option payload"
                        .to_string(),
            });
        }
        if let Some(reason) = mapping_unverified {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report.unverified_actions.push(BundleUnverifiedAction {
                idx: action.idx,
                action: action.raw.clone(),
                reason,
            });
        }
        if let Some(game_action) = mapped {
            let outcome = engine.step_game(&game_action);
            if outcome.status != ActionStatus::Accepted {
                let previous_record = position
                    .checked_sub(1)
                    .and_then(|previous| bundle.records.get(previous));
                if is_unavailable_sapphire_click(action, previous_record, record) {
                    report.mark_uncertified();
                    report.unverified_action_semantics += 1;
                    report
                        .ignored_recorder_actions
                        .push(BundleIgnoredRecorderAction {
                            idx: action.idx,
                            action: action.raw.clone(),
                            reason: "recorder logged the unavailable Sapphire Key after its linked relic was claimed; Java state is unchanged"
                                .to_string(),
                        });
                } else {
                    report.status = BundleComparisonStatus::Diverged;
                    report.first_divergence = Some(BundleFirstDivergence {
                        idx: action.idx,
                        kind: "action_rejected".to_string(),
                        path: "action".to_string(),
                        expected: action.raw.clone(),
                        actual: serde_json::to_value(outcome.next_decision.legal_actions)
                            .unwrap_or_else(|_| json!("unserializable legal actions")),
                        detail: format!("canonical action {game_action:?} was rejected"),
                        differences: Vec::new(),
                    });
                    return Ok(report);
                }
            } else {
                report.replayed_actions += 1;
            }
        }

        let actual = project_oracle_state(&engine)?;
        // The recorder can attach one settled state to several adjacent action
        // callbacks. Defer comparison across the complete identical-state block
        // rather than pretending each callback has its own causal checkpoint.
        let state_coupled_to_following = mapping_coupled_to_following
            || bundle
                .records
                .get(position + 1)
                .is_some_and(|next| same_recorded_state(record, next));
        let lethal_reward_transition =
            matches!(action.action_type.as_str(), "PLAY_CARD" | "USE_POTION")
                && recorded_enemies_all_dead(record)
                && bundle
                    .actions
                    .get(position + 1)
                    .is_some_and(|next| next.action_type == "REWARD_TAKE");
        let mut expected = if state_coupled_to_following {
            // There is no independent oracle checkpoint for this action. The
            // final record in the identical-state block owns the comparison.
            json!({})
        } else {
            record.partial_oracle_state(!semantic_neow_selection)?
        };
        if lethal_reward_transition {
            if let Some(object) = expected.as_object_mut() {
                object.remove("phase");
                object.remove("rng");
                strip_recorded_combat_transients(object);
            }
        }
        let partial = diff_partial_oracle_state(&expected, &actual);
        let skipped_fields = partial.skipped_fields_total();
        if skipped_fields != 0 {
            report.mark_uncertified();
        }
        report.skipped_fields_total += skipped_fields;
        merge_counts(
            &mut report.skipped_fields_by_path,
            partial.skipped_fields_by_path,
        );
        if state_coupled_to_following {
            report.mark_uncertified();
            report.coupled_actions += 1;
            pending_coupled_actions += 1;
            continue;
        }
        if let Some(first) = partial.diffs.first() {
            let differing_paths = partial
                .diffs
                .iter()
                .take(12)
                .map(|diff| diff.path.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            report.status = BundleComparisonStatus::Diverged;
            report.first_divergence = Some(BundleFirstDivergence {
                idx: action.idx,
                kind: "state".to_string(),
                path: first.path.clone(),
                expected: first.expected.clone(),
                actual: first.actual.clone(),
                detail: format!(
                    "first of {} state differences after recorder action {}; paths: {}",
                    partial.diffs.len(),
                    action.action_type,
                    differing_paths,
                ),
                differences: partial.diffs.clone(),
            });
            return Ok(report);
        }
        report.matched_checkpoints += 1;
        report.matched_actions += 1;
        report.comparable_actions += 1 + pending_coupled_actions;
        pending_coupled_actions = 0;
    }
    Ok(report)
}

fn is_unavailable_sapphire_click(
    action: &RecorderAction,
    previous_record: Option<&RecordedTraceRecord>,
    record: &RecordedTraceRecord,
) -> bool {
    action.action_type == "REWARD_TAKE"
        && action.raw.get("reward_type").and_then(Value::as_str) == Some("SAPPHIRE_KEY")
        && previous_record.is_some_and(|previous| same_recorded_state(previous, record))
}

/// Bridge recorder UI omissions only when the next recorded action proves a
/// unique canonical route. The bridge is serialized in the report and never
/// counted as a matched recorder action.
fn infer_missing_prerequisites(
    engine: &mut RunEngine,
    action: &RecorderAction,
) -> Result<Vec<BundleInferredAction>, String> {
    if action.action_type == "EVENT_CHOICE" && engine.current_phase() == RunPhase::CardReward {
        let legal = engine.get_legal_actions();
        let mut item_opens = legal.iter().filter_map(|candidate| match candidate {
            GameAction::SelectRewardItem(item_index) => Some(*item_index),
            _ => None,
        });
        let first = item_opens.next();
        if item_opens.next().is_some() {
            return Err(format!(
                "recorder event-grid click omitted an ambiguous selection stage at action {}",
                action.idx
            ));
        }
        if let Some(item_index) = first {
            let game_action = GameAction::SelectRewardItem(item_index);
            let outcome = engine.step_game(&game_action);
            if outcome.status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical grid-open action {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            return Ok(vec![BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason:
                    "the recorder combines the event grid click with Rust's typed selection staging"
                        .to_string(),
            }]);
        }
    }

    if action.action_type != "PATH" {
        return Ok(Vec::new());
    }

    let mut inferred = Vec::new();
    if engine.current_phase() == RunPhase::Shop {
        let legal = engine.get_legal_actions();
        if legal.contains(&GameAction::ShopLeave) {
            let game_action = GameAction::ShopLeave;
            let outcome = engine.step_game(&game_action);
            if outcome.status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical inferred action {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "PATH proves the recorder left the active shop".to_string(),
            });
        }
    }
    while engine.current_phase() != RunPhase::MapChoice && engine.current_reward_screen().is_some()
    {
        let legal = engine.get_legal_actions();
        let mut skips = legal.iter().filter_map(|candidate| match candidate {
            GameAction::SkipRewardItem(item_index) => Some(*item_index),
            _ => None,
        });
        let first_skip = skips.next();
        if skips.next().is_some() {
            return Err(format!(
                "recorder omitted an ambiguous reward skip before PATH at action {}",
                action.idx
            ));
        }

        let (game_action, reason) = if let Some(item_index) = first_skip {
            (
                GameAction::SkipRewardItem(item_index),
                "PATH proves the recorder skipped the open card-reward choice",
            )
        } else if legal.contains(&GameAction::LeaveRewards) {
            (
                GameAction::LeaveRewards,
                "PATH proves the recorder left the completed reward screen",
            )
        } else {
            break;
        };

        let outcome = engine.step_game(&game_action);
        if outcome.status != ActionStatus::Accepted {
            return Err(format!(
                "canonical inferred action {game_action:?} was rejected before recorder action {}",
                action.idx
            ));
        }
        inferred.push(BundleInferredAction {
            before_idx: action.idx,
            action: game_action,
            reason: reason.to_string(),
        });
    }
    Ok(inferred)
}

/// Distilled Chaos resolves its internal card plays inside the canonical
/// potion action. The legacy recorder logs those internal callbacks afterward,
/// so they are non-actions only when the complete causal witness is present.
pub(crate) fn is_distilled_chaos_recorder_only_callback(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(record)) = (actions.get(position), records.get(position)) else {
        return false;
    };
    if !is_internal_card_play_callback(action) || position == 0 {
        return false;
    }

    let mut cursor = position;
    let mut preceding_internal_callbacks = 0usize;
    while cursor > 0 {
        cursor -= 1;
        let (Some(previous_action), Some(previous_record)) =
            (actions.get(cursor), records.get(cursor))
        else {
            return false;
        };
        if !same_recorded_state(previous_record, record) {
            return false;
        }
        if is_distilled_chaos_use(previous_action) {
            return preceding_internal_callbacks < 3;
        }
        if !is_internal_card_play_callback(previous_action) {
            return false;
        }
        preceding_internal_callbacks += 1;
        if preceding_internal_callbacks >= 3 {
            return false;
        }
    }
    false
}

fn is_internal_card_play_callback(action: &RecorderAction) -> bool {
    action.action_type == "PLAY_CARD"
        && action.raw.get("hand_idx").and_then(Value::as_i64) == Some(-1)
}

fn is_distilled_chaos_use(action: &RecorderAction) -> bool {
    action.action_type == "USE_POTION"
        && action.raw.get("potion_id").and_then(Value::as_str) == Some("DistilledChaos")
}

fn same_recorded_state(left: &RecordedTraceRecord, right: &RecordedTraceRecord) -> bool {
    left.floor == right.floor
        && left.act == right.act
        && left.turn == right.turn
        && left.phase == right.phase
        && left.screen == right.screen
        && left.map == right.map
        && left.post == right.post
        && left.deck == right.deck
}

fn recorded_enemies_all_dead(record: &RecordedTraceRecord) -> bool {
    record
        .post
        .get("enemies")
        .and_then(Value::as_array)
        .is_some_and(|enemies| {
            !enemies.is_empty()
                && enemies
                    .iter()
                    .all(|enemy| enemy.get("dead").and_then(Value::as_bool) == Some(true))
        })
}

fn strip_recorded_combat_transients(state: &mut Map<String, Value>) {
    state.remove("turn");
    state.remove("enemies");
    state.remove("piles");
    if let Some(player) = state.get_mut("player").and_then(Value::as_object_mut) {
        for key in ["block", "energy", "stance", "powers", "orbs"] {
            player.remove(key);
        }
    }
}

fn merge_counts(target: &mut BTreeMap<String, u64>, source: BTreeMap<String, u64>) {
    for (path, count) in source {
        *target.entry(path).or_default() += count;
    }
}

#[derive(Default)]
struct RecorderActionAdapter {
    neow_commits: usize,
    neow_grid_followup: bool,
    last_mapping_unverified: Option<String>,
    last_mapping_coupled_to_following: bool,
}

impl RecorderActionAdapter {
    fn take_last_mapping_unverified(&mut self) -> Option<String> {
        std::mem::take(&mut self.last_mapping_unverified)
    }

    fn take_last_mapping_coupled_to_following(&mut self) -> bool {
        std::mem::take(&mut self.last_mapping_coupled_to_following)
    }

    fn map(
        &mut self,
        engine: &RunEngine,
        action: &RecorderAction,
        record: &RecordedTraceRecord,
        next_record: Option<&RecordedTraceRecord>,
    ) -> Result<Option<GameAction>, String> {
        self.last_mapping_unverified = None;
        self.last_mapping_coupled_to_following = false;
        let raw = &action.raw;
        match action.action_type.as_str() {
            "NEOW" => {
                let commit = self.neow_commits;
                self.neow_commits += 1;
                match commit {
                    0 => Ok(Some(GameAction::Proceed)),
                    1 => {
                        self.neow_grid_followup = record.screen == "GRID";
                        Ok(Some(GameAction::ChooseNeowOption(value_usize(
                            raw, "choice",
                        )?)))
                    }
                    2 if self.neow_grid_followup => {
                        Err("Neow grid follow-up omits the selected deck-card identity".to_string())
                    }
                    2 => Ok(Some(GameAction::Proceed)),
                    _ => Err(format!(
                        "recorder emitted unexpected fourth Neow UI commit at action {}",
                        action.idx
                    )),
                }
            }
            "PATH" => {
                if engine.current_phase() != RunPhase::MapChoice {
                    if engine.current_reward_screen().is_some() {
                        return Err(
                            "recorder advanced to PATH while the canonical reward screen was still active; a reward skip/leave action is missing"
                                .to_string(),
                        );
                    }
                    return Err(format!(
                        "recorder advanced to PATH from canonical phase {:?}",
                        engine.current_phase()
                    ));
                }
                let x = value_i32(raw, "x")?;
                let y = value_i32(raw, "y")?;
                let context = engine.current_decision_context();
                let path = context
                    .map
                    .and_then(|map| {
                        map.paths
                            .into_iter()
                            .find(|path| path.x == x && path.y == y)
                    })
                    .ok_or_else(|| format!("no canonical map path reaches ({x}, {y})"))?;
                Ok(Some(GameAction::ChoosePath(path.choice)))
            }
            "PLAY_CARD" => Ok(Some(GameAction::CombatAction(Action::PlayCard {
                card_idx: validate_recorded_hand_card(engine, raw)?,
                target_idx: value_i32(raw, "target")?,
            }))),
            "END_TURN" => Ok(Some(GameAction::CombatAction(Action::EndTurn))),
            "USE_POTION" => {
                let slot = value_usize(raw, "slot")?;
                let target = value_i32(raw, "target")?;
                if engine.current_phase() == RunPhase::Combat {
                    Ok(Some(GameAction::CombatAction(Action::UsePotion {
                        potion_idx: slot,
                        target_idx: target,
                    })))
                } else {
                    Ok(Some(GameAction::UsePotion(slot)))
                }
            }
            "DISCARD_POTION" => Ok(Some(GameAction::DiscardPotion(value_usize(raw, "slot")?))),
            "EVENT_CHOICE" if engine.current_phase() == RunPhase::CardReward => {
                let (mapped, identity_verified) = map_recorded_event_grid_choice(engine, record)?;
                if !identity_verified {
                    self.last_mapping_unverified = Some(
                        "recorder grid result does not distinguish duplicate deck-card instances"
                            .to_string(),
                    );
                }
                Ok(Some(mapped))
            }
            "EVENT_CHOICE" => Ok(Some(GameAction::EventChoice(value_usize(raw, "choice")?))),
            "CHEST_OPEN" => Ok(Some(GameAction::OpenChest)),
            "CAMPFIRE" => match value_str(raw, "choice")? {
                "REST" => Ok(Some(GameAction::CampfireRest)),
                "RECALL" => Ok(Some(GameAction::CampfireRecall)),
                "LIFT" => Ok(Some(GameAction::CampfireLift)),
                "DIG" => Ok(Some(GameAction::CampfireDig)),
                "TOKE" => Ok(Some(GameAction::CampfireToke)),
                "SMITH" => {
                    let next_record = next_record.ok_or_else(|| {
                        "recorder CAMPFIRE/SMITH ends before the upgraded deck is visible"
                            .to_string()
                    })?;
                    let mapped = map_recorded_campfire_upgrade(engine, next_record)?;
                    self.last_mapping_unverified = Some(
                        "recorder SMITH action omits the selected deck-card identity; the next ordered deck determines the upgrade"
                            .to_string(),
                    );
                    self.last_mapping_coupled_to_following = true;
                    Ok(Some(mapped))
                }
                other => Err(format!("unknown campfire choice {other:?}")),
            },
            "SHOP_BUY_CARD" => {
                let card_id = value_str(raw, "card_id")?;
                let context = engine.current_decision_context();
                let offers = context
                    .shop
                    .ok_or_else(|| "card purchase occurred without a shop decision".to_string())?
                    .offers;
                let offer = unique_candidate(
                    offers.into_iter().filter(|offer| offer.card_id == card_id),
                    &format!("shop card offer {card_id:?}"),
                )?;
                Ok(Some(GameAction::ShopBuyCard(offer.index)))
            }
            "SHOP_BUY_RELIC" => {
                let relic_id = value_str(raw, "relic_id")?;
                let context = engine.current_decision_context();
                let offers = context
                    .shop
                    .ok_or_else(|| "relic purchase occurred without a shop decision".to_string())?
                    .relic_offers;
                let offer = unique_candidate(
                    offers
                        .into_iter()
                        .filter(|offer| offer.relic_id == relic_id),
                    &format!("shop relic offer {relic_id:?}"),
                )?;
                Ok(Some(GameAction::ShopBuyRelic(offer.index)))
            }
            "SHOP_BUY_POTION" => {
                let potion_id = value_str(raw, "potion_id")?;
                let context = engine.current_decision_context();
                let offers = context
                    .shop
                    .ok_or_else(|| "potion purchase occurred without a shop decision".to_string())?
                    .potion_offers;
                let offer = unique_candidate(
                    offers
                        .into_iter()
                        .filter(|offer| offer.potion_id == potion_id),
                    &format!("shop potion offer {potion_id:?}"),
                )?;
                Ok(Some(GameAction::ShopBuyPotion(offer.index)))
            }
            "SHOP_REMOVE" => {
                let next_record = next_record.ok_or_else(|| {
                    "recorder SHOP_REMOVE ends before the selected deck state is visible"
                        .to_string()
                })?;
                let (mapped, identity_verified) = map_recorded_shop_removal(engine, next_record)?;
                if !identity_verified {
                    self.last_mapping_unverified = Some(
                        "recorder SHOP_REMOVE result does not distinguish duplicate deck-card instances"
                            .to_string(),
                    );
                }
                self.last_mapping_coupled_to_following = true;
                Ok(Some(mapped))
            }
            "REWARD_TAKE" => Ok(Some(map_reward_take(engine, raw)?)),
            "CARD_REWARD" => Ok(Some(map_reward_choice(engine, raw, "card_id")?)),
            "BOSS_RELIC" => Err(
                "legacy BOSS_RELIC omits canonical reward-item and option-selection staging"
                    .to_string(),
            ),
            other => Err(format!("unsupported recorder action type {other:?}")),
        }
    }
}

fn map_recorded_shop_removal(
    engine: &RunEngine,
    next_record: &RecordedTraceRecord,
) -> Result<(GameAction, bool), String> {
    let expected_deck = recorded_ordered_deck(next_record, "shop removal")?;
    let mut candidates = engine
        .get_legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            GameAction::ShopRemoveCard(deck_index) => {
                let mut projected = engine.run_state.deck.clone();
                (deck_index < projected.len()).then(|| {
                    projected.remove(deck_index);
                    (deck_index, projected)
                })
            }
            _ => None,
        })
        .filter_map(|candidate| (candidate.1 == expected_deck).then_some(candidate.0))
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    let Some(&first_deck_index) = candidates.first() else {
        return Err("recorded deck does not identify any legal shop removal".to_string());
    };
    let identity_verified = candidates.len() == 1;
    if !identity_verified && !removal_candidates_are_equivalent(engine, &candidates) {
        return Err(format!(
            "recorded deck leaves {} semantically distinct shop removals",
            candidates.len()
        ));
    }
    Ok((
        GameAction::ShopRemoveCard(first_deck_index),
        identity_verified,
    ))
}

fn map_recorded_campfire_upgrade(
    engine: &RunEngine,
    next_record: &RecordedTraceRecord,
) -> Result<GameAction, String> {
    let expected_deck = recorded_ordered_deck(next_record, "campfire smith")?;
    let mut candidates = Vec::new();
    for action in engine.get_legal_actions() {
        if !matches!(action, GameAction::CampfireUpgrade(_)) {
            continue;
        }
        let mut projected = engine.clone();
        if projected.step_game(&action).status == ActionStatus::Accepted
            && projected.run_state.deck == expected_deck
        {
            candidates.push(action);
        }
    }
    unique_candidate(
        candidates,
        "campfire upgrade matching the recorded ordered deck",
    )
}

fn recorded_ordered_deck(
    record: &RecordedTraceRecord,
    context: &str,
) -> Result<Vec<String>, String> {
    record
        .deck
        .as_array()
        .ok_or_else(|| format!("{context} trace omitted the resulting ordered deck"))?
        .iter()
        .map(|card| {
            card.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{context} trace deck contains a non-string card"))
        })
        .collect()
}

fn removal_candidates_are_equivalent(engine: &RunEngine, candidates: &[usize]) -> bool {
    let Some(first) = candidates
        .first()
        .and_then(|index| engine.run_state.deck_card_states.get(*index))
    else {
        return false;
    };
    !is_bottled_instance(engine, first.instance_id)
        && candidates.iter().all(|index| {
            engine
                .run_state
                .deck_card_states
                .get(*index)
                .is_some_and(|state| {
                    !is_bottled_instance(engine, state.instance_id)
                        && same_card_semantics_ignoring_identity(first, state)
                })
        })
}

fn map_recorded_event_grid_choice(
    engine: &RunEngine,
    record: &RecordedTraceRecord,
) -> Result<(GameAction, bool), String> {
    let expected_deck = recorded_ordered_deck(record, "event grid")?;
    let screen = engine
        .current_reward_screen()
        .ok_or_else(|| "event grid selection has no canonical selection screen".to_string())?;
    let item_index = screen
        .active_item
        .ok_or_else(|| "event grid selection item was not opened".to_string())?;
    let item = screen
        .items
        .iter()
        .find(|item| item.index == item_index)
        .ok_or_else(|| format!("active event grid item {item_index} is absent"))?;
    if !matches!(
        item.label.as_str(),
        "deck_selection_purge"
            | "deck_selection_peace_pipe"
            | "deck_selection_event_remove"
            | "deck_selection_bonfire_offer"
    ) {
        return Err(format!(
            "recorder omitted semantic payload for unsupported event grid {:?}",
            item.label
        ));
    }

    let mut candidates = Vec::new();
    for (choice_index, choice) in item.choices.iter().enumerate() {
        let RewardChoice::Card {
            index: deck_index,
            card_id,
        } = choice
        else {
            continue;
        };
        if *deck_index >= engine.run_state.deck.len() {
            continue;
        }
        let mut projected = engine.run_state.deck.clone();
        projected.remove(*deck_index);
        if projected == expected_deck {
            candidates.push((choice_index, *deck_index, card_id));
        }
    }
    let Some(&(choice_index, first_deck_index, first_card_id)) = candidates.first() else {
        return Err(format!(
            "recorded deck does not identify any canonical choice for event grid {:?}",
            item.label
        ));
    };

    let identity_verified = candidates.len() == 1;
    if !identity_verified {
        let first_state = engine
            .run_state
            .deck_card_states
            .get(first_deck_index)
            .ok_or_else(|| "event grid canonical deck state is not reconciled".to_string())?;
        let first_is_bottled = is_bottled_instance(engine, first_state.instance_id);
        let equivalent = candidates.iter().all(|(_, deck_index, card_id)| {
            let Some(state) = engine.run_state.deck_card_states.get(*deck_index) else {
                return false;
            };
            *card_id == first_card_id
                && !first_is_bottled
                && !is_bottled_instance(engine, state.instance_id)
                && same_card_semantics_ignoring_identity(first_state, state)
        });
        if !equivalent {
            return Err(format!(
                "recorded deck leaves {} semantically distinct event-grid candidates",
                candidates.len()
            ));
        }
    }

    Ok((
        GameAction::ChooseRewardOption {
            item_index,
            choice_index,
        },
        identity_verified,
    ))
}

fn same_card_semantics_ignoring_identity(
    left: &crate::combat_types::CardInstance,
    right: &crate::combat_types::CardInstance,
) -> bool {
    left.def_id == right.def_id
        && left.cost == right.cost
        && left.base_cost == right.base_cost
        && left.misc == right.misc
        && left.flags == right.flags
}

fn is_bottled_instance(engine: &RunEngine, instance_id: u32) -> bool {
    [
        engine.run_state.bottled_flame_card_instance_id,
        engine.run_state.bottled_lightning_card_instance_id,
        engine.run_state.bottled_tornado_card_instance_id,
    ]
    .contains(&Some(instance_id))
}

fn validate_recorded_hand_card(engine: &RunEngine, raw: &Value) -> Result<usize, String> {
    let hand_idx = value_usize(raw, "hand_idx")?;
    let recorded_id = value_str(raw, "card_id")?;
    let combat = engine
        .get_combat_engine()
        .ok_or_else(|| "PLAY_CARD occurred without active combat".to_string())?;
    let card = combat
        .state
        .hand
        .get(hand_idx)
        .ok_or_else(|| format!("PLAY_CARD hand index {hand_idx} is out of range"))?;
    let actual_id = combat.card_registry.card_name(card.def_id);
    // SearingBlow.java preserves every upgrade in timesUpgraded/misc while
    // retaining one card ID. Other cards expose only the ordinary upgraded bit.
    let actual_upgrades = if actual_id.trim_end_matches('+') == "Searing Blow" {
        card.misc.max(i32::from(card.is_upgraded()))
    } else {
        i32::from(card.is_upgraded())
    };
    let recorded_upgrades = value_i32(raw, "upgrades")?;
    if actual_id.trim_end_matches('+') != recorded_id || actual_upgrades != recorded_upgrades {
        return Err(format!(
            "PLAY_CARD identity mismatch at hand index {hand_idx}: recorder {recorded_id:?}+{recorded_upgrades}, canonical {actual_id:?}"
        ));
    }
    Ok(hand_idx)
}

fn unique_candidate<T>(
    candidates: impl IntoIterator<Item = T>,
    description: &str,
) -> Result<T, String> {
    let mut candidates = candidates.into_iter();
    let first = candidates
        .next()
        .ok_or_else(|| format!("canonical state has no {description}"))?;
    if candidates.next().is_some() {
        return Err(format!(
            "canonical state has multiple {description} candidates; recorder omitted an authoritative index"
        ));
    }
    Ok(first)
}

fn map_reward_take(engine: &RunEngine, raw: &Value) -> Result<GameAction, String> {
    let reward_type = value_str(raw, "reward_type")?;
    let expected_kind = match reward_type {
        "CARD" => RewardItemKind::CardChoice,
        "RELIC" => RewardItemKind::Relic,
        "GOLD" => RewardItemKind::Gold,
        "STOLEN_GOLD" => RewardItemKind::StolenGold,
        "POTION" => RewardItemKind::Potion,
        "RUBY_KEY" => RewardItemKind::Key {
            color: RewardKeyColor::Ruby,
            linked_item_index: None,
        },
        "EMERALD_KEY" => RewardItemKind::Key {
            color: RewardKeyColor::Emerald,
            linked_item_index: None,
        },
        "SAPPHIRE_KEY" => RewardItemKind::Key {
            color: RewardKeyColor::Sapphire,
            linked_item_index: None,
        },
        other => return Err(format!("unknown reward type {other:?}")),
    };
    let screen = engine
        .current_reward_screen()
        .ok_or_else(|| "reward click occurred without a canonical reward screen".to_string())?;
    let item = unique_candidate(
        screen
            .items
            .iter()
            .filter(|item| reward_kind_matches(item.kind, expected_kind)),
        &format!("reward item of type {reward_type}"),
    )?;
    Ok(GameAction::SelectRewardItem(item.index))
}

fn reward_kind_matches(actual: RewardItemKind, expected: RewardItemKind) -> bool {
    match (actual, expected) {
        (
            RewardItemKind::Key { color: actual, .. },
            RewardItemKind::Key {
                color: expected, ..
            },
        ) => actual == expected,
        _ => actual == expected,
    }
}

fn map_reward_choice(engine: &RunEngine, raw: &Value, id_key: &str) -> Result<GameAction, String> {
    let wanted = value_str(raw, id_key)?;
    let screen = engine
        .current_reward_screen()
        .ok_or_else(|| "reward choice occurred without a canonical reward screen".to_string())?;
    let item = if let Some(index) = screen.active_item {
        screen
            .items
            .iter()
            .find(|item| item.index == index)
            .ok_or_else(|| format!("active reward item {index} is absent"))?
    } else {
        unique_candidate(
            screen.items.iter().filter(|item| !item.choices.is_empty()),
            "reward choice item",
        )?
    };
    let choice_index = unique_candidate(
        item.choices.iter().filter_map(|choice| match choice {
            RewardChoice::Card { index, card_id } if card_id == wanted => Some(*index),
            RewardChoice::Named { index, label } if label == wanted => Some(*index),
            _ => None,
        }),
        &format!("active reward choice {wanted:?}"),
    )?;
    Ok(GameAction::ChooseRewardOption {
        item_index: item.index,
        choice_index,
    })
}

fn canonical_recorder_phase(phase: &str, screen: &str, action: &Value) -> Option<&'static str> {
    let action_type = action.get("type").and_then(Value::as_str);
    if action_type == Some("NEOW") && screen != "MAP" {
        return Some("NEOW");
    }
    if action_type == Some("PATH") {
        return match action.get("symbol").and_then(Value::as_str) {
            Some("$") => Some("SHOP"),
            Some("R") => Some("CAMPFIRE"),
            Some("T") => Some("CHEST"),
            // EventRoom.java can roll an event, combat, shop, or treasure.
            // Trust the settled recorder state; COMPLETE/NONE is ambiguous.
            Some("?") => match (screen, phase) {
                ("SHOP", _) => Some("SHOP"),
                (_, "COMBAT") => Some("COMBAT"),
                (_, "EVENT") => Some("EVENT"),
                _ => None,
            },
            Some("M" | "E" | "B") => Some("COMBAT"),
            _ => None,
        };
    }
    match screen {
        // Java keeps the owning room in EVENT while GridSelectScreen is open.
        // The language-neutral phase names the active card-selection decision,
        // which RunEngine represents through its canonical reward surface.
        "GRID" | "CARD_REWARD" | "COMBAT_REWARD" => Some("REWARD"),
        "SHOP" => Some("SHOP"),
        "MAP" => Some("MAP"),
        "DEATH" => Some("GAME_OVER"),
        _ => match phase {
            "COMBAT" => Some("COMBAT"),
            "EVENT" => Some("EVENT"),
            // COMPLETE/INCOMPLETE are Java room phases, not decision phases.
            "COMPLETE" | "INCOMPLETE" | "NONE" => None,
            _ => None,
        },
    }
}

fn required_value<'a>(
    value: &'a Value,
    key: &str,
    path: &Path,
    line: usize,
) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{} line {} omitted {key}", path.display(), line))
}

fn required_str<'a>(
    value: &'a Value,
    key: &str,
    path: &Path,
    line: usize,
) -> Result<&'a str, String> {
    required_value(value, key, path, line)?
        .as_str()
        .ok_or_else(|| format!("{} line {} has non-string {key}", path.display(), line))
}

fn required_usize(value: &Value, key: &str, path: &Path, line: usize) -> Result<usize, String> {
    required_value(value, key, path, line)?
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| format!("{} line {} has invalid {key}", path.display(), line))
}

fn required_i32(value: &Value, key: &str, path: &Path, line: usize) -> Result<i32, String> {
    required_value(value, key, path, line)?
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| format!("{} line {} has invalid {key}", path.display(), line))
}

fn value_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("recorder action omitted string {key}"))
}

fn value_usize(value: &Value, key: &str) -> Result<usize, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| format!("recorder action omitted non-negative integer {key}"))
}

fn value_i32(value: &Value, key: &str) -> Result<i32, String> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| format!("recorder action omitted 32-bit integer {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Cursor, Write};

    fn gzip_member(line: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        writeln!(encoder, "{line}").unwrap();
        encoder.finish().unwrap()
    }

    fn trace_json(idx: usize) -> String {
        json!({
            "v": 1,
            "idx": idx,
            "floor": 0,
            "act": 1,
            "turn": 1,
            "phase": "EVENT",
            "screen": "NONE",
            "map": {"x": 0, "y": -1},
            "action": {"type": "NEOW", "choice": 0},
            "post": {},
            "deck": []
        })
        .to_string()
    }

    #[test]
    fn concatenated_gzip_members_preserve_every_sitting() {
        let mut bytes = gzip_member(&trace_json(0));
        bytes.extend(gzip_member(&trace_json(1)));
        let records = parse_trace_records(
            MultiGzDecoder::new(Cursor::new(bytes)),
            Path::new("two-sittings.gz"),
        )
        .expect("both gzip members must decode");
        assert_eq!(
            records.iter().map(|record| record.idx).collect::<Vec<_>>(),
            [0, 1]
        );
    }

    #[test]
    fn alignment_rejects_same_type_with_different_payload() {
        let meta = RecordingMeta {
            v: 1,
            run_id: "payload-mismatch".to_string(),
            seed_long: 0,
            seed_display: "0".to_string(),
            character: "WATCHER".to_string(),
            ascension: 0,
            game_version: "desktop-1.0".to_string(),
            status: "in_progress".to_string(),
            records: 1,
            sittings: Vec::new(),
            profile: None,
        };
        let actions = vec![RecorderAction {
            idx: 0,
            action_type: "PLAY_CARD".to_string(),
            raw: json!({"idx": 0, "type": "PLAY_CARD", "hand_idx": 0, "target": 0}),
        }];
        let records = vec![RecordedTraceRecord {
            idx: 0,
            floor: 1,
            act: 1,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": 0}),
            action: json!({"type": "PLAY_CARD", "hand_idx": 0, "target": 1}),
            post: json!({}),
            deck: json!([]),
        }];
        assert!(validate_bundle_alignment(&meta, &actions, &records)
            .unwrap_err()
            .contains("payload mismatch"));
    }

    #[test]
    fn path_debug_intent_skips_fields_captured_before_create_intent() {
        let record = RecordedTraceRecord {
            idx: 0,
            floor: 1,
            act: 1,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": 0}),
            action: json!({"type": "PATH", "x": 0, "y": 0, "symbol": "M"}),
            post: json!({
                "player": {"energy": 0},
                "enemies": [{"intent": {"name": "DEBUG", "dmg": 12, "move_id": 1, "hits": 1}}],
                "piles": {},
                "relics": [],
                "potions": [],
                "rng": {}
            }),
            deck: json!([]),
        };
        let partial = record.partial_oracle_state(true).unwrap();
        assert!(partial.pointer("/enemies/0/intent/dmg").is_none());
        assert!(partial.pointer("/enemies/0/intent/hits").is_none());
        assert!(partial.pointer("/enemies/0/intent/name").is_none());
        assert_eq!(partial.get("phase"), Some(&json!("COMBAT")));
    }

    #[test]
    fn path_symbols_define_unambiguous_destination_phases() {
        for (symbol, phase) in [
            ("$", "SHOP"),
            ("R", "CAMPFIRE"),
            ("T", "CHEST"),
            ("E", "COMBAT"),
        ] {
            assert_eq!(
                canonical_recorder_phase(
                    "COMPLETE",
                    "NONE",
                    &json!({"type": "PATH", "symbol": symbol}),
                ),
                Some(phase)
            );
        }
    }

    #[test]
    fn question_room_phase_uses_settled_recorder_evidence() {
        let action = json!({"type": "PATH", "symbol": "?"});
        assert_eq!(
            canonical_recorder_phase("EVENT", "NONE", &action),
            Some("EVENT")
        );
        assert_eq!(
            canonical_recorder_phase("COMBAT", "NONE", &action),
            Some("COMBAT")
        );
        assert_eq!(
            canonical_recorder_phase("COMPLETE", "SHOP", &action),
            Some("SHOP")
        );
        assert_eq!(canonical_recorder_phase("COMPLETE", "NONE", &action), None);
    }

    #[test]
    fn event_grid_uses_the_canonical_card_selection_phase() {
        assert_eq!(
            canonical_recorder_phase(
                "EVENT",
                "GRID",
                &json!({"type": "EVENT_CHOICE", "choice": 0}),
            ),
            Some("REWARD"),
        );
    }

    #[test]
    fn recorded_searing_blow_uses_exact_times_upgraded() {
        let mut engine = RunEngine::new(7, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        let mut card = combat.card_registry.make_card("Searing Blow+");
        card.misc = 3;
        combat.state.hand = vec![card];

        assert_eq!(
            validate_recorded_hand_card(
                &engine,
                &json!({"hand_idx": 0, "card_id": "Searing Blow", "upgrades": 3}),
            ),
            Ok(0)
        );
        assert!(validate_recorded_hand_card(
            &engine,
            &json!({"hand_idx": 0, "card_id": "Searing Blow", "upgrades": 1}),
        )
        .unwrap_err()
        .contains("identity mismatch"));
    }

    #[test]
    fn duplicate_candidates_are_never_resolved_by_first_match() {
        assert!(unique_candidate([1, 2], "duplicate reward")
            .unwrap_err()
            .contains("authoritative index"));
    }
}
