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
use crate::decision::{
    RewardChoice, RewardItemKind, RewardItemState, RewardKeyColor, RewardScreenSource,
};
use crate::effects::declarative::GeneratedCardPool;
use crate::engine::{ChoiceReason, CombatPhase};
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
    /// Recorder-captured post-generation process state. The legacy dialect is
    /// additive and still evolving, so preserve this envelope as JSON and
    /// consume only source-defined fields whose capture point is explicit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<Value>,
    /// Explicit Java execution environment inputs that change gameplay but
    /// are not derivable from the seed/profile (notably Settings.isDebug).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<RecordingEnvironmentSnapshot>,
    /// Replay-time provenance for `profile`. This is deliberately not part of
    /// `meta.json`: legacy bundles may resolve their missing profile through a
    /// separately committed operator attestation.
    #[serde(skip)]
    pub profile_authority: Option<String>,
    /// Missing profile fields that were filled only to enable diagnostic
    /// replay. Such a bundle is never promoted to certified initialization.
    #[serde(skip)]
    pub profile_quarantine: Option<BundleInitializationQuarantine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordingEnvironmentSnapshot {
    pub v: u32,
    pub settings_is_debug: bool,
    pub custom_modifiers: Vec<String>,
    pub loaded_mods: Vec<String>,
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

/// Authoritative terminal lifecycle emitted after the final room transition.
/// This is intentionally separate from action checkpoints: the legacy
/// ProceedButton hook can omit the click that enters TrueVictoryRoom, while
/// TraceWriter still records the settled run outcome and floor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedRunEnd {
    pub status: String,
    pub floor: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedLifecycleSummary {
    pub saw_run_start: bool,
    pub resume_count: usize,
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
        // GameActionManager.turn is a combat counter. Neow/noncombat recorder
        // callbacks can expose 0 or a stale prior-combat value, neither of
        // which is part of the canonical run state.
        if self.phase != "COMBAT" {
            state.remove("turn");
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
        // createIntent update and before queued battle-start powers resolve,
        // so Java still exposes transient DEBUG names and an empty player
        // power list. All causal move fields remain compared on this record.
        if path_combat_entry {
            state.remove("piles");
            if let Some(player) = state.get_mut("player").and_then(Value::as_object_mut) {
                // Anchor, Teardrop Locket, and other atBattleStart callbacks
                // are still queued at the recorder's map-click hook, just like
                // battle-start powers. Blood Vial and Pantograph likewise
                // enqueue HealAction from atBattleStart, so this callback can
                // expose the pre-heal HP while the settled engine already
                // contains the heal. The next semantic action compares HP.
                // Java: BloodVial.java/Pantograph.java::atBattleStart.
                player.remove("block");
                player.remove("hp");
                player.remove("stance");
                player.remove("powers");
            }
            if let Some(relics) = state.get_mut("relics").and_then(Value::as_array_mut) {
                for relic in relics {
                    let battle_start_mutates_counter =
                        relic.get("id").and_then(Value::as_str).is_some_and(|id| {
                            matches!(
                                id,
                                "Pocketwatch"
                                    | "HornCleat"
                                    | "Horn Cleat"
                                    | "CaptainsWheel"
                                    | "Captain's Wheel"
                                    | "StoneCalendar"
                                    | "Stone Calendar"
                                    | "Velvet Choker"
                                    | "NeowsBlessing"
                                    | "Ancient Tea Set"
                                    | "AncientTeaSet"
                                    | "Incense Burner"
                                    | "Kunai"
                                    | "Letter Opener"
                                    | "Ornamental Fan"
                                    | "Shuriken"
                            )
                        });
                    if battle_start_mutates_counter {
                        if let Some(relic) = relic.as_object_mut() {
                            // The recorder's PATH hook precedes
                            // AbstractRelic.atBattleStart, while canonical
                            // combat entry is settled. Compare these counters
                            // at the next semantic action instead.
                            // Java: Pocketwatch/HornCleat/CaptainsWheel/
                            // StoneCalendar/VelvetChoker/NeowsLament/
                            // IncenseBurner/Kunai/LetterOpener/
                            // OrnamentalFan/Shuriken.java.
                            relic.remove("counter");
                        }
                    }
                }
            }
        }
        if let Some(enemies) = state.get_mut("enemies").and_then(Value::as_array_mut) {
            for enemy in enemies {
                if path_combat_entry {
                    // PATH is sampled before the first start-of-turn queue.
                    // Mercury Hourglass and other battle-start/start-turn
                    // effects can mutate current HP before Rust reaches its
                    // first decision; the next semantic action compares HP.
                    // Preserve max_hp so constructor/ascension spawn values
                    // remain certified at the entry callback.
                    // Java: AbstractRoom.java::update and
                    // MercuryHourglass.java::atTurnStart.
                    if let Some(enemy) = enemy.as_object_mut() {
                        enemy.remove("hp");
                    }
                }
                if enemy.get("dead").and_then(Value::as_bool) == Some(true) {
                    // Java clears a dead monster's power list only when its
                    // frame-driven deathTimer expires. Adjacent settled action
                    // records can therefore expose either the old list or an
                    // empty list for identical causal game state. Java:
                    // AbstractMonster.java::updateDeathAnimation (860-874).
                    if let Some(enemy) = enemy.as_object_mut() {
                        enemy.remove("powers");
                        enemy.remove("intent");
                    }
                    continue;
                }
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
                    // AbstractMonster.createIntent leaves intentDmg,
                    // isMultiDmg, and intentMultiAmt stale when a non-attack
                    // follows an attack because baseDamage is -1. Neither
                    // damage nor hit count is causal for a non-attack intent.
                    // Java: AbstractMonster.java::createIntent/setMove.
                    intent.remove("dmg");
                    intent.remove("hits");
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
    pub run_end: Option<RecordedRunEnd>,
    pub lifecycle: RecordedLifecycleSummary,
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
pub struct BundleStateReconstruction {
    pub before_idx: usize,
    pub path: String,
    pub expected: Value,
    pub actual: Value,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleLifecycleQuarantine {
    pub kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleCheckpointQuarantine {
    pub kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleEnvironmentQuarantine {
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
    /// Authoritative source used for profile-sensitive run initialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_authority: Option<String>,
    /// Lifecycle framing that remains replayable but cannot certify a fresh,
    /// uninterrupted run from action zero through the terminal record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_quarantine: Option<BundleLifecycleQuarantine>,
    /// V1 recorder checkpoints are recursively partial objects rather than a
    /// versioned complete oracle-state schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_quarantine: Option<BundleCheckpointQuarantine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_quarantine: Option<BundleEnvironmentQuarantine>,
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
    /// Legacy state that the running-game record observes but the recorder did
    /// not capture causally. Reconstructions enable deeper diagnostics but are
    /// never credited as certified comparisons.
    pub state_reconstructions: Vec<BundleStateReconstruction>,
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

fn comparison_meets_strict_certification_contract(
    bundle: &RecordingBundle,
    report: &BundleComparisonReport,
) -> bool {
    bundle.meta.profile.as_ref().is_some_and(|profile| {
        !profile.note_for_yourself_card.is_empty() && profile.check_version().is_ok()
    }) && bundle.meta.profile_quarantine.is_none()
        && recording_initial_snapshot_issue(&bundle.meta, bundle.records.first()).is_none()
        && report.initialization_quarantine.is_none()
        && report.initialization_authority.is_some()
        && report.lifecycle_quarantine.is_none()
        && report.checkpoint_quarantine.is_none()
        && report.environment_quarantine.is_none()
        && bundle.lifecycle.saw_run_start
        && bundle.lifecycle.resume_count == 0
        && bundle.run_end.is_some()
        && matches!(bundle.meta.status.as_str(), "VICTORY" | "DEATH")
        && report.first_divergence.is_none()
        && report.total_actions != 0
        && report.replayed_actions == report.total_actions
        && report.matched_actions == report.total_actions
        && report.matched_checkpoints == report.total_actions
        && report.comparable_actions == report.total_actions
        && report.coupled_actions == 0
        && report.unverified_action_semantics == 0
        && report.unverified_actions.is_empty()
        && report.inferred_actions.is_empty()
        && report.ignored_recorder_actions.is_empty()
        && report.state_reconstructions.is_empty()
        && report.skipped_fields_total == 0
        && report.skipped_fields_by_path.is_empty()
}

pub fn load_recording_meta(path: impl AsRef<Path>) -> Result<RecordingMeta, String> {
    let path = path.as_ref();
    let meta_path = path.join("meta.json");
    let file = File::open(&meta_path)
        .map_err(|error| format!("failed to open {}: {error}", meta_path.display()))?;
    let mut value: Value = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| format!("failed to parse {}: {error}", meta_path.display()))?;
    let profile_quarantine = normalize_recording_profile(&mut value);
    let mut meta: RecordingMeta = serde_json::from_value(value)
        .map_err(|error| format!("failed to parse {}: {error}", meta_path.display()))?;
    meta.profile_quarantine = profile_quarantine;
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

fn normalize_recording_profile(value: &mut Value) -> Option<BundleInitializationQuarantine> {
    let profile = value.get_mut("profile")?.as_object_mut()?;
    let mut issues = Vec::new();
    let mut issue_kinds = Vec::new();
    match profile.get("note_for_yourself_card") {
        Some(Value::String(card)) if !card.is_empty() => {}
        Some(Value::Null) => {
            profile.insert("note_for_yourself_card".to_string(), json!("IronWave"));
            issues.push("recorder profile explicitly recorded an unknown process-persistent Note for Yourself card; diagnostic replay uses the historical IronWave compatibility value".to_string());
            issue_kinds.push("unknown_note_for_yourself_card");
        }
        Some(_) => {}
        None => {
            profile.insert("note_for_yourself_card".to_string(), json!("IronWave"));
            issues.push("recorder profile omitted the process-persistent Note for Yourself card; diagnostic replay uses the historical IronWave compatibility value".to_string());
            issue_kinds.push("missing_note_for_yourself_card");
        }
    }
    if !profile.contains_key("is_trial") {
        profile.insert("is_trial".to_string(), json!(false));
        issues.push(
            "recorder profile omitted the explicit trial-mode flag; diagnostic replay uses false"
                .to_string(),
        );
        issue_kinds.push("missing_trial_mode");
    }
    if !profile.contains_key("note_for_yourself_upgrades") {
        profile.insert("note_for_yourself_upgrades".to_string(), json!(0));
        issues.push(
            "recorder profile omitted the exact NOTE_UPGRADE preference; diagnostic replay uses 0"
                .to_string(),
        );
        issue_kinds.push("missing_note_for_yourself_upgrades");
    }
    (!issues.is_empty()).then(|| BundleInitializationQuarantine {
        kind: if issues.len() == 1 {
            issue_kinds[0].to_string()
        } else {
            "incomplete_profile_snapshot".to_string()
        },
        reason: issues.join("; "),
    })
}

/// Validate that the recorder hook actually ran after deterministic dungeon
/// generation. This narrow predicate controls whether captured process-global
/// streams are safe to restore for diagnostic replay. Full certification has
/// additional requirements in [`recording_initial_snapshot_issue`].
fn recording_initial_capture_point_issue(
    meta: &RecordingMeta,
    first_record: Option<&RecordedTraceRecord>,
) -> Option<String> {
    let Some(initial) = meta.initial.as_ref() else {
        return Some("recording omitted initial post-generation state".to_string());
    };
    if initial.get("captured_at").and_then(Value::as_str) != Some("first_run_frame_post_generation")
    {
        return Some("initial.captured_at is not first_run_frame_post_generation".to_string());
    }
    let Some(initial_map) = initial.pointer("/rng/map").and_then(Value::as_i64) else {
        return Some("initial.rng.map is missing or not an integer counter".to_string());
    };
    let Some(first_record) = first_record else {
        return Some("recording has no first checkpoint to settle the initial capture".to_string());
    };
    if first_record.action.get("type").and_then(Value::as_str) != Some("NEOW") {
        return Some(
            "first checkpoint is not a Neow action and cannot settle the initial capture"
                .to_string(),
        );
    }
    let Some(settled_map) = first_record
        .post
        .pointer("/rng/map")
        .and_then(Value::as_i64)
    else {
        return Some("first settled Neow checkpoint omitted post.rng.map".to_string());
    };
    (initial_map != settled_map).then(|| {
        format!(
            "initial envelope claims a post-generation capture but map RNG counter {initial_map} disagrees with the first settled Neow checkpoint counter {settled_map}; process-global RNG states from that envelope were not restored"
        )
    })
}

fn parse_rng_word(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().map(|word| word as u64))
        .or_else(|| {
            let raw = value.as_str()?;
            raw.parse::<i64>()
                .map(|word| word as u64)
                .ok()
                .or_else(|| u64::from_str_radix(raw.strip_prefix("0x").unwrap_or(raw), 16).ok())
        })
}

fn value_is_rng_word(value: &Value) -> bool {
    parse_rng_word(value).is_some()
}

fn required_array(value: &Value, pointer: &str, issues: &mut Vec<String>) {
    if !value.pointer(pointer).is_some_and(Value::is_array) {
        issues.push(format!("initial{pointer} is missing or not an array"));
    }
}

/// Return every reason an initial envelope cannot be used for a certified
/// comparison. The v1 recorder envelope is additive: legacy/partial snapshots
/// remain useful replay evidence, but certification requires the complete
/// causal state that Java carried across run generation.
fn recording_initial_snapshot_issue(
    meta: &RecordingMeta,
    first_record: Option<&RecordedTraceRecord>,
) -> Option<String> {
    let mut issues = Vec::new();
    if let Some(issue) = recording_initial_capture_point_issue(meta, first_record) {
        issues.push(issue);
    }
    let Some(initial) = meta.initial.as_ref() else {
        return Some(issues.join("; "));
    };
    if initial.get("v").and_then(Value::as_u64) != Some(1) {
        issues.push("initial.v is missing or is not supported version 1".to_string());
    }

    let counters = initial.get("rng").and_then(Value::as_object);
    for stream in crate::trace::CANONICAL_RNG_KEYS {
        if !counters
            .and_then(|rng| rng.get(*stream))
            .is_some_and(|value| value.as_i64().is_some())
        {
            issues.push(format!(
                "initial.rng.{stream} is missing or not an integer counter"
            ));
        }
    }
    let raw_states = initial.pointer("/rng/rawStates").and_then(Value::as_object);
    if raw_states.is_none() {
        issues.push(
            "initial.rng.rawStates omitted the exact native state/counter set for all 13 streams and neow"
                .to_string(),
        );
    } else {
        for stream in crate::trace::CANONICAL_RNG_KEYS
            .iter()
            .copied()
            .chain(std::iter::once("neow"))
        {
            let state = raw_states
                .and_then(|states| states.get(stream))
                .and_then(Value::as_object);
            for word in ["seed0", "seed1"] {
                if !state
                    .and_then(|state| state.get(word))
                    .is_some_and(value_is_rng_word)
                {
                    issues.push(format!(
                        "initial.rng.rawStates.{stream}.{word} is missing or not an exact RNG word"
                    ));
                }
            }
            let raw_counter = state.and_then(|state| state.get("counter"));
            if !raw_counter.is_some_and(|value| value.as_i64().is_some()) {
                issues.push(format!(
                    "initial.rng.rawStates.{stream}.counter is missing or not an integer"
                ));
            } else if stream != "neow"
                && counters
                    .and_then(|rng| rng.get(stream))
                    .and_then(Value::as_i64)
                    != raw_counter.and_then(Value::as_i64)
            {
                issues.push(format!(
                    "initial.rng.rawStates.{stream}.counter disagrees with initial.rng.{stream}"
                ));
            }
        }
    }

    for word in ["seed0", "seed1"] {
        if !initial
            .pointer(&format!("/ambient_mathutils/{word}"))
            .is_some_and(value_is_rng_word)
        {
            issues.push(format!(
                "initial.ambient_mathutils.{word} is missing or not an exact RNG word"
            ));
        }
    }
    if recording_java_collections_state(meta)
        .ok()
        .flatten()
        .is_none()
    {
        issues.push(
            "initial.java_collections.state is missing or not an exact 48-bit Random state"
                .to_string(),
        );
    }
    if initial
        .get("the_bomb_id_offset")
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .is_none()
    {
        issues.push("initial.the_bomb_id_offset is missing or not a 32-bit integer".to_string());
    }
    for field in [
        "seed_set",
        "has_ruby_key",
        "has_emerald_key",
        "has_sapphire_key",
    ] {
        if !initial.get(field).is_some_and(Value::is_boolean) {
            issues.push(format!("initial.{field} is missing or not a boolean"));
        }
    }

    if !initial
        .get("realized_generation")
        .is_some_and(Value::is_object)
    {
        issues.push(
            "initial.realized_generation omitted the ordered pools, queues, map, bosses, and Neow options"
                .to_string(),
        );
    } else {
        for pool in [
            "common",
            "uncommon",
            "rare",
            "colorless",
            "source_common",
            "source_uncommon",
            "source_rare",
        ] {
            required_array(
                initial,
                &format!("/realized_generation/card_pools/{pool}"),
                &mut issues,
            );
        }
        for pool in ["common", "uncommon", "rare", "shop", "boss"] {
            required_array(
                initial,
                &format!("/realized_generation/relic_pools/{pool}"),
                &mut issues,
            );
        }
        for pool in ["regular", "shrines", "one_time_shrines"] {
            required_array(
                initial,
                &format!("/realized_generation/event_pools/{pool}"),
                &mut issues,
            );
        }
        for field in [
            "boss_sequence",
            "monster_encounter_queue",
            "elite_encounter_queue",
            "neow_options",
        ] {
            required_array(
                initial,
                &format!("/realized_generation/{field}"),
                &mut issues,
            );
        }
        if !initial
            .pointer("/realized_generation/map")
            .is_some_and(Value::is_object)
        {
            issues.push(
                "initial/realized_generation/map is missing or not a canonical map object"
                    .to_string(),
            );
        }
    }

    if issues.is_empty() && meta.profile_quarantine.is_none() {
        if let Some(profile) = &meta.profile {
            match profile.to_engine_profile() {
                Ok(profile) => {
                    let generated =
                        RunEngine::new_with_profile(meta.seed_long as u64, meta.ascension, profile);
                    let generated_rng = generated.rng_state_tuples();
                    let generated_counters = generated.rng_counters();
                    for stream in crate::trace::CANONICAL_RNG_KEYS
                        .iter()
                        .copied()
                        .chain(std::iter::once("neow"))
                    {
                        let Some((seed0, seed1, counter)) = generated_rng.get(stream).copied()
                        else {
                            issues.push(format!(
                                "freshly generated Rust run omitted native RNG stream {stream}"
                            ));
                            continue;
                        };
                        let captured = initial.pointer(&format!("/rng/rawStates/{stream}"));
                        let captured_seed0 = captured
                            .and_then(|state| state.get("seed0"))
                            .and_then(parse_rng_word);
                        let captured_seed1 = captured
                            .and_then(|state| state.get("seed1"))
                            .and_then(parse_rng_word);
                        let captured_counter = captured
                            .and_then(|state| state.get("counter"))
                            .and_then(Value::as_i64);
                        if captured_seed0 != Some(seed0)
                            || captured_seed1 != Some(seed1)
                            || captured_counter != Some(i64::from(counter))
                        {
                            issues.push(format!(
                                "initial.rng.rawStates.{stream} disagrees with the freshly generated Rust run"
                            ));
                        }
                        if stream != "neow"
                            && initial
                                .pointer(&format!("/rng/{stream}"))
                                .and_then(Value::as_i64)
                                != generated_counters.get(stream).copied()
                        {
                            issues.push(format!(
                                "initial.rng.{stream} disagrees with the freshly generated Rust run"
                            ));
                        }
                    }

                    let expected = generated.recording_initialization_witness();
                    for component in [
                        "card_pools",
                        "relic_pools",
                        "event_pools",
                        "boss_sequence",
                        "monster_encounter_queue",
                        "elite_encounter_queue",
                        "map",
                        "neow_options",
                    ] {
                        let pointer = format!("/realized_generation/{component}");
                        if initial.pointer(&pointer) != expected.pointer(&pointer) {
                            issues.push(format!(
                                "initial.realized_generation.{component} disagrees with the freshly generated Rust run"
                            ));
                        }
                    }
                }
                Err(error) => issues.push(format!(
                    "profile cannot initialize an exact Rust run witness: {error}"
                )),
            }
        }
    }

    (!issues.is_empty()).then(|| issues.join("; "))
}

fn recording_ambient_math_state(
    meta: &RecordingMeta,
    first_record: Option<&RecordedTraceRecord>,
) -> Result<Option<(u64, u64)>, String> {
    let Some(initial) = meta.initial.as_ref() else {
        return Ok(None);
    };
    if initial.get("captured_at").and_then(Value::as_str) != Some("first_run_frame_post_generation")
    {
        return Ok(None);
    }
    // Neow inputs do not mutate mapRng. A true post-generation snapshot must
    // therefore agree with the first settled Neow checkpoint; tranche-1 hooks
    // that fired before dungeon generation used counters 0/-1 despite carrying
    // the newer label. Never restore process-global state from that envelope.
    if recording_initial_capture_point_issue(meta, first_record).is_some() {
        return Ok(None);
    }
    let Some(ambient) = initial.get("ambient_mathutils") else {
        return Ok(None);
    };
    let parse = |field: &str| -> Result<u64, String> {
        let value = ambient.get(field).ok_or_else(|| {
            format!(
                "recording {} initial.ambient_mathutils omitted {field}",
                meta.run_id
            )
        })?;
        if let Some(value) = value.as_i64() {
            return Ok(value as u64);
        }
        value
            .as_str()
            .ok_or_else(|| {
                format!(
                    "recording {} initial.ambient_mathutils.{field} is not a signed decimal string",
                    meta.run_id
                )
            })?
            .parse::<i64>()
            .map(|value| value as u64)
            .map_err(|error| {
                format!(
                    "recording {} has invalid initial.ambient_mathutils.{field}: {error}",
                    meta.run_id
                )
            })
    };
    Ok(Some((parse("seed0")?, parse("seed1")?)))
}

fn recording_java_collections_state(meta: &RecordingMeta) -> Result<Option<u64>, String> {
    const JAVA_RANDOM_STATE_MASK: u64 = (1_u64 << 48) - 1;
    let Some(initial) = meta.initial.as_ref() else {
        return Ok(None);
    };
    let value = initial
        .pointer("/java_collections/state")
        .or_else(|| initial.get("java_collections_rng_state"));
    let Some(value) = value else {
        return Ok(None);
    };
    if let Some(value) = value.as_u64() {
        if value > JAVA_RANDOM_STATE_MASK {
            return Err(format!(
                "recording {} initial Java Collections state exceeds 48 bits",
                meta.run_id
            ));
        }
        return Ok(Some(value));
    }
    let raw = value.as_str().ok_or_else(|| {
        format!(
            "recording {} initial Java Collections state is not a hexadecimal string",
            meta.run_id
        )
    })?;
    let raw = raw.strip_prefix("0x").unwrap_or(raw);
    let state = u64::from_str_radix(raw, 16).map_err(|error| {
        format!(
            "recording {} has invalid initial Java Collections state: {error}",
            meta.run_id
        )
    })?;
    if state > JAVA_RANDOM_STATE_MASK {
        return Err(format!(
            "recording {} initial Java Collections state exceeds 48 bits",
            meta.run_id
        ));
    }
    Ok(Some(state))
}

fn recording_the_bomb_id_offset(meta: &RecordingMeta) -> Result<Option<i32>, String> {
    let Some(initial) = meta.initial.as_ref() else {
        return Ok(None);
    };
    let Some(value) = initial.get("the_bomb_id_offset") else {
        return Ok(None);
    };
    let value = value.as_i64().ok_or_else(|| {
        format!(
            "recording {} initial.the_bomb_id_offset is not an integer",
            meta.run_id
        )
    })?;
    i32::try_from(value).map(Some).map_err(|_| {
        format!(
            "recording {} initial.the_bomb_id_offset is outside Java's 32-bit integer range",
            meta.run_id
        )
    })
}

pub fn load_recording_bundle(path: impl AsRef<Path>) -> Result<RecordingBundle, String> {
    let path = path.as_ref();
    let mut meta = load_recording_meta(path)?;
    if meta.profile.is_some() {
        meta.profile_authority = Some("recording meta.profile".to_string());
    } else if let Some((profile, source, quarantine)) =
        load_operator_profile_attestation(path, &meta)?
    {
        meta.profile = Some(profile);
        meta.profile_authority = Some(source);
        meta.profile_quarantine = meta.profile_quarantine.or(quarantine);
    }
    let actions = load_script_actions(&path.join("script.jsonl"))?;
    let trace = load_trace_records(&path.join("trace.jsonl.gz"))?;
    validate_trace_header(&meta, trace.header.as_ref())?;
    validate_bundle_alignment(&meta, &actions, &trace.records)?;
    Ok(RecordingBundle {
        path: path.to_path_buf(),
        meta,
        actions,
        records: trace.records,
        run_end: trace.run_end,
        lifecycle: trace.lifecycle,
    })
}

fn load_operator_profile_attestation(
    bundle_path: &Path,
    meta: &RecordingMeta,
) -> Result<
    Option<(
        TraceProfileSnapshot,
        String,
        Option<BundleInitializationQuarantine>,
    )>,
    String,
> {
    let Some(recordings_root) = bundle_path.parent() else {
        return Ok(None);
    };
    let attestation_path = recordings_root.join("profile-attestation.json");
    let file = match File::open(&attestation_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to open {}: {error}",
                attestation_path.display()
            ));
        }
    };
    let mut attestation: Value = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| format!("failed to parse {}: {error}", attestation_path.display()))?;
    if attestation.get("v").and_then(Value::as_u64) != Some(1)
        || attestation.get("kind").and_then(Value::as_str) != Some("operator_profile_attestation")
    {
        return Err(format!(
            "{} is not a supported operator profile attestation",
            attestation_path.display()
        ));
    }
    let applies_to = attestation
        .get("applies_to")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{} omitted applies_to", attestation_path.display()))?;
    if !applies_to.contains("all recording bundles in this directory recorded on or before") {
        return Err(format!(
            "{} has an unsupported applies_to scope",
            attestation_path.display()
        ));
    }
    let attested_at = attestation
        .get("attested_at")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{} omitted attested_at", attestation_path.display()))?;
    let cutoff = attested_at.replace('-', "");
    let recorded_at = meta
        .run_id
        .rsplit_once("-WATCHER-")
        .and_then(|(_, suffix)| suffix.get(..8));
    if cutoff.len() != 8 || recorded_at.is_none_or(|date| date > cutoff.as_str()) {
        return Ok(None);
    }

    let profile = attestation
        .get_mut("profile")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| format!("{} omitted profile", attestation_path.display()))?;
    let note_unknown = profile
        .get("note_for_yourself_card")
        .is_none_or(Value::is_null);
    if note_unknown {
        // Preserve diagnostic replay through Java's fallback, but an operator
        // attestation that says `null` (or omits the field) is not an exact
        // witness of CardCrawlGame.playerPref's process-persistent value.
        // Java: NoteForYourself.java:90-97.
        profile.insert(
            "note_for_yourself_card".to_string(),
            Value::String("IronWave".to_string()),
        );
    }
    let trial_mode_unknown = !profile.contains_key("is_trial");
    if trial_mode_unknown {
        profile.insert("is_trial".to_string(), Value::Bool(false));
    }
    let note_upgrades_unknown = !profile.contains_key("note_for_yourself_upgrades");
    if note_upgrades_unknown {
        profile.insert("note_for_yourself_upgrades".to_string(), json!(0));
    }
    let profile: TraceProfileSnapshot = serde_json::from_value(Value::Object(profile.clone()))
        .map_err(|error| {
            format!(
                "{} contains an invalid profile snapshot: {error}",
                attestation_path.display()
            )
        })?;
    profile.check_version().map_err(|error| {
        format!(
            "{} contains an invalid profile snapshot: {error}",
            attestation_path.display()
        )
    })?;
    Ok(Some((
        profile,
        "operator attestation data/traces/recordings/profile-attestation.json".to_string(),
        (note_unknown || trial_mode_unknown || note_upgrades_unknown).then(|| BundleInitializationQuarantine {
            kind: if [note_unknown, trial_mode_unknown, note_upgrades_unknown]
                .into_iter()
                .filter(|missing| *missing)
                .count()
                > 1
            {
                "incomplete_operator_profile_attestation"
            } else if note_unknown {
                "unknown_attested_note_for_yourself_card"
            } else if note_upgrades_unknown {
                "missing_attested_note_for_yourself_upgrades"
            } else {
                "missing_attested_trial_mode"
            }
            .to_string(),
            reason: [
                note_unknown.then_some("operator profile attestation did not provide an exact non-null Note for Yourself preference; diagnostic replay uses Java's IronWave fallback"),
                note_upgrades_unknown.then_some("operator profile attestation omitted the exact NOTE_UPGRADE preference; diagnostic replay uses 0"),
                trial_mode_unknown.then_some("operator profile attestation omitted the explicit trial-mode flag; diagnostic replay uses false"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("; ")
                + "; the profile is not authoritative for certification",
        }),
    )))
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

#[derive(Debug, Default)]
struct LoadedTraceRecords {
    header: Option<Value>,
    records: Vec<RecordedTraceRecord>,
    run_end: Option<RecordedRunEnd>,
    lifecycle: RecordedLifecycleSummary,
}

fn load_trace_records(path: &Path) -> Result<LoadedTraceRecords, String> {
    let file =
        File::open(path).map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    parse_trace_records(MultiGzDecoder::new(file), path)
}

fn parse_trace_records(reader: impl Read, path: &Path) -> Result<LoadedTraceRecords, String> {
    let mut reader = BufReader::new(reader);
    let mut header = None;
    let mut records = Vec::new();
    let mut run_end = None;
    let mut lifecycle = RecordedLifecycleSummary::default();
    let mut saw_payload = false;
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
        if value.get("kind").and_then(Value::as_str) == Some("header") {
            if header.is_some() || saw_payload {
                return Err(format!(
                    "{} line {} contains a repeated or out-of-order trace header",
                    path.display(),
                    line_index
                ));
            }
            let version = required_i32(&value, "v", path, line_index)?;
            if version != 1 {
                return Err(format!(
                    "unsupported trace header version {version} in {} line {}",
                    path.display(),
                    line_index
                ));
            }
            header = Some(value);
            continue;
        }
        if value.get("kind").and_then(Value::as_str) == Some("lifecycle") {
            let lifecycle_type = required_str(&value, "type", path, line_index)?;
            match lifecycle_type {
                "RUN_START" => {
                    if lifecycle.saw_run_start || saw_payload {
                        return Err(format!(
                            "{} line {} contains a repeated or late RUN_START lifecycle",
                            path.display(),
                            line_index
                        ));
                    }
                    lifecycle.saw_run_start = true;
                    saw_payload = true;
                }
                "RESUME" => {
                    if run_end.is_some() {
                        return Err(format!(
                            "{} line {} contains RESUME outside a live run lifecycle",
                            path.display(),
                            line_index
                        ));
                    }
                    lifecycle.resume_count += 1;
                    saw_payload = true;
                }
                "RUN_END" => {
                    if !lifecycle.saw_run_start && lifecycle.resume_count == 0 {
                        return Err(format!(
                            "{} line {} contains RUN_END before RUN_START",
                            path.display(),
                            line_index
                        ));
                    }
                    if run_end.is_some() {
                        return Err(format!(
                            "{} line {} repeats the terminal RUN_END lifecycle",
                            path.display(),
                            line_index
                        ));
                    }
                    run_end = Some(RecordedRunEnd {
                        status: required_str(&value, "status", path, line_index)?.to_string(),
                        floor: required_i32(&value, "floor", path, line_index)?,
                    });
                    saw_payload = true;
                }
                other => {
                    return Err(format!(
                        "{} line {} has unsupported lifecycle type {other:?}",
                        path.display(),
                        line_index
                    ));
                }
            }
            continue;
        }
        if let Some(kind) = value.get("kind") {
            return Err(format!(
                "{} line {} has unsupported trace record kind {kind}",
                path.display(),
                line_index
            ));
        }
        saw_payload = true;
        if !lifecycle.saw_run_start && lifecycle.resume_count == 0 {
            return Err(format!(
                "{} line {} contains an action checkpoint before RUN_START",
                path.display(),
                line_index
            ));
        }
        if run_end.is_some() {
            return Err(format!(
                "{} line {} contains an action checkpoint after RUN_END",
                path.display(),
                line_index
            ));
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
    Ok(LoadedTraceRecords {
        header,
        records,
        run_end,
        lifecycle,
    })
}

fn validate_trace_header(meta: &RecordingMeta, header: Option<&Value>) -> Result<(), String> {
    let Some(header) = header else {
        // Pre-header legacy bundles remain diagnostic inputs. Their missing
        // duplicate envelope is already reflected by their initialization
        // quarantine and must not make the action corpus unreadable.
        return Ok(());
    };
    for (field, expected) in [
        ("seed_long", json!(meta.seed_long)),
        ("seed_display", json!(meta.seed_display)),
        ("character", json!(meta.character)),
        ("ascension", json!(meta.ascension)),
        ("game_version", json!(meta.game_version)),
    ] {
        if header.get(field) != Some(&expected) {
            return Err(format!(
                "recording {} trace header {field} does not match meta.json (trace {}, meta {expected})",
                meta.run_id,
                header.get(field).cloned().unwrap_or(Value::Null),
            ));
        }
    }

    let mut header_profile =
        json!({"profile": header.get("profile").cloned().unwrap_or(Value::Null)});
    normalize_recording_profile(&mut header_profile);
    let expected_profile = meta
        .profile
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| format!("failed to serialize recording profile: {error}"))?
        .unwrap_or(Value::Null);
    if header_profile.get("profile") != Some(&expected_profile) {
        return Err(format!(
            "recording {} trace header profile does not match meta.json",
            meta.run_id
        ));
    }
    if header.get("initial").unwrap_or(&Value::Null)
        != meta.initial.as_ref().unwrap_or(&Value::Null)
    {
        return Err(format!(
            "recording {} trace header initial state does not match meta.json",
            meta.run_id
        ));
    }
    let expected_environment = meta
        .environment
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| format!("failed to serialize recording environment: {error}"))?
        .unwrap_or(Value::Null);
    if header.get("environment").unwrap_or(&Value::Null) != &expected_environment {
        return Err(format!(
            "recording {} trace header environment does not match meta.json",
            meta.run_id
        ));
    }
    Ok(())
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

fn incomplete_v1_checkpoint_path(record: &RecordedTraceRecord) -> Option<String> {
    let Some(post) = record.post.as_object() else {
        return Some("post".to_string());
    };
    let Some(player) = post.get("player").and_then(Value::as_object) else {
        return Some("post.player".to_string());
    };
    for field in ["hp", "max_hp", "block", "energy", "gold"] {
        if !player
            .get(field)
            .is_some_and(|value| value.as_i64().is_some())
        {
            return Some(format!("post.player.{field}"));
        }
    }
    if !player.get("stance").is_some_and(Value::is_string) {
        return Some("post.player.stance".to_string());
    }
    for field in ["powers", "orbs"] {
        if !player.get(field).is_some_and(Value::is_array) {
            return Some(format!("post.player.{field}"));
        }
    }
    let Some(piles) = post.get("piles").and_then(Value::as_object) else {
        return Some("post.piles".to_string());
    };
    for field in ["hand", "draw_ordered", "discard", "exhaust"] {
        if !piles.get(field).is_some_and(Value::is_array) {
            return Some(format!("post.piles.{field}"));
        }
    }
    let Some(rng) = post.get("rng").and_then(Value::as_object) else {
        return Some("post.rng".to_string());
    };
    for stream in crate::trace::CANONICAL_RNG_KEYS {
        if !rng
            .get(*stream)
            .is_some_and(|value| value.as_i64().is_some())
        {
            return Some(format!("post.rng.{stream}"));
        }
    }
    let Some(relics) = post.get("relics").and_then(Value::as_array) else {
        return Some("post.relics".to_string());
    };
    if relics.iter().any(|relic| {
        !relic.get("id").is_some_and(Value::is_string)
            || !relic
                .get("counter")
                .is_some_and(|value| value.as_i64().is_some())
    }) {
        return Some("post.relics[].{id,counter}".to_string());
    }
    let Some(potions) = post.get("potions").and_then(Value::as_array) else {
        return Some("post.potions".to_string());
    };
    if potions.iter().any(|potion| !potion.is_string()) {
        return Some("post.potions[]".to_string());
    }
    if !post.get("enemies").is_some_and(Value::is_array) {
        return Some("post.enemies".to_string());
    }
    if record.phase.is_empty() {
        return Some("phase".to_string());
    }
    if record.screen.is_empty() {
        return Some("screen".to_string());
    }
    if !record
        .map
        .get("x")
        .is_some_and(|value| value.as_i64().is_some())
        || !record
            .map
            .get("y")
            .is_some_and(|value| value.as_i64().is_some())
    {
        return Some("map.{x,y}".to_string());
    }
    None
}

fn v1_checkpoint_quarantine(bundle: &RecordingBundle) -> BundleCheckpointQuarantine {
    if let Some((idx, path)) = bundle
        .records
        .iter()
        .find_map(|record| incomplete_v1_checkpoint_path(record).map(|path| (record.idx, path)))
    {
        BundleCheckpointQuarantine {
            kind: "incomplete_v1_checkpoint_shape".to_string(),
            reason: format!(
                "v1 checkpoint {idx} is recursively partial at {path}; zero state differences or skipped-field counts cannot certify an omitted subtree"
            ),
        }
    } else {
        BundleCheckpointQuarantine {
            kind: "partial_v1_checkpoint_schema".to_string(),
            reason: "v1 recording checkpoints have no distinct complete-state schema/version and remain diagnostic even when every currently known field is present"
                .to_string(),
        }
    }
}

fn recording_environment_quarantine(meta: &RecordingMeta) -> Option<BundleEnvironmentQuarantine> {
    match &meta.environment {
        None => Some(BundleEnvironmentQuarantine {
            kind: "missing_execution_environment".to_string(),
            reason: "recording omitted Settings.isDebug, custom modifiers, and loaded-mod identity; release/debug reward rules cannot be certified"
                .to_string(),
        }),
        Some(environment) if environment.v != 1 => Some(BundleEnvironmentQuarantine {
            kind: "unsupported_execution_environment_version".to_string(),
            reason: format!(
                "recording execution environment version {} is unsupported",
                environment.v
            ),
        }),
        Some(environment) if environment.settings_is_debug => {
            Some(BundleEnvironmentQuarantine {
                kind: "debug_gameplay_environment".to_string(),
                reason: "Settings.isDebug changes gameplay/reward behavior and is outside the standard-run certification contract"
                    .to_string(),
            })
        }
        Some(environment) if !environment.custom_modifiers.is_empty() => {
            Some(BundleEnvironmentQuarantine {
                kind: "custom_run_modifiers".to_string(),
                reason: format!(
                    "custom modifiers are outside the standard-run certification contract: {:?}",
                    environment.custom_modifiers
                ),
            })
        }
        Some(environment) if {
            let mut loaded = environment
                .loaded_mods
                .iter()
                .map(|name| name.to_ascii_lowercase())
                .collect::<Vec<_>>();
            loaded.sort();
            loaded
                != vec![
                    "basemod".to_string(),
                    "stslib".to_string(),
                    "tracelab".to_string(),
                ]
        } => {
            Some(BundleEnvironmentQuarantine {
                kind: "unrecognized_gameplay_mods".to_string(),
                reason: format!(
                    "loaded mods exceed the recorder-infrastructure allowlist (BaseMod, StSLib, TraceLab): {:?}",
                    environment.loaded_mods
                ),
            })
        }
        Some(_) => None,
    }
}

pub fn compare_recording_bundle(
    bundle: &RecordingBundle,
) -> Result<BundleComparisonReport, String> {
    let initial_capture_point_issue =
        recording_initial_capture_point_issue(&bundle.meta, bundle.records.first());
    let initial_snapshot_issue =
        recording_initial_snapshot_issue(&bundle.meta, bundle.records.first());
    let profile_quarantine = bundle.meta.profile_quarantine.clone().or_else(|| {
        bundle
            .meta
            .profile
            .is_none()
            .then(|| BundleInitializationQuarantine {
                kind: "missing_profile_snapshot".to_string(),
                reason: "legacy recording omitted authoritative profile/unlock state; replay uses the ordinary API's all-unlocked compatibility default and cannot certify relic-pool initialization"
                    .to_string(),
            })
    });
    let initialization_quarantine = initial_snapshot_issue
        .as_ref()
        .map(|issue| BundleInitializationQuarantine {
            kind: "incomplete_initialization_snapshot".to_string(),
            reason: profile_quarantine
                .as_ref()
                .map(|quarantine| format!("{}; {issue}", quarantine.reason))
                .unwrap_or_else(|| issue.clone()),
        })
        .or(profile_quarantine);
    let lifecycle_quarantine = if bundle.lifecycle.resume_count != 0 {
        Some(BundleLifecycleQuarantine {
            kind: "unsupported_resumed_recording".to_string(),
            reason: format!(
                "recording contains {} RESUME lifecycle record(s); replay remains diagnostic until a lossless save/pre-action continuation witness is supported",
                bundle.lifecycle.resume_count
            ),
        })
    } else if bundle.run_end.is_none() {
        Some(BundleLifecycleQuarantine {
            kind: "missing_terminal_lifecycle".to_string(),
            reason: "recording omitted the terminal RUN_END lifecycle record".to_string(),
        })
    } else {
        None
    };
    let checkpoint_quarantine = Some(v1_checkpoint_quarantine(bundle));
    let environment_quarantine = recording_environment_quarantine(&bundle.meta);
    let mut report = BundleComparisonReport {
        status: if bundle.records.is_empty() {
            BundleComparisonStatus::NoActions
        } else if initialization_quarantine.is_some()
            || lifecycle_quarantine.is_some()
            || checkpoint_quarantine.is_some()
            || environment_quarantine.is_some()
        {
            BundleComparisonStatus::Uncertified
        } else {
            BundleComparisonStatus::Match
        },
        bundle: bundle.meta.run_id.clone(),
        seed: bundle.meta.seed_long,
        ascension: bundle.meta.ascension,
        outcome: bundle.meta.status.clone(),
        initialization_quarantine,
        initialization_authority: bundle.meta.profile_authority.clone().or_else(|| {
            bundle
                .meta
                .profile
                .is_some()
                .then(|| "provided profile snapshot".to_string())
        }),
        lifecycle_quarantine,
        checkpoint_quarantine,
        environment_quarantine,
        matched_actions: 0,
        comparable_actions: 0,
        matched_checkpoints: 0,
        coupled_actions: 0,
        unverified_action_semantics: 0,
        unverified_actions: Vec::new(),
        inferred_actions: Vec::new(),
        ignored_recorder_actions: Vec::new(),
        state_reconstructions: Vec::new(),
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
    // The recorder's tranche-1 snapshot is explicitly post-generation. Build
    // the deterministic run first, then restore the captured process-global
    // MathUtils state so subsequent constructors/effects continue from the
    // witnessed Java frame instead of consuming generation twice.
    if let Some(state) = recording_ambient_math_state(&bundle.meta, bundle.records.first())? {
        engine.restore_ambient_math_rng_state(state);
    }
    let recorded_java_collections_state = if initial_capture_point_issue.is_none() {
        recording_java_collections_state(&bundle.meta)?
    } else {
        None
    };
    if let Some(state) = recorded_java_collections_state {
        engine.restore_java_collections_rng_state(state);
    }
    if initial_capture_point_issue.is_none() {
        if let Some(offset) = recording_the_bomb_id_offset(&bundle.meta)? {
            engine.restore_the_bomb_id_offset(offset);
        }
    }
    let mut adapter = RecorderActionAdapter::default();
    let mut pending_coupled_actions = 0usize;
    for (position, (action, record)) in bundle.actions.iter().zip(&bundle.records).enumerate() {
        let previous_record = position
            .checked_sub(1)
            .and_then(|previous| bundle.records.get(previous));
        let recorder_only_distilled_chaos_callback =
            is_distilled_chaos_recorder_only_callback(&bundle.actions, &bundle.records, position);
        let recorder_only_necronomicon_callback =
            is_necronomicon_recorder_only_callback(&bundle.actions, &bundle.records, position);
        let recorder_only_card_reward_preview =
            is_unindexed_card_reward_preview(&bundle.actions, &bundle.records, position);
        let recorder_only_canceled_smith =
            is_canceled_campfire_smith(&bundle.actions, &bundle.records, position);
        let recorder_only_disabled_chest_relic =
            is_disabled_chest_relic_click(action, previous_record, record);
        let recorder_only_shop_remove_opener =
            is_shop_remove_grid_opener(&bundle.actions, &bundle.records, position);
        let recorder_only_early_sequence_end_turn =
            is_early_sequence_end_turn_callback(&bundle.actions, &bundle.records, position);
        let recorder_only_auto_burn =
            is_auto_played_burn_recorder_callback(&bundle.actions, &bundle.records, position);
        let recorder_only_necronomicon_exit = is_deferred_necronomicon_event_exit(
            &engine,
            &bundle.actions,
            &bundle.records,
            position,
        );
        let pending_combat_choice = engine
            .get_combat_engine()
            .is_some_and(|combat| combat.phase == crate::engine::CombatPhase::AwaitingChoice);
        let coupled_choice_lookahead =
            identical_state_combat_lookahead(&bundle.actions, &bundle.records, position);
        let inferred_result = if recorder_only_early_sequence_end_turn
            && pending_combat_choice
            && !coupled_choice_lookahead.is_empty()
        {
            infer_omitted_combat_choice_with_lookahead(
                &mut engine,
                action,
                record,
                coupled_choice_lookahead,
            )?
            .ok_or_else(|| {
                format!(
                    "recorder action {} did not expose its coupled pending combat choice",
                    action.idx
                )
            })
        } else if recorder_only_distilled_chaos_callback
            || recorder_only_necronomicon_callback
            || recorder_only_card_reward_preview
            || recorder_only_canceled_smith
            || recorder_only_disabled_chest_relic
            || recorder_only_shop_remove_opener
            // A MeditateAction choice sits between the recorded use-card and
            // PressEndTurnButtonAction callbacks. Do not let the duplicate
            // callback classifier hide that unresolved agent decision.
            || (recorder_only_early_sequence_end_turn && !pending_combat_choice)
            || recorder_only_auto_burn
            || recorder_only_necronomicon_exit
        {
            Ok(InferredPrerequisites::default())
        } else {
            infer_missing_prerequisites(&mut engine, action, record)
        };
        let recorder_only_event_grid_exit;
        match inferred_result {
            Ok(inferred) => {
                recorder_only_event_grid_exit = action.action_type == "EVENT_CHOICE"
                    && record.screen == "MAP"
                    && engine.current_phase() == RunPhase::MapChoice
                    && inferred.actions.iter().any(|entry| {
                        entry
                            .reason
                            .contains("recorder-omitted event transform-grid selection")
                    });
                if !inferred.actions.is_empty() {
                    report.mark_uncertified();
                }
                if !inferred.reconstructions.is_empty() {
                    report.mark_uncertified();
                    report
                        .state_reconstructions
                        .extend(inferred.reconstructions);
                }
                if let Some(reason) = inferred.unverified_reason {
                    report.mark_uncertified();
                    report.unverified_action_semantics += 1;
                    report.unverified_actions.push(BundleUnverifiedAction {
                        idx: action.idx,
                        action: action.raw.clone(),
                        reason,
                    });
                }
                report.inferred_actions.extend(inferred.actions);
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
        let recorder_only_collapsed_event_exit = engine.current_phase() == RunPhase::MapChoice
            && is_collapsed_event_exit_pair(&bundle.actions, &bundle.records, position);
        if action.action_type == "REWARD_TAKE"
            && engine.current_reward_screen().is_some_and(|screen| {
                matches!(
                    screen.source,
                    RewardScreenSource::Combat | RewardScreenSource::BossCombat
                ) && !screen.items.iter().any(|item| {
                    item.kind == RewardItemKind::Potion && item.state == RewardItemState::Available
                })
            })
        {
            if let Some(expected_id) = upcoming_recorded_potion_reward(&bundle.actions, position) {
                let expected_counter = record
                    .post
                    .pointer("/rng/potion")
                    .and_then(Value::as_i64)
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or_else(|| {
                        format!(
                            "trace action {} omitted a valid post.rng.potion counter",
                            action.idx
                        )
                    })?;
                if engine.reconcile_legacy_missing_potion_reward(expected_id, expected_counter)? {
                    report.mark_uncertified();
                    report.state_reconstructions.push(BundleStateReconstruction {
                        before_idx: action.idx,
                        path: "reward.potion".to_string(),
                        expected: json!(expected_id),
                        actual: json!("absent under the source-normal release reward-count gate"),
                        reason: "the legacy trace proves a potion reward and its exact RNG endpoint, but contradicts AbstractRoom.addPotionToRewards' four-item release gate; the adapter reconstructs the witnessed reward without changing production generation, pending recorder capture of runtime/debug settings and the ordered reward list"
                            .to_string(),
                    });
                }
            }
        }
        // Some Java callbacks prove mandatory UI prerequisites which create
        // their reward items (notably Calling Bell's post-curse relic screen).
        // Reconcile a legacy recorded relic only after those prerequisites
        // have materialized the canonical item.
        if action.action_type == "REWARD_TAKE"
            && action.raw.get("reward_type").and_then(Value::as_str) == Some("RELIC")
            && !recorder_only_disabled_chest_relic
        {
            let expected = value_str(&action.raw, "id")?;
            if !expected.is_empty() {
                if let Some(actual) = engine
                    .reconcile_legacy_recorded_relic_reward(expected)
                    .map_err(|error| {
                        format!(
                            "cannot reconcile recorded relic reward before action {}: {error}",
                            action.idx
                        )
                    })?
                {
                    report.mark_uncertified();
                    report.state_reconstructions.push(BundleStateReconstruction {
                        before_idx: action.idx,
                        path: "reward.relic.id".to_string(),
                        expected: json!(expected),
                        actual: json!(actual),
                        reason: "the legacy trace proves the claimed relic identity, but omitted the realized ordered relic pools; the adapter applies a minimal same-tier swap so later state can be diagnosed without changing production run generation"
                            .to_string(),
                    });
                }
            }
        }
        let mapped = if recorder_only_event_grid_exit || recorder_only_collapsed_event_exit {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: if recorder_only_event_grid_exit {
                        "the event callback follows recorder-omitted transform-grid selections; Rust settles the non-causal exit dialog with the final grid choice, and the MAP checkpoint proves no additional game-state mutation"
                            .to_string()
                    } else {
                        "Java exposes a separate non-causal event exit dialog after the preceding choice; Rust settles that UI stage immediately, and the adjacent MAP checkpoint proves identical game state"
                            .to_string()
                    },
                });
            None
        } else if recorder_only_distilled_chaos_callback {
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
        } else if recorder_only_necronomicon_callback {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "NecronomiconPower queued an internal PLAY_CARD callback with hand_idx=-1 after the canonical card action had already settled both plays; the adjacent callback has identical Java state and card payload"
                        .to_string(),
                });
            None
        } else if recorder_only_card_reward_preview {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "the unindexed card-reward opener produced no persistent card choice; the trace proves a skipped preview but cannot identify which Prayer Wheel item was opened"
                        .to_string(),
                });
            None
        } else if recorder_only_canceled_smith {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "SmithOption opened Java's upgrade grid, but the unchanged deck and following campfire choice prove that the unrecorded grid selection was canceled"
                        .to_string(),
                });
            None
        } else if recorder_only_disabled_chest_relic {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "recorder logged the disabled chest relic after its linked Sapphire Key was claimed; Java marks both linked RewardItems done/ignored and the recorded state is unchanged"
                        .to_string(),
                });
            None
        } else if recorder_only_shop_remove_opener {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "recorder emitted SHOP_REMOVE when the merchant purge grid opened; that callback has no game-state mutation, while the later selected-card effect is either a repeated callback or is only visible in the following exact shop checkpoint"
                        .to_string(),
                });
            None
        } else if recorder_only_early_sequence_end_turn {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "recorder emitted END_TURN for Java's callEndTurnEarlySequence after the preceding card callback had already captured the new turn; both checkpoints have identical Java state"
                        .to_string(),
                });
            None
        } else if recorder_only_auto_burn {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "Burn.triggerOnEndOfTurnForPlayingCard queued an internal useCard callback before the recorder's END_TURN callback; the identical settled checkpoints prove this is not a second agent action"
                        .to_string(),
                });
            None
        } else if recorder_only_necronomicon_exit {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report
                .ignored_recorder_actions
                .push(BundleIgnoredRecorderAction {
                    idx: action.idx,
                    action: action.raw.clone(),
                    reason: "Cursed Tome's final EVENT_CHOICE callback occurs after Necronomicon's deferred ShowCardAndObtainEffect; canonical replay already settled the curse and event exit at the preceding relic selection"
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
        let mapping_inferred_reason = adapter.take_last_mapping_inferred_reason();
        if let (Some(reason), Some(game_action)) = (mapping_inferred_reason, mapped.as_ref()) {
            report.mark_uncertified();
            report.inferred_actions.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action.clone(),
                reason,
            });
        }
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
        let mut terminal_combat_callback = None;
        let mut smoke_bomb_callback = false;
        if let Some(game_action) = mapped {
            terminal_combat_callback =
                if record.phase == "COMBAT" && recorded_combat_victory_pending(record) {
                    match &game_action {
                        GameAction::CombatAction(action) => {
                            engine.project_combat_callback_action(action)
                        }
                        _ => None,
                    }
                } else if action.action_type == "USE_POTION"
                    && action.raw.get("potion_id").and_then(Value::as_str) == Some("SmokeBomb")
                    && record.phase == "COMBAT"
                    && bundle
                        .actions
                        .get(position + 1)
                        .is_some_and(|next| next.action_type == "PATH")
                {
                    smoke_bomb_callback = true;
                    match &game_action {
                        GameAction::CombatAction(action) => {
                            engine.project_combat_callback_action(action)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
            let outcome = engine.step_game(&game_action);
            if outcome.status != ActionStatus::Accepted {
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

            let recorder_omitted_post_action_choice = engine
                .get_combat_engine()
                .filter(|combat| combat.phase == CombatPhase::AwaitingChoice)
                .is_some_and(|combat| {
                    action.action_type == "USE_POTION"
                        || (action.action_type == "PLAY_CARD"
                            && combat.choice.as_ref().is_some_and(|choice| {
                                choice.reason == ChoiceReason::ExhaustFromHand
                                    && combat.runtime_played_card.is_some_and(|card| {
                                        matches!(
                                            combat.card_registry.card_name(card.def_id),
                                            "Purity" | "Purity+"
                                        )
                                    })
                            }))
                });
            if outcome.status == ActionStatus::Accepted && recorder_omitted_post_action_choice {
                // Choice potions queue a combat selection, while the legacy
                // USE_POTION record is sampled only after that queue settles.
                // Purity's ExhaustAction is sampled the same way after its
                // any-number hand grid resolves. The recorder emits no CHOOSE
                // payload for either path. Reuse the bounded exact checkpoint
                // search and keep the result uncertified. Unsupported
                // multi-select contexts fail closed inside
                // infer_omitted_combat_choice.
                //
                // For generated-card potions the complete checkpoint,
                // including the generated hand card and cardRandom counter,
                // is the oracle witness.
                // Java: potions/StancePotion.java, LiquidMemories.java,
                // AttackPotion.java, SkillPotion.java, PowerPotion.java, and
                // ColorlessPotion.java; actions/unique/DiscoveryAction.java;
                // cards/colorless/Purity.java and actions/common/ExhaustAction.java.
                let inferred = infer_omitted_combat_choice(&mut engine, action, record)?
                    .ok_or_else(|| {
                        format!(
                            "recorder action {} did not expose its canonical post-action choice",
                            action.idx
                        )
                    })?;
                report.mark_uncertified();
                report
                    .state_reconstructions
                    .extend(inferred.reconstructions);
                if let Some(reason) = inferred.unverified_reason {
                    report.unverified_action_semantics += 1;
                    report.unverified_actions.push(BundleUnverifiedAction {
                        idx: action.idx,
                        action: action.raw.clone(),
                        reason,
                    });
                }
                report.inferred_actions.extend(inferred.actions);
            }
        }

        let mut actual = if let Some(callback) = terminal_combat_callback.as_ref() {
            // The legacy hook records AbstractPlayer.useCard before
            // AbstractRoom's endBattleTimer settles the non-agent victory
            // transition. Compare that exact Java callback frame while the
            // canonical engine remains advanced to its next decision.
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report.unverified_actions.push(BundleUnverifiedAction {
                idx: action.idx,
                action: action.raw.clone(),
                reason: if smoke_bomb_callback {
                    "legacy Smoke Bomb hook commits before SmokeBomb.use, then samples after potion destruction but before the escape timer advances the room; comparison uses that source-derived combat callback frame while canonical replay remains settled on the map"
                        .to_string()
                } else {
                    "legacy lethal PLAY_CARD checkpoint precedes Java's delayed room-victory transition; comparison uses the source-derived useCard callback frame while canonical replay remains settled"
                        .to_string()
                },
            });
            project_oracle_state(callback)?
        } else {
            project_oracle_state(&engine)?
        };
        let event_combat_entry_coupled = is_event_combat_entry_callback_coupled(
            &engine,
            &bundle.actions,
            &bundle.records,
            position,
        )?;
        if event_combat_entry_coupled {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report.unverified_actions.push(BundleUnverifiedAction {
                idx: action.idx,
                action: action.raw.clone(),
                reason: "the event callback sampled Java's pre-battle combat-entry frame before onEnterRoom/startBattle populated the opening hand and relic powers; the fully indexed following card and its exact checkpoint prove the canonical settled entry"
                    .to_string(),
            });
        }
        // The recorder can attach one settled state to several adjacent action
        // callbacks. Defer comparison across the complete identical-state block
        // rather than pretending each callback has its own causal checkpoint.
        // SpireHeart.goToFinalAct similarly exposes the non-agent
        // DOOR_UNLOCK animation before the next PATH proves Act 4 is ready.
        let mut state_coupled_to_following = mapping_coupled_to_following
            || recorder_only_shop_remove_opener
            || event_combat_entry_coupled
            // PressEndTurnButtonAction and TimeWarpPower emit an internal
            // END_TURN callback. Recorder FIFO sampling can attach either the
            // pre-turn frame or the already-settled next-turn frame to both
            // it and the preceding card, so the duplicate owns no independent
            // checkpoint. The next distinct indexed input is the witness.
            || recorder_only_early_sequence_end_turn
            // A lethal Necronomicon duplicate is the same pre-endBattle
            // callback already compared for the preceding semantic play.
            // Canonical replay has intentionally settled to rewards, so this
            // recorder-only callback owns no additional state checkpoint.
            || (recorder_only_necronomicon_callback
                && recorded_combat_victory_pending(record))
            || bundle
                .records
                .get(position + 1)
                .is_some_and(|next| same_recorded_state(record, next))
            || is_event_choice_followed_by_collapsed_exit(
                &bundle.actions,
                &bundle.records,
                position,
            )
            || is_necronomicon_reward_followed_by_event_exit(
                &bundle.actions,
                &bundle.records,
                position,
            )
            || is_campfire_effect_followed_by_path(&bundle.actions, position)
            || is_deferred_event_effect_followed_by_exit(
                &bundle.actions,
                &bundle.records,
                position,
            )
            || (record.screen == "DOOR_UNLOCK"
                && bundle
                    .actions
                    .get(position + 1)
                    .is_some_and(|next| next.action_type == "PATH"));
        let lethal_room_transition =
            matches!(action.action_type.as_str(), "PLAY_CARD" | "USE_POTION")
                && recorded_combat_victory_pending(record)
                && bundle.actions.get(position + 1).is_some_and(|next| {
                    next.action_type == "REWARD_TAKE"
                        || (next.action_type == "EVENT_CHOICE"
                            && next.raw.get("event").and_then(Value::as_str) == Some("SpireHeart"))
                });
        let mut expected = if state_coupled_to_following {
            // There is no independent oracle checkpoint for this action. The
            // final record in the identical-state block owns the comparison.
            json!({})
        } else {
            record.partial_oracle_state(!semantic_neow_selection)?
        };
        if lethal_room_transition {
            if let Some(object) = expected.as_object_mut() {
                object.remove("phase");
                object.remove("rng");
                strip_recorded_combat_transients(object);
            }
        }
        let mut partial = diff_partial_oracle_state(&expected, &actual);
        if recorded_java_collections_state.is_none()
            && action.action_type == "END_TURN"
            && !partial.diffs.is_empty()
            && partial
                .diffs
                .iter()
                .all(|diff| diff.path.starts_with("piles.exhaust["))
        {
            let expected_exhaust = expected
                .pointer("/piles/exhaust")
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .map(Value::as_str)
                        .map(|value| value.map(str::to_string))
                        .collect::<Option<Vec<_>>>()
                });
            if let Some(expected_exhaust) = expected_exhaust {
                if let Some(previous_order) =
                    engine.reconcile_legacy_ethereal_exhaust_order(&expected_exhaust)
                {
                    report.mark_uncertified();
                    report.state_reconstructions.push(BundleStateReconstruction {
                        before_idx: action.idx,
                        path: "piles.exhaust".to_string(),
                        expected: json!(expected_exhaust),
                        actual: json!(previous_order),
                        reason: "the legacy initial envelope omitted the raw process-global java.util.Collections default Random state; after the source-correct unshuffled status phase and shuffled Ethereal phase produced the exact same all-Ethereal suffix multiset, the adapter restored the recorded suffix order for deeper diagnostics without inventing an RNG state"
                            .to_string(),
                    });
                    actual = project_oracle_state(&engine)?;
                    partial = diff_partial_oracle_state(&expected, &actual);
                }
            }
        }
        if !state_coupled_to_following
            && !partial.diffs.is_empty()
            && matches!(action.action_type.as_str(), "PLAY_CARD" | "END_TURN")
            && following_combat_action_reaches_recorded_checkpoint(
                &engine,
                bundle.actions.get(position + 1),
                record,
            )?
        {
            // TraceLab can enqueue the next click before TraceWriter samples
            // the current callback. The fully indexed following card action
            // must reproduce every recorded causal field before this
            // checkpoint is treated as coupled; no state is reconstructed.
            state_coupled_to_following = true;
            expected = json!({});
            partial = diff_partial_oracle_state(&expected, &actual);
        }
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

    match reconcile_recorded_run_end(
        &mut engine,
        bundle.run_end.as_ref(),
        &bundle.meta.status,
        bundle.records.len(),
    ) {
        Ok(Some(inferred)) => {
            report.mark_uncertified();
            report.unverified_action_semantics += 1;
            report.inferred_actions.push(BundleInferredAction {
                before_idx: bundle.records.len(),
                action: inferred.action,
                reason: inferred.reason.clone(),
            });
            report.unverified_actions.push(BundleUnverifiedAction {
                idx: bundle.records.len(),
                action: inferred.lifecycle,
                reason: inferred.reason,
            });
        }
        Ok(None) => {}
        Err(divergence) => {
            report.status = BundleComparisonStatus::Diverged;
            report.first_divergence = Some(divergence);
        }
    }
    if report.status == BundleComparisonStatus::Match
        && !comparison_meets_strict_certification_contract(bundle, &report)
    {
        report.mark_uncertified();
    }
    Ok(report)
}

#[derive(Debug, Clone, PartialEq)]
struct TerminalRunEndInference {
    action: GameAction,
    lifecycle: Value,
    reason: String,
}

/// Settle a recorder-omitted final Proceed only from authoritative RUN_END
/// evidence and only when it is the canonical engine's sole progression
/// action. Optional potion discards may remain legal in the transition, but
/// cannot contribute to reaching TrueVictoryRoom and are never inferred.
/// ProceedButton routes an Ending boss room to TrueVictoryRoom, whose room
/// transition increments the floor before the run becomes a victory.
///
/// Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java
/// (`update`, `goToTrueVictoryRoom`) and rooms/TrueVictoryRoom.java.
fn reconcile_recorded_run_end(
    engine: &mut RunEngine,
    run_end: Option<&RecordedRunEnd>,
    recorded_outcome: &str,
    eof_idx: usize,
) -> Result<Option<TerminalRunEndInference>, BundleFirstDivergence> {
    let expected_lifecycle = run_end
        .map(|run_end| {
            json!({
                "kind": "lifecycle",
                "type": "RUN_END",
                "status": run_end.status,
                "floor": run_end.floor,
            })
        })
        .unwrap_or(Value::Null);
    let actual_shape = |engine: &RunEngine| {
        json!({
            "phase": format!("{:?}", engine.current_phase()),
            "legal_actions": engine.get_legal_actions(),
            "run_over": engine.run_state.run_over,
            "run_won": engine.run_state.run_won,
            "floor": engine.run_state.floor,
        })
    };
    let divergence = |engine: &RunEngine, detail: String| BundleFirstDivergence {
        idx: eof_idx,
        kind: "terminal_lifecycle".to_string(),
        path: "lifecycle.RUN_END".to_string(),
        expected: expected_lifecycle.clone(),
        actual: actual_shape(engine),
        detail,
        differences: Vec::new(),
    };

    let Some(run_end) = run_end else {
        if recorded_outcome == "VICTORY" {
            return Err(divergence(
                engine,
                "recording metadata declares VICTORY but the trace omitted authoritative RUN_END lifecycle evidence"
                    .to_string(),
            ));
        }
        return Ok(None);
    };
    if run_end.status != recorded_outcome {
        return Err(divergence(
            engine,
            format!(
            "recording metadata outcome {recorded_outcome:?} disagrees with RUN_END status {:?}",
            run_end.status
        ),
        ));
    }
    let expected_win = run_end.status == "VICTORY";
    if engine.current_phase() == RunPhase::GameOver
        && engine.run_state.run_over
        && engine.run_state.run_won == expected_win
        && engine.run_state.floor == run_end.floor
        && matches!(run_end.status.as_str(), "VICTORY" | "DEATH")
    {
        // Exact terminal recordings include the final canonical action and
        // then a separate RUN_END lifecycle record. No replay bridge or
        // semantic inference is needed in this shape.
        return Ok(None);
    }
    if run_end.status != "VICTORY" {
        return Err(divergence(
            engine,
            format!(
                "RUN_END status {:?} has no source-proven terminal replay bridge",
                run_end.status
            ),
        ));
    }
    let progression_actions = engine
        .get_legal_actions()
        .into_iter()
        .filter(|action| !matches!(action, GameAction::DiscardPotion(_)))
        .collect::<Vec<_>>();
    if engine.current_phase() != RunPhase::Transition
        || progression_actions != vec![GameAction::Proceed]
    {
        return Err(divergence(
            engine,
            "victory RUN_END does not follow a unique canonical Transition/Proceed progression decision"
                .to_string(),
        ));
    }

    let action = GameAction::Proceed;
    let outcome = engine.step_game(&action);
    if outcome.status != ActionStatus::Accepted
        || !engine.run_state.run_over
        || !engine.run_state.run_won
        || engine.run_state.floor != run_end.floor
    {
        return Err(divergence(
            engine,
            format!(
                "final Proceed did not settle the recorded TrueVictory outcome at floor {}",
                run_end.floor
            ),
        ));
    }

    Ok(Some(TerminalRunEndInference {
        action,
        lifecycle: expected_lifecycle,
        reason: "RUN_END VICTORY and the sole canonical Transition/Proceed progression action prove the recorder omitted ProceedButton.goToTrueVictoryRoom; optional potion discards were not inferred, and terminal victory/floor were checked after replay"
            .to_string(),
    }))
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

fn is_disabled_chest_relic_click(
    action: &RecorderAction,
    previous_record: Option<&RecordedTraceRecord>,
    record: &RecordedTraceRecord,
) -> bool {
    action.action_type == "REWARD_TAKE"
        && action.raw.get("reward_type").and_then(Value::as_str) == Some("RELIC")
        && previous_record.is_some_and(|previous| {
            previous.action.get("type").and_then(Value::as_str) == Some("REWARD_TAKE")
                && previous.action.get("reward_type").and_then(Value::as_str)
                    == Some("SAPPHIRE_KEY")
                && same_recorded_state(previous, record)
        })
}

fn is_shop_remove_grid_opener(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let Some(action) = actions.get(position) else {
        return false;
    };
    let Some(next_action) = actions.get(position + 1) else {
        return false;
    };
    let Some(record) = records.get(position) else {
        return false;
    };
    let Some(next_record) = records.get(position + 1) else {
        return false;
    };
    if action.action_type != "SHOP_REMOVE" || record.screen != "GRID" {
        return false;
    }
    let repeated_opener =
        next_action.action_type == "SHOP_REMOVE" && same_recorded_state(record, next_record);
    let selection_settled_with_purchase = position
        .checked_sub(1)
        .and_then(|index| records.get(index))
        .is_some_and(|previous| same_recorded_causal_state_ignoring_event_ui(previous, record))
        && matches!(
            next_action.action_type.as_str(),
            "SHOP_BUY_CARD" | "SHOP_BUY_RELIC" | "SHOP_BUY_POTION"
        );
    repeated_opener || selection_settled_with_purchase
}

fn is_early_sequence_end_turn_callback(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let Some(action) = actions.get(position) else {
        return false;
    };
    let Some(record) = records.get(position) else {
        return false;
    };
    let Some(mut source_position) = position.checked_sub(1) else {
        return false;
    };
    // Burn autoplay callbacks can sit between a card-owned
    // PressEndTurnButtonAction and Recorder's later END_TURN callback. Walk
    // backward only across callbacks already proven recorder-only and sharing
    // the exact settled state; the first causal action remains subject to the
    // normal terminating-card/Time-Warp safeguards below.
    // Java: Burn.java, GameActionManager.callEndOfTurnActions,
    // Meditate.java, and PressEndTurnButtonAction.java.
    while is_auto_played_burn_recorder_callback(actions, records, source_position)
        && records
            .get(source_position)
            .is_some_and(|candidate| same_recorded_state(candidate, record))
    {
        let Some(previous) = source_position.checked_sub(1) else {
            return false;
        };
        source_position = previous;
    }
    let (Some(source_action), Some(source_record)) =
        (actions.get(source_position), records.get(source_position))
    else {
        return false;
    };
    let source_turn_advanced = source_position
        .checked_sub(1)
        .and_then(|index| records.get(index))
        .is_some_and(|before_source| source_record.turn > before_source.turn);
    let source_card_ends_turn = source_action
        .raw
        .get("card_id")
        .and_then(Value::as_str)
        .is_some_and(|card_id| matches!(card_id, "Meditate" | "Conclude" | "Vault"));
    let time_warp_ends_turn = source_record
        .post
        .get("enemies")
        .and_then(Value::as_array)
        .is_some_and(|enemies| {
            enemies.iter().any(|enemy| {
                enemy.get("id").and_then(Value::as_str) == Some("TimeEater")
                    && enemy
                        .get("powers")
                        .and_then(Value::as_array)
                        .is_some_and(|powers| {
                            powers.iter().any(|power| {
                                power.get("id").and_then(Value::as_str) == Some("Time Warp")
                                    && power.get("amt").and_then(Value::as_i64) == Some(0)
                            })
                        })
            })
        });
    action.action_type == "END_TURN"
        && source_action.action_type == "PLAY_CARD"
        // The named cards necessarily queue PressEndTurnButtonAction, even if
        // recorder FIFO timing hid the turn boundary in an earlier callback.
        // For an ordinary card, retain the visible turn-boundary witness for
        // Time Warp so an unrelated player END_TURN cannot be swallowed.
        && (source_card_ends_turn || (time_warp_ends_turn && source_turn_advanced))
        && same_recorded_state(source_record, record)
}

fn following_combat_action_reaches_recorded_checkpoint(
    engine: &RunEngine,
    next_action: Option<&RecorderAction>,
    record: &RecordedTraceRecord,
) -> Result<bool, String> {
    let Some(next_action) = next_action else {
        return Ok(false);
    };
    let game_action = match next_action.action_type.as_str() {
        "PLAY_CARD" => {
            let Ok(card_idx) = validate_recorded_hand_card(engine, &next_action.raw) else {
                return Ok(false);
            };
            let Some(target_idx) = next_action.raw.get("target").and_then(Value::as_i64) else {
                return Ok(false);
            };
            GameAction::CombatAction(Action::PlayCard {
                card_idx,
                target_idx: target_idx as i32,
            })
        }
        "END_TURN" => GameAction::CombatAction(Action::EndTurn),
        _ => return Ok(false),
    };
    let mut lookahead = engine.clone();
    let outcome = lookahead.step_game(&game_action);
    if outcome.status != ActionStatus::Accepted {
        return Ok(false);
    }
    let expected = record.partial_oracle_state(true)?;
    let actual = project_oracle_state(&lookahead)?;
    Ok(diff_partial_oracle_state(&expected, &actual)
        .diffs
        .is_empty())
}

fn is_event_combat_entry_callback_coupled(
    engine: &RunEngine,
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> Result<bool, String> {
    let (Some(action), Some(record), Some(next_action), Some(next_record)) = (
        actions.get(position),
        records.get(position),
        actions.get(position + 1),
        records.get(position + 1),
    ) else {
        return Ok(false);
    };
    let prebattle_shape = action.action_type == "EVENT_CHOICE"
        && record.phase == "COMBAT"
        && record
            .post
            .pointer("/piles/hand")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        && record
            .post
            .pointer("/player/energy")
            .and_then(Value::as_i64)
            == Some(0)
        && record
            .post
            .get("enemies")
            .and_then(Value::as_array)
            .is_some_and(|enemies| !enemies.is_empty())
        && matches!(next_action.action_type.as_str(), "PLAY_CARD" | "END_TURN");
    if !prebattle_shape {
        return Ok(false);
    }
    // EventRoom.startEvent/onEnterRoom stages the fight before the ordinary
    // opening combat actions settle. Accept this recorder timing only when the
    // next fully indexed combat input reproduces its complete later checkpoint.
    // Java: rooms/EventRoom.java; events/beyond/MysteriousSphere.java;
    // rooms/AbstractRoom.java::onPlayerEntry.
    following_combat_action_reaches_recorded_checkpoint(engine, Some(next_action), next_record)
}

/// Return the fully indexed combat callbacks that share this recorder sample.
/// Recorder drains only one FIFO entry after each three-frame stable interval,
/// so fast inputs can all receive the state reached by the last queued input.
/// Stop at the first distinct state or non-combat action and fail closed in the
/// candidate replay if any indexed card is not legal from the inferred route.
///
/// Java recorder: packages/harness-java/src/main/java/tracelab/Recorder.java
fn identical_state_combat_lookahead<'a>(
    actions: &'a [RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> &'a [RecorderAction] {
    let Some(record) = records.get(position) else {
        return &[];
    };
    let start = position.saturating_add(1).min(actions.len());
    let mut end = start;
    while end < actions.len()
        && records
            .get(end)
            .is_some_and(|next| same_recorded_state(record, next))
        && matches!(actions[end].action_type.as_str(), "PLAY_CARD" | "END_TURN")
    {
        end += 1;
    }
    &actions[start..end]
}

struct CoupledCombatLookaheadOutcome {
    terminal_callback: Option<RunEngine>,
}

fn apply_coupled_combat_lookahead(
    engine: &mut RunEngine,
    lookahead: &[RecorderAction],
    project_terminal_callback: bool,
) -> Result<Option<CoupledCombatLookaheadOutcome>, String> {
    let mut terminal_callback = None;
    for (offset, action) in lookahead.iter().enumerate() {
        let combat_action = match action.action_type.as_str() {
            "PLAY_CARD" => {
                let card_idx = match validate_recorded_hand_card(engine, &action.raw) {
                    Ok(index) => index,
                    Err(_) => return Ok(None),
                };
                let Some(target_idx) = action.raw.get("target").and_then(Value::as_i64) else {
                    return Ok(None);
                };
                Action::PlayCard {
                    card_idx,
                    target_idx: i32::try_from(target_idx).map_err(|_| {
                        format!("lookahead action {} target is out of range", action.idx)
                    })?,
                }
            }
            "END_TURN" => Action::EndTurn,
            _ => return Ok(None),
        };
        if project_terminal_callback && offset + 1 == lookahead.len() {
            terminal_callback = engine.project_combat_callback_action(&combat_action);
        }
        if engine
            .step_game(&GameAction::CombatAction(combat_action))
            .status
            != ActionStatus::Accepted
        {
            return Ok(None);
        }
    }
    Ok(Some(CoupledCombatLookaheadOutcome { terminal_callback }))
}

/// Bridge recorder UI omissions only when the next recorded action proves a
/// unique canonical route. The bridge is serialized in the report and never
/// counted as a matched recorder action.
#[derive(Default)]
struct InferredPrerequisites {
    actions: Vec<BundleInferredAction>,
    unverified_reason: Option<String>,
    reconstructions: Vec<BundleStateReconstruction>,
}

#[derive(Clone)]
struct SettledCombatChoiceCandidate {
    actions: Vec<GameAction>,
    discovery_idle_rolls: usize,
}

#[derive(Clone, Copy)]
struct DiscoveryIdleRngAlignment {
    expected_counter: i32,
    max_inclusive: i32,
}

/// Recover a recorder-omitted combat grid selection only when a later settled
/// checkpoint is an exact oracle witness for the complete choice resolution.
///
/// Single-card choices share the canonical `Choose` surface and are safe to
/// search generically. Multi-select search is deliberately limited to two
/// source-specific paths: Meditate's mandatory discard-pile return and
/// Purity's optional hand exhaust. Both Java actions fix the source pile,
/// effect, and cardinality bounds. This keeps the bridge from inventing
/// semantics for an unrelated unresolved multi-select UI.
///
/// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/MeditateAction.java
/// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Meditate.java
/// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Purity.java
/// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
fn infer_omitted_combat_choice(
    engine: &mut RunEngine,
    action: &RecorderAction,
    record: &RecordedTraceRecord,
) -> Result<Option<InferredPrerequisites>, String> {
    infer_omitted_combat_choice_with_lookahead(engine, action, record, &[])
}

fn infer_omitted_combat_choice_with_lookahead(
    engine: &mut RunEngine,
    action: &RecorderAction,
    record: &RecordedTraceRecord,
    settled_lookahead: &[RecorderAction],
) -> Result<Option<InferredPrerequisites>, String> {
    let Some(combat) = engine
        .get_combat_engine()
        .filter(|combat| combat.phase == CombatPhase::AwaitingChoice)
    else {
        return Ok(None);
    };
    let choice = combat.choice.as_ref().ok_or_else(|| {
        format!(
            "canonical combat is awaiting a missing choice context before recorder action {}",
            action.idx
        )
    })?;
    let active_card_name = combat
        .runtime_played_card
        .map(|card| combat.card_registry.card_name(card.def_id));
    let meditate_multi_select = choice.max_picks > 1
        && choice.reason == ChoiceReason::ReturnFromDiscard
        && active_card_name.is_some_and(|name| matches!(name, "Meditate" | "Meditate+"));
    let purity_multi_select = choice.max_picks > 1
        && choice.reason == ChoiceReason::ExhaustFromHand
        && active_card_name.is_some_and(|name| matches!(name, "Purity" | "Purity+"));
    let source_specific_multi_select = meditate_multi_select || purity_multi_select;
    if choice.max_picks > 1 && !source_specific_multi_select {
        return Err(format!(
            "recorder omitted unsupported multi-select combat choice {:?} before action {}; no source-specific inference rule applies",
            choice.reason, action.idx
        ));
    }

    let expected = record.partial_oracle_state(true)?;
    let discovery_idle_alignment =
        if choice.reason == ChoiceReason::DiscoverCard && action.action_type == "USE_POTION" {
            let pool = match action.raw.get("potion_id").and_then(Value::as_str) {
                Some("AttackPotion") => Some(GeneratedCardPool::Attack),
                Some("SkillPotion") => Some(GeneratedCardPool::Skill),
                Some("PowerPotion") => Some(GeneratedCardPool::Power),
                Some("ColorlessPotion") => Some(GeneratedCardPool::Colorless),
                _ => None,
            };
            pool.and_then(|pool| {
                let pool_len = crate::effects::interpreter::generated_card_pool(combat, pool).len();
                let expected_counter = expected
                    .pointer("/rng/cardRandom")
                    .and_then(Value::as_i64)
                    .and_then(|value| i32::try_from(value).ok())?;
                (pool_len > 0).then_some(DiscoveryIdleRngAlignment {
                    expected_counter,
                    max_inclusive: i32::try_from(pool_len - 1).ok()?,
                })
            })
        } else {
            None
        };
    let mut candidates = Vec::new();
    // Purity's ordered exhaust pile observes click order. At the normal
    // ten-card hand cap, Purity+ can select five of the nine remaining cards,
    // so exhaustively checking its source-specific permutations needs more
    // than the generic single-choice/Meditate bound while remaining finite.
    let mut search_budget = if purity_multi_select { 65_536 } else { 512 };
    collect_settled_combat_choice_candidates(
        engine,
        &expected,
        discovery_idle_alignment,
        settled_lookahead,
        record.phase == "COMBAT" && recorded_combat_victory_pending(record),
        source_specific_multi_select,
        source_specific_multi_select,
        None,
        Vec::new(),
        &mut search_budget,
        &mut candidates,
    )?;
    candidates.sort_by_key(|candidate| {
        candidate
            .actions
            .iter()
            .map(GameAction::canonical_sort_key)
            .collect::<Vec<_>>()
    });

    let Some(candidate) = candidates.first().cloned() else {
        return Err(format!(
            "settled recorder action {} does not exactly match any canonical resolution of pending combat choice {:?}",
            action.idx, choice.reason
        ));
    };
    for game_action in &candidate.actions {
        if engine.step_game(game_action).status != ActionStatus::Accepted {
            return Err(format!(
                "canonical inferred combat choice {game_action:?} was rejected before recorder action {}",
                action.idx
            ));
        }
    }
    let reconstructions = if candidate.discovery_idle_rolls > 0 {
        let alignment = discovery_idle_alignment.expect("idle rolls require Discovery alignment");
        let actual_counter = alignment
            .expected_counter
            .saturating_sub(i32::try_from(candidate.discovery_idle_rolls).unwrap_or(i32::MAX));
        let applied = engine.reconcile_discovery_idle_card_random(
            alignment.expected_counter,
            alignment.max_inclusive,
        )?;
        if applied != candidate.discovery_idle_rolls {
            return Err(format!(
                "Discovery idle-RNG reconciliation expected {} rolls but applied {applied}",
                candidate.discovery_idle_rolls
            ));
        }
        vec![BundleStateReconstruction {
            before_idx: action.idx,
            path: "rng.cardRandom".to_string(),
            expected: json!(alignment.expected_counter),
            actual: json!(actual_counter),
            reason: format!(
                "DiscoveryAction regenerated and discarded choices on UI update frames before the recorder sampled the selection; advanced {} equal-bound cardRandom rolls to the exact witnessed endpoint",
                candidate.discovery_idle_rolls
            ),
        }]
    } else {
        Vec::new()
    };

    let reason = if candidates.len() == 1 {
        "the following exact settled checkpoint uniquely identifies this recorder-omitted combat choice"
            .to_string()
    } else {
        format!(
            "the following exact settled checkpoint identifies this recorder-omitted combat choice effect; {} canonical input sequences reach the same recorded oracle state",
            candidates.len()
        )
    };
    Ok(Some(InferredPrerequisites {
        actions: candidate
            .actions
            .into_iter()
            .map(|game_action| BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: reason.clone(),
            })
            .collect(),
        // Even a unique state-derived route is not a directly recorded agent
        // action. Keep the replay useful, but never count it as certified.
        unverified_reason: Some(if candidates.len() == 1 {
            "recorder omitted the combat-choice callback and payload; the following exact settled state uniquely proves the result, not a directly recorded action"
                .to_string()
        } else {
            format!(
                "recorder omitted the combat-choice callback and payload; the exact settled state leaves {} canonical input sequences observationally equivalent",
                candidates.len()
            )
        }),
        reconstructions,
    }))
}

fn collect_settled_combat_choice_candidates(
    engine: &RunEngine,
    expected: &Value,
    discovery_idle_alignment: Option<DiscoveryIdleRngAlignment>,
    settled_lookahead: &[RecorderAction],
    project_terminal_combat_callback: bool,
    allow_source_specific_multi_select: bool,
    preserve_multi_select_order: bool,
    last_choice_index: Option<usize>,
    prefix: Vec<GameAction>,
    search_budget: &mut usize,
    matches: &mut Vec<SettledCombatChoiceCandidate>,
) -> Result<(), String> {
    if *search_budget == 0 {
        return Err(
            "pending combat-choice inference exceeded its 512-state safety bound".to_string(),
        );
    }
    *search_budget -= 1;

    let pending_choice = engine
        .get_combat_engine()
        .filter(|combat| combat.phase == CombatPhase::AwaitingChoice)
        .and_then(|combat| combat.choice.as_ref());
    let Some(choice) = pending_choice else {
        let mut aligned = engine.clone();
        let discovery_idle_rolls = match discovery_idle_alignment {
            Some(alignment) => match aligned.reconcile_discovery_idle_card_random(
                alignment.expected_counter,
                alignment.max_inclusive,
            ) {
                Ok(rolls) => rolls,
                Err(_) => return Ok(()),
            },
            None => 0,
        };
        let Some(lookahead) = apply_coupled_combat_lookahead(
            &mut aligned,
            settled_lookahead,
            project_terminal_combat_callback,
        )?
        else {
            return Ok(());
        };
        let actual = if let Some(callback) = lookahead.terminal_callback.as_ref() {
            project_oracle_state(callback)?
        } else {
            project_oracle_state(&aligned)?
        };
        let settled_diff = diff_partial_oracle_state(expected, &actual);
        if settled_diff.diffs.is_empty() {
            matches.push(SettledCombatChoiceCandidate {
                actions: prefix,
                discovery_idle_rolls,
            });
        }
        return Ok(());
    };

    if choice.max_picks > 1 && !allow_source_specific_multi_select {
        return Ok(());
    }
    let can_choose_more = choice.selected.len() < choice.max_picks;
    let mut legal = engine
        .get_legal_actions()
        .into_iter()
        .filter(|candidate| {
            matches!(
                candidate,
                GameAction::CombatAction(Action::Choose(_))
                    | GameAction::CombatAction(Action::ConfirmSelection)
            )
        })
        .filter(|candidate| match candidate {
            GameAction::CombatAction(Action::Choose(index)) => {
                can_choose_more
                    && (!allow_source_specific_multi_select
                        || preserve_multi_select_order
                        || last_choice_index.is_none_or(|last| *index > last))
            }
            GameAction::CombatAction(Action::ConfirmSelection) => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    legal.sort_by_key(GameAction::canonical_sort_key);

    for game_action in legal {
        let next_last_choice = match game_action {
            GameAction::CombatAction(Action::Choose(index)) => Some(index),
            _ => last_choice_index,
        };
        let mut projected = engine.clone();
        if projected.step_game(&game_action).status != ActionStatus::Accepted {
            continue;
        }
        let mut next_prefix = prefix.clone();
        next_prefix.push(game_action);
        collect_settled_combat_choice_candidates(
            &projected,
            expected,
            discovery_idle_alignment,
            settled_lookahead,
            project_terminal_combat_callback,
            allow_source_specific_multi_select,
            preserve_multi_select_order,
            next_last_choice,
            next_prefix,
            search_budget,
            matches,
        )?;
    }
    Ok(())
}

/// Close every uniquely determined layer of a combat reward screen.
///
/// The legacy recorder hooks `RewardItem.claimReward`, but not
/// `SkipCardButton.closeCurrentScreen` or the final ProceedButton commit. A
/// later PATH or COMBAT checkpoint can therefore witness both a skipped active
/// card choice and the backing reward-screen exit. Infer only when each layer
/// exposes exactly one skip, and keep every recovered input visible in the
/// diagnostic report.
///
/// Java: ui/buttons/SkipCardButton.java::update;
/// screens/CardRewardScreen.java::takeReward;
/// rewards/RewardItem.java::claimReward; ui/buttons/ProceedButton.java::update.
fn infer_unique_reward_screen_exit(
    engine: &mut RunEngine,
    recorder_idx: usize,
    witness: &str,
) -> Result<Vec<BundleInferredAction>, String> {
    let mut inferred = Vec::new();
    while engine.current_phase() != RunPhase::MapChoice && engine.current_reward_screen().is_some()
    {
        let legal = engine.get_legal_actions();
        let skips = legal
            .iter()
            .filter_map(|candidate| match candidate {
                GameAction::SkipRewardItem(item_index) => Some(*item_index),
                _ => None,
            })
            .collect::<Vec<_>>();
        let (game_action, reason) = match skips.as_slice() {
            [item_index] => (
                GameAction::SkipRewardItem(*item_index),
                format!("{witness} proves the recorder skipped the open card-reward choice"),
            ),
            [] if legal.contains(&GameAction::LeaveRewards) => (
                GameAction::LeaveRewards,
                format!("{witness} proves the recorder left the completed reward screen"),
            ),
            [] => break,
            _ => {
                return Err(format!(
                    "recorder omitted an ambiguous reward skip before action {recorder_idx}: {legal:?}"
                ));
            }
        };

        if engine.step_game(&game_action).status != ActionStatus::Accepted {
            return Err(format!(
                "canonical inferred action {game_action:?} was rejected before recorder action {recorder_idx}"
            ));
        }
        inferred.push(BundleInferredAction {
            before_idx: recorder_idx,
            action: game_action,
            reason,
        });
    }
    Ok(inferred)
}

fn infer_missing_prerequisites(
    engine: &mut RunEngine,
    action: &RecorderAction,
    record: &RecordedTraceRecord,
) -> Result<InferredPrerequisites, String> {
    if action.action_type == "PLAY_CARD" {
        if let Some(game_action) = infer_omitted_wish_choice(engine, record)? {
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical Wish choice {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            return Ok(InferredPrerequisites {
                actions: vec![BundleInferredAction {
                    before_idx: action.idx,
                    action: game_action,
                    reason: "the settled player powers/gold uniquely identify Wish's recorder-omitted option"
                        .to_string(),
                }],
                unverified_reason: None,
                reconstructions: Vec::new(),
            });
        }
    }

    if engine.current_phase() == RunPhase::Shop
        && matches!(
            action.action_type.as_str(),
            "SHOP_BUY_CARD" | "SHOP_BUY_RELIC" | "SHOP_BUY_POTION"
        )
    {
        // The purge-grid selection can settle in the same sampled frame as a
        // subsequent merchant purchase. Search every legal removal, then
        // require that removal plus the fully identified recorded purchase to
        // reproduce the complete checkpoint exactly.
        // Java: ShopScreen.java::updatePurge and TraceWriter's deferred sample.
        let expected = record.partial_oracle_state(true)?;
        let mut candidates = Vec::new();
        for removal in engine
            .get_legal_actions()
            .into_iter()
            .filter(|candidate| matches!(candidate, GameAction::ShopRemoveCard(_)))
        {
            let mut projected = engine.clone();
            if projected.step_game(&removal).status != ActionStatus::Accepted {
                continue;
            }
            let mut purchase_adapter = RecorderActionAdapter::default();
            let Ok(Some(purchase)) = purchase_adapter.map(&projected, action, record, None) else {
                continue;
            };
            if projected.step_game(&purchase).status != ActionStatus::Accepted {
                continue;
            }
            let actual = project_oracle_state(&projected)?;
            if diff_partial_oracle_state(&expected, &actual)
                .diffs
                .is_empty()
            {
                candidates.push(removal);
            }
        }
        candidates.sort_by_key(GameAction::canonical_sort_key);
        if let Some(removal) = candidates.first().cloned() {
            let candidate_indices = candidates
                .iter()
                .filter_map(|candidate| match candidate {
                    GameAction::ShopRemoveCard(index) => Some(*index),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if candidates.len() > 1
                && !removal_candidates_are_equivalent(engine, &candidate_indices)
            {
                return Err(format!(
                    "purchase checkpoint after recorder action {} leaves {} semantically distinct omitted shop removals",
                    action.idx,
                    candidates.len()
                ));
            }
            if engine.step_game(&removal).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical inferred shop removal {removal:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            return Ok(InferredPrerequisites {
                actions: vec![BundleInferredAction {
                    before_idx: action.idx,
                    action: removal,
                    reason: "the following merchant purchase checkpoint exactly proves the recorder-omitted purge-grid selection"
                        .to_string(),
                }],
                unverified_reason: Some(if candidates.len() == 1 {
                    "recorder omitted the merchant purge selection callback; the combined purchase checkpoint uniquely proves the removed card"
                        .to_string()
                } else {
                    format!(
                        "recorder omitted the merchant purge selection callback; {} equivalent duplicate-card removals reach the same purchase checkpoint",
                        candidates.len()
                    )
                }),
                reconstructions: Vec::new(),
            });
        }
    }

    if let Some(inferred) = infer_omitted_combat_choice(engine, action, record)? {
        return Ok(inferred);
    }

    if action.action_type == "REWARD_TAKE"
        && engine.current_phase() == RunPhase::CardReward
        && engine.current_reward_screen().is_some_and(|screen| {
            matches!(
                screen.source,
                RewardScreenSource::Combat | RewardScreenSource::BossCombat
            )
        })
    {
        // TraceLab records RewardItem.claimReward, but SkipCardButton only
        // calls closeCurrentScreen. Therefore a click on an underlying combat
        // reward proves the open card screen was closed first even though no
        // callback exists for that close. The backing card RewardItem remains.
        // Java: SkipCardButton.java::update, CardRewardScreen.java::takeReward,
        // RewardItem.java::claimReward.
        let legal = engine.get_legal_actions();
        let skips = legal
            .iter()
            .filter_map(|candidate| match candidate {
                GameAction::SkipRewardItem(item_index) => Some(*item_index),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !skips.is_empty() {
            let [item_index] = skips.as_slice() else {
                return Err(format!(
                    "recorder reward click omitted a non-unique card-screen close before action {}: {legal:?}",
                    action.idx
                ));
            };
            let game_action = GameAction::SkipRewardItem(*item_index);
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical card-screen close {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            return Ok(InferredPrerequisites {
                actions: vec![BundleInferredAction {
                    before_idx: action.idx,
                    action: game_action,
                    reason: "the underlying combat-reward click proves the recorder omitted SkipCardButton's closeCurrentScreen callback"
                        .to_string(),
                }],
                unverified_reason: None,
                reconstructions: Vec::new(),
            });
        }
    }

    if action.action_type == "REWARD_TAKE"
        && action.raw.get("reward_type").and_then(Value::as_str) == Some("RELIC")
        && engine.current_reward_screen().is_some_and(|screen| {
            screen.items.first().is_some_and(|item| {
                item.kind == RewardItemKind::CardChoice
                    && item.label == "calling_bell_curse"
                    && item.state == RewardItemState::Available
                    && matches!(
                        item.choices.as_slice(),
                        [RewardChoice::Card { card_id, .. }] if card_id == "CurseOfTheBell"
                    )
            })
        })
    {
        // CallingBell.onEquip first opens a mandatory Curse of the Bell
        // confirmation, then grants its three relics screenlessly. The legacy
        // recorder hooks RewardItem.claimReward for the relics but not the
        // intervening card-grid staging. The first named relic callback proves
        // both uniquely legal mandatory actions.
        let inferred = [
            GameAction::SelectRewardItem(0),
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            },
        ];
        let mut actions = Vec::new();
        for game_action in inferred {
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical Calling Bell prerequisite {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            actions.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "the first Calling Bell relic callback proves the recorder omitted its mandatory Curse of the Bell confirmation"
                    .to_string(),
            });
        }
        return Ok(InferredPrerequisites {
            actions,
            unverified_reason: None,
            reconstructions: Vec::new(),
        });
    }

    if action.action_type == "BOSS_RELIC" {
        let mut inferred = Vec::new();

        if engine
            .current_reward_screen()
            .is_some_and(|screen| screen.source == RewardScreenSource::BossCombat)
            && engine
                .get_legal_actions()
                .contains(&GameAction::LeaveRewards)
        {
            let game_action = GameAction::LeaveRewards;
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical boss-reward leave {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "BOSS_RELIC proves the recorder left the completed boss-combat rewards"
                    .to_string(),
            });
        }

        if engine.current_phase() == RunPhase::Chest
            && engine.get_legal_actions().contains(&GameAction::OpenChest)
        {
            let game_action = GameAction::OpenChest;
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical boss-chest open {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "BOSS_RELIC proves the recorder opened Java's intervening BossChest"
                    .to_string(),
            });
        }

        if engine.current_reward_screen().is_some_and(|screen| {
            screen.source == RewardScreenSource::BossRelic && screen.active_item.is_none()
        }) {
            let legal = engine.get_legal_actions();
            let openers = legal
                .iter()
                .filter_map(|candidate| match candidate {
                    GameAction::SelectRewardItem(item_index) => Some(*item_index),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if openers.as_slice() != [0] {
                return Err(format!(
                    "legacy BOSS_RELIC omitted a non-unique boss-relic opener before action {}: {legal:?}",
                    action.idx
                ));
            }
            let game_action = GameAction::SelectRewardItem(0);
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical boss-relic opener {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "SelectRewardItem(0) is the uniquely legal typed opener for BossRelicSelectScreen"
                    .to_string(),
            });
        }

        if !inferred.is_empty() {
            return Ok(InferredPrerequisites {
                actions: inferred,
                unverified_reason: None,
                reconstructions: Vec::new(),
            });
        }
    }

    if action.action_type == "NEOW" && engine.current_phase() == RunPhase::CardReward {
        return infer_neow_grid_prerequisites(engine, action, record);
    }

    if action.action_type == "EVENT_CHOICE"
        && engine.current_phase() == RunPhase::CardReward
        && engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.label == "deck_selection_event_transform")
        })
    {
        // Drug Dealer/Designer record the dialog button before and after their
        // grid, but the legacy hook omits every selected card. Search only the
        // canonical typed grid choices and require their fully settled ordered
        // deck (including miscRng transforms) to equal the recorded result.
        let expected_deck = recorded_ordered_deck(record, "event transform grid")?;
        let mut inferred = Vec::new();
        if engine
            .current_reward_screen()
            .is_some_and(|screen| screen.active_item.is_none())
        {
            let openers = engine
                .get_legal_actions()
                .into_iter()
                .filter(|candidate| matches!(candidate, GameAction::SelectRewardItem(_)))
                .collect::<Vec<_>>();
            let [open_action] = openers.as_slice() else {
                return Err(format!(
                    "event-transform grid has a non-unique opener before action {}: {openers:?}",
                    action.idx
                ));
            };
            let open_action = open_action.clone();
            if engine.step_game(&open_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical event-transform opener {open_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: open_action,
                reason: "the recorder combines the event transform-grid opener with its unrecorded card selections"
                    .to_string(),
            });
        }
        let mut sequences = Vec::new();
        collect_event_grid_sequences(engine, &expected_deck, 3, Vec::new(), &mut sequences);
        sequences.sort_by_key(|sequence| {
            sequence
                .iter()
                .map(GameAction::canonical_sort_key)
                .collect::<Vec<_>>()
        });
        let Some(sequence) = sequences.first().cloned() else {
            return Err(format!(
                "recorded ordered deck does not match any canonical event-transform selection sequence before action {}",
                action.idx
            ));
        };
        let ambiguous = sequences.len() > 1;
        for game_action in sequence {
            if engine.step_game(&game_action).status != ActionStatus::Accepted {
                return Err(format!(
                    "canonical event-transform choice {game_action:?} was rejected before recorder action {}",
                    action.idx
                ));
            }
            inferred.push(BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "the final ordered deck and miscRng state identify a recorder-omitted event transform-grid selection"
                    .to_string(),
            });
        }
        return Ok(InferredPrerequisites {
            actions: inferred,
            unverified_reason: ambiguous.then(|| {
                format!(
                    "the recorded event-transform result is compatible with {} canonical card-identity sequences",
                    sequences.len()
                )
            }),
            reconstructions: Vec::new(),
        });
    }

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
            return Ok(InferredPrerequisites {
                actions: vec![BundleInferredAction {
                    before_idx: action.idx,
                    action: game_action,
                    reason:
                        "the recorder combines the event grid click with Rust's typed selection staging"
                            .to_string(),
                }],
                unverified_reason: None,
                reconstructions: Vec::new(),
            });
        }
    }

    if action.action_type == "EVENT_CHOICE"
        && action.raw.get("event").and_then(Value::as_str) == Some("SpireHeart")
        && engine.current_phase() == RunPhase::Transition
        && engine.get_legal_actions().contains(&GameAction::Proceed)
    {
        let game_action = GameAction::Proceed;
        if engine.step_game(&game_action).status != ActionStatus::Accepted {
            return Err(format!(
                "canonical Spire Heart transition {game_action:?} was rejected before recorder action {}",
                action.idx
            ));
        }
        return Ok(InferredPrerequisites {
            actions: vec![BundleInferredAction {
                before_idx: action.idx,
                action: game_action,
                reason: "the Spire Heart event callback proves the recorder omitted the preceding boss-room transition Proceed"
                    .to_string(),
            }],
            unverified_reason: None,
            reconstructions: Vec::new(),
        });
    }

    // TraceLab's MapRoomNode patch records ordinary node clicks, but the
    // synthetic boss icon is handled by DungeonMap/DungeonMapScreen instead.
    // A combat checkpoint plus exactly one legal path proves the omitted boss
    // click without guessing among agent choices. Java: DungeonMapScreen.java
    // (`nodes.isEmpty()` selects `bossHb`) and DungeonMap.java (`bossHb`).
    let recorded_combat_started = record.phase == "COMBAT"
        && record
            .post
            .get("enemies")
            .and_then(Value::as_array)
            .is_some_and(|enemies| !enemies.is_empty());
    if action.action_type != "PATH" && recorded_combat_started {
        let mut inferred =
            infer_unique_reward_screen_exit(engine, action.idx, "the following combat checkpoint")?;
        if engine.current_phase() == RunPhase::MapChoice {
            let legal = engine.get_legal_actions();
            let path_actions = legal
                .iter()
                .filter(|candidate| matches!(candidate, GameAction::ChoosePath(_)))
                .collect::<Vec<_>>();
            if path_actions.as_slice() == [&GameAction::ChoosePath(0)] {
                let game_action = GameAction::ChoosePath(0);
                let outcome = engine.step_game(&game_action);
                if outcome.status != ActionStatus::Accepted {
                    return Err(format!(
                        "canonical inferred boss-path action {game_action:?} was rejected before recorder action {}",
                        action.idx
                    ));
                }
                inferred.push(BundleInferredAction {
                    before_idx: action.idx,
                    action: game_action,
                    reason: "the combat checkpoint and sole synthetic boss destination prove the boss-icon click omitted by the recorder"
                        .to_string(),
                });
            }
        }
        if !inferred.is_empty() {
            return Ok(InferredPrerequisites {
                actions: inferred,
                unverified_reason: None,
                reconstructions: Vec::new(),
            });
        }
    }

    if action.action_type != "PATH" {
        return Ok(InferredPrerequisites::default());
    }

    let mut inferred = Vec::new();
    if engine.current_phase() == RunPhase::Transition
        && engine.get_legal_actions().contains(&GameAction::Proceed)
    {
        // The recorder's next PATH is the settled witness for the intervening
        // DungeonTransitionScreen proceed. Java creates the next dungeon only
        // after this click. Java: DungeonTransitionScreen.java::update and
        // AbstractDungeon.java::nextRoomTransition.
        let game_action = GameAction::Proceed;
        let outcome = engine.step_game(&game_action);
        if outcome.status != ActionStatus::Accepted {
            return Err(format!(
                "canonical inferred transition {game_action:?} was rejected before recorder action {}",
                action.idx
            ));
        }
        inferred.push(BundleInferredAction {
            before_idx: action.idx,
            action: game_action,
            reason:
                "PATH in the next act proves the recorder omitted DungeonTransitionScreen Proceed"
                    .to_string(),
        });
    }
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
    inferred.extend(infer_unique_reward_screen_exit(engine, action.idx, "PATH")?);
    if engine.current_phase() == RunPhase::Transition
        && engine.get_legal_actions().contains(&GameAction::Proceed)
    {
        // A reward-screen Proceed can reveal DungeonTransitionScreen during
        // this same omitted UI chain (Calling Bell is the concrete Java path).
        // The next-act PATH proves the second Proceed as well.
        let game_action = GameAction::Proceed;
        if engine.step_game(&game_action).status != ActionStatus::Accepted {
            return Err(format!(
                "canonical post-reward transition {game_action:?} was rejected before recorder action {}",
                action.idx
            ));
        }
        inferred.push(BundleInferredAction {
            before_idx: action.idx,
            action: game_action,
            reason: "PATH in the next act proves the recorder omitted the post-reward DungeonTransitionScreen Proceed"
                .to_string(),
        });
    }
    Ok(InferredPrerequisites {
        actions: inferred,
        unverified_reason: None,
        reconstructions: Vec::new(),
    })
}

fn collect_event_grid_sequences(
    engine: &RunEngine,
    expected_deck: &[String],
    depth_remaining: usize,
    prefix: Vec<GameAction>,
    matches: &mut Vec<Vec<GameAction>>,
) {
    if engine.run_state.deck == expected_deck {
        matches.push(prefix);
        return;
    }
    if depth_remaining == 0 || engine.current_phase() != RunPhase::CardReward {
        return;
    }
    let mut legal = engine
        .get_legal_actions()
        .into_iter()
        .filter(|candidate| matches!(candidate, GameAction::ChooseRewardOption { .. }))
        .collect::<Vec<_>>();
    if legal.is_empty() {
        let openers = engine
            .get_legal_actions()
            .into_iter()
            .filter(|candidate| matches!(candidate, GameAction::SelectRewardItem(_)))
            .collect::<Vec<_>>();
        if let [open_action] = openers.as_slice() {
            let mut projected = engine.clone();
            if projected.step_game(open_action).status == ActionStatus::Accepted {
                let mut next_prefix = prefix;
                next_prefix.push(open_action.clone());
                collect_event_grid_sequences(
                    &projected,
                    expected_deck,
                    depth_remaining,
                    next_prefix,
                    matches,
                );
            }
        }
        return;
    }
    legal.sort_by_key(GameAction::canonical_sort_key);
    for game_action in legal {
        let mut projected = engine.clone();
        if projected.step_game(&game_action).status != ActionStatus::Accepted {
            continue;
        }
        let mut next_prefix = prefix.clone();
        next_prefix.push(game_action);
        collect_event_grid_sequences(
            &projected,
            expected_deck,
            depth_remaining - 1,
            next_prefix,
            matches,
        );
    }
}

fn infer_omitted_wish_choice(
    engine: &RunEngine,
    record: &RecordedTraceRecord,
) -> Result<Option<GameAction>, String> {
    let Some(combat) = engine.get_combat_engine() else {
        return Ok(None);
    };
    let Some(active_card) = combat.runtime_played_card else {
        return Ok(None);
    };
    if !combat
        .card_registry
        .card_name(active_card.def_id)
        .starts_with("Wish")
    {
        return Ok(None);
    }
    let legal = engine
        .get_legal_actions()
        .into_iter()
        .filter(|candidate| matches!(candidate, GameAction::CombatAction(Action::Choose(_))))
        .collect::<Vec<_>>();
    if legal.is_empty() {
        return Ok(None);
    }
    let recorded_player = record
        .post
        .get("player")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("trace action {} omitted post.player", record.idx))?;
    let recorded_gold = recorded_player.get("gold");
    let recorded_powers = recorded_player
        .get("powers")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("trace action {} omitted post.player.powers", record.idx))?;
    let mut matching = Vec::new();
    for candidate in legal {
        let mut projected_engine = engine.clone();
        if projected_engine.step_game(&candidate).status != ActionStatus::Accepted {
            continue;
        }
        let projected = serde_json::to_value(project_oracle_state(&projected_engine)?)
            .map_err(|error| format!("failed to project Wish choice: {error}"))?;
        let Some(player) = projected.get("player").and_then(Value::as_object) else {
            continue;
        };
        if player.get("gold") != recorded_gold {
            continue;
        }
        let Some(projected_powers) = player.get("powers").and_then(Value::as_array) else {
            continue;
        };
        if projected_powers
            .iter()
            .all(|power| recorded_powers.contains(power))
        {
            matching.push(candidate);
        }
    }
    match matching.as_slice() {
        [] => Err(format!(
            "settled trace action {} does not identify any canonical Wish option",
            record.idx
        )),
        [choice] => Ok(Some(choice.clone())),
        _ => Err(format!(
            "settled trace action {} leaves {} Wish options ambiguous",
            record.idx,
            matching.len()
        )),
    }
}

fn infer_neow_grid_prerequisites(
    engine: &mut RunEngine,
    action: &RecorderAction,
    record: &RecordedTraceRecord,
) -> Result<InferredPrerequisites, String> {
    let legal = engine.get_legal_actions();
    if legal.as_slice() != [GameAction::SelectRewardItem(0)] {
        return Err(format!(
            "recorder Neow grid omitted a non-unique canonical opener before action {}: {legal:?}",
            action.idx
        ));
    }

    let open_action = GameAction::SelectRewardItem(0);
    if engine.step_game(&open_action).status != ActionStatus::Accepted {
        return Err(format!(
            "canonical Neow grid-open action {open_action:?} was rejected before recorder action {}",
            action.idx
        ));
    }

    let (choice_action, identity_verified) = map_recorded_neow_grid_choice(engine, record)?;
    if engine.step_game(&choice_action).status != ActionStatus::Accepted {
        return Err(format!(
            "canonical Neow grid choice {choice_action:?} was rejected before recorder action {}",
            action.idx
        ));
    }

    // NeowReward.update applies GridSelectScreen.selectedCards before the final
    // event Continue. Java: NeowReward.java:119-170, 241-246, 274-280.
    let choice_reason = if identity_verified {
        "the recorded ordered deck uniquely identifies the omitted Neow grid choice"
    } else {
        "the recorded ordered deck identifies one semantic Neow removal but cannot certify which equivalent duplicate card instance Java selected"
    };
    Ok(InferredPrerequisites {
        actions: vec![
            BundleInferredAction {
                before_idx: action.idx,
                action: open_action,
                reason: "SelectRewardItem(0) is the uniquely legal typed opener for the recorded Neow grid"
                    .to_string(),
            },
            BundleInferredAction {
                before_idx: action.idx,
                action: choice_action,
                reason: choice_reason.to_string(),
            },
        ],
        unverified_reason: (!identity_verified).then(|| {
            "recorded ordered deck proves the Neow removal semantics but does not distinguish equivalent duplicate deck-card instances"
                .to_string()
        }),
        reconstructions: Vec::new(),
    })
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

/// NecronomiconPower queues a second `UseCardAction` for the same eligible
/// attack. The legacy AbstractPlayer.useCard hook reports that internal play
/// with `hand_idx=-1` after the canonical card action has settled both uses.
/// Java: powers/NecronomiconPower.java::onUseCard.
fn is_necronomicon_recorder_only_callback(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(record), Some(previous_action), Some(previous_record)) = (
        actions.get(position),
        records.get(position),
        position.checked_sub(1).and_then(|index| actions.get(index)),
        position.checked_sub(1).and_then(|index| records.get(index)),
    ) else {
        return false;
    };
    is_internal_card_play_callback(action)
        && previous_action.action_type == "PLAY_CARD"
        && previous_action.raw.get("hand_idx").and_then(Value::as_i64) != Some(-1)
        && ["card_id", "upgrades", "target", "energy_on_use"]
            .into_iter()
            .all(|key| previous_action.raw.get(key) == action.raw.get(key))
        && same_recorded_state(previous_record, record)
        && record
            .post
            .get("relics")
            .and_then(Value::as_array)
            .is_some_and(|relics| {
                relics
                    .iter()
                    .any(|relic| relic.get("id").and_then(Value::as_str) == Some("Necronomicon"))
            })
}

fn is_unindexed_card_reward_preview(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(record), Some(next_action), Some(next_record)) = (
        actions.get(position),
        records.get(position),
        actions.get(position + 1),
        records.get(position + 1),
    ) else {
        return false;
    };
    let is_card_opener = |candidate: &RecorderAction| {
        candidate.action_type == "REWARD_TAKE"
            && candidate.raw.get("reward_type").and_then(Value::as_str) == Some("CARD")
    };
    // RewardItem.claimReward exposes no item index and SkipCardButton exposes
    // no recorder callback. Consecutive openers with identical settled state
    // therefore prove that the earlier card screen was only previewed/skipped,
    // while providing no evidence for which Prayer Wheel item it represented.
    // The final opener is mapped normally from its following CARD_REWARD pick.
    let followed_by_another_preview =
        is_card_opener(next_action) && same_recorded_state(record, next_record);
    let followed_by_close_without_card = next_action.action_type != "CARD_REWARD"
        && record.screen == "CARD_REWARD"
        && next_record.screen != "CARD_REWARD"
        && record.deck == next_record.deck;
    is_card_opener(action) && (followed_by_another_preview || followed_by_close_without_card)
}

fn is_canceled_campfire_smith(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(record), Some(next_action), Some(next_record)) = (
        actions.get(position),
        records.get(position),
        actions.get(position + 1),
        records.get(position + 1),
    ) else {
        return false;
    };
    action.action_type == "CAMPFIRE"
        && action.raw.get("choice").and_then(Value::as_str) == Some("SMITH")
        && next_action.action_type == "CAMPFIRE"
        && record.deck == next_record.deck
}

fn is_internal_card_play_callback(action: &RecorderAction) -> bool {
    action.action_type == "PLAY_CARD"
        && action.raw.get("hand_idx").and_then(Value::as_i64) == Some(-1)
}

fn upcoming_recorded_potion_reward(actions: &[RecorderAction], position: usize) -> Option<&str> {
    actions
        .iter()
        .skip(position)
        .take_while(|action| action.action_type != "PATH")
        .find_map(|action| {
            (action.action_type == "REWARD_TAKE"
                && action.raw.get("reward_type").and_then(Value::as_str) == Some("POTION"))
            .then(|| action.raw.get("id").and_then(Value::as_str))
            .flatten()
        })
}

/// Burn queues itself for automatic use during the end-turn sequence. The
/// legacy recorder observes AbstractPlayer.useCard before AbstractRoom.endTurn,
/// so the internal Burn callback precedes the actual agent END_TURN callback.
/// Require the exact Java card identity, adjacency, and identical settled state
/// before treating it as recorder-only.
fn is_auto_played_burn_recorder_callback(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(record)) = (actions.get(position), records.get(position)) else {
        return false;
    };
    if action.action_type != "PLAY_CARD"
        || action.raw.get("card_id").and_then(Value::as_str) != Some("Burn")
    {
        return false;
    }
    let mut cursor = position + 1;
    while actions.get(cursor).is_some_and(|next| {
        next.action_type == "PLAY_CARD"
            && next.raw.get("card_id").and_then(Value::as_str) == Some("Burn")
    }) && records
        .get(cursor)
        .is_some_and(|next| same_recorded_state(record, next))
    {
        cursor += 1;
    }
    actions
        .get(cursor)
        .is_some_and(|next| next.action_type == "END_TURN")
        && records
            .get(cursor)
            .is_some_and(|next| same_recorded_state(record, next))
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

fn same_recorded_causal_state_ignoring_event_ui(
    left: &RecordedTraceRecord,
    right: &RecordedTraceRecord,
) -> bool {
    left.floor == right.floor
        && left.act == right.act
        && left.turn == right.turn
        && left.map == right.map
        && left.post == right.post
        && left.deck == right.deck
}

fn is_event_choice_followed_by_collapsed_exit(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(next_action), Some(record), Some(next_record)) = (
        actions.get(position),
        actions.get(position + 1),
        records.get(position),
        records.get(position + 1),
    ) else {
        return false;
    };
    action.action_type == "EVENT_CHOICE"
        && next_action.action_type == "EVENT_CHOICE"
        && action.raw.get("event") == next_action.raw.get("event")
        && next_record.screen == "MAP"
        && same_recorded_causal_state_ignoring_event_ui(record, next_record)
}

fn is_collapsed_event_exit_pair(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    position.checked_sub(1).is_some_and(|previous| {
        is_event_choice_followed_by_collapsed_exit(actions, records, previous)
    })
}

/// Necronomicon.onEquip queues ShowCardAndObtainEffect for Necronomicurse.
/// The RewardItem hook observes the relic before that effect settles; Cursed
/// Tome's following Leave callback observes the curse and map transition.
/// Java: CursedTome.java::randomBook, Necronomicon.java::onEquip.
fn is_necronomicon_reward_followed_by_event_exit(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(next_action), Some(record), Some(next_record)) = (
        actions.get(position),
        actions.get(position + 1),
        records.get(position),
        records.get(position + 1),
    ) else {
        return false;
    };
    action.action_type == "REWARD_TAKE"
        && action.raw.get("reward_type").and_then(Value::as_str) == Some("RELIC")
        && action.raw.get("id").and_then(Value::as_str) == Some("Necronomicon")
        && next_action.action_type == "EVENT_CHOICE"
        && next_action.raw.get("event").and_then(Value::as_str) == Some("CursedTome")
        && record.screen == "COMBAT_REWARD"
        && next_record.screen == "MAP"
}

fn is_deferred_necronomicon_event_exit(
    engine: &RunEngine,
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let Some(previous) = position.checked_sub(1) else {
        return false;
    };
    is_necronomicon_reward_followed_by_event_exit(actions, records, previous)
        && engine.current_phase() == RunPhase::MapChoice
        && engine
            .run_state
            .deck
            .iter()
            .any(|card| card == "Necronomicurse")
}

/// CampfireOption.use queues its concrete CampfireEffect and closes the room
/// before that effect settles. The next PATH is the recorder's first witness
/// for Rest/Recall/Lift/Dig/Toke results. Java: ui/campfire/*.java.
fn is_campfire_effect_followed_by_path(actions: &[RecorderAction], position: usize) -> bool {
    actions.get(position).is_some_and(|action| {
        action.action_type == "CAMPFIRE"
            && actions
                .get(position + 1)
                .is_some_and(|next| next.action_type == "PATH")
    })
}

/// Ghosts queues five/three ShowCardAndObtainEffects before its explicit Leave
/// page. The first callback contains the max-HP loss but not the new cards;
/// the adjacent Leave callback is the first settled witness.
/// Java: events/city/Ghosts.java::buttonEffect/becomeGhost.
fn is_deferred_event_effect_followed_by_exit(
    actions: &[RecorderAction],
    records: &[RecordedTraceRecord],
    position: usize,
) -> bool {
    let (Some(action), Some(next_action), Some(next_record)) = (
        actions.get(position),
        actions.get(position + 1),
        records.get(position + 1),
    ) else {
        return false;
    };
    action.action_type == "EVENT_CHOICE"
        && next_action.action_type == "EVENT_CHOICE"
        && action.raw.get("event").and_then(Value::as_str) == Some("Ghosts")
        && next_action.raw.get("event") == action.raw.get("event")
        && next_record.screen == "MAP"
}

fn recorded_combat_victory_pending(record: &RecordedTraceRecord) -> bool {
    let Some(enemies) = record.post.get("enemies").and_then(Value::as_array) else {
        return false;
    };
    if enemies.is_empty() {
        return false;
    }
    let is_minion = |enemy: &Value| {
        enemy
            .get("powers")
            .and_then(Value::as_array)
            .is_some_and(|powers| {
                powers
                    .iter()
                    .any(|power| power.get("id").and_then(Value::as_str) == Some("Minion"))
            })
    };
    let owner_dead = enemies
        .iter()
        .any(|enemy| enemy.get("dead").and_then(Value::as_bool) == Some(true) && !is_minion(enemy));
    let ordinary_cleanup_pending = enemies
        .iter()
        .all(|enemy| enemy.get("dead").and_then(Value::as_bool) == Some(true) || is_minion(enemy));
    let awakened_one_cleanup_pending = enemies.iter().any(|enemy| {
        enemy.get("dead").and_then(Value::as_bool) == Some(true)
            && matches!(
                enemy.get("id").and_then(Value::as_str),
                Some("AwakenedOne" | "Awakened One")
            )
    }) && enemies.iter().all(|enemy| {
        enemy.get("dead").and_then(Value::as_bool) == Some(true)
            || enemy.get("id").and_then(Value::as_str) == Some("Cultist")
    });
    // AwakenedOne.die queues EscapeAction for its surviving Cultists. They do
    // not carry MinionPower, but the lethal useCard callback is still a
    // victory-pending frame just like other boss/minion cleanup callbacks.
    // Java: reference/extracted/methods/monster/AwakenedOne.java::die.
    owner_dead && (ordinary_cleanup_pending || awakened_one_cleanup_pending)
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
    last_mapping_inferred_reason: Option<String>,
}

impl RecorderActionAdapter {
    fn take_last_mapping_unverified(&mut self) -> Option<String> {
        std::mem::take(&mut self.last_mapping_unverified)
    }

    fn take_last_mapping_coupled_to_following(&mut self) -> bool {
        std::mem::take(&mut self.last_mapping_coupled_to_following)
    }

    fn take_last_mapping_inferred_reason(&mut self) -> Option<String> {
        std::mem::take(&mut self.last_mapping_inferred_reason)
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
        self.last_mapping_inferred_reason = None;
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
                    2 if self.neow_grid_followup => Ok(Some(GameAction::Proceed)),
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
                // ShopScreen's purge button callback opens GridCardSelectScreen;
                // Java performs the purchase/removal later in ShopScreen.update
                // after selectedCards becomes non-empty. The legacy recorder
                // captures only that opener. The following checkpoint can prove
                // the deck delta, but it cannot turn the omitted selection into
                // a recorded agent action.
                // Java: shop/ShopScreen.java::updatePurgeCard, purchasePurge,
                // and updatePurge.
                self.last_mapping_unverified = Some(if identity_verified {
                    "recorder SHOP_REMOVE omits the selected-card callback; the following ordered deck uniquely identifies the removed card"
                        .to_string()
                } else {
                    "recorder SHOP_REMOVE omits the selected-card callback, and the resulting deck does not distinguish equivalent duplicate card instances"
                        .to_string()
                });
                self.last_mapping_inferred_reason = Some(
                    "the following ordered deck proves the recorder-omitted merchant grid selection"
                        .to_string(),
                );
                self.last_mapping_coupled_to_following = true;
                Ok(Some(mapped))
            }
            "REWARD_TAKE" => {
                let (mapped, identity_verified) = map_reward_take(engine, raw, next_record)?;
                if !identity_verified {
                    self.last_mapping_unverified = Some(
                        "the recorded card identity appears in multiple Prayer Wheel offers; the persistent pick is proven but the opened reward-item identity is not"
                            .to_string(),
                    );
                }
                Ok(Some(mapped))
            }
            "CARD_REWARD" => Ok(Some(map_reward_choice(engine, raw, "card_id")?)),
            "BOSS_RELIC" => {
                let relic_id = value_str(raw, "relic_id")?;
                let screen = engine.current_reward_screen().ok_or_else(|| {
                    "legacy BOSS_RELIC occurred without BossRelicSelectScreen".to_string()
                })?;
                if screen.source != RewardScreenSource::BossRelic {
                    return Err(format!(
                        "legacy BOSS_RELIC occurred on {:?} reward screen",
                        screen.source
                    ));
                }
                let item_index = screen.active_item.ok_or_else(|| {
                    "legacy BOSS_RELIC omitted canonical reward-item staging".to_string()
                })?;
                let item = screen
                    .items
                    .get(item_index)
                    .ok_or_else(|| format!("active boss-relic item {item_index} is absent"))?;
                let choice_index = unique_candidate(
                    item.choices.iter().enumerate().filter_map(
                        |(choice_index, choice)| match choice {
                            RewardChoice::Named { label, .. } if label == relic_id => {
                                Some(choice_index)
                            }
                            _ => None,
                        },
                    ),
                    &format!("boss relic choice {relic_id:?}"),
                )?;
                Ok(Some(GameAction::ChooseRewardOption {
                    item_index,
                    choice_index,
                }))
            }
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

fn map_recorded_neow_grid_choice(
    engine: &RunEngine,
    record: &RecordedTraceRecord,
) -> Result<(GameAction, bool), String> {
    let expected_deck = recorded_ordered_deck(record, "Neow grid")?;
    let screen = engine
        .current_reward_screen()
        .ok_or_else(|| "Neow grid selection has no canonical selection screen".to_string())?;
    let item_index = screen
        .active_item
        .ok_or_else(|| "Neow grid selection item was not opened".to_string())?;
    let item = screen
        .items
        .iter()
        .find(|item| item.index == item_index)
        .ok_or_else(|| format!("active Neow grid item {item_index} is absent"))?;
    if !matches!(
        item.label.as_str(),
        "deck_selection_neow_remove"
            | "deck_selection_neow_upgrade"
            | "deck_selection_neow_transform"
    ) {
        return Err(format!(
            "recorder omitted semantic payload for unsupported Neow grid {:?}",
            item.label
        ));
    }

    let mut candidates = Vec::new();
    for candidate in engine.get_legal_actions() {
        let GameAction::ChooseRewardOption {
            item_index: candidate_item,
            choice_index,
        } = candidate
        else {
            continue;
        };
        if candidate_item != item_index {
            continue;
        }
        let Some(RewardChoice::Card {
            index: deck_index, ..
        }) = item.choices.get(choice_index)
        else {
            continue;
        };
        let mut projected = engine.clone();
        if projected.step_game(&candidate).status == ActionStatus::Accepted
            && projected.run_state.deck == expected_deck
        {
            candidates.push((candidate, *deck_index));
        }
    }
    candidates.sort_by_key(|(candidate, _)| candidate.canonical_sort_key());
    let Some((first_action, _)) = candidates.first() else {
        return Err(format!(
            "recorded ordered deck does not match any legal canonical choice for Neow grid {:?}",
            item.label
        ));
    };

    let identity_verified = candidates.len() == 1;
    if !identity_verified {
        let deck_indices = candidates
            .iter()
            .map(|(_, deck_index)| *deck_index)
            .collect::<Vec<_>>();
        if item.label != "deck_selection_neow_remove"
            || !removal_candidates_are_equivalent(engine, &deck_indices)
        {
            return Err(format!(
                "recorded ordered deck leaves {} semantically distinct Neow-grid candidates",
                candidates.len()
            ));
        }
    }

    Ok((first_action.clone(), identity_verified))
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
    if item.label == "event_library_card_reward" {
        // The legacy hook records The Library's dialog callback before and
        // after GridSelectScreen, but omits the selected card. Java offers 20
        // distinct base IDs, so the exact ordered deck delta identifies the
        // semantic grid choice without relying on display order.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/city/TheLibrary.java
        let mut candidates = Vec::new();
        for choice_index in 0..item.choices.len() {
            let game_action = GameAction::ChooseRewardOption {
                item_index,
                choice_index,
            };
            let mut projected = engine.clone();
            if projected.step_game(&game_action).status == ActionStatus::Accepted
                && projected.run_state.deck == expected_deck
            {
                candidates.push(game_action);
            }
        }
        let Some(first) = candidates.first().cloned() else {
            return Err(
                "recorded deck does not identify any canonical Library card choice".to_string(),
            );
        };
        if candidates.len() > 1 {
            return Err(format!(
                "recorded deck leaves {} semantically distinct Library card choices",
                candidates.len()
            ));
        }
        return Ok((first, true));
    }
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

fn map_reward_take(
    engine: &RunEngine,
    raw: &Value,
    next_record: Option<&RecordedTraceRecord>,
) -> Result<(GameAction, bool), String> {
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
    let mut candidates = screen
        .items
        .iter()
        .filter(|item| reward_kind_matches(item.kind, expected_kind))
        .collect::<Vec<_>>();
    if expected_kind == RewardItemKind::CardChoice {
        // Claimed Prayer Wheel card items remain in the ordered screen but
        // cannot be opened a second time. Unavailable linked keys deliberately
        // remain mappable below so the legacy duplicate Sapphire click can be
        // classified by is_unavailable_sapphire_click after rejection.
        candidates.retain(|item| item.state == RewardItemState::Available);
    }
    let (item, identity_verified) = if candidates.len() == 1 {
        (candidates[0], true)
    } else if expected_kind == RewardItemKind::Relic {
        // Calling Bell exposes three simultaneously claimable relic RewardItems.
        // The legacy callback omits the item index but does include the exact
        // relic ID, which authoritatively identifies the item. Java:
        // CallingBell.java::onEquip and RewardItem.java::claimReward.
        let wanted = value_str(raw, "id")?;
        if wanted.is_empty() {
            return Err(format!(
                "canonical state has multiple reward item of type {reward_type} candidates; recorder omitted both index and relic identity"
            ));
        }
        let matching = candidates.iter().copied().filter(|item| {
            item.label == wanted
                || item.choices.iter().any(
                    |choice| matches!(choice, RewardChoice::Named { label, .. } if label == wanted),
                )
        });
        (
            unique_candidate(matching, &format!("relic reward item {wanted:?}"))?,
            true,
        )
    } else if expected_kind == RewardItemKind::CardChoice {
        // Prayer Wheel exposes two card reward items. The legacy opener has no
        // item index, but the immediately following CARD_REWARD callback names
        // the chosen card and therefore identifies the unique containing item.
        // Java: CombatRewardScreen.java::setupItemReward.
        let next_action = next_record
            .filter(|record| {
                record.action.get("type").and_then(Value::as_str) == Some("CARD_REWARD")
            })
            .ok_or_else(|| {
                format!(
                    "canonical state has multiple reward item of type {reward_type} candidates; recorder omitted an authoritative index and no following card identity resolves it"
                )
            })?;
        let wanted = recorded_card_reward_id(&next_action.action)?;
        let matching = candidates
            .iter()
            .copied()
            .filter(|item| {
                item.choices.iter().any(|choice| {
                    matches!(choice, RewardChoice::Card { card_id, .. } if card_id == &wanted)
                })
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            let offers = candidates
                .iter()
                .map(|item| {
                    (
                        item.index,
                        item.choices
                            .iter()
                            .filter_map(|choice| match choice {
                                RewardChoice::Card { card_id, .. } => Some(card_id.as_str()),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();
            return Err(format!(
                "canonical state has no card reward item containing {wanted:?}; generated offers were {offers:?}"
            ));
        }
        // Two separately generated Prayer Wheel offers may contain the same
        // card. The recorder proves the persistent card choice but omits the
        // backing RewardItem index, so select the first equivalent opener and
        // report the identity ambiguity instead of inventing certainty.
        let identity_verified = matching.len() == 1;
        (matching[0], identity_verified)
    } else {
        (
            unique_candidate(candidates, &format!("reward item of type {reward_type}"))?,
            true,
        )
    };
    Ok((GameAction::SelectRewardItem(item.index), identity_verified))
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
    let wanted = if id_key == "card_id" {
        recorded_card_reward_id(raw)?
    } else {
        value_str(raw, id_key)?.to_string()
    };
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
            RewardChoice::Card { index, card_id } if card_id == &wanted => Some(*index),
            RewardChoice::Named { index, label } if label == &wanted => Some(*index),
            _ => None,
        }),
        &format!("active reward choice {wanted:?}"),
    )?;
    Ok(GameAction::ChooseRewardOption {
        item_index: item.index,
        choice_index,
    })
}

fn recorded_card_reward_id(raw: &Value) -> Result<String, String> {
    let card_id = value_str(raw, "card_id")?;
    let upgrades = raw.get("upgrades").and_then(Value::as_i64).unwrap_or(0);
    Ok(if upgrades > 0 && !card_id.ends_with('+') {
        format!("{card_id}+")
    } else {
        card_id.to_string()
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

    fn run_start_json() -> String {
        json!({"v": 1, "kind": "lifecycle", "type": "RUN_START", "floor": 0}).to_string()
    }

    fn exact_combat_record(engine: &RunEngine, idx: usize, action: Value) -> RecordedTraceRecord {
        let state = serde_json::to_value(project_oracle_state(engine).unwrap()).unwrap();
        RecordedTraceRecord {
            idx,
            floor: state["floor"].as_i64().unwrap() as i32,
            act: state["act"].as_i64().unwrap() as i32,
            turn: state["turn"].as_i64().unwrap() as i32,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: state["map"].clone(),
            action,
            post: json!({
                "player": state["player"].clone(),
                "enemies": state["enemies"].clone(),
                "piles": state["piles"].clone(),
                "relics": state["relics"].clone(),
                "potions": state["potions"].clone(),
                "rng": state["rng"].clone(),
            }),
            deck: state["deck"].clone(),
        }
    }

    fn pending_meditate(card_id: &str, discard: &[&str]) -> RunEngine {
        let mut engine = RunEngine::new(7, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.hand = vec![combat.card_registry.make_card(card_id)];
        combat.state.draw_pile = [
            "Strike_P",
            "Defend_P",
            "Vigilance",
            "Eruption",
            "Strike_P",
            "Defend_P",
            "Strike_P",
            "Defend_P",
        ]
        .into_iter()
        .map(|id| combat.card_registry.make_card(id))
        .collect();
        combat.state.discard_pile = discard
            .iter()
            .map(|id| combat.card_registry.make_card(id))
            .collect();
        combat.state.exhaust_pile.clear();
        combat.state.energy = 3;
        combat.state.stance = crate::state::Stance::Wrath;

        let outcome = engine.step_game(&GameAction::CombatAction(Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        }));
        assert_eq!(outcome.status, ActionStatus::Accepted);
        assert_eq!(
            engine.get_combat_engine().unwrap().phase,
            CombatPhase::AwaitingChoice
        );
        engine
    }

    #[test]
    fn omitted_base_meditate_choice_is_inferred_only_from_exact_settled_state() {
        // MeditateAction opens a mandatory one-card discard grid, then only
        // after it resolves do ChangeStanceAction and
        // PressEndTurnButtonAction run. The legacy recorder has no grid-choice
        // callback, so the following duplicate END_TURN checkpoint is the
        // first exact witness for the selected card.
        // Java: MeditateAction.java::update and Meditate.java::use.
        let mut pending = pending_meditate("Meditate", &["Evaluate", "Worship", "Strike_P"]);
        let mut settled = pending.clone();
        assert_eq!(
            settled
                .step_game(&GameAction::CombatAction(Action::Choose(1)))
                .status,
            ActionStatus::Accepted
        );
        assert_ne!(
            settled.get_combat_engine().unwrap().phase,
            CombatPhase::AwaitingChoice
        );

        let raw = json!({"idx": 248, "type": "END_TURN"});
        let action = RecorderAction {
            idx: 248,
            action_type: "END_TURN".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&settled, 248, raw);
        let inferred = infer_missing_prerequisites(&mut pending, &action, &record).unwrap();

        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            [GameAction::CombatAction(Action::Choose(1))]
        );
        assert!(inferred
            .unverified_reason
            .as_deref()
            .unwrap()
            .contains("omitted the combat-choice callback"));
        assert_eq!(
            serde_json::to_value(project_oracle_state(&pending).unwrap()).unwrap(),
            serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap()
        );
    }

    #[test]
    fn omitted_upgraded_meditate_choice_infers_full_mandatory_selection() {
        // Meditate+ raises magicNumber to two. Java does not continue to Calm
        // or the end-turn action until exactly two selectedCards are returned.
        // Java: Meditate.java::upgrade and MeditateAction.java::update.
        let mut pending =
            pending_meditate("Meditate+", &["Evaluate", "Worship", "Strike_P", "Protect"]);
        let expected_actions = [
            GameAction::CombatAction(Action::Choose(0)),
            GameAction::CombatAction(Action::Choose(2)),
            GameAction::CombatAction(Action::ConfirmSelection),
        ];
        let mut settled = pending.clone();
        for game_action in &expected_actions {
            assert_eq!(
                settled.step_game(game_action).status,
                ActionStatus::Accepted
            );
        }

        let raw = json!({"idx": 300, "type": "END_TURN"});
        let action = RecorderAction {
            idx: 300,
            action_type: "END_TURN".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&settled, 300, raw);
        let inferred = infer_missing_prerequisites(&mut pending, &action, &record).unwrap();

        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            expected_actions
        );
        assert!(inferred.unverified_reason.is_some());
        assert_eq!(
            serde_json::to_value(project_oracle_state(&pending).unwrap()).unwrap(),
            serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap()
        );
    }

    #[test]
    fn omitted_upgraded_meditate_choice_infers_observable_click_order() {
        // GridCardSelectScreen stores selectedCards in click order, which
        // MeditateAction preserves when returning the cards to hand. The
        // settled hand therefore distinguishes selection permutations; the
        // inference search must not collapse them into index order.
        // Java: GridCardSelectScreen.java::update and MeditateAction.java::update.
        let mut pending =
            pending_meditate("Meditate+", &["Evaluate", "Worship", "Strike_P", "Protect"]);
        let expected_actions = [
            GameAction::CombatAction(Action::Choose(3)),
            GameAction::CombatAction(Action::Choose(0)),
            GameAction::CombatAction(Action::ConfirmSelection),
        ];
        let mut settled = pending.clone();
        for game_action in &expected_actions {
            assert_eq!(
                settled.step_game(game_action).status,
                ActionStatus::Accepted
            );
        }

        let raw = json!({"idx": 301, "type": "END_TURN"});
        let action = RecorderAction {
            idx: 301,
            action_type: "END_TURN".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&settled, 301, raw);
        let inferred = infer_missing_prerequisites(&mut pending, &action, &record).unwrap();

        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            expected_actions
        );
        assert_eq!(
            serde_json::to_value(project_oracle_state(&pending).unwrap()).unwrap(),
            serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap()
        );
    }

    #[test]
    fn ordinary_card_coupled_checkpoint_does_not_infer_a_combat_choice() {
        let mut engine = RunEngine::new(7, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.hand = vec![combat.card_registry.make_card("Strike_P")];
        combat.state.energy = 3;
        assert_eq!(
            engine
                .step_game(&GameAction::CombatAction(Action::PlayCard {
                    card_idx: 0,
                    target_idx: 0,
                }))
                .status,
            ActionStatus::Accepted
        );
        assert_ne!(
            engine.get_combat_engine().unwrap().phase,
            CombatPhase::AwaitingChoice
        );

        let raw = json!({"idx": 12, "type": "END_TURN"});
        let action = RecorderAction {
            idx: 12,
            action_type: "END_TURN".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&engine, 12, raw);
        assert!(infer_omitted_combat_choice(&mut engine, &action, &record)
            .unwrap()
            .is_none());
    }

    #[test]
    fn omitted_attack_potion_choice_is_inferred_from_the_exact_settled_checkpoint() {
        // AttackPotion delegates to DiscoveryAction: Java advances
        // cardRandomRng while building three unique ATTACK choices, then adds
        // the selected zero-cost card only after the grid selection resolves.
        // The legacy recorder logs USE_POTION after that resolution but has no
        // separate selection payload. Its complete post-state is therefore a
        // valid diagnostic witness, while the inferred input stays uncertified.
        // Java: potions/AttackPotion.java::use and
        // actions/unique/DiscoveryAction.java::update.
        let mut pending = RunEngine::new(7, 0);
        pending.debug_enter_specific_combat(&["JawWorm"]);
        let combat = pending.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.potions[0] = "AttackPotion".to_string();

        assert_eq!(
            pending
                .step_game(&GameAction::CombatAction(Action::UsePotion {
                    potion_idx: 0,
                    target_idx: -1,
                }))
                .status,
            ActionStatus::Accepted
        );
        assert_eq!(
            pending.get_combat_engine().unwrap().phase,
            CombatPhase::AwaitingChoice
        );

        let mut settled = pending.clone();
        assert_eq!(
            settled
                .step_game(&GameAction::CombatAction(Action::Choose(1)))
                .status,
            ActionStatus::Accepted
        );
        // DiscoveryAction.generateCardChoices is (incorrectly but
        // authoritatively) called before its duration gate on every UI frame.
        // Model seven unrecorded frames of human selection latency. Every
        // ATTACK draw uses the 25-card Watcher source pool.
        // Java: actions/unique/DiscoveryAction.java::update.
        for _ in 0..7 {
            settled
                .debug_combat_engine_mut()
                .card_random_rng
                .random_int(24);
        }
        let raw = json!({
            "idx": 378,
            "type": "USE_POTION",
            "potion_id": "AttackPotion",
            "slot": 0,
            "target": -1
        });
        let action = RecorderAction {
            idx: 378,
            action_type: "USE_POTION".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&settled, 378, raw);
        let inferred = infer_omitted_combat_choice(&mut pending, &action, &record)
            .unwrap()
            .expect("exact Attack Potion post-state should identify the selected card");

        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            [GameAction::CombatAction(Action::Choose(1))]
        );
        assert!(inferred.unverified_reason.is_some());
        assert_eq!(inferred.reconstructions.len(), 1);
        assert_eq!(inferred.reconstructions[0].path, "rng.cardRandom");
        assert_eq!(
            inferred.reconstructions[0].expected.as_i64().unwrap()
                - inferred.reconstructions[0].actual.as_i64().unwrap(),
            7
        );
        assert_eq!(
            serde_json::to_value(project_oracle_state(&pending).unwrap()).unwrap(),
            serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap()
        );
    }

    #[test]
    fn omitted_purity_multi_select_is_inferred_from_the_exact_settled_checkpoint() {
        // Purity.java constructs ExhaustAction(magicNumber, false, true,
        // true): the hand grid accepts zero through three cards for base
        // Purity, then exhausts the selected batch. TraceLab records useCard
        // only after GridCardSelectScreen settles and emits no per-card choice
        // payload. The complete ordered zones therefore prove the selection,
        // while the recovered input remains explicitly uncertified.
        // HandCardSelectScreen appends clicks to selectedCards; ExhaustAction
        // iterates that group and moveToExhaustPile appends each card to the
        // exhaust pile. The final pile preserves click order, so permutations
        // are observably distinct.
        // Java: cards/colorless/Purity.java;
        // screens/select/HandCardSelectScreen.java;
        // actions/common/ExhaustAction.java::update; cards/CardGroup.java.
        let mut pending = RunEngine::new(7, 0);
        pending.debug_enter_specific_combat(&["JawWorm"]);
        let combat = pending.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.hand = ["Purity", "Strike_P", "Defend_P", "Vigilance"]
            .into_iter()
            .map(|id| combat.card_registry.make_card(id))
            .collect();
        combat.state.energy = 3;

        assert_eq!(
            pending
                .step_game(&GameAction::CombatAction(Action::PlayCard {
                    card_idx: 0,
                    target_idx: -1,
                }))
                .status,
            ActionStatus::Accepted
        );
        let choice = pending
            .get_combat_engine()
            .and_then(|combat| combat.choice.as_ref())
            .expect("Purity hand-exhaust choice");
        assert_eq!(choice.reason, ChoiceReason::ExhaustFromHand);
        assert_eq!((choice.min_picks, choice.max_picks), (0, 3));

        let mut settled = pending.clone();
        for action in [
            Action::Choose(2),
            Action::Choose(0),
            Action::ConfirmSelection,
        ] {
            assert_eq!(
                settled.step_game(&GameAction::CombatAction(action)).status,
                ActionStatus::Accepted
            );
        }
        let raw = json!({
            "idx": 778,
            "type": "PLAY_CARD",
            "card_id": "Purity",
            "upgrades": 0,
            "hand_idx": 6,
            "target": -1,
            "energy_on_use": 3
        });
        let action = RecorderAction {
            idx: 778,
            action_type: "PLAY_CARD".to_string(),
            raw: raw.clone(),
        };
        let record = exact_combat_record(&settled, 778, raw);
        let inferred = infer_omitted_combat_choice(&mut pending, &action, &record)
            .unwrap()
            .expect("exact Purity post-state should identify the exhausted cards");

        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            [
                GameAction::CombatAction(Action::Choose(2)),
                GameAction::CombatAction(Action::Choose(0)),
                GameAction::CombatAction(Action::ConfirmSelection),
            ]
        );
        assert!(inferred.unverified_reason.is_some());
        assert_eq!(
            serde_json::to_value(project_oracle_state(&pending).unwrap()).unwrap(),
            serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap()
        );
    }

    #[test]
    fn combat_checkpoint_closes_open_card_reward_before_unique_boss_path() {
        // The final Act 4 elite reward may still have its CardRewardScreen
        // open when the player skips it, leaves the backing combat rewards,
        // and clicks the sole Heart boss icon. TraceLab records none of those
        // commits, so the first Heart combat callback must recover all three
        // typed actions in order.
        // Java: SkipCardButton.java::update, ProceedButton.java::update,
        // DungeonMapScreen.java/DungeonMap.java bossHb branches.
        let mut engine = RunEngine::new(29, 0);
        engine.run_state.act = 4;
        engine.run_state.map_x = 3;
        engine.run_state.map_y = 2;
        engine.debug_roll_boss_sequence_for_act(4);
        engine.debug_build_combat_reward_screen(crate::map::RoomType::Elite);
        let card_item = engine
            .current_reward_screen()
            .unwrap()
            .items
            .iter()
            .find(|item| item.kind == RewardItemKind::CardChoice)
            .unwrap()
            .index;
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(card_item))
            .accepted());
        assert!(engine
            .get_legal_actions()
            .contains(&GameAction::SkipRewardItem(card_item)));

        let raw = json!({
            "idx": 803,
            "type": "USE_POTION",
            "potion_id": "Swift Potion",
            "slot": 0,
            "target": -1
        });
        let action = RecorderAction {
            idx: 803,
            action_type: "USE_POTION".to_string(),
            raw: raw.clone(),
        };
        let record = RecordedTraceRecord {
            idx: 803,
            floor: 55,
            act: 4,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": -1, "y": 15}),
            action: raw,
            post: json!({"enemies": [{"id": "CorruptHeart"}]}),
            deck: json!([]),
        };

        let inferred = infer_missing_prerequisites(&mut engine, &action, &record).unwrap();
        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            [
                GameAction::SkipRewardItem(card_item),
                GameAction::LeaveRewards,
                GameAction::ChoosePath(0),
            ]
        );
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        assert_eq!(
            engine.get_combat_engine().unwrap().state.enemies[0].id,
            "CorruptHeart"
        );
    }

    #[test]
    fn delayed_card_checkpoint_can_be_proven_by_following_end_turn() {
        // TraceWriter may sample a PLAY_CARD callback only after the already
        // clicked End Turn has settled. The fully typed following action is a
        // valid coupling witness only when its complete projected state exactly
        // matches the earlier callback record.
        let mut engine = RunEngine::new(17, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.hand = vec![combat.card_registry.make_card("Defend_P")];

        let mut settled = engine.clone();
        assert_eq!(
            settled
                .step_game(&GameAction::CombatAction(Action::EndTurn))
                .status,
            ActionStatus::Accepted
        );
        let raw = json!({"idx": 41, "type": "PLAY_CARD", "card_id": "Ghostly"});
        let record = exact_combat_record(&settled, 41, raw);
        let next = RecorderAction {
            idx: 42,
            action_type: "END_TURN".to_string(),
            raw: json!({"idx": 42, "type": "END_TURN"}),
        };

        assert!(
            following_combat_action_reaches_recorded_checkpoint(&engine, Some(&next), &record,)
                .unwrap()
        );
    }

    #[test]
    fn recorder_fifo_lookahead_stops_at_the_first_distinct_combat_state() {
        // Recorder drains one stable FIFO callback at a time. Meditate's own
        // PressEndTurnButtonAction can therefore leave PLAY_CARD and END_TURN
        // callbacks sharing one sample, but a later distinct sample is not
        // part of that coupled sequence.
        // Java recorder: packages/harness-java/src/main/java/tracelab/Recorder.java;
        // Java: Meditate.java::use and PressEndTurnButtonAction.java::update.
        let actions = vec![
            RecorderAction {
                idx: 0,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 0, "type": "PLAY_CARD", "card_id": "Meditate"}),
            },
            RecorderAction {
                idx: 1,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 1, "type": "END_TURN"}),
            },
            RecorderAction {
                idx: 2,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 2, "type": "PLAY_CARD", "card_id": "Burn"}),
            },
            RecorderAction {
                idx: 3,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 3, "type": "END_TURN"}),
            },
        ];
        let base = RecordedTraceRecord {
            idx: 0,
            floor: 1,
            act: 1,
            turn: 2,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": 0}),
            action: actions[0].raw.clone(),
            post: json!({"player": {"hp": 50}}),
            deck: json!(["Meditate"]),
        };
        let mut records = vec![base.clone(), base.clone(), base.clone(), base];
        for (index, record) in records.iter_mut().enumerate() {
            record.idx = index;
            record.action = actions[index].raw.clone();
        }
        records[3].turn = 3;

        assert_eq!(
            identical_state_combat_lookahead(&actions, &records, 0)
                .iter()
                .map(|action| action.idx)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn event_combat_entry_couples_only_to_an_exact_following_checkpoint() {
        // EventRoom stages combat before AbstractRoom's ordinary battle-start
        // actions settle. A following typed combat input is a valid witness
        // only if replaying it reaches the entire recorded checkpoint.
        // Java: EventRoom.java::startEvent and AbstractRoom.java::onPlayerEntry.
        let mut engine = RunEngine::new(42, 0);
        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.phase = CombatPhase::PlayerTurn;
        combat.state.hand = vec![combat.card_registry.make_card("Strike_P")];
        combat.state.energy = 3;

        let next_action = RecorderAction {
            idx: 1,
            action_type: "PLAY_CARD".to_string(),
            raw: json!({
                "idx": 1,
                "type": "PLAY_CARD",
                "card_id": "Strike_P",
                "upgrades": 0,
                "hand_idx": 0,
                "target": 0,
                "energy_on_use": 3
            }),
        };
        let mut settled = engine.clone();
        assert!(settled
            .step_game(&GameAction::CombatAction(Action::PlayCard {
                card_idx: 0,
                target_idx: 0,
            }))
            .accepted());
        let next_record = exact_combat_record(&settled, 1, next_action.raw.clone());
        let mut prebattle = next_record.clone();
        prebattle.idx = 0;
        prebattle.action = json!({"type": "EVENT_CHOICE", "choice": 0});
        prebattle.post["piles"]["hand"] = json!([]);
        prebattle.post["player"]["energy"] = json!(0);
        let actions = vec![
            RecorderAction {
                idx: 0,
                action_type: "EVENT_CHOICE".to_string(),
                raw: json!({"idx": 0, "type": "EVENT_CHOICE", "choice": 0}),
            },
            next_action,
        ];

        assert!(is_event_combat_entry_callback_coupled(
            &engine,
            &actions,
            &[prebattle.clone(), next_record.clone()],
            0,
        )
        .unwrap());

        let mut mismatched = next_record;
        mismatched.post["player"]["hp"] = json!(1);
        assert!(!is_event_combat_entry_callback_coupled(
            &engine,
            &actions,
            &[prebattle, mismatched],
            0,
        )
        .unwrap());
    }

    #[test]
    fn trace_header_is_retained_and_must_match_meta() {
        let header = json!({
            "v": 1,
            "kind": "header",
            "seed_long": 7,
            "seed_display": "7",
            "character": "WATCHER",
            "ascension": 0,
            "game_version": "desktop-1.0",
            "profile": null,
            "initial": null
        });
        let input = format!("{header}\n{}\n{}\n", run_start_json(), trace_json(0));
        let loaded = parse_trace_records(Cursor::new(input), Path::new("header.jsonl"))
            .expect("supported header should be retained");
        assert_eq!(loaded.header.as_ref(), Some(&header));

        let meta = RecordingMeta {
            v: 1,
            run_id: "header-test".to_string(),
            seed_long: 7,
            seed_display: "7".to_string(),
            character: "WATCHER".to_string(),
            ascension: 0,
            game_version: "desktop-1.0".to_string(),
            status: "IN_PROGRESS".to_string(),
            records: 1,
            sittings: Vec::new(),
            profile: None,
            initial: None,
            environment: None,
            profile_authority: None,
            profile_quarantine: None,
        };
        validate_trace_header(&meta, loaded.header.as_ref()).unwrap();

        let mut mismatched = header;
        mismatched["seed_long"] = json!(8);
        assert!(validate_trace_header(&meta, Some(&mismatched))
            .unwrap_err()
            .contains("seed_long does not match"));
    }

    #[test]
    fn concatenated_gzip_members_preserve_every_sitting() {
        let mut bytes = gzip_member(&format!("{}\n{}", run_start_json(), trace_json(0)));
        bytes.extend(gzip_member(&trace_json(1)));
        let loaded = parse_trace_records(
            MultiGzDecoder::new(Cursor::new(bytes)),
            Path::new("two-sittings.gz"),
        )
        .expect("both gzip members must decode");
        assert_eq!(
            loaded
                .records
                .iter()
                .map(|record| record.idx)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(loaded.run_end, None);
    }

    #[test]
    fn run_end_lifecycle_is_retained_and_duplicates_fail_closed() {
        let input = format!(
            "{}\n{}\n{}\n{}\n",
            run_start_json(),
            trace_json(0),
            json!({"v": 1, "kind": "lifecycle", "type": "RUN_END", "status": "VICTORY", "floor": 56}),
            json!({"v": 1, "kind": "lifecycle", "type": "RUN_END", "status": "VICTORY", "floor": 56}),
        );
        let error = parse_trace_records(Cursor::new(input), Path::new("duplicate-run-end.jsonl"))
            .expect_err("two terminal lifecycle records are not a unique oracle witness");
        assert!(error.contains("repeats the terminal RUN_END"));

        let input = format!(
            "{}\n{}\n{}\n",
            run_start_json(),
            trace_json(0),
            json!({"v": 1, "kind": "lifecycle", "type": "RUN_END", "status": "VICTORY", "floor": 56}),
        );
        let loaded = parse_trace_records(Cursor::new(input), Path::new("run-end.jsonl"))
            .expect("one terminal lifecycle is authoritative");
        assert_eq!(
            loaded.run_end,
            Some(RecordedRunEnd {
                status: "VICTORY".to_string(),
                floor: 56,
            })
        );
    }

    #[test]
    fn lifecycle_fsm_rejects_duplicate_or_late_start_and_tracks_resume() {
        let duplicate = format!(
            "{}\n{}\n{}\n",
            run_start_json(),
            run_start_json(),
            trace_json(0)
        );
        assert!(
            parse_trace_records(Cursor::new(duplicate), Path::new("duplicate-start.jsonl"))
                .unwrap_err()
                .contains("repeated or late RUN_START")
        );

        let late = format!("{}\n{}\n", trace_json(0), run_start_json());
        assert!(
            parse_trace_records(Cursor::new(late), Path::new("late-start.jsonl"))
                .unwrap_err()
                .contains("before RUN_START")
        );

        let resumed = format!(
            "{}\n{}\n{}\n",
            json!({"v": 1, "kind": "lifecycle", "type": "RESUME", "floor": 1}),
            trace_json(0),
            json!({"v": 1, "kind": "lifecycle", "type": "RUN_END", "status": "DEATH", "floor": 1}),
        );
        let resumed = parse_trace_records(Cursor::new(resumed), Path::new("resumed.jsonl"))
            .expect("RESUME remains readable for diagnostic replay");
        assert!(!resumed.lifecycle.saw_run_start);
        assert_eq!(resumed.lifecycle.resume_count, 1);
    }

    fn heart_waiting_for_true_victory() -> RunEngine {
        let mut run = RunEngine::new(42, 0);
        run.run_state.act = 4;
        // enter_specific_combat identifies a boss from the canonical boss
        // floor formula (act * 17 - 1).
        run.run_state.floor = 67;
        run.debug_enter_specific_combat(&["CorruptHeart"]);
        let combat = run.debug_combat_engine_mut();
        combat.state.enemies[0].entity.hp = 1;
        combat.state.hand = crate::tests::support::make_deck(&["Strike_P"]);
        combat.state.draw_pile.clear();
        combat.state.discard_pile.clear();
        combat.state.energy = 3;
        let callback = Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        };

        let projected = run
            .project_combat_callback_action(&callback)
            .expect("AbstractPlayer.useCard callback frame");
        assert_eq!(projected.current_phase(), RunPhase::Combat);
        assert!(projected.get_combat_engine().unwrap().state.enemies[0]
            .entity
            .is_dead());

        assert!(run
            .step_game(&GameAction::CombatAction(callback))
            .accepted());
        assert_eq!(run.current_phase(), RunPhase::Transition);
        assert_eq!(run.get_legal_actions(), vec![GameAction::Proceed]);
        run
    }

    #[test]
    fn heart_run_end_replays_unique_true_victory_proceed() {
        // The lethal useCard callback precedes AbstractRoom's victory timer;
        // ProceedButton later routes TheEnding to TrueVictoryRoom and
        // nextRoomTransition increments floorNum.
        // Java: AbstractPlayer.java::useCard; AbstractRoom.java::update;
        // ProceedButton.java::update/goToTrueVictoryRoom.
        let mut run = heart_waiting_for_true_victory();
        run.run_state.potions[2] = "Dexterity Potion".to_string();
        assert!(run
            .get_legal_actions()
            .contains(&GameAction::DiscardPotion(2)));
        let run_end = RecordedRunEnd {
            status: "VICTORY".to_string(),
            floor: 68,
        };
        let inferred = reconcile_recorded_run_end(&mut run, Some(&run_end), "VICTORY", 890)
            .expect("unique final Proceed must be source-proven")
            .expect("RUN_END requires one inferred action");

        assert_eq!(inferred.action, GameAction::Proceed);
        assert!(inferred.reason.contains("goToTrueVictoryRoom"));
        assert!(run.run_state.run_over);
        assert!(run.run_state.run_won);
        assert_eq!(run.run_state.floor, run_end.floor);
    }

    #[test]
    fn already_recorded_terminal_action_needs_no_lifecycle_inference() {
        let mut run = heart_waiting_for_true_victory();
        assert!(run.step_game(&GameAction::Proceed).accepted());
        let run_end = RecordedRunEnd {
            status: "VICTORY".to_string(),
            floor: 68,
        };

        assert_eq!(
            reconcile_recorded_run_end(&mut run, Some(&run_end), "VICTORY", 891)
                .expect("an exact GameOver state must validate against RUN_END"),
            None
        );
    }

    #[test]
    fn victory_run_end_mismatch_or_nonunique_shape_fails_closed() {
        let mut wrong_shape = RunEngine::new(42, 0);
        let run_end = RecordedRunEnd {
            status: "VICTORY".to_string(),
            floor: 1,
        };
        let error = reconcile_recorded_run_end(&mut wrong_shape, Some(&run_end), "VICTORY", 0)
            .expect_err("Neow's Proceed is not a TrueVictory terminal shape");
        assert!(error
            .detail
            .contains("unique canonical Transition/Proceed progression"));

        let mut wrong_floor = heart_waiting_for_true_victory();
        let wrong_end = RecordedRunEnd {
            status: "VICTORY".to_string(),
            floor: 69,
        };
        let error = reconcile_recorded_run_end(&mut wrong_floor, Some(&wrong_end), "VICTORY", 890)
            .expect_err("terminal floor mismatch must not be masked");
        assert!(error.detail.contains("did not settle"));
    }

    #[test]
    fn missing_note_profile_field_is_explicitly_quarantined() {
        let mut meta = json!({
            "profile": {
                "v": 1,
                "note_for_yourself_upgrades": 0,
                "highest_unlocked_ascension": 20,
                "is_daily_run": false,
                "is_trial": false,
                "final_act_available": true,
                "bosses_seen": [],
                "locked_cards": [],
                "locked_relics": []
            }
        });

        let quarantine = normalize_recording_profile(&mut meta).expect("missing field quarantine");
        assert_eq!(quarantine.kind, "missing_note_for_yourself_card");
        assert_eq!(
            meta.pointer("/profile/note_for_yourself_card"),
            Some(&json!("IronWave"))
        );

        meta["profile"]["note_for_yourself_card"] = Value::Null;
        let quarantine = normalize_recording_profile(&mut meta).expect("null field quarantine");
        assert_eq!(quarantine.kind, "unknown_note_for_yourself_card");

        meta["profile"]["note_for_yourself_card"] = json!("IronWave");
        meta["profile"]
            .as_object_mut()
            .unwrap()
            .remove("note_for_yourself_upgrades");
        let quarantine =
            normalize_recording_profile(&mut meta).expect("missing NOTE_UPGRADE quarantine");
        assert_eq!(quarantine.kind, "missing_note_for_yourself_upgrades");
    }

    fn complete_initial_envelope() -> (RecordingMeta, RecordedTraceRecord) {
        let raw_state = |counter: i64| {
            json!({
                "seed0": "0123456789abcdef",
                "seed1": "fedcba9876543210",
                "counter": counter
            })
        };
        let initial = json!({
            "v": 1,
            "captured_at": "first_run_frame_post_generation",
            "seed_set": false,
            "has_ruby_key": false,
            "has_emerald_key": false,
            "has_sapphire_key": false,
            "rng": {
                "card": 0,
                "monster": 0,
                "event": 0,
                "relic": 0,
                "treasure": 0,
                "potion": 0,
                "merchant": 0,
                "monsterHp": 0,
                "ai": 0,
                "shuffle": 0,
                "cardRandom": 0,
                "misc": 0,
                "map": 98,
                "rawStates": {
                    "card": raw_state(0),
                    "monster": raw_state(0),
                    "event": raw_state(0),
                    "relic": raw_state(0),
                    "treasure": raw_state(0),
                    "potion": raw_state(0),
                    "merchant": raw_state(0),
                    "monsterHp": raw_state(0),
                    "ai": raw_state(0),
                    "shuffle": raw_state(0),
                    "cardRandom": raw_state(0),
                    "misc": raw_state(0),
                    "map": raw_state(98),
                    "neow": raw_state(0)
                }
            },
            "ambient_mathutils": {
                "seed0": "0123456789abcdef",
                "seed1": "fedcba9876543210"
            },
            "java_collections": {"state": "bb20b4600a74"},
            "the_bomb_id_offset": 0,
            "realized_generation": {
                "card_pools": {
                    "common": [],
                    "uncommon": [],
                    "rare": [],
                    "colorless": [],
                    "source_common": [],
                    "source_uncommon": [],
                    "source_rare": []
                },
                "relic_pools": {
                    "common": [],
                    "uncommon": [],
                    "rare": [],
                    "shop": [],
                    "boss": []
                },
                "event_pools": {
                    "regular": [],
                    "shrines": [],
                    "one_time_shrines": []
                },
                "boss_sequence": [],
                "monster_encounter_queue": [],
                "elite_encounter_queue": [],
                "map": [],
                "neow_options": []
            }
        });
        let mut meta: RecordingMeta = serde_json::from_value(json!({
            "v": 1,
            "run_id": "complete-initial-envelope",
            "seed_long": 7,
            "seed_display": "7",
            "character": "WATCHER",
            "ascension": 0,
            "game_version": "desktop-1.0",
            "status": "IN_PROGRESS",
            "records": 1,
                "environment": {
                "v": 1,
                "settings_is_debug": false,
                "custom_modifiers": [],
                "loaded_mods": ["basemod", "stslib", "tracelab"]
            },
            "profile": {
                "v": 1,
                "note_for_yourself_card": "IronWave",
                "note_for_yourself_upgrades": 0,
                "highest_unlocked_ascension": 20,
                "is_daily_run": false,
                "is_trial": false,
                "final_act_available": true,
                "bosses_seen": [
                    "GUARDIAN", "GHOST", "SLIME", "CHAMP", "AUTOMATON",
                    "COLLECTOR", "CROW", "DONUT", "WIZARD"
                ],
                "locked_cards": [],
                "locked_relics": []
            },
            "initial": initial
        }))
        .unwrap();
        let generated = RunEngine::new_with_profile(
            meta.seed_long as u64,
            meta.ascension,
            meta.profile.as_ref().unwrap().to_engine_profile().unwrap(),
        );
        let generated_witness = generated.recording_initialization_witness();
        let initial = meta.initial.as_mut().unwrap();
        initial["rng"] = generated_witness["rng"].clone();
        initial["realized_generation"] = generated_witness["realized_generation"].clone();
        let settled_map = initial.pointer("/rng/map").unwrap().clone();
        let first = RecordedTraceRecord {
            idx: 0,
            floor: 0,
            act: 1,
            turn: 0,
            phase: "EVENT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": -1}),
            action: json!({"type": "NEOW", "choice": 0}),
            post: json!({"rng": {"map": settled_map}}),
            deck: json!([]),
        };
        (meta, first)
    }

    #[test]
    fn certification_requires_complete_post_generation_causal_state() {
        let (complete, first) = complete_initial_envelope();
        assert_eq!(
            recording_initial_snapshot_issue(&complete, Some(&first)),
            None
        );
        assert_eq!(recording_the_bomb_id_offset(&complete).unwrap(), Some(0));
        assert!(recording_environment_quarantine(&complete).is_none());

        let mut debug_environment = complete.clone();
        debug_environment
            .environment
            .as_mut()
            .unwrap()
            .settings_is_debug = true;
        assert_eq!(
            recording_environment_quarantine(&debug_environment)
                .as_ref()
                .map(|quarantine| quarantine.kind.as_str()),
            Some("debug_gameplay_environment")
        );

        let mut custom_environment = complete.clone();
        custom_environment
            .environment
            .as_mut()
            .unwrap()
            .custom_modifiers
            .push("Draft".to_string());
        assert_eq!(
            recording_environment_quarantine(&custom_environment)
                .as_ref()
                .map(|quarantine| quarantine.kind.as_str()),
            Some("custom_run_modifiers")
        );

        let mut modded_environment = complete.clone();
        modded_environment
            .environment
            .as_mut()
            .unwrap()
            .loaded_mods
            .push("GameplayMod".to_string());
        assert_eq!(
            recording_environment_quarantine(&modded_environment)
                .as_ref()
                .map(|quarantine| quarantine.kind.as_str()),
            Some("unrecognized_gameplay_mods")
        );

        let mut missing_raw_state = complete.clone();
        missing_raw_state
            .initial
            .as_mut()
            .unwrap()
            .pointer_mut("/rng/rawStates/card/seed0")
            .unwrap()
            .take();
        assert!(
            recording_initial_snapshot_issue(&missing_raw_state, Some(&first))
                .is_some_and(|issue| issue.contains("rawStates.card.seed0"))
        );

        let mut mismatched_counter = complete.clone();
        mismatched_counter.initial.as_mut().unwrap()["rng"]["rawStates"]["map"]["counter"] =
            json!(97);
        assert!(
            recording_initial_snapshot_issue(&mismatched_counter, Some(&first))
                .is_some_and(|issue| issue.contains("disagrees with initial.rng.map"))
        );

        let mut altered_native_state = complete.clone();
        altered_native_state.initial.as_mut().unwrap()["rng"]["rawStates"]["card"]["seed0"] =
            json!("0000000000000000");
        assert!(
            recording_initial_snapshot_issue(&altered_native_state, Some(&first)).is_some_and(
                |issue| issue.contains(
                    "initial.rng.rawStates.card disagrees with the freshly generated Rust run"
                )
            )
        );

        let mut altered_relic_pool = complete.clone();
        altered_relic_pool.initial.as_mut().unwrap()["realized_generation"]["relic_pools"]
            ["common"]
            .as_array_mut()
            .unwrap()
            .push(json!("ArbitraryRelic"));
        assert!(
            recording_initial_snapshot_issue(&altered_relic_pool, Some(&first))
                .is_some_and(|issue| issue.contains(
                    "initial.realized_generation.relic_pools disagrees with the freshly generated Rust run"
                ))
        );

        let mut invalid_bomb_offset = complete.clone();
        invalid_bomb_offset.initial.as_mut().unwrap()["the_bomb_id_offset"] =
            json!(i64::from(i32::MAX) + 1);
        assert!(
            recording_initial_snapshot_issue(&invalid_bomb_offset, Some(&first))
                .is_some_and(|issue| issue.contains("32-bit integer"))
        );

        let mut missing_generation_witness = complete;
        missing_generation_witness.initial.as_mut().unwrap()["realized_generation"]
            .as_object_mut()
            .unwrap()
            .remove("relic_pools");
        assert!(
            recording_initial_snapshot_issue(&missing_generation_witness, Some(&first))
                .is_some_and(|issue| issue.contains("realized_generation/relic_pools"))
        );
    }

    #[test]
    fn match_requires_direct_semantics_checkpoints_and_terminal_lifecycle() {
        let (mut meta, first) = complete_initial_envelope();
        meta.status = "VICTORY".to_string();
        meta.profile_authority = Some("recording meta.profile".to_string());
        let action = RecorderAction {
            idx: 0,
            action_type: "NEOW".to_string(),
            raw: first.action.clone(),
        };
        let mut bundle = RecordingBundle {
            path: PathBuf::new(),
            meta,
            actions: vec![action],
            records: vec![first],
            run_end: Some(RecordedRunEnd {
                status: "VICTORY".to_string(),
                floor: 68,
            }),
            lifecycle: RecordedLifecycleSummary {
                saw_run_start: true,
                resume_count: 0,
            },
        };
        let exact = BundleComparisonReport {
            status: BundleComparisonStatus::Match,
            bundle: "strict-contract".to_string(),
            seed: 7,
            ascension: 0,
            outcome: "VICTORY".to_string(),
            initialization_quarantine: None,
            initialization_authority: Some("recording meta.profile".to_string()),
            lifecycle_quarantine: None,
            checkpoint_quarantine: None,
            environment_quarantine: None,
            matched_actions: 1,
            comparable_actions: 1,
            matched_checkpoints: 1,
            coupled_actions: 0,
            unverified_action_semantics: 0,
            unverified_actions: Vec::new(),
            inferred_actions: Vec::new(),
            ignored_recorder_actions: Vec::new(),
            state_reconstructions: Vec::new(),
            total_actions: 1,
            replayed_actions: 1,
            skipped_fields_total: 0,
            skipped_fields_by_path: BTreeMap::new(),
            first_divergence: None,
        };
        assert!(comparison_meets_strict_certification_contract(
            &bundle, &exact
        ));

        let mut resumed_bundle = bundle.clone();
        resumed_bundle.lifecycle.resume_count = 1;
        assert!(!comparison_meets_strict_certification_contract(
            &resumed_bundle,
            &exact
        ));

        let mut coupled = exact.clone();
        coupled.coupled_actions = 1;
        assert!(!comparison_meets_strict_certification_contract(
            &bundle, &coupled
        ));

        let mut inferred = exact.clone();
        inferred.inferred_actions.push(BundleInferredAction {
            before_idx: 1,
            action: GameAction::Proceed,
            reason: "legacy bridge".to_string(),
        });
        assert!(!comparison_meets_strict_certification_contract(
            &bundle, &inferred
        ));

        let mut skipped = exact.clone();
        skipped.skipped_fields_total = 1;
        skipped
            .skipped_fields_by_path
            .insert("player.energy".to_string(), 1);
        assert!(!comparison_meets_strict_certification_contract(
            &bundle, &skipped
        ));

        let mut indirect = exact.clone();
        indirect.matched_checkpoints = 0;
        assert!(!comparison_meets_strict_certification_contract(
            &bundle, &indirect
        ));

        bundle.run_end = None;
        assert!(!comparison_meets_strict_certification_contract(
            &bundle, &exact
        ));
    }

    #[test]
    fn recursively_partial_v1_checkpoint_never_counts_as_complete() {
        let partial = RecordedTraceRecord {
            idx: 0,
            floor: 0,
            act: 1,
            turn: 0,
            phase: "EVENT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": -1}),
            action: json!({"type": "NEOW", "choice": 0}),
            post: json!({
                "player": {},
                "enemies": [],
                "piles": {},
                "relics": [],
                "potions": [],
                "rng": {}
            }),
            deck: json!([]),
        };
        assert_eq!(
            incomplete_v1_checkpoint_path(&partial).as_deref(),
            Some("post.player.hp")
        );
    }

    #[test]
    fn post_generation_initial_envelope_parses_process_global_rng_states() {
        let meta: RecordingMeta = serde_json::from_value(json!({
            "v": 1,
            "run_id": "initial-envelope",
            "seed_long": 7,
            "seed_display": "7",
            "character": "WATCHER",
            "ascension": 0,
            "game_version": "desktop-1.0",
            "status": "IN_PROGRESS",
            "records": 0,
            "initial": {
                "v": 1,
                "captured_at": "first_run_frame_post_generation",
                "rng": {"map": 98},
                "ambient_mathutils": {
                    "seed0": "-1",
                    "seed1": "3251836536433006742"
                },
                "java_collections": {"state": "bb20b4600a74"}
            }
        }))
        .unwrap();
        let first = RecordedTraceRecord {
            idx: 0,
            floor: 0,
            act: 1,
            turn: 0,
            phase: "EVENT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": -1}),
            action: json!({"type": "NEOW", "choice": 0}),
            post: json!({"rng": {"map": 98}}),
            deck: json!([]),
        };

        assert_eq!(
            recording_ambient_math_state(&meta, Some(&first)).unwrap(),
            Some((u64::MAX, 3_251_836_536_433_006_742))
        );
        assert_eq!(
            recording_java_collections_state(&meta).unwrap(),
            Some(0xBB20_B460_0A74)
        );

        let mut premature = meta;
        premature.initial.as_mut().unwrap()["rng"]["map"] = json!(0);
        assert!(recording_initial_snapshot_issue(&premature, Some(&first)).is_some());
        assert_eq!(
            recording_ambient_math_state(&premature, Some(&first)).unwrap(),
            None
        );
    }

    #[test]
    fn path_combat_entry_defers_battle_start_relic_counter_comparison() {
        let record = RecordedTraceRecord {
            idx: 110,
            floor: 18,
            act: 2,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 5, "y": 0}),
            action: json!({"type": "PATH", "x": 5, "y": 0}),
            post: json!({
                "player": {"hp": 60, "energy": 0},
                "enemies": [{"id": "JawWorm", "hp": 40, "max_hp": 40}],
                "piles": {},
                "relics": [
                    {"id": "Pocketwatch", "counter": -1},
                    {"id": "Ancient Tea Set", "counter": -2},
                    {"id": "Akabeko", "counter": -1},
                    {"id": "Incense Burner", "counter": 0},
                    {"id": "Ornamental Fan", "counter": -1},
                    {"id": "Kunai", "counter": -1}
                ],
                "potions": [],
                "rng": {}
            }),
            deck: json!([]),
        };

        let projected = record.partial_oracle_state(true).unwrap();
        assert_eq!(projected.pointer("/relics/0/counter"), None);
        assert_eq!(projected.pointer("/relics/1/counter"), None);
        assert_eq!(projected.pointer("/relics/2/counter"), Some(&json!(-1)));
        assert_eq!(projected.pointer("/relics/3/counter"), None);
        assert_eq!(projected.pointer("/relics/4/counter"), None);
        assert_eq!(projected.pointer("/relics/5/counter"), None);
        assert_eq!(projected.pointer("/enemies/0/hp"), None);
        assert_eq!(projected.pointer("/enemies/0/max_hp"), Some(&json!(40)));
    }

    #[test]
    fn noncombat_turn_is_not_a_causal_oracle_field() {
        let record = RecordedTraceRecord {
            idx: 0,
            floor: 0,
            act: 1,
            turn: 0,
            phase: "EVENT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": -1}),
            action: json!({"type": "NEOW", "choice": 0}),
            post: json!({
                "player": {}, "enemies": [], "piles": {},
                "relics": [], "potions": [], "rng": {}
            }),
            deck: json!([]),
        };

        assert!(record
            .partial_oracle_state(true)
            .unwrap()
            .get("turn")
            .is_none());
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
            initial: None,
            environment: None,
            profile_authority: None,
            profile_quarantine: None,
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
                "player": {
                    "hp": 42,
                    "block": 10,
                    "energy": 0,
                    "stance": "Neutral",
                    "powers": [{"id": "Strength", "amt": 3}]
                },
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
        assert!(partial.pointer("/player/block").is_none());
        assert!(partial.pointer("/player/hp").is_none());
        assert!(partial.pointer("/player/stance").is_none());
        assert!(partial.pointer("/player/powers").is_none());
        assert_eq!(partial.get("phase"), Some(&json!("COMBAT")));
    }

    #[test]
    fn dead_monster_powers_are_not_a_causal_legacy_oracle_field() {
        // AbstractMonster.updateDeathAnimation clears powers only after a
        // frame-time deathTimer expires, so two action checkpoints can expose
        // different lists for the same dead monster. Java:
        // AbstractMonster.java:860-874.
        let record = RecordedTraceRecord {
            idx: 0,
            floor: 1,
            act: 1,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": 0}),
            action: json!({"type": "END_TURN"}),
            post: json!({
                "player": {},
                "enemies": [{
                    "dead": true,
                    "intent": {"name": "DEBUFF", "dmg": -1, "move_id": 3, "hits": 1},
                    "powers": [{"id": "Artifact", "amt": 1}]
                }],
                "piles": {},
                "relics": [],
                "potions": [],
                "rng": {}
            }),
            deck: json!([]),
        };

        let partial = record.partial_oracle_state(true).unwrap();
        assert!(partial.pointer("/enemies/0/powers").is_none());
        assert!(partial.pointer("/enemies/0/intent").is_none());
    }

    #[test]
    fn non_attack_intent_ignores_java_stale_attack_damage_and_hits() {
        // Chosen's two-hit Poke followed by Hex leaves isMultiDmg and
        // intentMultiAmt stale because the Hex EnemyMoveInfo has baseDamage
        // -1. The strong-debuff identity and move ID remain causal.
        // Java: Chosen.java::getMove and AbstractMonster.java::createIntent.
        let record = RecordedTraceRecord {
            idx: 0,
            floor: 17,
            act: 2,
            turn: 2,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 6, "y": 0}),
            action: json!({"type": "END_TURN"}),
            post: json!({
                "player": {},
                "enemies": [{
                    "dead": false,
                    "intent": {"name": "STRONG_DEBUFF", "dmg": 5, "move_id": 4, "hits": 2},
                    "powers": []
                }],
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
        assert_eq!(
            partial.pointer("/enemies/0/intent/name"),
            Some(&json!("STRONG_DEBUFF"))
        );
        assert_eq!(
            partial.pointer("/enemies/0/intent/move_id"),
            Some(&json!(4))
        );
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
    fn linked_chest_relic_callback_after_sapphire_key_is_recorder_only() {
        // RewardItem.claimReward marks the linked reward isDone/ignoreReward
        // when either the Sapphire Key or relic is claimed. The legacy hook can
        // still emit the disabled relic callback, but it cannot mutate Java.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/RewardItem.java
        let previous = RecordedTraceRecord {
            idx: 10,
            floor: 8,
            act: 1,
            turn: 1,
            phase: "COMPLETE".to_string(),
            screen: "COMBAT_REWARD".to_string(),
            map: json!({"x": 3, "y": 7}),
            action: json!({"type": "REWARD_TAKE", "reward_type": "SAPPHIRE_KEY", "id": ""}),
            post: json!({"player": {"hp": 70}, "relics": []}),
            deck: json!(["Strike"]),
        };
        let current = RecordedTraceRecord {
            idx: 11,
            action: json!({"type": "REWARD_TAKE", "reward_type": "RELIC", "id": "InkBottle"}),
            ..previous.clone()
        };
        let action = RecorderAction {
            idx: 11,
            action_type: "REWARD_TAKE".to_string(),
            raw: current.action.clone(),
        };

        assert!(is_disabled_chest_relic_click(
            &action,
            Some(&previous),
            &current,
        ));
    }

    #[test]
    fn calling_bell_relic_id_resolves_the_omitted_item_index() {
        let mut engine = RunEngine::new(7, 0);
        engine.debug_set_reward_screen(crate::decision::RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: ["Omamori", "Yang", "Calipers"]
                .into_iter()
                .enumerate()
                .map(|(index, relic)| crate::decision::RewardItem {
                    index,
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: relic.to_string(),
                    claimable: true,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                })
                .collect(),
        });

        let (mapped, verified) = map_reward_take(
            &engine,
            &json!({"reward_type": "RELIC", "id": "Yang"}),
            None,
        )
        .unwrap();
        assert_eq!(mapped, GameAction::SelectRewardItem(1));
        assert!(verified);
    }

    #[test]
    fn calling_bell_first_relic_infers_the_mandatory_curse_confirmation() {
        let mut engine = RunEngine::new(7, 0);
        engine.debug_set_reward_screen(crate::decision::RewardScreen {
            source: RewardScreenSource::BossCombat,
            ordered: true,
            active_item: None,
            items: vec![
                crate::decision::RewardItem {
                    index: 0,
                    kind: RewardItemKind::CardChoice,
                    state: RewardItemState::Available,
                    label: "calling_bell_curse".to_string(),
                    claimable: true,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: vec![RewardChoice::Card {
                        index: 0,
                        card_id: "CurseOfTheBell".to_string(),
                    }],
                },
                crate::decision::RewardItem {
                    index: 1,
                    kind: RewardItemKind::Relic,
                    state: RewardItemState::Available,
                    label: "Omamori".to_string(),
                    claimable: false,
                    active: false,
                    skip_allowed: false,
                    skip_label: None,
                    choices: Vec::new(),
                },
            ],
        });
        let action = RecorderAction {
            idx: 159,
            action_type: "REWARD_TAKE".to_string(),
            raw: json!({"reward_type": "RELIC", "id": "Omamori"}),
        };
        let record = RecordedTraceRecord {
            idx: 159,
            floor: 16,
            act: 1,
            turn: 0,
            phase: "COMPLETE".to_string(),
            screen: "COMBAT_REWARD".to_string(),
            map: json!({}),
            action: action.raw.clone(),
            post: json!({}),
            deck: json!([]),
        };

        let inferred = infer_missing_prerequisites(&mut engine, &action, &record).unwrap();
        assert_eq!(inferred.actions.len(), 2);
        assert!(engine
            .run_state
            .deck
            .iter()
            .any(|card| card == "CurseOfTheBell"));
        assert!(engine
            .get_legal_actions()
            .contains(&GameAction::SelectRewardItem(1)));
    }

    #[test]
    fn repeated_shop_remove_with_identical_state_marks_only_the_grid_opener() {
        let actions = [
            RecorderAction {
                idx: 20,
                action_type: "SHOP_REMOVE".to_string(),
                raw: json!({"idx": 20, "type": "SHOP_REMOVE"}),
            },
            RecorderAction {
                idx: 21,
                action_type: "SHOP_REMOVE".to_string(),
                raw: json!({"idx": 21, "type": "SHOP_REMOVE"}),
            },
        ];
        let first = RecordedTraceRecord {
            idx: 20,
            floor: 10,
            act: 1,
            turn: 1,
            phase: "COMPLETE".to_string(),
            screen: "GRID".to_string(),
            map: json!({"x": 5, "y": 9}),
            action: actions[0].raw.clone(),
            post: json!({"player": {"gold": 231}, "relics": []}),
            deck: json!(["Strike", "Defend"]),
        };
        let second = RecordedTraceRecord {
            idx: 21,
            action: actions[1].raw.clone(),
            ..first.clone()
        };

        assert!(is_shop_remove_grid_opener(&actions, &[first, second], 0,));
    }

    #[test]
    fn necronomicon_reward_callback_is_coupled_to_deferred_curse_exit() {
        let actions = [
            RecorderAction {
                idx: 132,
                action_type: "REWARD_TAKE".to_string(),
                raw: json!({"type": "REWARD_TAKE", "reward_type": "RELIC", "id": "Necronomicon"}),
            },
            RecorderAction {
                idx: 133,
                action_type: "EVENT_CHOICE".to_string(),
                raw: json!({"type": "EVENT_CHOICE", "choice": 0, "event": "CursedTome"}),
            },
        ];
        let reward = RecordedTraceRecord {
            idx: 132,
            floor: 20,
            act: 2,
            turn: 1,
            phase: "EVENT".to_string(),
            screen: "COMBAT_REWARD".to_string(),
            map: json!({"x": 4, "y": 2}),
            action: actions[0].raw.clone(),
            post: json!({}),
            deck: json!(["Strike_P"]),
        };
        let exit = RecordedTraceRecord {
            idx: 133,
            screen: "MAP".to_string(),
            action: actions[1].raw.clone(),
            deck: json!(["Strike_P", "Necronomicurse"]),
            ..reward.clone()
        };

        assert!(is_necronomicon_reward_followed_by_event_exit(
            &actions,
            &[reward, exit],
            0,
        ));
    }

    #[test]
    fn campfire_effect_callback_is_coupled_to_following_path() {
        let actions = [
            RecorderAction {
                idx: 179,
                action_type: "CAMPFIRE".to_string(),
                raw: json!({"type": "CAMPFIRE", "choice": "REST"}),
            },
            RecorderAction {
                idx: 180,
                action_type: "PATH".to_string(),
                raw: json!({"type": "PATH", "x": 2, "y": 7}),
            },
        ];
        assert!(is_campfire_effect_followed_by_path(&actions, 0));

        let not_settled = [
            actions[0].clone(),
            RecorderAction {
                idx: 180,
                action_type: "CAMPFIRE".to_string(),
                raw: json!({"type": "CAMPFIRE", "choice": "REST"}),
            },
        ];
        assert!(!is_campfire_effect_followed_by_path(&not_settled, 0));
    }

    #[test]
    fn ghosts_card_effects_are_coupled_to_the_explicit_exit_page() {
        let actions = [
            RecorderAction {
                idx: 181,
                action_type: "EVENT_CHOICE".to_string(),
                raw: json!({"type": "EVENT_CHOICE", "choice": 0, "event": "Ghosts"}),
            },
            RecorderAction {
                idx: 182,
                action_type: "EVENT_CHOICE".to_string(),
                raw: json!({"type": "EVENT_CHOICE", "choice": 0, "event": "Ghosts"}),
            },
        ];
        let first = RecordedTraceRecord {
            idx: 181,
            floor: 22,
            act: 2,
            turn: 0,
            phase: "EVENT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 2, "y": 7}),
            action: actions[0].raw.clone(),
            post: json!({}),
            deck: json!(["Strike_P"]),
        };
        let exit = RecordedTraceRecord {
            idx: 182,
            screen: "MAP".to_string(),
            action: actions[1].raw.clone(),
            deck: json!(["Strike_P", "Apparition"]),
            ..first.clone()
        };
        assert!(is_deferred_event_effect_followed_by_exit(
            &actions,
            &[first, exit],
            0,
        ));
    }

    #[test]
    fn shop_remove_infers_omitted_grid_selection_from_following_ordered_deck() {
        // ShopScreen.purchasePurge only opens GridCardSelectScreen. The later
        // updatePurge observes selectedCards, charges gold, and removes the
        // card; the legacy hook does not record that selected-card identity.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java.
        let mut engine = RunEngine::new(7, 0);
        engine.run_state.gold = 999;
        engine.debug_set_shop_state(crate::run::ShopState {
            cards: Vec::new(),
            relics: Vec::new(),
            potions: Vec::new(),
            remove_price: 75,
            removal_used: false,
        });
        let removed_index = 4;
        let mut expected_deck = engine.run_state.deck.clone();
        expected_deck.remove(removed_index);
        let action = RecorderAction {
            idx: 57,
            action_type: "SHOP_REMOVE".to_string(),
            raw: json!({"idx": 57, "type": "SHOP_REMOVE"}),
        };
        let current = RecordedTraceRecord {
            idx: 57,
            floor: 10,
            act: 1,
            turn: 0,
            phase: "COMPLETE".to_string(),
            screen: "GRID".to_string(),
            map: json!({"x": 4, "y": 9}),
            action: action.raw.clone(),
            post: json!({"player": {"gold": 999}}),
            deck: json!(engine.run_state.deck),
        };
        let following = RecordedTraceRecord {
            idx: 58,
            screen: "NONE".to_string(),
            post: json!({"player": {"gold": 924}}),
            deck: json!(expected_deck),
            action: json!({"type": "PATH", "x": 4, "y": 10}),
            ..current.clone()
        };

        let mut adapter = RecorderActionAdapter::default();
        let mapped = adapter
            .map(&engine, &action, &current, Some(&following))
            .expect("following deck should identify the omitted selection");
        assert_eq!(mapped, Some(GameAction::ShopRemoveCard(removed_index)));
        assert!(adapter
            .take_last_mapping_unverified()
            .expect("omitted recorder payload must remain unverified")
            .contains("omits the selected-card callback"));
        assert!(adapter
            .take_last_mapping_inferred_reason()
            .expect("omitted selection must be serialized as inferred")
            .contains("following ordered deck"));
        assert!(adapter.take_last_mapping_coupled_to_following());
    }

    #[test]
    fn shop_purchase_checkpoint_can_prove_selection_after_purge_opener() {
        // A purge selection and a subsequent shop purchase can settle before
        // TraceWriter samples the purchase callback. The combined exact state
        // must uniquely prove the omitted removal before the purchase maps.
        let mut engine = RunEngine::new(19, 0);
        engine.run_state.gold = 999;
        engine.debug_set_shop_state(crate::run::ShopState {
            cards: vec![("Madness".to_string(), 50)],
            relics: Vec::new(),
            potions: Vec::new(),
            remove_price: 75,
            removal_used: false,
        });
        let removed_index = engine
            .run_state
            .deck
            .iter()
            .position(|card| card == "Eruption")
            .expect("Watcher starter deck has a unique Eruption");

        let mut settled = engine.clone();
        assert_eq!(
            settled
                .step_game(&GameAction::ShopRemoveCard(removed_index))
                .status,
            ActionStatus::Accepted
        );
        assert_eq!(
            settled.step_game(&GameAction::ShopBuyCard(0)).status,
            ActionStatus::Accepted
        );
        let projected = serde_json::to_value(project_oracle_state(&settled).unwrap()).unwrap();
        let raw = json!({
            "idx": 81,
            "type": "SHOP_BUY_CARD",
            "card_id": "Madness",
            "upgrades": 0
        });
        let action = RecorderAction {
            idx: 81,
            action_type: "SHOP_BUY_CARD".to_string(),
            raw: raw.clone(),
        };
        let record = RecordedTraceRecord {
            idx: 81,
            floor: projected["floor"].as_i64().unwrap() as i32,
            act: projected["act"].as_i64().unwrap() as i32,
            turn: projected["turn"].as_i64().unwrap() as i32,
            phase: "COMPLETE".to_string(),
            screen: "SHOP".to_string(),
            map: projected["map"].clone(),
            action: raw,
            post: json!({
                "player": projected["player"].clone(),
                "enemies": projected["enemies"].clone(),
                "piles": projected["piles"].clone(),
                "relics": projected["relics"].clone(),
                "potions": projected["potions"].clone(),
                "rng": projected["rng"].clone(),
            }),
            deck: projected["deck"].clone(),
        };

        let inferred = infer_missing_prerequisites(&mut engine, &action, &record).unwrap();
        assert_eq!(
            inferred
                .actions
                .iter()
                .map(|entry| entry.action.clone())
                .collect::<Vec<_>>(),
            [GameAction::ShopRemoveCard(removed_index)]
        );
        assert!(inferred.unverified_reason.is_some());
        assert!(!engine.run_state.deck.iter().any(|card| card == "Eruption"));
    }

    #[test]
    fn time_warp_end_turn_callback_is_not_a_second_player_end_turn() {
        // TimeWarpPower.onAfterUseCard calls callEndTurnEarlySequence at 12.
        // If the card checkpoint already moved to the next turn, an identical
        // END_TURN checkpoint is the recorder observing that internal call.
        let actions = [
            RecorderAction {
                idx: 30,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 30, "type": "PLAY_CARD", "card_id": "Eruption"}),
            },
            RecorderAction {
                idx: 31,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 31, "type": "PLAY_CARD", "card_id": "Strike"}),
            },
            RecorderAction {
                idx: 32,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 32, "type": "END_TURN"}),
            },
        ];
        let before = RecordedTraceRecord {
            idx: 30,
            floor: 16,
            act: 1,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 3, "y": 15}),
            action: actions[0].raw.clone(),
            post: json!({
                "player": {"hp": 70},
                "enemies": [{
                    "id": "TimeEater",
                    "powers": [{"id": "Time Warp", "amt": 0}]
                }]
            }),
            deck: json!(["Strike"]),
        };
        let after_card = RecordedTraceRecord {
            idx: 31,
            turn: 2,
            action: actions[1].raw.clone(),
            ..before.clone()
        };
        let callback = RecordedTraceRecord {
            idx: 32,
            action: actions[2].raw.clone(),
            ..after_card.clone()
        };

        assert!(is_early_sequence_end_turn_callback(
            &actions,
            &[before, after_card, callback],
            2,
        ));
    }

    #[test]
    fn awakened_one_with_live_cultists_is_a_victory_pending_callback() {
        // Awakened One's final die() queues EscapeAction for Cultists, which
        // lack MinionPower. The recorder's lethal useCard frame therefore has
        // one dead boss plus live Cultists even though the room is already
        // committed to victory.
        // Java: reference/extracted/methods/monster/AwakenedOne.java::die.
        let record = RecordedTraceRecord {
            idx: 419,
            floor: 50,
            act: 3,
            turn: 6,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": -1, "y": 15}),
            action: json!({"type": "PLAY_CARD", "card_id": "Tantrum", "target": 2}),
            post: json!({
                "enemies": [
                    {"id": "Cultist", "dead": false, "powers": []},
                    {"id": "Cultist", "dead": false, "powers": []},
                    {"id": "AwakenedOne", "dead": true, "powers": []}
                ]
            }),
            deck: json!([]),
        };

        assert!(recorded_combat_victory_pending(&record));
    }

    #[test]
    fn press_end_turn_card_callback_is_not_a_second_player_end_turn() {
        // Meditate queues PressEndTurnButtonAction, which calls the same
        // GameActionManager early-end sequence as Time Warp. The recorder sees
        // that internal room callback after the card's settled checkpoint.
        // Java: Meditate.java and PressEndTurnButtonAction.java.
        let actions = [
            RecorderAction {
                idx: 214,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 214, "type": "PLAY_CARD", "card_id": "Miracle"}),
            },
            RecorderAction {
                idx: 215,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 215, "type": "PLAY_CARD", "card_id": "Meditate"}),
            },
            RecorderAction {
                idx: 216,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 216, "type": "END_TURN"}),
            },
        ];
        let turn_one = RecordedTraceRecord {
            idx: 214,
            floor: 20,
            act: 2,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 4, "y": 5}),
            action: actions[0].raw.clone(),
            post: json!({"player": {"hp": 69}}),
            deck: json!(["Meditate"]),
        };
        let settled_turn_two = RecordedTraceRecord {
            idx: 215,
            turn: 2,
            action: actions[1].raw.clone(),
            ..turn_one.clone()
        };
        let callback = RecordedTraceRecord {
            idx: 216,
            action: actions[2].raw.clone(),
            ..settled_turn_two.clone()
        };

        assert!(is_early_sequence_end_turn_callback(
            &actions,
            &[turn_one.clone(), settled_turn_two.clone(), callback.clone()],
            2,
        ));

        let mut ordinary_card_actions = actions;
        ordinary_card_actions[1].raw["card_id"] = json!("Strike_P");
        assert!(
            !is_early_sequence_end_turn_callback(
                &ordinary_card_actions,
                &[turn_one, settled_turn_two, callback],
                2,
            ),
            "an identical future checkpoint after an ordinary card is coupling, not an internal end-turn callback"
        );
    }

    #[test]
    fn terminal_press_end_turn_callback_does_not_require_a_visible_turn_increment() {
        // Meditate queues PressEndTurnButtonAction. If that internal end-turn
        // resolves Transient's final Fading turn, AbstractRoom commits combat
        // victory before the recorder ever observes a larger turn number. The
        // repeated terminal frame is still the internal callback, not a new
        // player END_TURN.
        // Java: cards/purple/Meditate.java;
        // actions/common/PressEndTurnButtonAction.java;
        // monsters/beyond/Transient.java and powers/FadingPower.java.
        let actions = [
            RecorderAction {
                idx: 628,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 628, "type": "PLAY_CARD", "card_id": "FearNoEvil"}),
            },
            RecorderAction {
                idx: 629,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 629, "type": "PLAY_CARD", "card_id": "Meditate"}),
            },
            RecorderAction {
                idx: 630,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 630, "type": "END_TURN"}),
            },
        ];
        let before = RecordedTraceRecord {
            idx: 628,
            floor: 44,
            act: 3,
            turn: 5,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 5, "y": 9}),
            action: actions[0].raw.clone(),
            post: json!({
                "player": {"hp": 28},
                "enemies": [{"id": "Transient", "dead": false, "powers": []}]
            }),
            deck: json!(["Meditate+"]),
        };
        let terminal_card = RecordedTraceRecord {
            idx: 629,
            action: actions[1].raw.clone(),
            post: json!({
                "player": {"hp": 28},
                "enemies": [{"id": "Transient", "dead": true, "powers": []}]
            }),
            ..before.clone()
        };
        let callback = RecordedTraceRecord {
            idx: 630,
            action: actions[2].raw.clone(),
            ..terminal_card.clone()
        };

        assert_eq!(before.turn, terminal_card.turn);
        assert!(recorded_combat_victory_pending(&terminal_card));
        assert!(is_early_sequence_end_turn_callback(
            &actions,
            &[before.clone(), terminal_card.clone(), callback.clone()],
            2,
        ));

        // Recorder.java drains callbacks FIFO after a stable-frame delay. A
        // still-live fight can therefore repeat a frame whose turn number was
        // already attached to an earlier queued card callback as well.
        let mut live_card = terminal_card.clone();
        live_card.post["enemies"][0]["dead"] = json!(false);
        let live_callback = RecordedTraceRecord {
            post: live_card.post.clone(),
            ..callback
        };
        assert!(!recorded_combat_victory_pending(&live_card));
        assert!(is_early_sequence_end_turn_callback(
            &actions,
            &[before, live_card, live_callback],
            2,
        ));
    }

    #[test]
    fn press_end_turn_callback_skips_intervening_auto_played_burn_callbacks() {
        // Meditate queues PressEndTurnButtonAction. During that same early-end
        // sequence, callEndOfTurnActions queues each Burn for automatic use;
        // Recorder drains those useCard callbacks before its END_TURN callback.
        // All callbacks can therefore expose the same already-settled turn.
        // Java: cards/purple/Meditate.java, cards/status/Burn.java,
        // actions/GameActionManager.java::callEndOfTurnActions, and
        // actions/common/PressEndTurnButtonAction.java.
        let actions = [
            RecorderAction {
                idx: 859,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 859, "type": "PLAY_CARD", "card_id": "Miracle"}),
            },
            RecorderAction {
                idx: 860,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 860, "type": "PLAY_CARD", "card_id": "Meditate"}),
            },
            RecorderAction {
                idx: 861,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 861, "type": "PLAY_CARD", "card_id": "Burn", "hand_idx": 2}),
            },
            RecorderAction {
                idx: 862,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"idx": 862, "type": "PLAY_CARD", "card_id": "Burn", "hand_idx": 3}),
            },
            RecorderAction {
                idx: 863,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 863, "type": "END_TURN"}),
            },
        ];
        let before = RecordedTraceRecord {
            idx: 859,
            floor: 55,
            act: 4,
            turn: 3,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": -1, "y": 15}),
            action: actions[0].raw.clone(),
            post: json!({"player": {"hp": 68}, "enemies": [{"id": "CorruptHeart"}]}),
            deck: json!(["Meditate+"]),
        };
        let settled = RecordedTraceRecord {
            idx: 860,
            turn: 4,
            action: actions[1].raw.clone(),
            ..before.clone()
        };
        let records = [
            before,
            settled.clone(),
            RecordedTraceRecord {
                idx: 861,
                action: actions[2].raw.clone(),
                ..settled.clone()
            },
            RecordedTraceRecord {
                idx: 862,
                action: actions[3].raw.clone(),
                ..settled.clone()
            },
            RecordedTraceRecord {
                idx: 863,
                action: actions[4].raw.clone(),
                ..settled
            },
        ];

        assert!(is_auto_played_burn_recorder_callback(&actions, &records, 2));
        assert!(is_auto_played_burn_recorder_callback(&actions, &records, 3));
        assert!(is_early_sequence_end_turn_callback(&actions, &records, 4));

        let mut changed = records;
        changed[4].post["player"]["hp"] = json!(67);
        assert!(
            !is_early_sequence_end_turn_callback(&actions, &changed, 4),
            "a distinct later player state must not be swallowed"
        );
    }

    #[test]
    fn auto_played_burn_callback_is_not_a_player_card_action() {
        // Burn.triggerOnEndOfTurnForPlayingCard queues the status card with
        // dontTriggerOnUseCard=true before AbstractRoom.endTurn is observed.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java.
        let actions = [
            RecorderAction {
                idx: 40,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({
                    "idx": 40,
                    "type": "PLAY_CARD",
                    "card_id": "Burn",
                    "hand_idx": 1
                }),
            },
            RecorderAction {
                idx: 41,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({
                    "idx": 41,
                    "type": "PLAY_CARD",
                    "card_id": "Burn",
                    "hand_idx": 2
                }),
            },
            RecorderAction {
                idx: 42,
                action_type: "END_TURN".to_string(),
                raw: json!({"idx": 42, "type": "END_TURN"}),
            },
        ];
        let burn = RecordedTraceRecord {
            idx: 40,
            floor: 55,
            act: 4,
            turn: 3,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": -1, "y": 15}),
            action: actions[0].raw.clone(),
            post: json!({"player": {"hp": 68}, "piles": {"hand": ["Burn"]}}),
            deck: json!(["Eruption+"]),
        };
        let second_burn = RecordedTraceRecord {
            idx: 41,
            action: actions[1].raw.clone(),
            ..burn.clone()
        };
        let end_turn = RecordedTraceRecord {
            idx: 42,
            action: actions[2].raw.clone(),
            ..burn.clone()
        };
        let records = [burn, second_burn, end_turn];

        assert!(is_auto_played_burn_recorder_callback(&actions, &records, 0));
        assert!(is_auto_played_burn_recorder_callback(&actions, &records, 1));
    }

    #[test]
    fn necronomicon_internal_play_is_not_a_second_agent_card_action() {
        let actions = [
            RecorderAction {
                idx: 136,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"type": "PLAY_CARD", "card_id": "ReachHeaven", "upgrades": 1, "hand_idx": 3, "target": 0, "energy_on_use": 2}),
            },
            RecorderAction {
                idx: 137,
                action_type: "PLAY_CARD".to_string(),
                raw: json!({"type": "PLAY_CARD", "card_id": "ReachHeaven", "upgrades": 1, "hand_idx": -1, "target": 0, "energy_on_use": 2}),
            },
        ];
        let settled = RecordedTraceRecord {
            idx: 136,
            floor: 21,
            act: 2,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 3, "y": 3}),
            action: actions[0].raw.clone(),
            post: json!({"relics": [{"id": "Necronomicon", "counter": -1}]}),
            deck: json!(["ReachHeaven+"]),
        };
        let internal = RecordedTraceRecord {
            idx: 137,
            action: actions[1].raw.clone(),
            ..settled.clone()
        };

        assert!(is_necronomicon_recorder_only_callback(
            &actions,
            &[settled, internal],
            1,
        ));
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
