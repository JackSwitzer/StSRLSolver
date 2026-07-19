//! Trace schema v2 wire types.
//!
use crate::checkpoint::CoreCheckpoint;
use crate::run::GameAction;
use serde::{Deserialize, Deserializer, Serialize};

pub const TRACE_SCHEMA_NAME: &str = "sts.trace";
pub const TRACE_SCHEMA_MAJOR: u32 = 2;
pub const TRACE_SCHEMA_MINOR: u32 = 0;

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
        if self.minor > TRACE_SCHEMA_MINOR {
            return Err(format!(
                "unsupported trace schema minor {} (maximum supported {TRACE_SCHEMA_MINOR})",
                self.minor
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
        self.schema.validate()?;
        self.payload.validate()
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
        let envelope = Self {
            schema: wire.schema,
            capabilities: wire.capabilities,
            producer: wire.producer,
            payload: wire.payload,
        };
        envelope.validate().map_err(serde::de::Error::custom)?;
        Ok(envelope)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TracePayloadV2 {
    Header(HeaderV2),
    Transition(TransitionV2),
    End(EndV2),
}

impl TracePayloadV2 {
    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Header(header) => {
                let engine = header.initial_checkpoint.engine();
                if header.seed_long as u64 != engine.seed {
                    return Err("trace header seed does not match its initial checkpoint".to_string());
                }
                if header.ascension != engine.run_state.ascension {
                    return Err(
                        "trace header ascension does not match its initial checkpoint".to_string(),
                    );
                }
            }
            Self::Transition(transition) => {
                if matches!(&transition.outcome, ActionOutcome::Rejected { .. })
                    && transition.pre != transition.post
                {
                    return Err(
                        "rejected trace transition must preserve its causal checkpoint".to_string(),
                    );
                }
            }
            Self::End(_) => {}
        }
        Ok(())
    }
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
    pub initial_checkpoint: CoreCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionV2 {
    pub idx: u64,
    /// Serialized directly from the core action type. V2 has no parallel
    /// trace-only action vocabulary or semantic adapter.
    pub action: GameAction,
    pub outcome: ActionOutcome,
    pub pre: CoreCheckpoint,
    pub post: CoreCheckpoint,
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
    pub final_checkpoint: CoreCheckpoint,
}
