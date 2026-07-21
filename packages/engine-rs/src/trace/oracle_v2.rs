//! Language-neutral post-action state used by the Java/Rust oracle.
//!
//! `CoreCheckpoint` remains the Rust continuation format. This module contains
//! only stable gameplay identities and values that both implementations can
//! emit and compare.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::run::{RunEngine, RunPhase};

pub const ORACLE_STATE_SCHEMA_NAME: &str = "sts.oracle_state";
pub const ORACLE_STATE_SCHEMA_MAJOR: u32 = 2;
pub const ORACLE_STATE_SCHEMA_MINOR: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleStateSchemaVersion {
    pub name: String,
    pub major: u32,
    pub minor: u32,
}

impl OracleStateSchemaVersion {
    pub fn current() -> Self {
        Self {
            name: ORACLE_STATE_SCHEMA_NAME.to_string(),
            major: ORACLE_STATE_SCHEMA_MAJOR,
            minor: ORACLE_STATE_SCHEMA_MINOR,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name != ORACLE_STATE_SCHEMA_NAME {
            return Err(format!(
                "unsupported oracle-state schema name {:?}",
                self.name
            ));
        }
        if self.major != ORACLE_STATE_SCHEMA_MAJOR {
            return Err(format!(
                "unsupported oracle-state schema major {}",
                self.major
            ));
        }
        if self.minor > ORACLE_STATE_SCHEMA_MINOR {
            return Err(format!(
                "unsupported oracle-state schema minor {}",
                self.minor
            ));
        }
        Ok(())
    }
}

impl Default for OracleStateSchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OracleStateV2 {
    pub schema: OracleStateSchemaVersion,
    pub floor: i32,
    pub act: i32,
    pub turn: i32,
    pub phase: String,
    pub map: OracleMapPositionV2,
    pub keys: OracleKeyStateV2,
    pub player: OraclePlayerStateV2,
    pub enemies: Vec<OracleEnemyStateV2>,
    pub piles: OraclePileStateV2,
    pub deck: Vec<String>,
    pub relics: Vec<OracleRelicStateV2>,
    pub potions: Vec<String>,
    pub rng: OracleRngStateV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neow: Option<OracleNeowStateV2>,
}

#[derive(Deserialize)]
struct OracleStateV2Wire {
    schema: OracleStateSchemaVersion,
    floor: i32,
    act: i32,
    turn: i32,
    phase: String,
    map: OracleMapPositionV2,
    keys: OracleKeyStateV2,
    player: OraclePlayerStateV2,
    enemies: Vec<OracleEnemyStateV2>,
    piles: OraclePileStateV2,
    deck: Vec<String>,
    relics: Vec<OracleRelicStateV2>,
    potions: Vec<String>,
    rng: OracleRngStateV2,
    #[serde(default)]
    neow: Option<OracleNeowStateV2>,
}

impl<'de> Deserialize<'de> for OracleStateV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = OracleStateV2Wire::deserialize(deserializer)?;
        let state = Self {
            schema: wire.schema,
            floor: wire.floor,
            act: wire.act,
            turn: wire.turn,
            phase: wire.phase,
            map: wire.map,
            keys: wire.keys,
            player: wire.player,
            enemies: wire.enemies,
            piles: wire.piles,
            deck: wire.deck,
            relics: wire.relics,
            potions: wire.potions,
            rng: wire.rng,
            neow: wire.neow,
        };
        state.validate().map_err(serde::de::Error::custom)?;
        Ok(state)
    }
}

impl OracleStateV2 {
    pub fn validate(&self) -> Result<(), String> {
        self.schema.validate()?;
        self.rng.validate()?;
        if self.floor < 0 {
            return Err(format!("oracle-state floor {} is negative", self.floor));
        }
        if !(1..=4).contains(&self.act) {
            return Err(format!(
                "oracle-state act {} is outside 1 through 4",
                self.act
            ));
        }
        if self.turn < 1 {
            return Err(format!("oracle-state turn {} must be positive", self.turn));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleMapPositionV2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleKeyStateV2 {
    pub ruby: bool,
    pub emerald: bool,
    pub sapphire: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OraclePlayerStateV2 {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub energy: i32,
    pub stance: String,
    pub gold: i32,
    pub powers: Vec<OraclePowerStateV2>,
    pub orbs: Vec<OracleOrbStateV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OraclePowerStateV2 {
    pub id: String,
    pub amt: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleOrbStateV2 {
    pub id: String,
    pub evoke_amount: i32,
    pub passive_amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleEnemyStateV2 {
    pub id: String,
    pub idx: usize,
    pub dead: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub intent: OracleIntentStateV2,
    pub powers: Vec<OraclePowerStateV2>,
    pub move_history: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleIntentStateV2 {
    pub move_id: i32,
    pub name: String,
    pub dmg: i32,
    pub hits: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OraclePileStateV2 {
    pub hand: Vec<String>,
    pub draw_ordered: Vec<String>,
    pub discard: Vec<String>,
    pub exhaust: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleRelicStateV2 {
    pub id: String,
    pub counter: i32,
}

/// The complete named StS dungeon RNG counter set.
///
/// Typed required fields make a missing Java stream a schema error instead of
/// a value that can silently default to zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleRngStateV2 {
    pub card: i64,
    pub monster: i64,
    pub event: i64,
    pub relic: i64,
    pub treasure: i64,
    pub potion: i64,
    pub merchant: i64,
    #[serde(rename = "monsterHp")]
    pub monster_hp: i64,
    pub ai: i64,
    pub shuffle: i64,
    #[serde(rename = "cardRandom")]
    pub card_random: i64,
    pub misc: i64,
    pub map: i64,
    #[serde(rename = "ambientMath")]
    pub ambient_math: OracleRandomXs128StateV2,
    #[serde(rename = "javaCollections")]
    pub java_collections: String,
    #[serde(rename = "rawStates")]
    pub raw_states: BTreeMap<String, OracleCountedRngStateV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleRandomXs128StateV2 {
    pub seed0: String,
    pub seed1: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCountedRngStateV2 {
    pub seed0: String,
    pub seed1: String,
    pub counter: i64,
}

impl OracleRngStateV2 {
    fn validate(&self) -> Result<(), String> {
        parse_hex_state("rng.ambientMath.seed0", &self.ambient_math.seed0, 64)?;
        parse_hex_state("rng.ambientMath.seed1", &self.ambient_math.seed1, 64)?;
        parse_hex_state("rng.javaCollections", &self.java_collections, 48)?;
        for name in crate::trace::CANONICAL_RNG_KEYS
            .iter()
            .copied()
            .chain(std::iter::once("neow"))
        {
            let state = self
                .raw_states
                .get(name)
                .ok_or_else(|| format!("rng.rawStates omitted required stream {name:?}"))?;
            parse_hex_state(&format!("rng.rawStates.{name}.seed0"), &state.seed0, 64)?;
            parse_hex_state(&format!("rng.rawStates.{name}.seed1"), &state.seed1, 64)?;
            if let Some(flat_counter) = self.flat_counter(name) {
                if state.counter != flat_counter {
                    return Err(format!(
                        "rng.rawStates.{name}.counter {} disagrees with rng.{name} {}",
                        state.counter, flat_counter
                    ));
                }
            }
        }
        Ok(())
    }

    fn flat_counter(&self, name: &str) -> Option<i64> {
        match name {
            "card" => Some(self.card),
            "monster" => Some(self.monster),
            "event" => Some(self.event),
            "relic" => Some(self.relic),
            "treasure" => Some(self.treasure),
            "potion" => Some(self.potion),
            "merchant" => Some(self.merchant),
            "monsterHp" => Some(self.monster_hp),
            "ai" => Some(self.ai),
            "shuffle" => Some(self.shuffle),
            "cardRandom" => Some(self.card_random),
            "misc" => Some(self.misc),
            "map" => Some(self.map),
            "neow" => None,
            _ => None,
        }
    }
}

fn parse_hex_state(path: &str, value: &str, bits: u32) -> Result<u64, String> {
    let expected_digits = (bits / 4) as usize;
    if value.len() != expected_digits || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "{path} must be exactly {expected_digits} hexadecimal digits"
        ));
    }
    u64::from_str_radix(value, 16).map_err(|_| format!("{path} is not valid hexadecimal state"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleNeowStateV2 {
    pub mode: String,
    pub options: Vec<OracleNeowOptionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<OracleNeowOptionV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleNeowOptionV2 {
    pub category: u8,
    pub reward_id: String,
    pub drawback_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleStateFieldDiff {
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialOracleStateDiff {
    pub diffs: Vec<OracleStateFieldDiff>,
    /// Canonical leaves that the recorder did not emit, grouped by path.
    pub skipped_fields_by_path: BTreeMap<String, u64>,
}

impl PartialOracleStateDiff {
    pub fn skipped_fields_total(&self) -> u64 {
        self.skipped_fields_by_path.values().sum()
    }
}

pub fn project_oracle_state(engine: &RunEngine) -> Result<OracleStateV2, String> {
    let post = crate::trace::build_post_state(engine);
    let combat = engine.get_combat_engine();
    let enemies = post
        .enemies
        .into_iter()
        .map(|enemy| {
            let dead = combat
                .and_then(|combat| combat.state.enemies.get(enemy.idx))
                .is_some_and(|state| state.entity.hp <= 0);
            OracleEnemyStateV2 {
                id: enemy.id,
                idx: enemy.idx,
                dead,
                hp: enemy.hp,
                max_hp: enemy.max_hp,
                block: enemy.block,
                intent: OracleIntentStateV2 {
                    move_id: enemy.intent.move_id,
                    name: enemy.intent.name,
                    dmg: enemy.intent.dmg,
                    hits: enemy.intent.hits,
                },
                powers: enemy
                    .powers
                    .into_iter()
                    .map(|power| OraclePowerStateV2 {
                        id: power.id,
                        amt: power.amt,
                    })
                    .collect(),
                move_history: enemy.move_history,
            }
        })
        .collect();
    let player = OraclePlayerStateV2 {
        hp: post.player.hp,
        max_hp: post.player.max_hp,
        block: post.player.block,
        energy: post.player.energy,
        stance: post.player.stance,
        gold: post.player.gold,
        powers: post
            .player
            .powers
            .into_iter()
            .map(|power| OraclePowerStateV2 {
                id: power.id,
                amt: power.amt,
            })
            .collect(),
        orbs: post
            .player
            .orbs
            .into_iter()
            .map(|orb| OracleOrbStateV2 {
                id: orb.id,
                evoke_amount: orb.evoke_amount,
                passive_amount: orb.passive_amount,
            })
            .collect(),
    };
    let neow = engine
        .oracle_neow_witness()
        .map(|(options, selected)| OracleNeowStateV2 {
            mode: "four_choices".to_string(),
            options: options
                .into_iter()
                .map(|option| OracleNeowOptionV2 {
                    category: option.category,
                    reward_id: option.reward_id,
                    drawback_id: option.drawback_id,
                    label: option.label,
                })
                .collect(),
            selected: selected.map(|option| OracleNeowOptionV2 {
                category: option.category,
                reward_id: option.reward_id,
                drawback_id: option.drawback_id,
                label: option.label,
            }),
        });
    let state = OracleStateV2 {
        schema: OracleStateSchemaVersion::current(),
        floor: engine.run_state.floor,
        act: engine.run_state.act,
        turn: combat.map_or(1, |combat| combat.state.turn.max(1)),
        phase: phase_label(engine.current_phase()).to_string(),
        map: OracleMapPositionV2 {
            x: engine.run_state.map_x,
            y: engine.run_state.map_y,
        },
        keys: OracleKeyStateV2 {
            ruby: engine.run_state.has_ruby_key,
            emerald: engine.run_state.has_emerald_key,
            sapphire: engine.run_state.has_sapphire_key,
        },
        player,
        enemies,
        piles: OraclePileStateV2 {
            hand: post.piles.hand,
            draw_ordered: post.piles.draw_ordered,
            discard: post.piles.discard,
            exhaust: post.piles.exhaust,
        },
        deck: engine.run_state.deck.clone(),
        relics: post
            .relics
            .into_iter()
            .map(|relic| OracleRelicStateV2 {
                id: relic.id,
                counter: relic.counter,
            })
            .collect(),
        potions: post.potions,
        rng: OracleRngStateV2::from_engine(engine)?,
        neow,
    };
    state.validate()?;
    Ok(state)
}

impl OracleRngStateV2 {
    fn from_engine(engine: &RunEngine) -> Result<Self, String> {
        let counters = engine.rng_counters();
        let get = |name: &str| {
            counters
                .get(name)
                .copied()
                .ok_or_else(|| format!("engine omitted required RNG counter {name:?}"))
        };
        let (ambient_seed0, ambient_seed1) = engine.ambient_math_rng_state();
        let raw_states = engine
            .rng_state_tuples()
            .into_iter()
            .map(|(name, (seed0, seed1, counter))| {
                (
                    name,
                    OracleCountedRngStateV2 {
                        seed0: format!("{seed0:016x}"),
                        seed1: format!("{seed1:016x}"),
                        counter: i64::from(counter),
                    },
                )
            })
            .collect();
        Ok(Self {
            card: get("card")?,
            monster: get("monster")?,
            event: get("event")?,
            relic: get("relic")?,
            treasure: get("treasure")?,
            potion: get("potion")?,
            merchant: get("merchant")?,
            monster_hp: get("monsterHp")?,
            ai: get("ai")?,
            shuffle: get("shuffle")?,
            card_random: get("cardRandom")?,
            misc: get("misc")?,
            map: get("map")?,
            ambient_math: OracleRandomXs128StateV2 {
                seed0: format!("{ambient_seed0:016x}"),
                seed1: format!("{ambient_seed1:016x}"),
            },
            java_collections: format!("{:012x}", engine.java_collections_rng_state()),
            raw_states,
        })
    }
}

pub fn diff_oracle_states(
    expected: &OracleStateV2,
    actual: &OracleStateV2,
) -> Vec<OracleStateFieldDiff> {
    let mut expected = serde_json::to_value(expected).expect("oracle state must serialize");
    let mut actual = serde_json::to_value(actual).expect("oracle state must serialize");
    let expected_rng = expected
        .as_object_mut()
        .and_then(|object| object.remove("rng"))
        .expect("oracle state must contain rng");
    let actual_rng = actual
        .as_object_mut()
        .and_then(|object| object.remove("rng"))
        .expect("oracle state must contain rng");
    let mut diffs = Vec::new();
    collect_json_diffs("rng", &expected_rng, &actual_rng, &mut diffs);
    collect_json_diffs("", &expected, &actual, &mut diffs);
    diffs
}

/// Compare a recorder projection that may omit not-yet-supported fields.
///
/// Missing recorder object fields are counted, never defaulted. Arrays are
/// atomic collections in the wire contract: a length mismatch is a real
/// divergence rather than partial coverage.
pub fn diff_partial_oracle_state(
    expected_partial: &serde_json::Value,
    actual: &OracleStateV2,
) -> PartialOracleStateDiff {
    let mut expected = expected_partial.clone();
    let mut actual = serde_json::to_value(actual).expect("oracle state must serialize");
    let expected_rng = expected
        .as_object_mut()
        .and_then(|object| object.remove("rng"));
    let actual_rng = actual
        .as_object_mut()
        .and_then(|object| object.remove("rng"))
        .expect("oracle state must contain rng");
    let mut result = PartialOracleStateDiff {
        diffs: Vec::new(),
        skipped_fields_by_path: BTreeMap::new(),
    };
    if let Some(expected_rng) = expected_rng {
        collect_partial_json_diffs("rng", &expected_rng, &actual_rng, &mut result);
    } else {
        count_skipped_leaves("rng", &actual_rng, &mut result.skipped_fields_by_path);
    }
    collect_partial_json_diffs("", &expected, &actual, &mut result);
    result
}

fn collect_partial_json_diffs(
    path: &str,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    result: &mut PartialOracleStateDiff,
) {
    match (expected, actual) {
        (serde_json::Value::Object(expected), serde_json::Value::Object(actual)) => {
            for (key, actual_value) in actual {
                let child = child_path(path, key);
                if let Some(expected_value) = expected.get(key) {
                    collect_partial_json_diffs(&child, expected_value, actual_value, result);
                } else {
                    count_skipped_leaves(&child, actual_value, &mut result.skipped_fields_by_path);
                }
            }
            for (key, expected_value) in expected {
                if !actual.contains_key(key) {
                    result.diffs.push(OracleStateFieldDiff {
                        path: child_path(path, key),
                        expected: expected_value.clone(),
                        actual: serde_json::json!("absent"),
                    });
                }
            }
        }
        (serde_json::Value::Array(expected), serde_json::Value::Array(actual)) => {
            for index in 0..expected.len().max(actual.len()) {
                let child = format!("{path}[{index}]");
                match (expected.get(index), actual.get(index)) {
                    (Some(expected), Some(actual)) => {
                        collect_partial_json_diffs(&child, expected, actual, result)
                    }
                    (Some(expected), None) => result.diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: expected.clone(),
                        actual: serde_json::json!("absent"),
                    }),
                    (None, Some(actual)) => result.diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: serde_json::json!("absent"),
                        actual: actual.clone(),
                    }),
                    (None, None) => {}
                }
            }
        }
        _ if expected != actual => result.diffs.push(OracleStateFieldDiff {
            path: path.to_string(),
            expected: expected.clone(),
            actual: actual.clone(),
        }),
        _ => {}
    }
}

fn count_skipped_leaves(
    path: &str,
    value: &serde_json::Value,
    skipped: &mut BTreeMap<String, u64>,
) {
    match value {
        serde_json::Value::Object(object) if !object.is_empty() => {
            for (key, value) in object {
                count_skipped_leaves(&child_path(path, key), value, skipped);
            }
        }
        serde_json::Value::Array(values) if !values.is_empty() => {
            for (index, value) in values.iter().enumerate() {
                count_skipped_leaves(&format!("{path}[{index}]"), value, skipped);
            }
        }
        _ => *skipped.entry(path.to_string()).or_default() += 1,
    }
}

fn child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

fn collect_json_diffs(
    path: &str,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    diffs: &mut Vec<OracleStateFieldDiff>,
) {
    match (expected, actual) {
        (serde_json::Value::Object(expected), serde_json::Value::Object(actual)) => {
            let keys: BTreeSet<&String> = expected.keys().chain(actual.keys()).collect();
            for key in keys {
                let child = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                match (expected.get(key), actual.get(key)) {
                    (Some(expected), Some(actual)) => {
                        collect_json_diffs(&child, expected, actual, diffs)
                    }
                    (Some(expected), None) => diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: expected.clone(),
                        actual: serde_json::json!("absent"),
                    }),
                    (None, Some(actual)) => diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: serde_json::json!("absent"),
                        actual: actual.clone(),
                    }),
                    (None, None) => {}
                }
            }
        }
        (serde_json::Value::Array(expected), serde_json::Value::Array(actual)) => {
            for index in 0..expected.len().max(actual.len()) {
                let child = format!("{path}[{index}]");
                match (expected.get(index), actual.get(index)) {
                    (Some(expected), Some(actual)) => {
                        collect_json_diffs(&child, expected, actual, diffs)
                    }
                    (Some(expected), None) => diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: expected.clone(),
                        actual: serde_json::json!("absent"),
                    }),
                    (None, Some(actual)) => diffs.push(OracleStateFieldDiff {
                        path: child,
                        expected: serde_json::json!("absent"),
                        actual: actual.clone(),
                    }),
                    (None, None) => {}
                }
            }
        }
        _ if expected != actual => diffs.push(OracleStateFieldDiff {
            path: path.to_string(),
            expected: expected.clone(),
            actual: actual.clone(),
        }),
        _ => {}
    }
}

fn phase_label(phase: RunPhase) -> &'static str {
    match phase {
        RunPhase::Neow => "NEOW",
        RunPhase::MapChoice => "MAP",
        RunPhase::Chest => "CHEST",
        RunPhase::Combat => "COMBAT",
        RunPhase::CardReward => "REWARD",
        RunPhase::Campfire => "CAMPFIRE",
        RunPhase::Shop => "SHOP",
        RunPhase::Event => "EVENT",
        RunPhase::Transition => "TRANSITION",
        RunPhase::GameOver => "GAME_OVER",
    }
}
