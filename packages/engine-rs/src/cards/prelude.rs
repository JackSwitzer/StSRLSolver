//! Shared imports for per-card definition files.
#![allow(unused_imports)]

pub use std::collections::HashMap;
pub use super::{CardSpec as CardDef, CardType, CardTarget};
pub(crate) use super::insert;
pub use crate::effects::declarative::{
    Effect as E, SimpleEffect as SE, Target as T, AmountSource as A,
    Pile as P, Condition as Cond, BoolFlag as BF,
    CardFilter, ChoiceAction, BulkAction,
};
pub use crate::status_ids::sid;
pub use crate::state::Stance;
pub use crate::orbs::OrbType;
