//! Meal Ticket: heal 15 when entering a shop room.
//!
//! Source: decompiled `relics/MealTicket.java` (`justEnteredRoom` checks
//! `ShopRoom` and calls `player.heal(15)`). Run-level handling lives in `run.rs`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "MealTicket",
    name: "Meal Ticket",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
