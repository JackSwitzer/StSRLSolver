//! Lee's Waffle: on pickup, gain 7 Max HP and heal to full.
//!
//! Source: `reference/extracted/methods/relic/Waffle.java` (`onEquip` calls
//! `increaseMaxHp(7, false)` and then `heal(maxHealth)`). Run acquisition is
//! handled in `run.rs` so Mark of the Bloom can block only the heal.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Lee's Waffle",
    name: "Lee's Waffle",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
