use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus, TypedEventDef};

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

#[test]
fn wave8_supported_branches_are_no_longer_marked_blocked() {
    let dead_adventurer = typed_event(1, "Dead Adventurer");
    assert!(matches!(dead_adventurer.options[0].status, EventRuntimeStatus::Supported));

    let secret_portal = typed_event(3, "Secret Portal");
    assert!(matches!(secret_portal.options[0].status, EventRuntimeStatus::Supported));

    let wheel = typed_shrine_event("Wheel of Change");
    assert!(matches!(wheel.options[0].status, EventRuntimeStatus::Supported));
}
