//! Enemy turn-start power definitions.
//!
//! Powers that trigger at the start of the enemy's turn.
//! Note: Target::Player is used as a proxy for "the entity that owns this
//! power" since the Target enum doesn't have a Self variant. When these
//! are wired to dispatch, the interpreter will need to resolve the target
//! to the correct entity (enemy or player) based on context.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

// ===========================================================================
// Ritual — EnemyTurnStart + NotFirstTurn: gain Strength
// ===========================================================================

static RITUAL_TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::EnemyTurnStart,
        condition: TriggerCondition::NotFirstTurn,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnEnd,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

fn ritual_hook(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    // RitualPower.java uses distinct boundaries by owner: player-controlled
    // Ritual gains Strength at player turn end, while enemy Ritual gains it at
    // end of round after skipping its first round.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RitualPower.java
    match (owner, event.kind) {
        (EffectOwner::PlayerPower, Trigger::TurnEnd) => {
            let amount = engine.state.player.status(sid::RITUAL);
            engine.state.player.add_status(sid::STRENGTH, amount);
        }
        (EffectOwner::EnemyPower { enemy_idx }, Trigger::EnemyTurnStart) => {
            let idx = enemy_idx as usize;
            if idx < engine.state.enemies.len() {
                let amount = engine.state.enemies[idx].entity.status(sid::RITUAL);
                engine.state.enemies[idx].entity.add_status(sid::STRENGTH, amount);
            }
        }
        _ => {}
    }
}

pub static DEF_RITUAL: EntityDef = EntityDef {
    id: "ritual",
    name: "Ritual",
    kind: EntityKind::Power,
    triggers: &RITUAL_TRIGGERS,
    complex_hook: Some(ritual_hook),
    status_guard: Some(sid::RITUAL),
};

// ===========================================================================
// Regeneration — EnemyTurnEnd: heal HP equal to stacks after all monsters act
// ===========================================================================

static REGENERATION_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::HealHp(
    Target::SelfEntity,
    AmountSource::StatusValue(sid::REGENERATION),
))];

static REGENERATION_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnEnd,
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
    status_guard: Some(sid::REGENERATION),
};

// ===========================================================================
// Growth — EnemyTurnStart: gain Strength and Block
// ===========================================================================

static GROWTH_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(
        Target::SelfEntity,
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
    status_guard: Some(sid::GROWTH),
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
    status_guard: Some(sid::METALLICIZE),
};

// ===========================================================================
// Plated Armor (Enemy) — gain block after the monster group acts
// ===========================================================================

static PLATED_ARMOR_ENEMY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainBlock(
    AmountSource::StatusValue(sid::PLATED_ARMOR),
))];

static PLATED_ARMOR_ENEMY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::EnemyTurnEnd,
    condition: TriggerCondition::Always,
    effects: &PLATED_ARMOR_ENEMY_EFFECTS,
    counter: None,
}];

pub static DEF_PLATED_ARMOR_ENEMY: EntityDef = EntityDef {
    id: "plated_armor_enemy",
    name: "Plated Armor (Enemy)",
    kind: EntityKind::Power,
    triggers: &PLATED_ARMOR_ENEMY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::PLATED_ARMOR),
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
        assert_eq!(DEF_REGENERATION.triggers[0].trigger, Trigger::EnemyTurnEnd);
    }

    #[test]
    fn test_growth_has_two_effects() {
        assert_eq!(DEF_GROWTH.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_all_enemy_defs_fire_on_enemy_turn() {
        let defs = [&DEF_RITUAL, &DEF_GROWTH, &DEF_METALLICIZE_ENEMY];
        for def in &defs {
            assert_eq!(def.triggers[0].trigger, Trigger::EnemyTurnStart);
        }
    }
}
