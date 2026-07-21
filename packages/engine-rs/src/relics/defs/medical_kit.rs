//! Medical Kit: status cards become playable (exhaust on play).
//!
//! Source: `reference/extracted/methods/relic/MedicalKit.java` (`onUseCard`
//! sets both the STATUS card and its UseCardAction to exhaust).
//! Passive check via has_relic in card playability pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Medical Kit",
    name: "Medical Kit",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
