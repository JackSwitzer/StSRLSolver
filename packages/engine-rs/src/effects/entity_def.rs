//! Entity definitions for the unified effect system.
//!
//! An `EntityDef` describes a relic, power, or potion as a static data
//! structure with declarative triggered effects and an optional complex hook.
//! This mirrors the declarative card effect pattern: most behavior is data,
//! with fn pointers only for irreducible logic.

use crate::effects::declarative::Effect;
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::ids::StatusId;
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};

// ===========================================================================
// TriggeredEffect — a single trigger -> effects binding
// ===========================================================================

/// A triggered effect: fires when `trigger` occurs and `condition` is met,
/// then executes `effects`. Optionally tracks a counter status.
#[derive(Debug, Clone, Copy)]
pub struct TriggeredEffect {
    /// When this effect fires.
    pub trigger: Trigger,
    /// Additional condition that must be true.
    pub condition: TriggerCondition,
    /// The declarative effects to execute.
    pub effects: &'static [Effect],
    /// Optional counter: (counter_status, threshold).
    /// The counter is incremented on each trigger; effects only fire
    /// when the counter reaches the threshold, then it resets to 0.
    pub counter: Option<(StatusId, i32)>,
}

// ===========================================================================
// EntityKind — what type of entity this definition describes
// ===========================================================================

/// The kind of entity (relic, power, potion).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Relic,
    Power,
    Potion,
}

// ===========================================================================
// EntityDef — the full definition of a relic/power/potion
// ===========================================================================

/// A static definition of a relic, power, or potion.
///
/// Most behavior is expressed as `TriggeredEffect` entries in `triggers`.
/// Only irreducible logic (e.g., modifying damage pipeline, complex
/// multi-step interactions) uses the `complex_hook` fn pointer.
#[derive(Debug, Clone, Copy)]
pub struct EntityDef {
    /// Unique string identifier (e.g., "Pen Nib", "Noxious Fumes").
    pub id: &'static str,
    /// Human-readable display name.
    pub name: &'static str,
    /// What kind of entity this is.
    pub kind: EntityKind,
    /// Declarative triggered effects (static slice for const construction).
    pub triggers: &'static [TriggeredEffect],
    /// Optional complex hook for irreducible logic.
    /// Receives the engine, owner, concrete emitted event, and per-instance
    /// mutable runtime state for this installed entity.
    pub complex_hook: Option<fn(&mut CombatEngine, EffectOwner, &GameEvent, &mut EffectState)>,
    /// Optional status guard: if set, skip this entity unless the player
    /// has this status > 0. Used by powers so they only fire when installed.
    pub status_guard: Option<StatusId>,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::declarative::{SimpleEffect, AmountSource};
    use crate::status_ids::sid;

    #[test]
    fn test_entity_kind_is_copy() {
        let k = EntityKind::Relic;
        let _k2 = k;
        let _k3 = k;
    }

    #[test]
    fn test_triggered_effect_is_copy() {
        static EFFECTS: [Effect; 1] = [
            Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Fixed(1))),
        ];
        let te = TriggeredEffect {
            trigger: Trigger::TurnStart,
            condition: TriggerCondition::Always,
            effects: &EFFECTS,
            counter: None,
        };
        let _te2 = te;
        let _te3 = te;
    }

    #[test]
    fn test_entity_def_static_construction() {
        static TRIGGER_EFFECTS: [Effect; 1] = [
            Effect::Simple(SimpleEffect::DrawCards(AmountSource::Fixed(1))),
        ];
        static TRIGGERS: [TriggeredEffect; 1] = [
            TriggeredEffect {
                trigger: Trigger::TurnStart,
                condition: TriggerCondition::Always,
                effects: &TRIGGER_EFFECTS,
                counter: None,
            },
        ];
        static DEF: EntityDef = EntityDef {
            id: "test_relic",
            name: "Test Relic",
            kind: EntityKind::Relic,
            triggers: &TRIGGERS,
            complex_hook: None,
    status_guard: None,
        };
        assert_eq!(DEF.id, "test_relic");
        assert_eq!(DEF.kind, EntityKind::Relic);
        assert_eq!(DEF.triggers.len(), 1);
        assert!(DEF.complex_hook.is_none());
    }

    #[test]
    fn test_entity_def_with_counter() {
        static EFFECTS: [Effect; 1] = [
            Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Fixed(1))),
        ];
        static TRIGGERS: [TriggeredEffect; 1] = [
            TriggeredEffect {
                trigger: Trigger::OnAnyCardPlayed,
                condition: TriggerCondition::Always,
                effects: &EFFECTS,
                counter: Some((sid::PEN_NIB_COUNTER, 10)),
            },
        ];
        let te = TRIGGERS[0];
        assert_eq!(te.counter, Some((sid::PEN_NIB_COUNTER, 10)));
    }

    #[test]
    fn test_entity_def_size() {
        let size = std::mem::size_of::<EntityDef>();
        // Contains &'static str (16) + &'static str (16) + EntityKind (1) +
        // &'static [TriggeredEffect] (16) + Option<fn ptr> (8+8) + padding
        assert!(size <= 80, "EntityDef is {} bytes, expected <= 80", size);
    }
}
