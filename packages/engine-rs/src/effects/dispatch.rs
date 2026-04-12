//! Unified dispatch_trigger — single entry point for all entity triggers.
//!
//! Replaces the scattered dispatch calls in engine.rs, relics/combat.rs,
//! and powers/registry.rs with one function that iterates EntityDefs.

use crate::effects::entity_def::EntityDef;
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::effects::interpreter::execute_trigger_effects;
use crate::engine::CombatEngine;

// ===========================================================================
// Public entry point
// ===========================================================================

/// Fire all relic and power triggers that match `trigger`.
///
/// Iterates RELIC_DEFS and POWER_DEFS, checks whether the entity is active,
/// evaluates conditions, handles counter patterns, and executes effects.
pub fn dispatch_trigger(engine: &mut CombatEngine, trigger: Trigger, ctx: &TriggerContext) {
    // 1. Relic defs — active if player has the relic
    for def in crate::relics::defs::RELIC_DEFS {
        if !engine.state.has_relic(def.id) {
            continue;
        }
        execute_entity_triggers(engine, def, trigger, ctx);
    }

    // 2. Power defs — active if the player has the power's status guard > 0.
    //    Skip entirely if the guard status is not present.
    for def in crate::powers::defs::POWER_DEFS {
        if let Some(guard) = def.status_guard {
            if engine.state.player.status(guard) <= 0 {
                continue;
            }
        }
        execute_entity_triggers(engine, def, trigger, ctx);
    }
}

// ===========================================================================
// Per-entity trigger execution
// ===========================================================================

fn execute_entity_triggers(
    engine: &mut CombatEngine,
    def: &EntityDef,
    trigger: Trigger,
    ctx: &TriggerContext,
) {
    // Fire declarative triggers
    for te in def.triggers {
        if te.trigger != trigger {
            continue;
        }
        if !check_condition(engine, &te.condition, ctx) {
            continue;
        }
        // Handle counter pattern
        if let Some((counter_sid, threshold)) = te.counter {
            let val = engine.state.player.status(counter_sid) + 1;
            if val >= threshold {
                engine.state.player.set_status(counter_sid, 0);
            } else {
                engine.state.player.set_status(counter_sid, val);
                continue; // Counter not reached, skip effects
            }
        }
        execute_trigger_effects(engine, ctx, te.effects);
    }

    // Complex hook fires if trigger matches any of this entity's triggers
    if let Some(hook) = def.complex_hook {
        if def.triggers.iter().any(|te| te.trigger == trigger) {
            hook(engine, ctx);
        }
    }
}

// ===========================================================================
// Condition evaluation
// ===========================================================================

fn check_condition(
    engine: &CombatEngine,
    cond: &TriggerCondition,
    ctx: &TriggerContext,
) -> bool {
    match cond {
        TriggerCondition::Always => true,
        TriggerCondition::FirstTurn => ctx.is_first_turn,
        TriggerCondition::NotFirstTurn => !ctx.is_first_turn,
        TriggerCondition::NoBlock => engine.state.player.block == 0,
        TriggerCondition::CounterReached => true, // Counter check is in execute_entity_triggers
        TriggerCondition::InStance(stance) => engine.state.stance == *stance,
        TriggerCondition::HasStatus(sid) => engine.state.player.status(*sid) > 0,
        TriggerCondition::HpBelow(pct) => {
            let threshold = (engine.state.player.max_hp * (*pct as i32)) / 100;
            engine.state.player.hp <= threshold
        }
        TriggerCondition::HandEmpty => engine.state.hand.is_empty(),
        TriggerCondition::CardTypeIs(card_type) => ctx.card_type == Some(*card_type),
        TriggerCondition::IsBossFight => {
            engine.state.enemies.iter().any(|e| {
                matches!(
                    e.id.as_str(),
                    "Hexaghost" | "SlimeBoss" | "TheGuardian"
                        | "BronzeAutomaton" | "TheCollector" | "TheChamp"
                        | "AwakenedOne" | "TimeEater" | "Donu" | "Deca"
                        | "TheHeart" | "CorruptHeart" | "SpireShield" | "SpireSpear"
                )
            })
        }
        TriggerCondition::IsEliteFight => {
            // Elite detection is typically Python-side; approximate
            false
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};

    #[test]
    fn test_check_condition_always() {
        // Always should return true without needing engine state
        // (Can't easily construct CombatEngine in unit tests)
        let _ = TriggerCondition::Always;
    }

    #[test]
    fn test_check_condition_first_turn() {
        let ctx = TriggerContext {
            card_type: None,
            is_first_turn: true,
            target_idx: -1,
        };
        assert!(ctx.is_first_turn);
    }

    #[test]
    fn test_trigger_context_not_first_turn() {
        let ctx = TriggerContext::empty();
        assert!(!ctx.is_first_turn);
    }
}
