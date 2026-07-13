//! Turn-end power definitions.
//!
//! Powers that trigger at the end of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
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
    status_guard: Some(sid::METALLICIZE),
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
    status_guard: Some(sid::PLATED_ARMOR),
};

// ===========================================================================
// Combust — TurnEnd: lose HP, then deal THORNS damage to all enemies
// ===========================================================================

static COMBUST_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_combust(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    if event.kind != Trigger::TurnEnd {
        return;
    }
    let targets = engine.state.living_enemy_indices();
    if targets.is_empty() {
        return;
    }

    // CombustPower.atEndOfTurn queues LoseHPAction before a source-less
    // DamageAllEnemiesAction with DamageType.THORNS. stackPower adds incoming
    // damage to amount but increments the private hpLoss field by one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CombustPower.java
    engine.player_lose_hp_from_damage(state.get(0).max(1));
    let damage = engine.state.player.status(sid::COMBUST);
    for target in targets {
        engine.deal_thorns_damage_to_enemy(target, damage);
    }
}

pub static DEF_COMBUST: EntityDef = EntityDef {
    id: "combust",
    name: "Combust",
    kind: EntityKind::Power,
    triggers: &COMBUST_TRIGGERS,
    complex_hook: Some(hook_combust),
    status_guard: Some(sid::COMBUST),
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
    status_guard: Some(sid::OMEGA),
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
    status_guard: Some(sid::LIKE_WATER),
};

// ===========================================================================
// Study — TurnEnd: add Insight(s) to draw pile
// ===========================================================================

// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/StudyPower.java
// MakeTempCardInDrawPileAction(..., randomSpot=true) uses cardRandomRng for
// each Insight rather than shuffling the existing draw pile.
static STUDY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCardToRandomDrawSpot(
    "Insight",
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
    status_guard: Some(sid::STUDY),
};

// ===========================================================================
// No Draw — TurnEnd: remove the one-turn draw restriction
// ===========================================================================

// Source: powers/NoDrawPower.java::atEndOfTurn queues removal of power ID
// "No Draw" when the player's turn ends.
static NO_DRAW_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::SetStatus(
    Target::Player,
    sid::NO_DRAW,
    AmountSource::Fixed(0),
))];

static NO_DRAW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &NO_DRAW_EFFECTS,
    counter: None,
}];

pub static DEF_NO_DRAW: EntityDef = EntityDef {
    id: "no_draw",
    name: "No Draw",
    kind: EntityKind::Power,
    triggers: &NO_DRAW_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::NO_DRAW),
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
    fn test_combust_uses_ordered_complex_hook() {
        assert!(DEF_COMBUST.triggers[0].effects.is_empty());
        assert!(DEF_COMBUST.complex_hook.is_some());
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
