//! Blue Candle: curse cards become playable (1 HP + exhaust).
//! Passive check via has_relic in card playability pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

pub static DEF: EntityDef = EntityDef {
    id: "Blue Candle",
    name: "Blue Candle",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
