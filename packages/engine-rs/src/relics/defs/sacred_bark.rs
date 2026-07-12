//! Sacred Bark: doubles potion potency, including already-owned potions.
//!
//! Source: `reference/extracted/methods/relic/SacredBark.java` calls
//! initializeData on every owned potion when canonical ID `SacredBark` is
//! equipped. Potion potency reads ownership dynamically in the Rust runtime.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "SacredBark",
    name: "Sacred Bark",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
