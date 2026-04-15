use crate::events::{
    shrine_events, typed_shrine_events, EventProgramOp, EventRuntimeStatus, TypedEventDef,
};
use crate::run::{RunAction, RunEngine, RunPhase};

// Blocker note:
// - Match and Keep! is the full Java GremlinMatchGame minigame from
//   /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java
// - The current runtime only has event-option and reward-screen flows, so we keep this event
//   explicitly blocked until a dedicated card-grid event runtime lands.

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

#[test]
fn match_and_keep_is_explicitly_blocked_until_the_match_game_runtime_lands() {
    let event = typed_shrine_event("Match and Keep!");
    assert!(matches!(event.options[0].status, EventRuntimeStatus::Blocked { .. }));
    assert!(matches!(
        event.options[0].program.ops.as_slice(),
        [EventProgramOp::BlockedPlaceholder { .. }]
    ));
    if let EventRuntimeStatus::Blocked { reason } = &event.options[0].status {
        assert!(reason.contains("GremlinMatchGame"));
    }
}

#[test]
fn match_and_keep_does_not_enter_the_temporary_reward_path_anymore() {
    let mut engine = RunEngine::new(421, 20);
    let deck_before = engine.run_state.deck.clone();
    engine.debug_set_typed_event_state(typed_shrine_event("Match and Keep!"));

    let start = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(start.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.current_reward_screen(), None);
    assert_eq!(engine.run_state.deck, deck_before);
}

#[test]
fn typed_and_legacy_shrine_catalog_sizes_still_match_after_match_and_keep_blocker_cutover() {
    assert_eq!(typed_shrine_events().len(), shrine_events().len());
}
