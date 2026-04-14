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
fn dead_adventurer_is_still_blocked_on_the_remaining_room_reward_queue_semantics() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    match &dead_adventurer.options[0].status {
        EventRuntimeStatus::Blocked { reason } => {
            assert!(reason.contains("persistent search-state tracking"));
            assert!(reason.contains("room-reward queue"));
            assert!(reason.contains("elite-combat continuation"));
        }
        other => panic!("expected Dead Adventurer to remain blocked, found {other:?}"),
    }
}
