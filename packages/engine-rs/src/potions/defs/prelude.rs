//! Shared imports for per-potion EntityDef files.
#![allow(unused_imports)]

pub use crate::effects::declarative::{
    Effect as E, SimpleEffect as SE, Target as T, AmountSource as A,
};
pub use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
pub use crate::effects::trigger::{Trigger, TriggerCondition};
pub use crate::status_ids::sid;
