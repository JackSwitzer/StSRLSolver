//! Turn-end power definitions.
//!
//! Powers that trigger at the end of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::state::Stance;
use crate::status_ids::sid;

// ===========================================================================
// Metallicize — TurnEnd: gain block equal to stacks
// ===========================================================================

static METALLICIZE_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::METALLICIZE),
))];

static METALLICIZE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &METALLICIZE_EFFECTS,
    counter: None,
}];

pub static DEF_METALLICIZE: EntityDef = EntityDef {
    id: "metallicize",
    name: "Metallicize",
    kind: EntityKind::Power,
    triggers: &METALLICIZE_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Plated Armor — TurnEnd: gain block equal to stacks
// ===========================================================================

static PLATED_ARMOR_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::PLATED_ARMOR),
))];

static PLATED_ARMOR_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &PLATED_ARMOR_EFFECTS,
    counter: None,
}];

pub static DEF_PLATED_ARMOR: EntityDef = EntityDef {
    id: "plated_armor",
    name: "Plated Armor",
    kind: EntityKind::Power,
    triggers: &PLATED_ARMOR_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Combust — TurnEnd: lose 1 HP, deal damage to all enemies
// ===========================================================================

static COMBUST_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::DealDamage(
        Target::Player,
        AmountSource::Fixed(1),
    )),
    Effect::Simple(SimpleEffect::DealDamage(
        Target::AllEnemies,
        AmountSource::StatusValue(sid::COMBUST),
    )),
];

static COMBUST_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &COMBUST_EFFECTS,
    counter: None,
}];

pub static DEF_COMBUST: EntityDef = EntityDef {
    id: "combust",
    name: "Combust",
    kind: EntityKind::Power,
    triggers: &COMBUST_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Omega — TurnEnd: deal 50 damage to all enemies
// ===========================================================================

static OMEGA_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DealDamage(
    Target::AllEnemies,
    AmountSource::StatusValue(sid::OMEGA),
))];

static OMEGA_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &OMEGA_EFFECTS,
    counter: None,
}];

pub static DEF_OMEGA: EntityDef = EntityDef {
    id: "omega",
    name: "Omega",
    kind: EntityKind::Power,
    triggers: &OMEGA_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Like Water — TurnEnd: gain block if in Calm stance
// ===========================================================================

static LIKE_WATER_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::LIKE_WATER),
))];

static LIKE_WATER_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::InStance(Stance::Calm),
    effects: &LIKE_WATER_EFFECTS,
    counter: None,
}];

pub static DEF_LIKE_WATER: EntityDef = EntityDef {
    id: "like_water",
    name: "Like Water",
    kind: EntityKind::Power,
    triggers: &LIKE_WATER_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Study — TurnEnd: add Insight(s) to draw pile
// ===========================================================================

static STUDY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Insight",
    Pile::Draw,
    AmountSource::StatusValue(sid::STUDY),
))];

static STUDY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &STUDY_EFFECTS,
    counter: None,
}];

pub static DEF_STUDY: EntityDef = EntityDef {
    id: "study",
    name: "Study",
    kind: EntityKind::Power,
    triggers: &STUDY_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metallicize_def() {
        assert_eq!(DEF_METALLICIZE.triggers.len(), 1);
        assert_eq!(DEF_METALLICIZE.triggers[0].trigger, Trigger::TurnEnd);
    }

    #[test]
    fn test_combust_has_two_effects() {
        assert_eq!(DEF_COMBUST.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_like_water_requires_calm() {
        assert_eq!(
            DEF_LIKE_WATER.triggers[0].condition,
            TriggerCondition::InStance(Stance::Calm)
        );
    }

    #[test]
    fn test_all_turn_end_defs() {
        let defs = [
            &DEF_METALLICIZE, &DEF_PLATED_ARMOR, &DEF_COMBUST,
            &DEF_OMEGA, &DEF_LIKE_WATER, &DEF_STUDY,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert_eq!(def.triggers[0].trigger, Trigger::TurnEnd);
        }
    }
}
