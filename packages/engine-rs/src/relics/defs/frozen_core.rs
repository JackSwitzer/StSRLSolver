//! Frozen Core: If no orbs channeled at end of turn, channel 1 Frost.
//!
//! complex_hook needed: requires checking orb slots and calling engine.channel_orb().
//! Old dispatch: sets FROZEN_CORE_TRIGGER flag, engine handles orb channeling.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "FrozenCore",
    name: "Frozen Core",
    kind: EntityKind::Relic,
    // Cannot be declarative: needs to check if orb slots are empty and
    // call engine.channel_orb(OrbType::Frost). Requires complex_hook.
    triggers: &[],
    complex_hook: None, // TODO: wire complex_hook for orb channeling
    status_guard: None,
};
