use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus, TypedEventDef};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn dead_adventurer_keeps_the_ascension_sensitive_search_blocker_only() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    match &dead_adventurer.options[0].status {
        EventRuntimeStatus::Blocked { reason } => {
            assert!(reason.contains("ascension-sensitive initial encounter chance"));
        }
        other => panic!("expected Dead Adventurer to remain blocked, found {other:?}"),
    }
}

#[test]
fn dead_adventurer_carries_the_search_chain_and_room_reward_skeleton() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(matches!(
        dead_adventurer.options[0].program.ops.as_slice(),
        [EventProgramOp::RandomOutcomeTable { outcomes }] if outcomes.len() == 6
    ));

    let EventProgramOp::RandomOutcomeTable { outcomes } = &dead_adventurer.options[0].program.ops[0] else {
        panic!("expected top-level shuffled reward-order table");
    };
    let first_order = &outcomes[0];
    let Some(EventProgramOp::RandomOutcomeTable { outcomes: search_outcomes }) =
        first_order.ops.first()
    else {
        panic!("expected search chance table inside Dead Adventurer order");
    };
    assert_eq!(search_outcomes.len(), 100);

    let Some(EventProgramOp::RandomOutcomeTable { outcomes: first_page_outcomes }) =
        search_outcomes[0].ops.first()
    else {
        panic!("expected first page fight/reward split");
    };
    assert_eq!(first_page_outcomes.len(), 3);
}
