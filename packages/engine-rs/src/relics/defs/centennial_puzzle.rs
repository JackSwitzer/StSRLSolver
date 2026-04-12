//! Centennial Puzzle: Draw 3 cards on first HP loss per combat.
//!
//! complex_hook needed: one-shot flag (CENTENNIAL_PUZZLE_READY) check
//! and engine.draw_cards(3). Old dispatch: checks CENTENNIAL_PUZZLE_READY,
//! sets CENTENNIAL_PUZZLE_DRAW = 3 for engine to handle.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
        condition: TriggerCondition::HasStatus(sid::CENTENNIAL_PUZZLE_READY),
        effects: &[], // complex_hook handles draw + flag reset
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Centennial Puzzle",
    name: "Centennial Puzzle",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for draw + flag reset
    status_guard: None,
};
