use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn dead_adventurer_no_longer_depends_on_blocked_room_reward_queue_semantics() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(matches!(
        dead_adventurer.options[0].status,
        EventRuntimeStatus::Supported
    ));
}
