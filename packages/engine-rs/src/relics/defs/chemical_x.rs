//! Chemical X: X-cost effects receive +2.
//!
//! Source: `reference/extracted/methods/relic/ChemicalX.java` defines the
//! canonical ID and SHOP tier. The active +2 behavior is applied by the
//! card-effect X-cost pipeline, matching the individual Java X-cost actions.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Chemical X",
    name: "Chemical X",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
