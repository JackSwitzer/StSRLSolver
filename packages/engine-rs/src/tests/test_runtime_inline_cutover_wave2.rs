use crate::effects::runtime::{EffectExecutionPhase, EffectOwner, EventRecordPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_with, engine_with_enemies, ensure_in_hand, make_deck_n, play_on_enemy,
    play_self,
};

#[test]
fn envenom_engine_path_applies_poison_through_runtime_hook() {
    let mut engine = engine_with(make_deck_n("Strike", 6), 50, 0);
    engine.state.player.set_status(sid::ENVENOM, 2);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2);
    assert!(engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == crate::effects::trigger::Trigger::DamageResolved
            && record.def_id == Some("envenom")
            && record.execution == Some(EffectExecutionPhase::Hook)
    }));
}

#[test]
fn thousand_cuts_engine_path_hits_all_enemies_after_card_play() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_with_enemies(make_deck_n("Defend", 6), enemies, 99);
    engine.state.player.set_status(sid::THOUSAND_CUTS, 1);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Defend");

    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(engine.state.enemies[0].entity.hp, 39);
    assert_eq!(engine.state.enemies[1].entity.hp, 34);
    assert!(engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.def_id == Some("thousand_cuts")
            && record.execution == Some(EffectExecutionPhase::Hook)
    }));
}

#[test]
fn panache_engine_path_tracks_hidden_counter_and_bursts_on_fifth_card() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 60, 60),
        enemy_no_intent("Cultist", 55, 55),
    ];
    let mut engine = engine_with_enemies(make_deck_n("Defend", 8), enemies, 99);
    engine.state.player.set_status(sid::PANACHE, 10);
    engine.rebuild_effect_runtime();

    for expected in 1..=4 {
        ensure_in_hand(&mut engine, "Defend");
        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(
            engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
            expected
        );
        assert_eq!(engine.state.enemies[0].entity.hp, 60);
        assert_eq!(engine.state.enemies[1].entity.hp, 55);
    }

    ensure_in_hand(&mut engine, "Defend");
    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 50);
    assert_eq!(engine.state.enemies[1].entity.hp, 45);
}

#[test]
fn electrodynamics_lightning_targets_all_enemies_via_runtime_active_power_query() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_with_enemies(make_deck_n("Strike", 4), enemies, 3);
    engine.init_defect_orbs(1);
    engine.state.player.set_status(sid::ELECTRODYNAMICS, 1);
    engine.rebuild_effect_runtime();
    engine.channel_orb(crate::orbs::OrbType::Lightning);

    assert!(engine
        .effect_runtime
        .has_instance("electrodynamics", EffectOwner::PlayerPower));

    engine.evoke_front_orb();

    assert_eq!(engine.state.enemies[0].entity.hp, 32);
    assert_eq!(engine.state.enemies[1].entity.hp, 27);
}

#[test]
fn double_tap_replay_runs_through_runtime_replay_window() {
    let mut engine = engine_with(make_deck_n("Strike", 6), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 0);
    assert!(engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == crate::effects::trigger::Trigger::OnAttackPlayed
            && record.def_id == Some("double_tap")
            && record.execution == Some(EffectExecutionPhase::Hook)
    }));
}

#[test]
fn burst_replay_is_suppressed_when_time_warp_force_ends_the_turn() {
    let mut engine = engine_with(make_deck_n("Defend", 6), 50, 0);
    engine.state.player.set_status(sid::BURST, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP, 11);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Defend");

    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(engine.state.player.status(sid::BURST), 1);
    assert_eq!(engine.state.turn, 2);
    assert!(engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.def_id == Some("time_warp")
            && record.execution == Some(EffectExecutionPhase::Hook)
    }));
}
