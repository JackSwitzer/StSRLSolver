use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus, TypedEventDef};
use crate::run::{GameAction, RunEngine, RunPhase};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn spire_heart_branch_is_supported_and_uses_canonical_final_act_op() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert!(matches!(
        spire_heart.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(matches!(
        spire_heart.options[0].program.ops.as_slice(),
        [EventProgramOp::ResolveFinalAct]
    ));
}

#[test]
fn spire_heart_without_keys_ends_run_on_canonical_terminal_path() {
    let mut engine = RunEngine::new(313, 20);
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    assert_eq!(engine.current_phase(), RunPhase::GameOver);
    assert!(engine.run_state.run_won);
    assert!(engine.run_state.run_over);
    assert!(engine.current_reward_screen().is_none());
}

#[test]
fn spire_heart_with_keys_starts_act_four_on_event_runtime_path() {
    let mut engine = RunEngine::new(313, 20);
    engine.run_state.has_ruby_key = true;
    engine.run_state.has_emerald_key = true;
    engine.run_state.has_sapphire_key = true;
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.act, 4);
    assert_eq!(engine.boss_name(), "CorruptHeart");
    assert_eq!(engine.map.get_start_nodes()[0].room_type, crate::map::RoomType::Rest);
    assert!(engine.pending_event_combat_summary().is_none());
}

#[test]
fn spire_heart_act_four_chain_reaches_heart_and_ends_without_boss_reward() {
    let mut engine = RunEngine::new(313, 20);
    engine.run_state.has_ruby_key = true;
    engine.run_state.has_emerald_key = true;
    engine.run_state.has_sapphire_key = true;
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let enter = engine.step_game(&GameAction::EventChoice(0));
    assert!(enter.accepted());

    // The Ending is Rest -> Shop -> Elite -> Heart, with each transition a
    // real action rather than the previous direct elite/Heart shortcut.
    let rest = engine.get_legal_actions()[0].clone();
    assert!(engine.step_game(&rest).accepted());
    assert_eq!(engine.current_phase(), RunPhase::Campfire);
    assert!(engine.step_game(&GameAction::CampfireRest).accepted());
    let shop = engine.get_legal_actions()[0].clone();
    assert!(engine.step_game(&shop).accepted());
    assert_eq!(engine.current_phase(), RunPhase::Shop);
    assert!(engine.step_game(&GameAction::ShopLeave).accepted());
    let elite = engine.get_legal_actions()[0].clone();
    assert!(engine.step_game(&elite).accepted());
    assert_eq!(engine.debug_current_enemy_ids().len(), 2);

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert!(engine.step_game(&GameAction::LeaveRewards).accepted());
    let heart_room = engine.get_legal_actions()[0].clone();
    assert!(engine.step_game(&heart_room).accepted());
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    assert_eq!(engine.debug_current_enemy_ids(), vec!["CorruptHeart".to_string()]);

    let heart_floor = engine.run_state.floor;
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::Transition);
    assert!(!engine.run_state.run_over);
    assert_eq!(engine.run_state.floor, heart_floor);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.floor, heart_floor + 1);
    assert_eq!(engine.current_phase(), RunPhase::GameOver);
    assert!(engine.run_state.run_won);
    assert!(engine.run_state.run_over);
    assert!(engine.current_reward_screen().is_none());
}
