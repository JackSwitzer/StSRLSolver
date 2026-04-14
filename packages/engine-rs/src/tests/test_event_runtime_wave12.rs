use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus, TypedEventDef};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn test_event_runtime_wave12_spire_heart_is_supported_in_the_typed_catalog() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert_eq!(spire_heart.options.len(), 1);
    assert!(matches!(spire_heart.options[0].status, EventRuntimeStatus::Supported));
    assert_eq!(
        spire_heart.options[0]
            .program
            .ops
            .iter()
            .filter(|op| matches!(op, EventProgramOp::BlockedPlaceholder { .. }))
            .count(),
        0
    );
}

#[test]
fn spire_heart_is_supported_with_no_blocked_placeholder_ops() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert_eq!(spire_heart.options.len(), 1);
    assert!(matches!(
        spire_heart.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(spire_heart.options[0]
        .program
        .ops
        .iter()
        .all(|op| !matches!(op, EventProgramOp::BlockedPlaceholder { .. })));
}
