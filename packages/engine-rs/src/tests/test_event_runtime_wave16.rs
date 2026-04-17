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
fn dead_adventurer_is_supported_and_keeps_the_search_chain_skeleton() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(matches!(
        dead_adventurer.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(matches!(
        dead_adventurer.options[0].program.ops.as_slice(),
        [EventProgramOp::RandomOutcomeTable { outcomes }] if outcomes.len() == 6
    ));
}
