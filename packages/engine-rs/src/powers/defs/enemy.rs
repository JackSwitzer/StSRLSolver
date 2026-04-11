//! Enemy turn-start power definitions.
//!
//! Powers that trigger at the start of the enemy's turn.
//! Note: Target::Player is used as a proxy for "the entity that owns this
//! power" since the Target enum doesn't have a Self variant. When these
//! are wired to dispatch, the interpreter will need to resolve the target
//! to the correct entity (enemy or player) based on context.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

// ===========================================================================
// Ritual — EnemyTurnStart + NotFirstTurn: gain Strength
// ===========================================================================

static RITUAL_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player, // proxy for "self" (the enemy that owns this power)
    sid::STRENGTH,
    AmountSource::StatusValue(sid::RITUAL),
))];

static RITUAL_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnStart,
    condition: TriggerCondition::NotFirstTurn,
    effects: &RITUAL_EFFECTS,
    counter: None,
}];

pub static DEF_RITUAL: EntityDef = EntityDef {
    id: "ritual",
    name: "Ritual",
    kind: EntityKind::Power,
    triggers: &RITUAL_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Regeneration — EnemyTurnStart: heal HP equal to stacks (turn-based)
// ===========================================================================

static REGENERATION_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::HealHp(
    Target::Player, // proxy for "self" (the enemy that owns this power)
    AmountSource::StatusValue(sid::REGENERATION),
))];

static REGENERATION_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnStart,
    condition: TriggerCondition::Always,
    effects: &REGENERATION_EFFECTS,
    counter: None,
}];

pub static DEF_REGENERATION: EntityDef = EntityDef {
    id: "regeneration",
    name: "Regeneration",
    kind: EntityKind::Power,
    triggers: &REGENERATION_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Growth — EnemyTurnStart: gain Strength and Block
// ===========================================================================

static GROWTH_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player, // proxy for "self"
        sid::STRENGTH,
        AmountSource::StatusValue(sid::GROWTH),
    )),
    Effect::Simple(SimpleEffect::GainBlock(
        AmountSource::StatusValue(sid::GROWTH),
    )),
];

static GROWTH_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnStart,
    condition: TriggerCondition::Always,
    effects: &GROWTH_EFFECTS,
    counter: None,
}];

pub static DEF_GROWTH: EntityDef = EntityDef {
    id: "growth",
    name: "Growth",
    kind: EntityKind::Power,
    triggers: &GROWTH_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Metallicize (Enemy) — EnemyTurnStart: gain block equal to stacks
// ===========================================================================

static METALLICIZE_ENEMY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::METALLICIZE),
))];

static METALLICIZE_ENEMY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnStart,
    condition: TriggerCondition::Always,
    effects: &METALLICIZE_ENEMY_EFFECTS,
    counter: None,
}];

pub static DEF_METALLICIZE_ENEMY: EntityDef = EntityDef {
    id: "metallicize_enemy",
    name: "Metallicize (Enemy)",
    kind: EntityKind::Power,
    triggers: &METALLICIZE_ENEMY_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ritual_skips_first_turn() {
        assert_eq!(DEF_RITUAL.triggers[0].trigger, Trigger::EnemyTurnStart);
        assert_eq!(
            DEF_RITUAL.triggers[0].condition,
            TriggerCondition::NotFirstTurn
        );
    }

    #[test]
    fn test_regeneration_always_fires() {
        assert_eq!(DEF_REGENERATION.triggers[0].condition, TriggerCondition::Always);
    }

    #[test]
    fn test_growth_has_two_effects() {
        assert_eq!(DEF_GROWTH.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_all_enemy_defs_fire_on_enemy_turn() {
        let defs = [&DEF_RITUAL, &DEF_REGENERATION, &DEF_GROWTH, &DEF_METALLICIZE_ENEMY];
        for def in &defs {
            assert_eq!(def.triggers[0].trigger, Trigger::EnemyTurnStart);
        }
    }
}
