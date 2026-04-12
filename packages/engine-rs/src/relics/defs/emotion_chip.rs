//! Emotion Chip: On HP loss, trigger front orb passive next turn.
//!
//! complex_hook needed: requires triggering orb passive via engine.
//! Old dispatch: sets EMOTION_CHIP_TRIGGER = 1 flag.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
        condition: TriggerCondition::Always,
        effects: &[], // complex_hook handles orb passive trigger
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Emotion Chip",
    name: "Emotion Chip",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for orb passive trigger
    status_guard: None,
};
