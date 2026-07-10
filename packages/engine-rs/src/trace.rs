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
//! `BTreeMap<String, u64>` — rather than a fixed struct — so the schema
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
// T2 — Action vocabulary (tagged enum, matches script file + RunAction 1:1)
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
    pub rng: BTreeMap<String, u64>,
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
    pub java: BTreeMap<String, u64>,
    pub rust: BTreeMap<String, u64>,
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
    }
    Ok(masks)
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
