//! Trace schema v2 wire types.
//!
//! This module is intentionally schema-only. The `pre`, `post`, initial, and
//! final checkpoint fields remain opaque JSON until the checkpoint stack lands
//! a typed `CoreCheckpoint`. Their presence must not be interpreted as a claim
//! that the current engine can capture or restore complete causal state.

use crate::run::GameAction;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub const TRACE_SCHEMA_NAME: &str = "sts.trace";
pub const TRACE_SCHEMA_MAJOR: u32 = 2;
pub const TRACE_SCHEMA_MINOR: u32 = 0;

/// A checkpoint-shaped payload whose structure is deliberately unspecified in
/// schema 2.0. This alias is replaced by typed core checkpoint data in the next
/// stacked PR.
pub type OpaqueCheckpoint = Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub name: String,
    pub major: u32,
    pub minor: u32,
}

impl SchemaVersion {
    pub fn current() -> Self {
        Self {
            name: TRACE_SCHEMA_NAME.to_string(),
            major: TRACE_SCHEMA_MAJOR,
            minor: TRACE_SCHEMA_MINOR,
        }
    }

    /// V2 readers accept future v2 minor revisions and reject other schema
    /// names or majors. Serde ignores unknown fields, allowing additive minor
    /// revisions to remain readable.
    pub fn validate(&self) -> Result<(), String> {
        if self.name != TRACE_SCHEMA_NAME {
            return Err(format!(
                "unsupported trace schema name {:?} (expected {TRACE_SCHEMA_NAME:?})",
                self.name
            ));
        }
        if self.major != TRACE_SCHEMA_MAJOR {
            return Err(format!(
                "unsupported trace schema major {} (expected {TRACE_SCHEMA_MAJOR})",
                self.major
            ));
        }
        Ok(())
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

/// Machine-readable declarations of optional trace features.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Capabilities(pub Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceEnvelopeV2 {
    pub schema: SchemaVersion,
    #[serde(default)]
    pub capabilities: Capabilities,
    pub producer: Producer,
    pub payload: TracePayloadV2,
}

impl TraceEnvelopeV2 {
    pub fn new(capabilities: Capabilities, producer: Producer, payload: TracePayloadV2) -> Self {
        Self {
            schema: SchemaVersion::current(),
            capabilities,
            producer,
            payload,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.schema.validate()
    }
}

#[derive(Deserialize)]
struct TraceEnvelopeWire {
    schema: SchemaVersion,
    #[serde(default)]
    capabilities: Capabilities,
    producer: Producer,
    payload: TracePayloadV2,
}

impl<'de> Deserialize<'de> for TraceEnvelopeV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TraceEnvelopeWire::deserialize(deserializer)?;
        wire.schema.validate().map_err(serde::de::Error::custom)?;
        Ok(Self {
            schema: wire.schema,
            capabilities: wire.capabilities,
            producer: wire.producer,
            payload: wire.payload,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TracePayloadV2 {
    Header(HeaderV2),
    Transition(TransitionV2),
    End(EndV2),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderV2 {
    pub trace_id: String,
    pub seed: String,
    pub seed_long: i64,
    pub character: String,
    pub ascension: i32,
    pub game_version: String,
    #[serde(default)]
    pub mods: Vec<String>,
    /// Opaque until `CoreCheckpoint` is introduced.
    pub initial_checkpoint: OpaqueCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionV2 {
    pub idx: u64,
    /// Serialized directly from the core action type. V2 has no parallel
    /// trace-only action vocabulary or semantic adapter.
    pub action: GameAction,
    pub outcome: ActionOutcome,
    /// Opaque until `CoreCheckpoint` is introduced.
    pub pre: OpaqueCheckpoint,
    /// Opaque until `CoreCheckpoint` is introduced.
    pub post: OpaqueCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ActionOutcome {
    Accepted,
    Rejected {
        code: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndV2 {
    pub transition_count: u64,
    pub result: String,
    /// Opaque until `CoreCheckpoint` is introduced.
    pub final_checkpoint: OpaqueCheckpoint,
}
