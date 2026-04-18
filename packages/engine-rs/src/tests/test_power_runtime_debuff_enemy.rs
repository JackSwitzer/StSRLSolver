#![cfg(test)]

use crate::effects::runtime::{EffectExecutionPhase, EffectOwner, EventRecordPhase};
use crate::effects::trigger::Trigger;
use crate::powers::defs::{DEF_SADISTIC_NATURE, DEF_TIME_WARP};
use crate::state::EnemyCombatState;
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_with, engine_with_enemies, ensure_in_hand, make_deck_n, play_on_enemy,
};

#[test]
fn sadistic_nature_runtime_triggers_on_skill_debuff_application() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::SADISTIC, 5);
    ensure_in_hand(&mut engine, "Trip");

    assert!(play_on_enemy(&mut engine, "Trip", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, 45);
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
}

#[test]
fn sadistic_nature_runtime_skips_artifact_blocked_debuffs() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::SADISTIC, 5);
    engine.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
    ensure_in_hand(&mut engine, "Trip");

    assert!(play_on_enemy(&mut engine, "Trip", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, 50);
    assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 0);
}

#[test]
fn sadistic_nature_def_uses_debuff_applied_trigger() {
    assert_eq!(DEF_SADISTIC_NATURE.triggers.len(), 1);
    assert_eq!(DEF_SADISTIC_NATURE.triggers[0].trigger, Trigger::OnDebuffApplied);
}

#[test]
fn envenom_runtime_triggers_only_on_unblocked_attack_damage() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::ENVENOM, 2);
    engine.state.enemies[0].entity.block = 20;
    engine.rebuild_effect_runtime();
    ensure_in_hand(&mut engine, "Strike");

    assert!(engine
        .effect_runtime
        .has_instance("envenom", EffectOwner::PlayerPower));

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 0);

    ensure_in_hand(&mut engine, "Strike");
    engine.state.enemies[0].entity.block = 0;
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2);
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::DamageResolved
            && record.def_id == Some("envenom")
            && record.amount > 0
    }));
}

#[test]
fn time_warp_def_uses_after_use_card_runtime_trigger() {
    assert_eq!(DEF_TIME_WARP.triggers.len(), 1);
    assert_eq!(DEF_TIME_WARP.triggers[0].trigger, Trigger::OnAfterUseCard);
    assert_eq!(DEF_TIME_WARP.status_guard, Some(sid::TIME_WARP_ACTIVE));
    assert!(DEF_TIME_WARP.complex_hook.is_some());
}

#[test]
fn time_warp_runtime_snapshot_installs_enemy_power_when_active() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 250, 0);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP, 5);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.rebuild_effect_runtime();

    assert!(engine
        .effect_runtime
        .has_instance("time_warp", EffectOwner::EnemyPower { enemy_idx: 0 }));
}

#[test]
fn time_warp_twelfth_card_ends_turn_resets_counter_and_buffs_all_monsters() {
    // Java oracle:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/TimeWarpPower.java
    let mut second_enemy = enemy_no_intent("Cultist", 40, 40);
    second_enemy.entity.set_status(sid::STRENGTH, 1);
    let mut engine = engine_with_enemies(
        make_deck_n("Strike", 12),
        vec![EnemyCombatState::new("TimeEater", 250, 250), second_enemy],
        3,
    );
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP, 11);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::TIME_WARP), 0);
    assert_eq!(engine.state.enemies[0].entity.strength(), 2);
    assert_eq!(engine.state.enemies[1].entity.strength(), 3);
    assert_eq!(engine.state.turn, 2, "Time Warp should call the early end-turn sequence");

    let time_warp_hook_seen = engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.def_id == Some("time_warp")
            && record.execution == Some(EffectExecutionPhase::Hook)
            && record.event == Trigger::OnAfterUseCard
    });
    assert!(time_warp_hook_seen, "time_warp should execute through the runtime hook path");
}
