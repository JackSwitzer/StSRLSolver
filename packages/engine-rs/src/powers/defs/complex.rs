//! Complex power definitions.
//!
//! Powers whose behavior cannot be fully expressed as declarative effects.
//! These use `complex_hook` fn pointers for their primary logic.
//! The triggers array may still contain declarative parts where applicable.
//!
//! Complex powers include:
//! - Card replay logic: Echo Form, Double Tap, Burst, Necronomicon
//! - On-attacked reactions: Thorns, Flame Barrier
//! - Damage/debuff reactions: Envenom, Sadistic Nature
//! - Per-card triggers with side effects: Thousand Cuts, Panache, Electrodynamics

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

// ===========================================================================
// Complex hooks
// ===========================================================================
// Migrated powers implement owner-aware hooks here. Remaining no-op hooks mark
// the surfaces that still execute inline in `engine.rs` / `combat_hooks.rs`.

fn hook_noop(
    _engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {}

fn hook_time_warp(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnAfterUseCard
        || engine.state.combat_over
        || engine.phase != crate::engine::CombatPhase::PlayerTurn
    {
        return;
    }

    let enemy_idx = match owner {
        EffectOwner::EnemyPower { enemy_idx } => enemy_idx as usize,
        _ => return,
    };
    if enemy_idx >= engine.state.enemies.len() || !engine.state.enemies[enemy_idx].is_alive() {
        return;
    }

    if crate::powers::increment_time_warp(&mut engine.state.enemies[enemy_idx].entity) {
        for idx in engine.state.living_enemy_indices() {
            engine.state.enemies[idx].entity.add_status(sid::STRENGTH, 2);
        }
        engine.end_turn();
    }
}

fn player_power_amount(engine: &CombatEngine, owner: EffectOwner, status_id: crate::ids::StatusId) -> i32 {
    match owner {
        EffectOwner::PlayerPower => engine.state.player.status(status_id),
        EffectOwner::EnemyPower { enemy_idx } => {
            let idx = enemy_idx as usize;
            if idx < engine.state.enemies.len() {
                engine.state.enemies[idx].entity.status(status_id)
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn hook_thousand_cuts(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnAfterCardPlayed || engine.state.combat_over {
        return;
    }

    let damage = player_power_amount(engine, owner, sid::THOUSAND_CUTS);
    if damage <= 0 {
        return;
    }

    let living = engine.state.living_enemy_indices();
    for idx in living {
        engine.deal_damage_to_enemy(idx, damage);
        if engine.state.combat_over {
            break;
        }
    }
}

fn hook_panache(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    if event.kind == Trigger::TurnStart {
        state.set(0, 0);
        return;
    }

    if event.kind != Trigger::OnUseCard || engine.state.combat_over {
        return;
    }

    let damage = player_power_amount(engine, owner, sid::PANACHE);
    if damage <= 0 {
        return;
    }

    let next = state.get(0) + 1;
    if next < 5 {
        state.set(0, next);
        return;
    }
    state.set(0, 0);

    let living = engine.state.living_enemy_indices();
    for idx in living {
        engine.deal_damage_to_enemy(idx, damage);
        if engine.state.combat_over {
            break;
        }
    }
}

fn hook_sadistic_nature(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnDebuffApplied || engine.state.combat_over {
        return;
    }

    let damage = player_power_amount(engine, owner, sid::SADISTIC);
    if damage <= 0 {
        return;
    }

    let target_idx = event.enemy_idx.max(event.target_idx);
    if target_idx < 0 {
        return;
    }
    let idx = target_idx as usize;
    if idx >= engine.state.enemies.len() || !engine.state.enemies[idx].is_alive() {
        return;
    }

    engine.deal_damage_to_enemy(idx, damage);
}

fn hook_envenom(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::DamageResolved
        || engine.state.combat_over
        || event.amount <= 0
        || matches!(
            event.card_type,
            Some(crate::cards::CardType::Skill | crate::cards::CardType::Power)
        )
    {
        return;
    }

    let poison = player_power_amount(engine, owner, sid::ENVENOM);
    if poison <= 0 {
        return;
    }

    let idx = event.enemy_idx.max(event.target_idx);
    if idx < 0 {
        return;
    }
    let idx = idx as usize;
    if idx >= engine.state.enemies.len() || !engine.state.enemies[idx].is_alive() {
        return;
    }

    engine.apply_player_debuff_to_enemy(idx, sid::POISON, poison);
}

fn hook_double_tap(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if !engine.runtime_replay_window || event.kind != Trigger::OnAttackPlayed || engine.state.combat_over {
        return;
    }

    let card_inst = match engine.runtime_played_card {
        Some(card_inst) => card_inst,
        None => return,
    };

    let remaining = player_power_amount(engine, owner, sid::DOUBLE_TAP);
    if remaining <= 0 {
        return;
    }

    match owner {
        EffectOwner::PlayerPower => engine.state.player.add_status(sid::DOUBLE_TAP, -1),
        EffectOwner::EnemyPower { enemy_idx } => {
            let idx = enemy_idx as usize;
            if idx < engine.state.enemies.len() {
                engine.state.enemies[idx].entity.add_status(sid::DOUBLE_TAP, -1);
            }
        }
        _ => return,
    }

    let card = engine.card_registry.card_def_by_id(card_inst.def_id).clone();
    crate::card_effects::execute_card_effects(engine, &card, card_inst, event.target_idx);
}

fn hook_burst(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if !engine.runtime_replay_window || event.kind != Trigger::OnSkillPlayed || engine.state.combat_over {
        return;
    }

    let card_inst = match engine.runtime_played_card {
        Some(card_inst) => card_inst,
        None => return,
    };

    let remaining = player_power_amount(engine, owner, sid::BURST);
    if remaining <= 0 {
        return;
    }

    match owner {
        EffectOwner::PlayerPower => engine.state.player.add_status(sid::BURST, -1),
        EffectOwner::EnemyPower { enemy_idx } => {
            let idx = enemy_idx as usize;
            if idx < engine.state.enemies.len() {
                engine.state.enemies[idx].entity.add_status(sid::BURST, -1);
            }
        }
        _ => return,
    }

    let card = engine.card_registry.card_def_by_id(card_inst.def_id).clone();
    crate::card_effects::execute_card_effects(engine, &card, card_inst, event.target_idx);
}

fn hook_echo_form(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if !engine.runtime_replay_window || event.kind != Trigger::OnCardPlayedPost || engine.state.combat_over {
        return;
    }

    let card_inst = match engine.runtime_played_card {
        Some(card_inst) => card_inst,
        None => return,
    };

    let card_type = match event.card_type {
        Some(card_type) => card_type,
        None => return,
    };
    if card_type == crate::cards::CardType::Power {
        return;
    }

    let echo_count = player_power_amount(engine, owner, sid::ECHO_FORM);
    if echo_count <= 0 || engine.state.cards_played_this_turn > echo_count {
        return;
    }

    let card = engine.card_registry.card_def_by_id(card_inst.def_id).clone();
    crate::card_effects::execute_card_effects(engine, &card, card_inst, event.target_idx);
}

// ===========================================================================
// Echo Form — replays the first card played each turn
// ===========================================================================

pub static DEF_ECHO_FORM: EntityDef = EntityDef {
    id: "echo_form",
    name: "Echo Form",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnCardPlayedPost,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_echo_form),
    status_guard: Some(sid::ECHO_FORM),
};

// ===========================================================================
// Double Tap — replays the next Attack played this turn
// ===========================================================================

pub static DEF_DOUBLE_TAP: EntityDef = EntityDef {
    id: "double_tap",
    name: "Double Tap",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnAttackPlayed,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_double_tap),
    status_guard: Some(sid::DOUBLE_TAP),
};

// ===========================================================================
// Burst — replays the next Skill played this turn
// ===========================================================================

pub static DEF_BURST: EntityDef = EntityDef {
    id: "burst",
    name: "Burst",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnSkillPlayed,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_burst),
    status_guard: Some(sid::BURST),
};

// ===========================================================================
// Thorns — deal damage back when attacked
// ===========================================================================

pub static DEF_THORNS: EntityDef = EntityDef {
    id: "thorns",
    name: "Thorns",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::THORNS),
};

// ===========================================================================
// Flame Barrier — deal damage back when attacked this turn
// ===========================================================================

pub static DEF_FLAME_BARRIER: EntityDef = EntityDef {
    id: "flame_barrier",
    name: "Flame Barrier",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::FLAME_BARRIER),
};

// ===========================================================================
// Envenom — apply Poison when dealing unblocked Attack damage
// ===========================================================================

pub static DEF_ENVENOM: EntityDef = EntityDef {
    id: "envenom",
    name: "Envenom",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::DamageResolved,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_envenom),
    status_guard: Some(sid::ENVENOM),
};

// ===========================================================================
// Sadistic Nature — deal damage when applying a debuff
// ===========================================================================

pub static DEF_SADISTIC_NATURE: EntityDef = EntityDef {
    id: "sadistic_nature",
    name: "Sadistic Nature",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnDebuffApplied,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_sadistic_nature),
    status_guard: Some(sid::SADISTIC),
};

// ===========================================================================
// Thousand Cuts — deal damage to all enemies on card play
// ===========================================================================

pub static DEF_THOUSAND_CUTS: EntityDef = EntityDef {
    id: "thousand_cuts",
    name: "Thousand Cuts",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnAfterCardPlayed,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_thousand_cuts),
    status_guard: Some(sid::THOUSAND_CUTS),
};

// ===========================================================================
// Panache — deal damage every 5 cards played
// ===========================================================================

pub static DEF_PANACHE: EntityDef = EntityDef {
    id: "panache",
    name: "Panache",
    kind: EntityKind::Power,
    triggers: &[
        TriggeredEffect {
            trigger: Trigger::OnUseCard,
            condition: TriggerCondition::Always,
            effects: &[],
            counter: None,
        },
        TriggeredEffect {
            trigger: Trigger::TurnStart,
            condition: TriggerCondition::Always,
            effects: &[],
            counter: None,
        },
    ],
    complex_hook: Some(hook_panache),
    status_guard: Some(sid::PANACHE),
};

// ===========================================================================
// Electrodynamics — Lightning orbs hit all enemies
// ===========================================================================

pub static DEF_ELECTRODYNAMICS: EntityDef = EntityDef {
    id: "electrodynamics",
    name: "Electrodynamics",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::ELECTRODYNAMICS),
};

// ===========================================================================
pub static DEF_TIME_WARP: EntityDef = EntityDef {
    id: "time_warp",
    name: "Time Warp",
    kind: EntityKind::Power,
    triggers: &[TriggeredEffect {
        trigger: Trigger::OnAfterUseCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    }],
    complex_hook: Some(hook_time_warp),
    status_guard: Some(sid::TIME_WARP_ACTIVE),
};

// ===========================================================================
// Static Discharge — On unblocked damage: channel Lightning
// ===========================================================================

pub static DEF_STATIC_DISCHARGE: EntityDef = EntityDef {
    id: "static_discharge",
    name: "Static Discharge",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::STATIC_DISCHARGE),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_complex_have_hooks() {
        let defs = [
            &DEF_ECHO_FORM, &DEF_DOUBLE_TAP, &DEF_BURST,
            &DEF_THORNS, &DEF_FLAME_BARRIER, &DEF_ENVENOM,
            &DEF_SADISTIC_NATURE, &DEF_THOUSAND_CUTS,
            &DEF_PANACHE, &DEF_ELECTRODYNAMICS,
            &DEF_TIME_WARP, &DEF_STATIC_DISCHARGE,
        ];
        for def in &defs {
            assert!(
                def.complex_hook.is_some(),
                "Complex power '{}' missing complex_hook",
                def.id
            );
        }
    }

    #[test]
    fn test_complex_have_empty_triggers() {
        // These remaining hook-only powers still have no declarative trigger surface.
        let defs = [&DEF_THORNS, &DEF_FLAME_BARRIER, &DEF_STATIC_DISCHARGE];
        for def in &defs {
            assert!(
                def.triggers.is_empty(),
                "Complex power '{}' should have empty triggers (logic is in hook)",
                def.id
            );
        }
    }
}

#[cfg(test)]
#[path = "../../tests/test_power_runtime_replay.rs"]
mod test_power_runtime_replay;

#[cfg(test)]
#[path = "../../tests/test_power_runtime_complex.rs"]
mod test_power_runtime_complex;

#[cfg(test)]
#[path = "../../tests/test_power_runtime_debuff_enemy.rs"]
mod test_power_runtime_debuff_enemy;

#[cfg(test)]
#[path = "../../tests/test_power_runtime_end_to_end.rs"]
mod test_power_runtime_end_to_end;

#[cfg(test)]
#[path = "../../tests/test_damage_followup_java_wave1.rs"]
mod test_damage_followup_java_wave1;
