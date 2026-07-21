//! Versioned causal checkpoints for deterministic simulator continuation.
//!
//! Checkpoints own the complete future-affecting `RunEngine` state. Static
//! registries and derived indexes are reconstructed during deserialization;
//! observations and emitted-event history are intentionally not part of the
//! causal contract.

use crate::run::RunEngine;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::OnceLock;

pub const CORE_CHECKPOINT_SCHEMA: &str = "sts.core_checkpoint";
pub const CORE_CHECKPOINT_MAJOR: u32 = 2;
pub const CORE_CHECKPOINT_MINOR: u32 = 0;
pub const CORE_SEMANTICS_REVISION: &str = "java-rng-actions-v2-checkpoint-v3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointVersion {
    pub name: String,
    pub major: u32,
    pub minor: u32,
}

impl CheckpointVersion {
    pub fn current() -> Self {
        Self {
            name: CORE_CHECKPOINT_SCHEMA.to_string(),
            major: CORE_CHECKPOINT_MAJOR,
            minor: CORE_CHECKPOINT_MINOR,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name != CORE_CHECKPOINT_SCHEMA {
            return Err(format!(
                "unsupported checkpoint schema name {:?} (expected {CORE_CHECKPOINT_SCHEMA:?})",
                self.name
            ));
        }
        if self.major != CORE_CHECKPOINT_MAJOR {
            return Err(format!(
                "unsupported checkpoint schema major {} (expected {CORE_CHECKPOINT_MAJOR})",
                self.major
            ));
        }
        if self.minor > CORE_CHECKPOINT_MINOR {
            return Err(format!(
                "unsupported checkpoint schema minor {} (maximum supported {CORE_CHECKPOINT_MINOR})",
                self.minor
            ));
        }
        Ok(())
    }
}

impl Default for CheckpointVersion {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Clone, Serialize)]
pub struct CoreCheckpoint {
    pub schema: CheckpointVersion,
    pub semantics_fingerprint: String,
    pub causal_hash: String,
    engine: RunEngine,
}

impl CoreCheckpoint {
    pub fn capture(engine: &RunEngine) -> Result<Self, String> {
        let mut engine = engine.clone();
        engine.validate_checkpoint_state()?;
        engine.rebuild_checkpoint_projections();
        engine.validate_checkpoint_state()?;
        let causal_hash = causal_hash(&engine).map_err(|error| error.to_string())?;
        Ok(Self {
            schema: CheckpointVersion::current(),
            semantics_fingerprint: core_semantics_fingerprint().to_string(),
            causal_hash,
            engine,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.schema.validate()?;
        validate_semantics_fingerprint(&self.semantics_fingerprint)?;
        self.engine.validate_checkpoint_state()?;
        let actual = causal_hash(&self.engine).map_err(|error| error.to_string())?;
        if actual != self.causal_hash {
            return Err(format!(
                "checkpoint causal hash mismatch (stored {}, computed {actual})",
                self.causal_hash
            ));
        }
        Ok(())
    }

    pub fn restore(&self) -> Result<RunEngine, String> {
        self.validate()?;
        let mut engine = self.engine.clone();
        engine.rebuild_checkpoint_projections();
        engine.validate_checkpoint_state()?;
        Ok(engine)
    }

    pub fn into_engine(mut self) -> Result<RunEngine, String> {
        self.validate()?;
        self.engine.rebuild_checkpoint_projections();
        self.engine.validate_checkpoint_state()?;
        Ok(self.engine)
    }

    pub fn engine(&self) -> &RunEngine {
        &self.engine
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl std::fmt::Debug for CoreCheckpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CoreCheckpoint")
            .field("schema", &self.schema)
            .field("semantics_fingerprint", &self.semantics_fingerprint)
            .field("causal_hash", &self.causal_hash)
            .field("phase", &self.engine.phase)
            .field("floor", &self.engine.run_state.floor)
            .finish_non_exhaustive()
    }
}

impl PartialEq for CoreCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.schema == other.schema
            && self.semantics_fingerprint == other.semantics_fingerprint
            && self.causal_hash == other.causal_hash
            && matches!(
                (
                    serde_json::to_vec(&self.engine),
                    serde_json::to_vec(&other.engine),
                ),
                (Ok(left), Ok(right)) if left == right
            )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoreCheckpointWire {
    schema: CheckpointVersion,
    semantics_fingerprint: String,
    causal_hash: String,
    engine: RunEngine,
}

impl<'de> Deserialize<'de> for CoreCheckpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let mut wire: CoreCheckpointWire =
            serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
        wire.schema.validate().map_err(serde::de::Error::custom)?;
        validate_semantics_fingerprint(&wire.semantics_fingerprint)
            .map_err(serde::de::Error::custom)?;
        wire.engine
            .validate_checkpoint_state()
            .map_err(serde::de::Error::custom)?;
        let canonical_engine =
            serde_json::to_value(&wire.engine).map_err(serde::de::Error::custom)?;
        if value.get("engine") != Some(&canonical_engine) {
            return Err(serde::de::Error::custom(
                "checkpoint engine must use the complete canonical causal representation",
            ));
        }
        let actual = causal_hash(&wire.engine).map_err(serde::de::Error::custom)?;
        if actual != wire.causal_hash {
            return Err(serde::de::Error::custom(format!(
                "checkpoint causal hash mismatch (stored {}, computed {actual})",
                wire.causal_hash
            )));
        }
        wire.engine.rebuild_checkpoint_projections();
        Ok(Self {
            schema: wire.schema,
            semantics_fingerprint: wire.semantics_fingerprint,
            causal_hash: wire.causal_hash,
            engine: wire.engine,
        })
    }
}

fn validate_semantics_fingerprint(fingerprint: &str) -> Result<(), String> {
    let expected = core_semantics_fingerprint();
    if fingerprint != expected {
        return Err(format!(
            "checkpoint semantics fingerprint mismatch (stored {fingerprint}, expected {expected})"
        ));
    }
    Ok(())
}

fn core_semantics_fingerprint() -> &'static str {
    static FINGERPRINT: OnceLock<String> = OnceLock::new();
    FINGERPRINT
        .get_or_init(|| core_semantics_fingerprint_for_revision(CORE_SEMANTICS_REVISION))
        .as_str()
}

pub(crate) fn core_semantics_fingerprint_for_revision(revision: &str) -> String {
    let catalog = format!("{:?}", crate::gameplay::global_registry().defs());
    hash_bytes(format!("{revision}\n{catalog}").as_bytes())
}

fn causal_hash(engine: &RunEngine) -> Result<String, serde_json::Error> {
    // RunEngine's causal representation contains only ordered collections. A
    // compact FNV-1a digest catches dropped/defaulted fields and accidental
    // edits without making the checkpoint depend on external crypto crates.
    let bytes = serde_json::to_vec(engine)?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
