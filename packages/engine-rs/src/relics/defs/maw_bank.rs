//! Maw Bank: gain 12 gold on room entry until any gold is spent.
//!
//! Source: `reference/extracted/methods/relic/MawBank.java`. `onEnterRoom`
//! gains 12 while active; `onSpendGold` permanently sets the used-up counter.
//! Run-level room and spending hooks live in `run.rs`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "MawBank",
    name: "Maw Bank",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
