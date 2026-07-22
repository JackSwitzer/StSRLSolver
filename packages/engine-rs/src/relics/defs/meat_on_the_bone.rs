//! Meat on the Bone: Heal 12 HP on victory if HP <= 50%.
//!
//! Source: `reference/extracted/methods/relic/MeatOnTheBone.java` (`onTrigger`
//! requires positive HP at or below maxHealth / 2.0, then calls `heal(12)`).

use crate::effects::entity_def::{EntityDef, EntityKind};

// Meat on the Bone is not an AbstractRelic.onVictory callback. AbstractRoom
// invokes its standalone onTrigger before AbstractPlayer dispatches onVictory,
// so CombatEngine::apply_player_on_victory owns the ordering-sensitive heal.
// Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
// (`endBattle`) and relics/MeatOnTheBone.java (`onTrigger`).

pub static DEF: EntityDef = EntityDef {
    id: "Meat on the Bone",
    name: "Meat on the Bone",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
