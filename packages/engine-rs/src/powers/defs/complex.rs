//! Complex power definitions.
//!
//! Powers whose behavior cannot be fully expressed as declarative effects.
//! These use `complex_hook` fn pointers for their primary logic.
//! The triggers array may still contain declarative parts where applicable.
//!
//! Complex powers include:
//! - Card replay logic: Echo Form, Double Tap, Burst, Necronomicon
//! - On-attacked reactions: Thorns, Flame Barrier
//! - On-debuff-applied reactions: Envenom, Sadistic Nature
//! - Per-card triggers with side effects: Thousand Cuts, Panache, Electrodynamics

use crate::effects::entity_def::{EntityDef, EntityKind};
use crate::effects::trigger::TriggerContext;
use crate::engine::CombatEngine;
use crate::status_ids::sid;

// ===========================================================================
// Complex hook stubs
// ===========================================================================
// These are placeholder hooks. When power dispatch is wired through EntityDef,
// each hook will contain the actual logic currently in engine.rs / combat_hooks.rs.
// For now they are no-ops to satisfy the type system.

fn hook_noop(_engine: &mut CombatEngine, _ctx: &TriggerContext) {}

// ===========================================================================
// Echo Form — replays the first card played each turn
// ===========================================================================

pub static DEF_ECHO_FORM: EntityDef = EntityDef {
    id: "echo_form",
    name: "Echo Form",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::ECHO_FORM),
};

// ===========================================================================
// Double Tap — replays the next Attack played this turn
// ===========================================================================

pub static DEF_DOUBLE_TAP: EntityDef = EntityDef {
    id: "double_tap",
    name: "Double Tap",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::DOUBLE_TAP),
};

// ===========================================================================
// Burst — replays the next Skill played this turn
// ===========================================================================

pub static DEF_BURST: EntityDef = EntityDef {
    id: "burst",
    name: "Burst",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::BURST),
};

// ===========================================================================
// Thorns — deal damage back when attacked
// ===========================================================================

pub static DEF_THORNS: EntityDef = EntityDef {
    id: "thorns",
    name: "Thorns",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::THORNS),
};

// ===========================================================================
// Flame Barrier — deal damage back when attacked this turn
// ===========================================================================

pub static DEF_FLAME_BARRIER: EntityDef = EntityDef {
    id: "flame_barrier",
    name: "Flame Barrier",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::FLAME_BARRIER),
};

// ===========================================================================
// Envenom — apply Poison when dealing unblocked Attack damage
// ===========================================================================

pub static DEF_ENVENOM: EntityDef = EntityDef {
    id: "envenom",
    name: "Envenom",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::ENVENOM),
};

// ===========================================================================
// Sadistic Nature — deal damage when applying a debuff
// ===========================================================================

pub static DEF_SADISTIC_NATURE: EntityDef = EntityDef {
    id: "sadistic_nature",
    name: "Sadistic Nature",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::SADISTIC),
};

// ===========================================================================
// Thousand Cuts — deal damage to all enemies on card play
// ===========================================================================

pub static DEF_THOUSAND_CUTS: EntityDef = EntityDef {
    id: "thousand_cuts",
    name: "Thousand Cuts",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::THOUSAND_CUTS),
};

// ===========================================================================
// Panache — deal damage every 5 cards played
// ===========================================================================

pub static DEF_PANACHE: EntityDef = EntityDef {
    id: "panache",
    name: "Panache",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::PANACHE),
};

// ===========================================================================
// Electrodynamics — Lightning orbs hit all enemies
// ===========================================================================

pub static DEF_ELECTRODYNAMICS: EntityDef = EntityDef {
    id: "electrodynamics",
    name: "Electrodynamics",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::ELECTRODYNAMICS),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_complex_have_hooks() {
        let defs = [
            &DEF_ECHO_FORM, &DEF_DOUBLE_TAP, &DEF_BURST,
            &DEF_THORNS, &DEF_FLAME_BARRIER, &DEF_ENVENOM,
            &DEF_SADISTIC_NATURE, &DEF_THOUSAND_CUTS,
            &DEF_PANACHE, &DEF_ELECTRODYNAMICS,
        ];
        for def in &defs {
            assert!(
                def.complex_hook.is_some(),
                "Complex power '{}' missing complex_hook",
                def.id
            );
        }
    }

    #[test]
    fn test_complex_have_empty_triggers() {
        // Complex powers currently have no declarative triggers
        let defs = [
            &DEF_ECHO_FORM, &DEF_DOUBLE_TAP, &DEF_BURST,
            &DEF_THORNS, &DEF_FLAME_BARRIER,
        ];
        for def in &defs {
            assert!(
                def.triggers.is_empty(),
                "Complex power '{}' should have empty triggers (logic is in hook)",
                def.id
            );
        }
    }
}
