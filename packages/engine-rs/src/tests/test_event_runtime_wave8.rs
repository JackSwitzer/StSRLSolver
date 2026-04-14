use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SecretPortal.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinWheelGame.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

fn blocked_reason(event: &TypedEventDef, option_index: usize) -> &str {
    match &event.options[option_index].status {
        EventRuntimeStatus::Blocked { reason } => reason.as_str(),
        other => panic!(
            "expected blocked status for {} option {}, found {other:?}",
            event.name, option_index
        ),
    }
}

#[test]
fn wave8_blocked_branches_keep_java_specific_reasons() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(blocked_reason(&dead_adventurer, 0).contains("encounter chance ramp"));
    assert!(blocked_reason(&dead_adventurer, 0).contains("elite combat continuation"));

    let secret_portal = typed_event(3, "Secret Portal");
    assert!(blocked_reason(&secret_portal, 0).contains("map-node transition"));
    assert!(blocked_reason(&secret_portal, 0).contains("boss room"));

    let wheel = typed_shrine_event("Wheel of Change");
    assert!(blocked_reason(&wheel, 0).contains("act-scaled gold"));
    assert!(blocked_reason(&wheel, 0).contains("nested purge choice"));
}

#[test]
fn wave8_blocked_branches_do_not_start_partial_runtime_side_effects() {
    let blocked_cases = [
        typed_event(1, "Dead Adventurer"),
        typed_event(3, "Secret Portal"),
        typed_shrine_event("Wheel of Change"),
    ];

    for event in blocked_cases {
        let mut engine = RunEngine::new(73, 20);
        engine.debug_set_typed_event_state(event.clone());

        let result = engine.step_with_result(&RunAction::EventChoice(0));
        assert!(result.action_accepted, "{} choice should still be accepted", event.name);
        assert_eq!(
            engine.current_phase(),
            RunPhase::MapChoice,
            "{} should currently fall back to map instead of starting a partial flow",
            event.name
        );
        assert!(
            engine.get_combat_engine().is_none(),
            "{} should not start combat on the blocked path",
            event.name
        );
        assert!(
            engine.current_reward_screen().is_none(),
            "{} should not open a reward screen on the blocked path",
            event.name
        );
        assert!(
            engine.pending_event_combat_summary().is_none(),
            "{} should not queue event combat on the blocked path",
            event.name
        );
    }
}
