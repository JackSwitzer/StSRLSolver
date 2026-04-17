use crate::effects::runtime::{EffectExecutionPhase, EventRecordPhase};
use crate::status_ids::sid;
use crate::tests::support::{engine_with, ensure_in_hand, play_on_enemy, make_deck_n};

#[test]
fn time_warp_runtime_ends_turn_before_double_tap_replay() {
    let mut engine = engine_with(make_deck_n("Strike_R", 12), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP, 11);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike_R");

    assert!(play_on_enemy(&mut engine, "Strike_R", 0));

    assert_eq!(
        engine.state.enemies[0].entity.hp, 44,
        "Time Warp should end the turn before Double Tap can replay the 12th card"
    );
    assert_eq!(
        engine.state.enemies[0].entity.strength(),
        2,
        "Time Warp should still grant the Time Eater strength buff on the 12th card"
    );
    assert_eq!(engine.state.turn, 2, "turn should advance after Time Warp ends the turn");

    let time_warp_hook_seen = engine.event_log.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.def_id == Some("time_warp")
            && record.execution == Some(EffectExecutionPhase::Hook)
    });
    assert!(time_warp_hook_seen, "time_warp hook should execute through the runtime");

}
