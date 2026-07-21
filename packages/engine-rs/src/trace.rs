//! Trace schema (v1) — the single source of truth for the parity-oracle
//! JSONL wire format described in `docs/goal/TOOLING.md` § T1.
//!
//! A trace file is JSONL: record 0 is a [`TraceHeader`], then one
//! [`TraceRecord`] after every executed action. Both are tagged with a
//! `v` (schema version) field; [`check_version`] rejects anything but `1`
//! with a clear error so a future schema bump can't be silently misread.
//!
//! This module also defines the action-script file format (T2, the
//! canonical action vocabulary consumed by `trace_replay`) and the
//! divergence-report structures produced by the differ (T3).
//!
//! ## RNG stream keys
//!
//! Slay the Spire has **13 independent RNG streams**
//! (`docs/vault/rng-system-analysis.md`, corroborated by
//! `docs/vault/rng-parity-audit.md` § 5). [`PostState::rng`] is a
//! `BTreeMap<String, i64>` — rather than a fixed struct — so the schema
//! tolerates streams being added/renamed without a version bump, and so
//! JSON key order is deterministic (`BTreeMap` sorts, which also gives
//! stable diffing/golden-fixture output). The canonical key set (vault
//! names win over any example elsewhere, including this file's docstring):
//!
//! Persistent (run-wide, counter-based):
//! - `card` (`cardRng`), `monster` (`monsterRng`), `event` (`eventRng`),
//!   `relic` (`relicRng`), `treasure` (`treasureRng`), `potion`
//!   (`potionRng`), `merchant` (`merchantRng`)
//!
//! Per-floor (reseeded `seed + floorNum` each floor):
//! - `monsterHp` (`monsterHpRng`), `ai` (`aiRng`), `shuffle` (`shuffleRng`),
//!   `cardRandom` (`cardRandomRng`), `misc` (`miscRng`)
//!
//! Special:
//! - `map` (`mapRng`, reseeded per-act)
//!
//! (`NeowEvent.rng` is documented in the vault as a separate, non-counted
//! stream and is intentionally not one of the 13 keys above.)
//!
//! Not every key is necessarily present on every record — a producer only
//! emits counters for streams it actually tracks distinctly.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Additive trace-v2 schema types. The existing definitions in this file stay
/// frozen as the read-only v1 contract.
pub mod v2;
pub mod oracle_v2;
pub mod bundle;

/// Current trace schema version. Every header/record carries `v` so a
/// consumer can immediately detect drift instead of silently misparsing.
pub const TRACE_SCHEMA_VERSION: u32 = 1;

/// Reject anything other than [`TRACE_SCHEMA_VERSION`] with a clear message.
///
/// Called explicitly by readers (not baked into `Deserialize`) so that a
/// malformed/future version produces a readable error rather than a serde
/// type-mismatch.
pub fn check_version(v: u32) -> Result<(), String> {
    if v != TRACE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported trace schema version {v} (expected {TRACE_SCHEMA_VERSION})"
        ));
    }
    Ok(())
}

// ===========================================================================
// T1 — Trace header + record
// ===========================================================================

/// Record 0 of every trace JSONL file.
///
/// Java ref for the fields this mirrors: `com.megacrit.cardcrawl.core.Settings`
/// (seed/character/ascension) and `com.megacrit.cardcrawl.core.CardCrawlGame`
/// (game_version). See `docs/goal/TOOLING.md` § T1/T2 for the wire example.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceHeader {
    /// Schema version. Must equal [`TRACE_SCHEMA_VERSION`]; validate with
    /// [`check_version`].
    pub v: u32,
    /// Always `"header"`. Lets a reader tell header apart from record 0
    /// on line-oriented JSONL without a separate framing byte.
    pub kind: String,
    /// Display-form seed string (base-34, `SeedHelper` encoding).
    pub seed: String,
    /// The same seed as a signed 64-bit integer (`SeedHelper.getLong`).
    pub seed_long: i64,
    /// Playable character, e.g. `"WATCHER"`.
    pub character: String,
    pub ascension: i32,
    /// Game build string, e.g. `"desktop-1.0"`.
    pub game_version: String,
    /// Mod ids active during capture, e.g. `["basemod","stslib","tracelab"]`.
    pub mods: Vec<String>,
}

impl TraceHeader {
    pub fn check_version(&self) -> Result<(), String> {
        check_version(self.v)
    }
}

/// One record emitted after every executed action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceRecord {
    pub v: u32,
    /// Monotonic 0-based index of this record within the trace (does not
    /// reset per floor/turn).
    pub idx: u64,
    pub floor: i32,
    pub turn: i32,
    /// Coarse phase label at the time of this record, e.g. `"COMBAT"`,
    /// `"MAP"`, `"SHOP"`, `"EVENT"`, `"CAMPFIRE"`, `"NEOW"`, `"GAME_OVER"`.
    pub phase: String,
    /// The action that produced this record.
    pub action: TraceAction,
    /// Full state snapshot *after* `action` executed.
    pub post: PostState,
}

impl TraceRecord {
    pub fn check_version(&self) -> Result<(), String> {
        check_version(self.v)
    }
}

// ===========================================================================
// T2 — Action vocabulary (tagged enum, matches script file + GameAction 1:1)
// ===========================================================================

/// The canonical action vocabulary shared by action scripts (T2) and trace
/// records. Tagged on `type` (matching the script JSON in
/// `docs/goal/TOOLING.md` § T2) so new variants can be added without
/// breaking old traces (`#[serde(other)]`-free deliberately: an unknown
/// `type` should be a hard parse error, not silently swallowed, since the
/// oracle must never guess semantics for an action it doesn't recognize).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceAction {
    /// Play a card from hand. `target` is an enemy index, or absent/`-1`
    /// for self/no-target cards.
    #[serde(rename = "PLAY_CARD")]
    PlayCard {
        hand_idx: usize,
        #[serde(default = "default_no_target", skip_serializing_if = "is_no_target")]
        target: i32,
        /// Optional card id for human-readable scripts/debugging; not
        /// required to resolve the action (hand_idx is authoritative).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        card_id: Option<String>,
    },
    #[serde(rename = "END_TURN")]
    EndTurn,
    #[serde(rename = "USE_POTION")]
    UsePotion {
        idx: usize,
        #[serde(default = "default_no_target", skip_serializing_if = "is_no_target")]
        target: i32,
    },
    #[serde(rename = "NEOW")]
    Neow { choice: usize },
    #[serde(rename = "PATH")]
    Path { choice: usize },
    #[serde(rename = "REWARD_TAKE")]
    RewardTake { item: usize },
    #[serde(rename = "REWARD_SKIP")]
    RewardSkip {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item: Option<usize>,
    },
    #[serde(rename = "EVENT_CHOICE")]
    EventChoice { choice: usize },
    #[serde(rename = "CAMPFIRE")]
    Campfire {
        /// `"REST"` or `"SMITH"` (upgrade); when `SMITH`, `card_idx` selects
        /// the deck index to upgrade.
        choice: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        card_idx: Option<usize>,
    },
    #[serde(rename = "SHOP_BUY")]
    ShopBuy { item: usize },
    #[serde(rename = "SHOP_REMOVE")]
    ShopRemove { deck_idx: usize },
    #[serde(rename = "SHOP_LEAVE")]
    ShopLeave,
}

fn default_no_target() -> i32 {
    -1
}

fn is_no_target(target: &i32) -> bool {
    *target < 0
}

// ===========================================================================
// PostState — full state snapshot after an action
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostState {
    pub player: PlayerPostState,
    pub enemies: Vec<EnemyPostState>,
    pub piles: PilePostState,
    pub relics: Vec<RelicPostState>,
    pub potions: Vec<String>,
    /// RNG stream counters keyed by the vault's canonical short names
    /// (`card`, `ai`, `shuffle`, ... — see module docs). `BTreeMap` so the
    /// key set can grow without a schema version bump and so serialized
    /// key order is deterministic.
    pub rng: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerPostState {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub energy: i32,
    /// `"Neutral" | "Wrath" | "Calm" | "Divinity"` for Watcher; other
    /// characters that never enter a stance report `"Neutral"`.
    pub stance: String,
    pub gold: i32,
    pub powers: Vec<PowerPostState>,
    /// Defect orb slots; always empty for Watcher/Ironclad/Silent.
    pub orbs: Vec<OrbPostState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerPostState {
    pub id: String,
    pub amt: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrbPostState {
    pub id: String,
    pub evoke_amount: i32,
    pub passive_amount: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyPostState {
    pub id: String,
    pub idx: usize,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub intent: IntentPostState,
    pub powers: Vec<PowerPostState>,
    /// Move-id history, oldest first — used to validate move-repeat/no-repeat
    /// constraints (`AbstractMonster.lastMove`/`lastTwoMoves`).
    pub move_history: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentPostState {
    pub move_id: i32,
    /// Human-readable move name when known (Java `moveName` constants);
    /// empty string when the engine has no string form for this move.
    #[serde(default)]
    pub name: String,
    pub dmg: i32,
    pub hits: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PilePostState {
    pub hand: Vec<String>,
    /// Draw pile in actual (post-shuffle) order, top of pile first.
    pub draw_ordered: Vec<String>,
    pub discard: Vec<String>,
    pub exhaust: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelicPostState {
    pub id: String,
    /// Relic counter (`AbstractRelic.counter`); `-1` when unused/not applicable.
    pub counter: i32,
}

// ===========================================================================
// T2 — Action script file (data/traces/scripts/<name>.json)
// ===========================================================================

/// An action script: the seeded, scripted action sequence that both the
/// Rust replay and the (human-attended) TraceLab mod consume 1:1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionScript {
    pub v: u32,
    pub seed: String,
    pub character: String,
    pub ascension: i32,
    pub stop: ScriptStopCondition,
    pub actions: Vec<TraceAction>,
}

impl ActionScript {
    pub fn check_version(&self) -> Result<(), String> {
        check_version(self.v)
    }
}

/// Parse a script seed using the same precedence as TraceLab.
///
/// Decimal strings are Java `long` values; display-form seeds fall back to
/// `SeedHelper.getLong`. The signed parse matters for seeds with the high bit
/// set because Java stores dungeon seeds in a signed `long`.
///
/// Source: packages/harness-java/src/main/java/tracelab/TraceLabMod.java
pub fn parse_script_seed(raw: &str) -> u64 {
    raw.trim()
        .parse::<i64>()
        .map(|seed| seed as u64)
        .unwrap_or_else(|_| crate::seed::seed_from_string(raw.trim()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptStopCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_floor: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_actions: Option<u64>,
}

// ===========================================================================
// T3 — Divergence report
// ===========================================================================

/// Top-level report emitted by the differ at `--diff <path>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivergenceReport {
    pub status: DivergenceStatus,
    pub script: String,
    pub seed: String,
    pub matched_actions: u64,
    pub total_actions: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_divergence: Option<FirstDivergence>,
    /// Masked diffs, in the order encountered (still counted as "matched"
    /// for `status` purposes, per T3: masked-only diffs still exit 0).
    #[serde(default)]
    pub masked: Vec<MaskedDiff>,
    /// Secondary field diffs at the same record as `first_divergence`
    /// (e.g. the RNG counter delta alongside the primary field), capped at
    /// 20 entries by the differ.
    #[serde(default)]
    pub secondary: Vec<FieldDiff>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DivergenceStatus {
    Match,
    Diverged,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirstDivergence {
    pub idx: u64,
    pub floor: i32,
    pub turn: i32,
    /// JSON-pointer-ish dotted path, e.g. `"post.enemies[0].intent.move_id"`.
    pub path: String,
    pub java: serde_json::Value,
    pub rust: serde_json::Value,
    /// RNG counters on each side at the point of divergence — checked
    /// first in canonical field order, so for enemy-AI work this is the
    /// primary diagnosis (`docs/goal/TOOLING.md` § T3).
    pub rng_at_divergence: RngSnapshotPair,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RngSnapshotPair {
    pub java: BTreeMap<String, i64>,
    pub rust: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDiff {
    pub path: String,
    pub java: serde_json::Value,
    pub rust: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskedDiff {
    pub path: String,
    pub java: serde_json::Value,
    pub rust: serde_json::Value,
    /// The mask id (`DEV-NNN`) that excluded this path from divergence.
    pub id: String,
}

// ===========================================================================
// Masks (docs/goal/masks.json)
// ===========================================================================

/// One entry in `docs/goal/masks.json`. Every entry MUST reference a
/// `DEV-NNN` id in `docs/work_units/parity-deviations-register.md` — a mask
/// without a `DEV-` id is a spec violation (`docs/goal/GOAL.md` § Edge-Case
/// Policy item 4) and must be rejected by loaders, not silently accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mask {
    /// Must start with `"DEV-"`; validate with [`Mask::check_id`].
    pub id: String,
    /// Dotted/JSON-pointer-ish path this mask excludes from divergence,
    /// e.g. `"post.neow.options"`.
    pub path: String,
    /// How broadly the mask applies, e.g. `"all"` for every record, or a
    /// narrower scope string understood by the differ.
    pub scope: String,
    pub reason: String,
    /// Path to the register document backing this mask (kept as data so
    /// the differ/report can surface it without hardcoding the doc path).
    pub register: String,
}

impl Mask {
    /// Validate the `DEV-` prefix invariant. Returns `Err` with a clear
    /// message naming the offending id if the mask is missing it.
    pub fn check_id(&self) -> Result<(), String> {
        if !self.id.starts_with("DEV-") {
            return Err(format!(
                "mask id '{}' must be DEV-NNN prefixed (path '{}')",
                self.id, self.path
            ));
        }
        Ok(())
    }
}

/// Parse and validate a masks file's contents (already-read JSON text).
/// Rejects the whole file if any entry fails [`Mask::check_id`].
pub fn parse_masks(json_text: &str) -> Result<Vec<Mask>, String> {
    let masks: Vec<Mask> =
        serde_json::from_str(json_text).map_err(|err| format!("invalid masks JSON: {err}"))?;
    for mask in &masks {
        mask.check_id()?;
        // The differ only implements `scope: "all"` today (`matching_mask`
        // applies a mask to every record). Silently accepting a narrower
        // scope string and then applying it globally would over-mask, so an
        // unknown scope is a hard error until narrower scopes are built.
        if mask.scope != "all" {
            return Err(format!(
                "mask '{}' has unsupported scope '{}' (only \"all\" is implemented; \
                 a narrower scope would be silently applied to every record)",
                mask.id, mask.scope
            ));
        }
    }
    Ok(masks)
}

// ===========================================================================
// T3 — Differ: field-by-field, canonical order, RNG counters first
// ===========================================================================

/// Secondary diffs at the first-diverging record are capped at this many
/// entries in the report (`docs/goal/TOOLING.md` § T3).
pub const MAX_SECONDARY_DIFFS: usize = 20;

/// Diff `java` vs `rust` trace records (already-parsed, header excluded)
/// field-by-field in canonical order — RNG counters first, per T3 ("for
/// enemy-AI work the counter delta is the diagnosis") — and build a
/// [`DivergenceReport`]. `masks` excludes matching paths from divergence
/// (still surfaced in the report's `masked` list).
///
/// Used by both `bin/trace_replay.rs` (against a real Java golden) and
/// `tests/test_trace_oracle.rs` (against an in-process synthetic "java"
/// side), so the diffing algorithm has exactly one implementation.
pub fn diff_records(
    script_name: &str,
    seed: &str,
    java: &[TraceRecord],
    rust: &[TraceRecord],
    masks: &[Mask],
) -> DivergenceReport {
    let total_actions = java.len().max(rust.len()) as u64;
    let mut matched_actions: u64 = 0;
    let mut first_divergence = None;
    let mut masked = Vec::new();
    let mut secondary = Vec::new();

    'records: for i in 0..java.len().min(rust.len()) {
        let j = &java[i];
        let r = &rust[i];
        let diffs = record_field_diffs(j, r);

        let mut record_diverged = false;
        for (path, jv, rv) in diffs {
            if let Some(mask) = matching_mask(masks, &path) {
                masked.push(MaskedDiff {
                    path,
                    java: jv,
                    rust: rv,
                    id: mask.id.clone(),
                });
                continue;
            }
            if first_divergence.is_none() {
                first_divergence = Some(FirstDivergence {
                    idx: j.idx,
                    floor: j.floor,
                    turn: j.turn,
                    path,
                    java: jv,
                    rust: rv,
                    rng_at_divergence: RngSnapshotPair {
                        java: j.post.rng.clone(),
                        rust: r.post.rng.clone(),
                    },
                });
                record_diverged = true;
            } else if secondary.len() < MAX_SECONDARY_DIFFS {
                secondary.push(FieldDiff { path, java: jv, rust: rv });
            }
        }

        if record_diverged {
            break 'records;
        }
        matched_actions += 1;
    }

    // A trace with fewer records than the other (e.g. one side stopped
    // early, or the script only partially replayed) is a divergence even
    // if every common-prefix record matched exactly — it must never be
    // silently reported as "match".
    if first_divergence.is_none() && java.len() != rust.len() {
        let java_ended = java.len() < rust.len();
        let cutoff = if java_ended { &rust[matched_actions as usize] } else { &java[matched_actions as usize] };
        let absent = serde_json::json!("absent (trace ended)");
        let present = serde_json::json!(cutoff.post.rng.clone());
        first_divergence = Some(FirstDivergence {
            idx: cutoff.idx,
            floor: cutoff.floor,
            turn: cutoff.turn,
            path: "post".to_string(),
            java: if java_ended { absent.clone() } else { present.clone() },
            rust: if java_ended { present } else { absent },
            rng_at_divergence: RngSnapshotPair {
                java: if java_ended { BTreeMap::new() } else { cutoff.post.rng.clone() },
                rust: if java_ended { cutoff.post.rng.clone() } else { BTreeMap::new() },
            },
        });
    }

    let status = if first_divergence.is_some() {
        DivergenceStatus::Diverged
    } else {
        DivergenceStatus::Match
    };

    DivergenceReport {
        status,
        script: script_name.to_string(),
        seed: seed.to_string(),
        matched_actions,
        total_actions,
        first_divergence,
        masked,
        secondary,
    }
}

fn matching_mask<'a>(masks: &'a [Mask], path: &str) -> Option<&'a Mask> {
    masks
        .iter()
        .find(|mask| path == mask.path || path.starts_with(&format!("{}.", mask.path)))
}

/// Field-by-field diff of a single record's `post`. Returns
/// `(json_pointer_ish_path, java_value, rust_value)` for every field that
/// differs, RNG counters first.
fn record_field_diffs(
    java: &TraceRecord,
    rust: &TraceRecord,
) -> Vec<(String, serde_json::Value, serde_json::Value)> {
    let mut diffs = Vec::new();

    // 1. RNG counters first.
    let mut rng_keys: Vec<&String> = java.post.rng.keys().chain(rust.post.rng.keys()).collect();
    rng_keys.sort();
    rng_keys.dedup();
    for key in rng_keys {
        let jv = java.post.rng.get(key).copied();
        let rv = rust.post.rng.get(key).copied();
        if jv != rv {
            diffs.push((format!("post.rng.{key}"), serde_json::json!(jv), serde_json::json!(rv)));
        }
    }

    // 2. Record identity and the executed action.
    push_diff(&mut diffs, "idx", java.idx, rust.idx);
    push_diff(&mut diffs, "floor", java.floor, rust.floor);
    push_diff(&mut diffs, "turn", java.turn, rust.turn);
    push_diff_str(&mut diffs, "phase", &java.phase, &rust.phase);
    push_diff(&mut diffs, "action", &java.action, &rust.action);

    // 3. Player.
    push_diff(&mut diffs, "post.player.hp", java.post.player.hp, rust.post.player.hp);
    push_diff(&mut diffs, "post.player.max_hp", java.post.player.max_hp, rust.post.player.max_hp);
    push_diff(&mut diffs, "post.player.block", java.post.player.block, rust.post.player.block);
    push_diff(&mut diffs, "post.player.energy", java.post.player.energy, rust.post.player.energy);
    push_diff_str(&mut diffs, "post.player.stance", &java.post.player.stance, &rust.post.player.stance);
    push_diff(&mut diffs, "post.player.gold", java.post.player.gold, rust.post.player.gold);
    push_power_diffs(
        &mut diffs,
        "post.player.powers",
        &java.post.player.powers,
        &rust.post.player.powers,
    );
    push_orb_diffs(
        &mut diffs,
        "post.player.orbs",
        &java.post.player.orbs,
        &rust.post.player.orbs,
    );

    // 4. Enemies (index-aligned).
    let enemy_count = java.post.enemies.len().max(rust.post.enemies.len());
    for idx in 0..enemy_count {
        let base = format!("post.enemies[{idx}]");
        match (java.post.enemies.get(idx), rust.post.enemies.get(idx)) {
            (Some(je), Some(re)) => {
                push_diff_str(&mut diffs, &format!("{base}.id"), &je.id, &re.id);
                push_diff(&mut diffs, &format!("{base}.idx"), je.idx, re.idx);
                push_diff(&mut diffs, &format!("{base}.hp"), je.hp, re.hp);
                push_diff(&mut diffs, &format!("{base}.max_hp"), je.max_hp, re.max_hp);
                push_diff(&mut diffs, &format!("{base}.block"), je.block, re.block);
                push_diff(&mut diffs, &format!("{base}.intent.move_id"), je.intent.move_id, re.intent.move_id);
                push_diff_str(&mut diffs, &format!("{base}.intent.name"), &je.intent.name, &re.intent.name);
                push_diff(&mut diffs, &format!("{base}.intent.dmg"), je.intent.dmg, re.intent.dmg);
                push_diff(&mut diffs, &format!("{base}.intent.hits"), je.intent.hits, re.intent.hits);
                push_power_diffs(
                    &mut diffs,
                    &format!("{base}.powers"),
                    &je.powers,
                    &re.powers,
                );
                push_diff(
                    &mut diffs,
                    &format!("{base}.move_history"),
                    &je.move_history,
                    &re.move_history,
                );
            }
            (Some(_), None) => {
                diffs.push((base, serde_json::json!("present"), serde_json::json!("absent")))
            }
            (None, Some(_)) => {
                diffs.push((base, serde_json::json!("absent"), serde_json::json!("present")))
            }
            (None, None) => {}
        }
    }

    // 5. Piles.
    push_diff_vec(&mut diffs, "post.piles.hand", &java.post.piles.hand, &rust.post.piles.hand);
    push_diff_vec(
        &mut diffs,
        "post.piles.draw_ordered",
        &java.post.piles.draw_ordered,
        &rust.post.piles.draw_ordered,
    );
    push_diff_vec(&mut diffs, "post.piles.discard", &java.post.piles.discard, &rust.post.piles.discard);
    push_diff_vec(&mut diffs, "post.piles.exhaust", &java.post.piles.exhaust, &rust.post.piles.exhaust);

    // 6. Relics + potions.
    let relic_count = java.post.relics.len().max(rust.post.relics.len());
    for idx in 0..relic_count {
        let base = format!("post.relics[{idx}]");
        match (java.post.relics.get(idx), rust.post.relics.get(idx)) {
            (Some(jr), Some(rr)) => {
                push_diff_str(&mut diffs, &format!("{base}.id"), &jr.id, &rr.id);
                push_diff(&mut diffs, &format!("{base}.counter"), jr.counter, rr.counter);
            }
            (Some(_), None) => {
                diffs.push((base, serde_json::json!("present"), serde_json::json!("absent")))
            }
            (None, Some(_)) => {
                diffs.push((base, serde_json::json!("absent"), serde_json::json!("present")))
            }
            (None, None) => {}
        }
    }
    push_diff_vec(&mut diffs, "post.potions", &java.post.potions, &rust.post.potions);

    diffs
}

fn push_power_diffs(
    diffs: &mut Vec<(String, serde_json::Value, serde_json::Value)>,
    base: &str,
    java: &[PowerPostState],
    rust: &[PowerPostState],
) {
    let count = java.len().max(rust.len());
    for idx in 0..count {
        let path = format!("{base}[{idx}]");
        match (java.get(idx), rust.get(idx)) {
            (Some(jp), Some(rp)) => {
                push_diff_str(diffs, &format!("{path}.id"), &jp.id, &rp.id);
                push_diff(diffs, &format!("{path}.amt"), jp.amt, rp.amt);
            }
            (Some(_), None) => diffs.push((
                path,
                serde_json::json!("present"),
                serde_json::json!("absent"),
            )),
            (None, Some(_)) => diffs.push((
                path,
                serde_json::json!("absent"),
                serde_json::json!("present"),
            )),
            (None, None) => {}
        }
    }
}

fn push_orb_diffs(
    diffs: &mut Vec<(String, serde_json::Value, serde_json::Value)>,
    base: &str,
    java: &[OrbPostState],
    rust: &[OrbPostState],
) {
    let count = java.len().max(rust.len());
    for idx in 0..count {
        let path = format!("{base}[{idx}]");
        match (java.get(idx), rust.get(idx)) {
            (Some(jo), Some(ro)) => {
                push_diff_str(diffs, &format!("{path}.id"), &jo.id, &ro.id);
                push_diff(
                    diffs,
                    &format!("{path}.evoke_amount"),
                    jo.evoke_amount,
                    ro.evoke_amount,
                );
                push_diff(
                    diffs,
                    &format!("{path}.passive_amount"),
                    jo.passive_amount,
                    ro.passive_amount,
                );
            }
            (Some(_), None) => diffs.push((
                path,
                serde_json::json!("present"),
                serde_json::json!("absent"),
            )),
            (None, Some(_)) => diffs.push((
                path,
                serde_json::json!("absent"),
                serde_json::json!("present"),
            )),
            (None, None) => {}
        }
    }
}

fn push_diff<T: PartialEq + serde::Serialize>(
    diffs: &mut Vec<(String, serde_json::Value, serde_json::Value)>,
    path: &str,
    java: T,
    rust: T,
) {
    if java != rust {
        diffs.push((path.to_string(), serde_json::json!(java), serde_json::json!(rust)));
    }
}

fn push_diff_str(
    diffs: &mut Vec<(String, serde_json::Value, serde_json::Value)>,
    path: &str,
    java: &str,
    rust: &str,
) {
    if java != rust {
        diffs.push((path.to_string(), serde_json::json!(java), serde_json::json!(rust)));
    }
}

fn push_diff_vec(
    diffs: &mut Vec<(String, serde_json::Value, serde_json::Value)>,
    path: &str,
    java: &[String],
    rust: &[String],
) {
    if java != rust {
        diffs.push((path.to_string(), serde_json::json!(java), serde_json::json!(rust)));
    }
}

// ===========================================================================
// Canonical RNG stream keys (for producers/differs that want the full list)
// ===========================================================================

/// The 13 canonical RNG stream keys in the vault's documented order
/// (persistent streams, then per-floor streams, then `map`). Not all
/// producers populate every key; this exists so tooling can iterate the
/// "expected" key set (e.g. to warn on an unrecognized key) without
/// hardcoding the list in multiple places.
pub const CANONICAL_RNG_KEYS: &[&str] = &[
    "card",
    "monster",
    "event",
    "relic",
    "treasure",
    "potion",
    "merchant",
    "monsterHp",
    "ai",
    "shuffle",
    "cardRandom",
    "misc",
    "map",
];

// ===========================================================================
// Replay support — TraceAction -> GameAction, and RunEngine -> PostState.
//
// Shared by `bin/trace_replay.rs` (against a real Java golden, T3/T4) and
// `tests/test_trace_oracle.rs` (in-process synthetic fixture, T5), so the
// action-mapping and state-snapshotting logic has exactly one
// implementation. This is deliberately a "layer on top of core" module
// (depends on `run`/`engine`/`state`, per `docs/goal/GOAL.md`'s
// architecture target — core sim never depends back on `trace`).
// ===========================================================================

/// Map a script `TraceAction` to the engine's `GameAction`.
///
/// Implemented today: `PLAY_CARD`, `END_TURN`, `USE_POTION` (combat), plus
/// `NEOW` and `PATH` (straightforward 1:1 index mappings off `run.rs`'s
/// `GameAction::ChooseNeowOption`/`ChoosePath`). Anything else is a hard
/// error naming the unsupported action type — per T3, the differ must
/// never guess semantics for an action it doesn't recognize.
pub fn map_action(
    engine: &crate::run::RunEngine,
    action: &TraceAction,
) -> Result<crate::run::GameAction, String> {
    use crate::actions::Action;
    use crate::run::GameAction;

    match action {
        TraceAction::PlayCard { hand_idx, target, .. } => {
            Ok(GameAction::CombatAction(Action::PlayCard { card_idx: *hand_idx, target_idx: *target }))
        }
        TraceAction::EndTurn => Ok(GameAction::CombatAction(Action::EndTurn)),
        TraceAction::UsePotion { idx, target } => {
            Ok(GameAction::CombatAction(Action::UsePotion { potion_idx: *idx, target_idx: *target }))
        }
        TraceAction::Neow { choice } => Ok(GameAction::ChooseNeowOption(*choice)),
        TraceAction::Path { choice } => Ok(GameAction::ChoosePath(*choice)),
        unsupported => Err(format!(
            "unsupported action type for trace_replay (engine phase={:?}): {unsupported:?}. \
             RunEngine mapping for this action type is not implemented yet; see \
             docs/goal/UNITS.md U05 for scope.",
            engine.current_phase()
        )),
    }
}

fn phase_label(phase: crate::run::RunPhase) -> &'static str {
    use crate::run::RunPhase;
    match phase {
        RunPhase::Neow => "NEOW",
        RunPhase::MapChoice => "MAP",
        RunPhase::Chest => "CHEST",
        RunPhase::Combat => "COMBAT",
        RunPhase::CardReward => "CARD_REWARD",
        RunPhase::Campfire => "CAMPFIRE",
        RunPhase::Shop => "SHOP",
        RunPhase::Event => "EVENT",
        RunPhase::Transition => "TRANSITION",
        RunPhase::GameOver => "GAME_OVER",
    }
}

/// Build a [`TraceRecord`] from the engine's *current* (post-step) state.
pub fn build_trace_record(engine: &crate::run::RunEngine, idx: u64, action: TraceAction) -> TraceRecord {
    let floor = engine.run_state.floor;
    let phase = engine.current_phase();
    let turn = engine.get_combat_engine().map(|c| c.state.turn).unwrap_or(0);

    TraceRecord {
        v: 1,
        idx,
        floor,
        turn,
        phase: phase_label(phase).to_string(),
        action,
        post: build_post_state(engine),
    }
}

fn trace_potion_id(potion_id: &str) -> String {
    // Java keeps vacant inventory entries as PotionSlot instances whose
    // stable POTION_ID is "Potion Slot"; traces must preserve those entries.
    // Source: decompiled/java-src/com/megacrit/cardcrawl/potions/PotionSlot.java
    if potion_id.is_empty() {
        "Potion Slot".to_string()
    } else {
        potion_id.to_string()
    }
}

fn run_relic_counter(relic_id: &str, counters: &[i16; crate::relic_flags::counter::NUM_COUNTERS]) -> Option<i32> {
    use crate::relic_flags::counter;

    let (index, normalize): (usize, fn(i16) -> i32) = match relic_id {
        "MawBank" | "Maw Bank" => (counter::MAW_BANK_GOLD, |value| if value == -2 { -2 } else { -1 }),
        "Omamori" => (counter::OMAMORI_USES, |value| value as i32),
        "Matryoshka" => (counter::MATRYOSHKA_USES, |value| if value <= 0 { -2 } else { value as i32 }),
        "Ancient Tea Set" | "AncientTeaSet" => (
            counter::ANCIENT_TEA_SET,
            |value| if value > 0 { -2 } else { -1 },
        ),
        "Girya" => (counter::GIRYA, |value| value as i32),
        "Tiny Chest" | "TinyChest" => (counter::TINY_CHEST, |value| value as i32),
        "NlothsMask" => (counter::NLOTHS_MASK, |value| value as i32),
        "WingedGreaves" => (counter::WINGED_GREAVES, |value| value as i32),
        "NeowsBlessing" => (counter::NEOWS_LAMENT, |value| value as i32),
        _ => return None,
    };
    Some(normalize(counters[index]))
}

fn runtime_counter_relic(relic_id: &str) -> bool {
    matches!(
        relic_id,
        "Pen Nib"
            | "Nunchaku"
            | "InkBottle"
            | "Happy Flower"
            | "Incense Burner"
            | "Sundial"
            | "Inserter"
            | "Ornamental Fan"
            | "Kunai"
            | "Shuriken"
            | "Letter Opener"
            | "StoneCalendar"
            | "Velvet Choker"
            | "Pocketwatch"
            | "Du-Vu Doll"
            | "HornCleat"
            | "CaptainsWheel"
    )
}

fn runtime_counter_persists_outside_combat(relic_id: &str) -> bool {
    matches!(
        relic_id,
        "Pen Nib"
            | "Nunchaku"
            | "InkBottle"
            | "Happy Flower"
            | "Incense Burner"
            | "Sundial"
            | "Inserter"
    )
}

fn outside_combat_relic_counter(engine: &crate::run::RunEngine, relic_id: &str) -> i32 {
    if let Some(counter) = run_relic_counter(relic_id, &engine.run_state.relic_flags.counters) {
        return counter;
    }
    match relic_id {
        "Lizard Tail" => return if engine.run_state.lizard_tail_used { -2 } else { -1 },
        "Circlet" => return 1,
        "Du-Vu Doll" => {
            let registry = crate::cards::global_registry();
            return engine
                .run_state
                .deck
                .iter()
                .filter(|id| {
                    registry
                        .get(id)
                        .is_some_and(|card| card.card_type == crate::cards::CardType::Curse)
                })
                .count() as i32;
        }
        _ => {}
    }
    if runtime_counter_persists_outside_combat(relic_id) {
        return engine
            .run_state
            .persisted_effect_states
            .iter()
            .find(|state| state.def_id == relic_id)
            .and_then(|state| state.values.first())
            .copied()
            .unwrap_or(0) as i32;
    }
    -1
}

fn combat_relic_counter(
    engine: &crate::run::RunEngine,
    combat: &crate::engine::CombatEngine,
    relic_id: &str,
    slot: usize,
) -> i32 {
    if let Some(counter) = run_relic_counter(relic_id, &combat.state.relic_counters) {
        return counter;
    }
    match relic_id {
        "Lizard Tail" => return if engine.run_state.lizard_tail_used { -2 } else { -1 },
        "Circlet" => return 1,
        _ => {}
    }
    if runtime_counter_relic(relic_id) {
        return combat.hidden_effect_value(
            relic_id,
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: slot as u16 },
            0,
        );
    }
    -1
}

/// Snapshot the engine's current state into a [`PostState`].
///
/// Outside combat there is no `CombatState` to report player/enemy detail
/// from; a minimal but well-formed `PostState` is built from `RunState` so
/// every record still round-trips through the schema.
pub fn build_post_state(engine: &crate::run::RunEngine) -> PostState {
    let rng: BTreeMap<String, i64> = engine.rng_counters().into_iter().collect();

    let Some(combat) = engine.get_combat_engine() else {
        return PostState {
            player: PlayerPostState {
                hp: engine.run_state.current_hp,
                max_hp: engine.run_state.max_hp,
                block: 0,
                energy: 0,
                stance: "Neutral".to_string(),
                gold: engine.run_state.gold,
                powers: vec![],
                orbs: vec![],
            },
            enemies: vec![],
            piles: PilePostState { hand: vec![], draw_ordered: vec![], discard: vec![], exhaust: vec![] },
            relics: engine
                .run_state
                .relics
                .iter()
                .map(|id| RelicPostState {
                    id: id.clone(),
                    counter: outside_combat_relic_counter(engine, id),
                })
                .collect(),
            potions: engine
                .run_state
                .potions
                .iter()
                .map(|potion| trace_potion_id(potion))
                .collect(),
            rng,
        };
    };

    let state = &combat.state;
    let registry = combat.card_registry;

    let card_name = |inst: &crate::combat_types::CardInstance| -> String {
        let mut name = registry.card_name(inst.def_id).to_string();
        if inst.is_upgraded() && !name.ends_with('+') {
            name.push('+');
        }
        name
    };

    let powers_from_statuses = |statuses: &[i32; crate::status_ids::sid::MAX_STATUS_ID]| -> Vec<PowerPostState> {
        statuses
            .iter()
            .enumerate()
            .filter(|(index, &amt)| {
                amt != 0
                    && !matches!(
                        *index as u16,
                        value if value == crate::status_ids::sid::STARTING_DMG.0
                            || value == crate::status_ids::sid::STR_AMT.0
                            || value == crate::status_ids::sid::HIGH_ASCENSION_AI.0
                    )
            })
            .map(|(i, &amt)| PowerPostState {
                id: trace_power_id(crate::status_ids::status_name(crate::ids::StatusId(
                    i as u16,
                )))
                .to_string(),
                amt,
            })
            .collect()
    };

    let enemies: Vec<EnemyPostState> = state
        .enemies
        .iter()
        .enumerate()
        .map(|(idx, enemy)| EnemyPostState {
            id: enemy.id.clone(),
            idx,
            hp: enemy.entity.hp,
            max_hp: enemy.entity.max_hp,
            block: enemy.entity.block,
            intent: IntentPostState {
                move_id: enemy.move_id,
                name: trace_intent_name(enemy.intent).to_string(),
                // AbstractMonster keeps -1 for non-attack intent damage and
                // the recorder reports one hit whenever isMultiDmg is false.
                // Java: AbstractMonster.java::{getIntentDmg,createIntent}.
                dmg: trace_intent_damage(combat, enemy),
                hits: if enemy.move_hits() > 1 {
                    enemy.move_hits()
                } else {
                    1
                },
            },
            powers: powers_from_statuses(&enemy.entity.statuses),
            // Rust keeps completed moves in history and the selected current
            // move in `move_id`; Java appends current in setMove().
            move_history: enemy
                .move_history
                .iter()
                .copied()
                .chain((enemy.move_id >= 0).then_some(enemy.move_id))
                .collect(),
        })
        .collect();

    let orbs: Vec<OrbPostState> = state
        .orb_slots
        .slots
        .iter()
        .filter(|orb| orb.orb_type != crate::orbs::OrbType::Empty)
        .map(|orb| OrbPostState {
            id: orb.orb_type.as_str().to_string(),
            evoke_amount: orb.evoke_amount,
            passive_amount: orb.base_passive,
        })
        .collect();

    PostState {
        player: PlayerPostState {
            hp: state.player.hp,
            max_hp: state.player.max_hp,
            block: state.player.block,
            energy: state.energy,
            stance: state.stance.as_str().to_string(),
            gold: engine.run_state.gold,
            powers: powers_from_statuses(&state.player.statuses),
            orbs,
        },
        enemies,
        piles: PilePostState {
            hand: state.hand.iter().map(card_name).collect(),
            draw_ordered: state.draw_pile.iter().map(card_name).collect(),
            discard: state.discard_pile.iter().map(card_name).collect(),
            exhaust: state.exhaust_pile.iter().map(card_name).collect(),
        },
        relics: state
            .relics
            .iter()
            .enumerate()
            .map(|(slot, id)| RelicPostState {
                id: id.clone(),
                counter: combat_relic_counter(engine, combat, id, slot),
            })
            .collect(),
        potions: state
            .potions
            .iter()
            .map(|potion| trace_potion_id(potion))
            .collect(),
        rng,
    }
}

fn trace_intent_name(intent: crate::combat_types::Intent) -> &'static str {
    use crate::combat_types::Intent;
    match intent {
        Intent::Attack { .. } => "ATTACK",
        Intent::Block { .. } => "DEFEND",
        Intent::Buff { .. } => "BUFF",
        Intent::Debuff { .. } => "DEBUFF",
        Intent::AttackBlock { .. } => "ATTACK_DEFEND",
        Intent::AttackBuff { .. } => "ATTACK_BUFF",
        Intent::AttackDebuff { .. } => "ATTACK_DEBUFF",
        Intent::DefendBuff { .. } => "DEFEND_BUFF",
        Intent::Escape => "ESCAPE",
        Intent::Sleep => "SLEEP",
        Intent::Stun => "STUN",
        Intent::Spawn | Intent::Unknown => "UNKNOWN",
    }
}

fn trace_power_id(runtime_id: &str) -> &str {
    match runtime_id {
        // CurlUpPower.java declares `POWER_ID = "Curl Up"`.
        "CurlUp" => "Curl Up",
        _ => runtime_id,
    }
}

fn trace_intent_damage(
    combat: &crate::engine::CombatEngine,
    enemy: &crate::state::EnemyCombatState,
) -> i32 {
    if !enemy.is_attacking() {
        return -1;
    }
    // AbstractMonster.calculateDamage carries one Java float through the
    // atDamageGive/Receive and stance pipeline, applies BackAttack's explicit
    // integer cast, then runs final modifiers such as Intangible before the
    // single final floor.
    let mut damage = (enemy.move_damage() + enemy.entity.strength()) as f32;
    if enemy.entity.is_weak() {
        let multiplier = if combat.state.has_relic("Paper Crane") {
            crate::damage::WEAK_MULT_PAPER_CRANE
        } else {
            crate::damage::WEAK_MULT
        };
        damage *= multiplier as f32;
    }
    if combat.state.player.is_vulnerable() {
        damage *= if combat.state.has_relic("Odd Mushroom") {
            crate::damage::VULN_MULT_ODD_MUSHROOM as f32
        } else {
            crate::damage::VULN_MULT as f32
        };
    }
    damage *= combat.state.stance.incoming_mult() as f32;
    if enemy.back_attack {
        damage = (damage * 1.5) as i32 as f32;
    }
    if combat.state.player.status(crate::status_ids::sid::INTANGIBLE) > 0 && damage > 1.0 {
        damage = 1.0;
    }
    damage.floor().max(0.0) as i32
}

/// Replay an [`ActionScript`] in-process through a fresh [`crate::run::RunEngine`],
/// returning one [`TraceRecord`] per executed action. Stops early on the
/// script's stop condition or run completion. Errors (unsupported action,
/// illegal action) mirror `bin/trace_replay.rs`'s behavior.
pub fn replay_script(script: &ActionScript) -> Result<Vec<TraceRecord>, String> {
    let seed = parse_script_seed(&script.seed);
    let mut engine = crate::run::RunEngine::new(seed, script.ascension);
    let mut records = Vec::with_capacity(script.actions.len());
    let mut legacy_neow_selection_pending = false;

    for (idx, action) in script.actions.iter().enumerate() {
        if matches!(action, TraceAction::Neow { .. }) {
            // Frozen v1 has one semantic Neow action and cannot encode the
            // canonical Intro Proceed. Supply only that unambiguous frame;
            // v2 records the core actions directly.
            step_legacy_neow_proceed(&mut engine, idx, "intro")?;
        }
        let run_action = map_action(&engine, action)?;
        let legal = engine.get_legal_actions();
        if !legal.contains(&run_action) {
            return Err(format!(
                "action {idx} ({action:?}) is not legal in the current engine state (phase={:?})",
                engine.current_phase()
            ));
        }
        engine.step_game(&run_action);
        records.push(build_trace_record(&engine, idx as u64, action.clone()));

        if matches!(action, TraceAction::Neow { .. }) {
            legacy_neow_selection_pending = true;
        }
        if legacy_neow_selection_pending
            && engine.current_phase() == crate::run::RunPhase::Neow
            && engine.get_legal_actions() == vec![crate::run::GameAction::Proceed]
        {
            // Keep the v1 record at the semantic selection checkpoint, matching
            // the frozen wire contract, then consume the otherwise unencodable
            // Exit Proceed before the next scripted action.
            step_legacy_neow_proceed(&mut engine, idx, "exit")?;
            legacy_neow_selection_pending = false;
        }

        if let Some(max_floor) = script.stop.max_floor {
            // Match the Java harness's stop semantics exactly
            // (packages/harness-java/.../tracelab/ScriptRunner.java:
            // `AbstractDungeon.floorNum > script.stop.max_floor`): actions ON
            // the max floor still execute; the run stops only once the floor
            // number exceeds it. `>=` here would truncate the Rust trace a
            // floor early and diff as a spurious length mismatch.
            if engine.run_state.floor > max_floor {
                break;
            }
        }
        if let Some(max_actions) = script.stop.max_actions {
            if records.len() as u64 >= max_actions {
                break;
            }
        }
        if engine.is_done() {
            break;
        }
    }

    Ok(records)
}

fn step_legacy_neow_proceed(
    engine: &mut crate::run::RunEngine,
    idx: usize,
    frame: &str,
) -> Result<(), String> {
    let legal = engine.get_legal_actions();
    if engine.current_phase() != crate::run::RunPhase::Neow
        || legal != vec![crate::run::GameAction::Proceed]
    {
        return Err(format!(
            "legacy v1 action {idx} cannot adapt Neow {frame}: expected sole legal Proceed, got {legal:?} in phase {:?}",
            engine.current_phase()
        ));
    }
    let outcome = engine.step_game(&crate::run::GameAction::Proceed);
    if !outcome.accepted() {
        return Err(format!(
            "legacy v1 action {idx} could not execute Neow {frame} Proceed"
        ));
    }
    Ok(())
}

/// Build a [`TraceHeader`] for a replayed script (Rust-produced trace).
pub fn header_for_script(script: &ActionScript) -> TraceHeader {
    TraceHeader {
        v: 1,
        kind: "header".to_string(),
        seed: script.seed.clone(),
        seed_long: parse_script_seed(&script.seed) as i64,
        character: script.character.clone(),
        ascension: script.ascension,
        game_version: "rust-sim".to_string(),
        mods: vec![],
    }
}
