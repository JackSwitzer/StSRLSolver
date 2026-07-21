//! Frozen Eye: reveals the draw pile's actual order.
//!
//! Sources: `reference/extracted/methods/relic/FrozenEye.java` defines the
//! canonical SHOP relic; `DrawPileViewScreen.java` preserves pile order only
//! when the player owns it. The information effect lives in CombatContext.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Frozen Eye",
    name: "Frozen Eye",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
