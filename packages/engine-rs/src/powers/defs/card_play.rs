//! Card-play power definitions.
//!
//! Powers that trigger when the player plays a card.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

// ===========================================================================
// After Image — OnAnyCardPlayed: gain block equal to stacks
// ===========================================================================

static AFTER_IMAGE_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::AFTER_IMAGE),
))];

static AFTER_IMAGE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAnyCardPlayed,
    condition: TriggerCondition::Always,
    effects: &AFTER_IMAGE_EFFECTS,
    counter: None,
}];

pub static DEF_AFTER_IMAGE: EntityDef = EntityDef {
    id: "after_image",
    name: "After Image",
    kind: EntityKind::Power,
    triggers: &AFTER_IMAGE_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Rage — OnAttackPlayed: gain block equal to stacks
// ===========================================================================

static RAGE_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::RAGE),
))];

static RAGE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAttackPlayed,
    condition: TriggerCondition::Always,
    effects: &RAGE_EFFECTS,
    counter: None,
}];

pub static DEF_RAGE: EntityDef = EntityDef {
    id: "rage",
    name: "Rage",
    kind: EntityKind::Power,
    triggers: &RAGE_TRIGGERS,
    complex_hook: None,
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_after_image_fires_on_any_card() {
        assert_eq!(DEF_AFTER_IMAGE.triggers[0].trigger, Trigger::OnAnyCardPlayed);
    }

    #[test]
    fn test_rage_fires_on_attack() {
        assert_eq!(DEF_RAGE.triggers[0].trigger, Trigger::OnAttackPlayed);
    }
}
