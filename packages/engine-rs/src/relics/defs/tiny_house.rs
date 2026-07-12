//! Tiny House: upgrade one card, gain 5 max HP, then offer 50 gold and a potion.
//!
//! Source: `reference/extracted/methods/relic/TinyHouse.java` performs these
//! effects in `onEquip` and constructs canonical BOSS-tier ID `Tiny House`.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Tiny House",
    name: "Tiny House",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
