use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus, TypedEventDef};

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn dead_adventurer_blocker_is_now_singular_and_explicit() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    match &dead_adventurer.options[0].status {
        EventRuntimeStatus::Blocked { reason } => {
            assert!(reason.contains("ascension-sensitive initial encounter chance"));
            assert!(reason.contains("first search roll"));
        }
        other => panic!("expected Dead Adventurer to remain blocked, found {other:?}"),
    }
}

#[test]
fn dead_adventurer_typed_program_still_carries_the_search_ramp_skeleton() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    let EventProgramOp::RandomOutcomeTable { outcomes } = &dead_adventurer.options[0].program.ops[0]
    else {
        panic!("expected top-level shuffled reward-order table");
    };
    assert_eq!(outcomes.len(), 6);

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
