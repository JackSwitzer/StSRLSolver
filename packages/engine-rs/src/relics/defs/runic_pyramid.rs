//! Runic Pyramid: the player does not discard their hand at end of turn.
//!
//! Source: `RunicPyramid.java` defines the canonical relic, while
//! `DiscardAtEndOfTurnAction.java` performs the relic check inline.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Runic Pyramid",
    name: "Runic Pyramid",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
