//! White Beast Statue: potion reward chance is 100 percent.
//!
//! Source: `WhiteBeast.java` constructs canonical UNCOMMON ID
//! `White Beast Statue`; `AbstractRoom.java::addPotionToRewards` sets chance
//! to 100 while it is owned.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "White Beast Statue",
    name: "White Beast Statue",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
