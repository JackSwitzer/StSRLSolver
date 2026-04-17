//! Exhaust power definitions.
//!
//! Powers that trigger when a card is exhausted.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

// ===========================================================================
// Feel No Pain — OnCardExhaust: gain block equal to stacks
// ===========================================================================

static FEEL_NO_PAIN_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::FEEL_NO_PAIN),
))];

static FEEL_NO_PAIN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardExhaust,
    condition: TriggerCondition::Always,
    effects: &FEEL_NO_PAIN_EFFECTS,
    counter: None,
}];

pub static DEF_FEEL_NO_PAIN: EntityDef = EntityDef {
    id: "feel_no_pain",
    name: "Feel No Pain",
    kind: EntityKind::Power,
    triggers: &FEEL_NO_PAIN_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::FEEL_NO_PAIN),
};

// ===========================================================================
// Dark Embrace — OnCardExhaust: draw cards equal to stacks
// ===========================================================================

static DARK_EMBRACE_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::StatusValue(sid::DARK_EMBRACE),
))];

static DARK_EMBRACE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardExhaust,
    condition: TriggerCondition::Always,
    effects: &DARK_EMBRACE_EFFECTS,
    counter: None,
}];

pub static DEF_DARK_EMBRACE: EntityDef = EntityDef {
    id: "dark_embrace",
    name: "Dark Embrace",
    kind: EntityKind::Power,
    triggers: &DARK_EMBRACE_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DARK_EMBRACE),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feel_no_pain_fires_on_exhaust() {
        assert_eq!(DEF_FEEL_NO_PAIN.triggers[0].trigger, Trigger::OnCardExhaust);
    }

    #[test]
    fn test_dark_embrace_fires_on_exhaust() {
        assert_eq!(DEF_DARK_EMBRACE.triggers[0].trigger, Trigger::OnCardExhaust);
    }

    #[test]
    fn test_feel_no_pain_gains_block() {
        let effect = &DEF_FEEL_NO_PAIN.triggers[0].effects[0];
        match effect {
            Effect::Simple(SimpleEffect::GainBlock(AmountSource::StatusValue(s))) => {
                assert_eq!(*s, sid::FEEL_NO_PAIN);
            }
            _ => panic!("Expected GainBlock(StatusValue(FEEL_NO_PAIN))"),
        }
    }
}
