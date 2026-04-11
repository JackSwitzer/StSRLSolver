//! Stance-change power definitions.
//!
//! Powers that trigger when the player changes stance.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::state::Stance;
use crate::status_ids::sid;

// ===========================================================================
// Mental Fortress — OnStanceChange: gain block equal to stacks
// ===========================================================================

static MENTAL_FORTRESS_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::MENTAL_FORTRESS),
))];

static MENTAL_FORTRESS_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnStanceChange,
    condition: TriggerCondition::Always,
    effects: &MENTAL_FORTRESS_EFFECTS,
    counter: None,
}];

pub static DEF_MENTAL_FORTRESS: EntityDef = EntityDef {
    id: "mental_fortress",
    name: "Mental Fortress",
    kind: EntityKind::Power,
    triggers: &MENTAL_FORTRESS_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Rushdown — OnStanceChange + entering Wrath: draw cards equal to stacks
// ===========================================================================

static RUSHDOWN_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::StatusValue(sid::RUSHDOWN),
))];

static RUSHDOWN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnStanceChange,
    condition: TriggerCondition::InStance(Stance::Wrath),
    effects: &RUSHDOWN_EFFECTS,
    counter: None,
}];

pub static DEF_RUSHDOWN: EntityDef = EntityDef {
    id: "rushdown",
    name: "Rushdown",
    kind: EntityKind::Power,
    triggers: &RUSHDOWN_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mental_fortress_fires_on_stance_change() {
        assert_eq!(
            DEF_MENTAL_FORTRESS.triggers[0].trigger,
            Trigger::OnStanceChange
        );
        assert_eq!(
            DEF_MENTAL_FORTRESS.triggers[0].condition,
            TriggerCondition::Always
        );
    }

    #[test]
    fn test_rushdown_requires_wrath() {
        assert_eq!(DEF_RUSHDOWN.triggers[0].trigger, Trigger::OnStanceChange);
        assert_eq!(
            DEF_RUSHDOWN.triggers[0].condition,
            TriggerCondition::InStance(Stance::Wrath)
        );
    }
}
