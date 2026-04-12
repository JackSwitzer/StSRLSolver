//! Mummified Hand: On Power play, reduce a random hand card's cost to 0.
//!
//! complex_hook needed: requires picking a random card from hand and
//! modifying its cost. Old dispatch sets MUMMIFIED_HAND_TRIGGER flag.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPowerPlayed,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Mummified Hand",
    name: "Mummified Hand",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for random card cost reduction
    status_guard: None,
};
