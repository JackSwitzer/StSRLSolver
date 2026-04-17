//! Card-play power definitions.
//!
//! Powers that trigger when the player plays a card.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::orbs::OrbType;
use crate::status_ids::sid;

// ===========================================================================
// After Image — OnAnyCardPlayed: gain block equal to stacks
// ===========================================================================

static AFTER_IMAGE_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::AFTER_IMAGE),
))];

static AFTER_IMAGE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardPlayedPre,
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
    status_guard: Some(sid::AFTER_IMAGE),
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
    status_guard: Some(sid::RAGE),
};

// ===========================================================================
// Heatsink — OnPowerPlayed: draw cards equal to stacks
// ===========================================================================

static HEATSINK_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::StatusValue(sid::HEATSINK),
))];

static HEATSINK_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnPowerPlayed,
    condition: TriggerCondition::Always,
    effects: &HEATSINK_EFFECTS,
    counter: None,
}];

pub static DEF_HEATSINK: EntityDef = EntityDef {
    id: "heatsink",
    name: "Heatsink",
    kind: EntityKind::Power,
    triggers: &HEATSINK_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::HEATSINK),
};

// ===========================================================================
// Storm — OnPowerPlayed: channel Lightning (complex_hook)
// ===========================================================================

static STORM_HOOK_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnPowerPlayed,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_storm(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::OnPowerPlayed && event.card_type == Some(crate::cards::CardType::Power) {
        engine.channel_orb(OrbType::Lightning);
    }
}

pub static DEF_STORM: EntityDef = EntityDef {
    id: "storm",
    name: "Storm",
    kind: EntityKind::Power,
    triggers: &STORM_HOOK_TRIGGERS,
    complex_hook: Some(hook_storm),
    status_guard: Some(sid::STORM),
};

// ===========================================================================
// Curiosity — OnPowerPlayed: enemy gains Strength (Awakened One)
// ===========================================================================

static CURIOSITY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::SelfEntity,
    sid::STRENGTH,
    AmountSource::StatusValue(sid::CURIOSITY),
))];

static CURIOSITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnPowerPlayed,
    condition: TriggerCondition::Always,
    effects: &CURIOSITY_EFFECTS,
    counter: None,
}];

pub static DEF_CURIOSITY: EntityDef = EntityDef {
    id: "curiosity",
    name: "Curiosity",
    kind: EntityKind::Power,
    triggers: &CURIOSITY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::CURIOSITY),
};

// ===========================================================================
// Beat of Death — OnAnyCardPlayed: deal damage to player per enemy stacks
// ===========================================================================

static BEAT_OF_DEATH_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DealDamage(
    Target::Player,
    AmountSource::StatusValue(sid::BEAT_OF_DEATH),
))];

static BEAT_OF_DEATH_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAnyCardPlayed,
    condition: TriggerCondition::Always,
    effects: &BEAT_OF_DEATH_EFFECTS,
    counter: None,
}];

pub static DEF_BEAT_OF_DEATH: EntityDef = EntityDef {
    id: "beat_of_death",
    name: "Beat of Death",
    kind: EntityKind::Power,
    triggers: &BEAT_OF_DEATH_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BEAT_OF_DEATH),
};

// ===========================================================================
// Slow — OnAnyCardPlayed: increment counter (damage calc modifier)
// ===========================================================================

static SLOW_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::IncrementCounter(
    sid::SLOW,
    1,
))];

static SLOW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAnyCardPlayed,
    condition: TriggerCondition::Always,
    effects: &SLOW_EFFECTS,
    counter: None,
}];

pub static DEF_SLOW: EntityDef = EntityDef {
    id: "slow",
    name: "Slow",
    kind: EntityKind::Power,
    triggers: &SLOW_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::SLOW),
};

// ===========================================================================
// Forcefield — OnAnyCardPlayed: decrement stacks by 1
// ===========================================================================

static FORCEFIELD_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::SelfEntity,
    sid::FORCEFIELD,
    AmountSource::Fixed(-1),
))];

static FORCEFIELD_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAnyCardPlayed,
    condition: TriggerCondition::Always,
    effects: &FORCEFIELD_EFFECTS,
    counter: None,
}];

pub static DEF_FORCEFIELD: EntityDef = EntityDef {
    id: "forcefield",
    name: "Forcefield",
    kind: EntityKind::Power,
    triggers: &FORCEFIELD_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::FORCEFIELD),
};

// ===========================================================================
// Skill Burn — OnSkillPlayed: deal damage to player
// ===========================================================================

static SKILL_BURN_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DealDamage(
    Target::Player,
    AmountSource::StatusValue(sid::SKILL_BURN),
))];

static SKILL_BURN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnSkillPlayed,
    condition: TriggerCondition::Always,
    effects: &SKILL_BURN_EFFECTS,
    counter: None,
}];

pub static DEF_SKILL_BURN: EntityDef = EntityDef {
    id: "skill_burn",
    name: "Skill Burn",
    kind: EntityKind::Power,
    triggers: &SKILL_BURN_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::SKILL_BURN),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_after_image_fires_on_any_card() {
        assert_eq!(DEF_AFTER_IMAGE.triggers[0].trigger, Trigger::OnCardPlayedPre);
    }

    #[test]
    fn test_rage_fires_on_attack() {
        assert_eq!(DEF_RAGE.triggers[0].trigger, Trigger::OnAttackPlayed);
    }

    #[test]
    fn test_heatsink_fires_on_power_played() {
        assert_eq!(DEF_HEATSINK.triggers[0].trigger, Trigger::OnPowerPlayed);
    }

    #[test]
    fn test_curiosity_fires_on_power_played() {
        assert_eq!(DEF_CURIOSITY.triggers[0].trigger, Trigger::OnPowerPlayed);
    }

    #[test]
    fn test_beat_of_death_fires_on_any_card() {
        assert_eq!(DEF_BEAT_OF_DEATH.triggers[0].trigger, Trigger::OnAnyCardPlayed);
    }

    #[test]
    fn test_skill_burn_fires_on_skill() {
        assert_eq!(DEF_SKILL_BURN.triggers[0].trigger, Trigger::OnSkillPlayed);
    }

    #[test]
    fn test_storm_is_complex() {
        assert!(DEF_STORM.complex_hook.is_some());
        assert_eq!(DEF_STORM.triggers[0].trigger, Trigger::OnPowerPlayed);
    }
}

#[cfg(test)]
#[path = "../../tests/test_power_runtime_card_play.rs"]
mod test_power_runtime_card_play;
