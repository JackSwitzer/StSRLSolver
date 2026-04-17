use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/exordium/GoldenWing.java
// - decompiled/java-src/com/megacrit/cardcrawl/helpers/CardHelper.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn dead_adventurer_is_supported_in_the_typed_catalog() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(matches!(
        dead_adventurer.options[0].status,
        EventRuntimeStatus::Supported
    ));
}

#[test]
fn golden_wing_branch_is_runtime_gated_by_attack_damage_in_the_deck() {
    let catalog = typed_event(1, "Golden Wing");
    assert!(matches!(
        catalog.options[1].status,
        EventRuntimeStatus::Supported
    ));

    let mut blocked_engine = RunEngine::new(241, 20);
    blocked_engine.run_state.deck = vec!["Defend_P".to_string(), "Strike_P".to_string()];
    blocked_engine.debug_set_typed_event_state(catalog.clone());

    let before = blocked_engine.run_state.gold;
    let blocked_step = blocked_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(blocked_step.action_accepted);
    assert_eq!(blocked_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(blocked_engine.run_state.gold, before);
    assert!(blocked_engine.current_reward_screen().is_none());

    let mut supported_engine = RunEngine::new(241, 20);
    supported_engine.run_state.deck = vec!["Feed".to_string(), "Strike_P".to_string()];
    supported_engine.debug_set_typed_event_state(catalog);

    let before = supported_engine.run_state.gold;
    let supported_step = supported_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(supported_step.action_accepted);
    assert_eq!(supported_engine.current_phase(), RunPhase::MapChoice);
    let gain = supported_engine.run_state.gold - before;
    assert!(
        (50..=80).contains(&gain),
        "expected Golden Wing gold gain between 50 and 80, got {gain}"
    );
    assert!(supported_engine.current_reward_screen().is_none());
}
