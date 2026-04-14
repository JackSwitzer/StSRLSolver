use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{shrine_events, typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Temporary parity note:
// - Match and Keep! is not yet the Java minigame from
//   /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java
// - We currently support it through the canonical reward runtime as a fixed Rushdown+ reward
//   so the event no longer remains unsupported in starter-seed/content-complete runs.

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

#[test]
fn match_and_keep_is_no_longer_blocked_in_the_typed_catalog() {
    let event = typed_shrine_event("Match and Keep!");
    assert!(matches!(
        event.options[0].status,
        EventRuntimeStatus::Supported
    ));
}

#[test]
fn match_and_keep_temporary_reward_flows_through_event_reward_runtime() {
    let mut engine = RunEngine::new(421, 20);
    engine.debug_set_typed_event_state(typed_shrine_event("Match and Keep!"));

    let start = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(start.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine
        .current_reward_screen()
        .expect("event reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert!(screen.items[0].claimable);
    assert_eq!(screen.items[0].choices.len(), 1);

    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);

    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck.last().map(String::as_str), Some("Adaptation+"));
}

#[test]
fn typed_and_legacy_shrine_catalog_sizes_still_match_after_match_and_keep_cutover() {
    assert_eq!(typed_shrine_events().len(), shrine_events().len());
}
