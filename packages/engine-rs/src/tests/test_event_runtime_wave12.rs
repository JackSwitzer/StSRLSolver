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
fn spire_heart_remains_the_single_supported_event_blocker_with_an_explicit_reason() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert_eq!(spire_heart.options.len(), 1);
    assert!(matches!(
        spire_heart.options[0].status,
        EventRuntimeStatus::Blocked { .. }
    ));
    assert_eq!(
        spire_heart.options[0]
            .program
            .ops
            .iter()
            .filter(|op| matches!(op, EventProgramOp::BlockedPlaceholder { .. }))
            .count(),
        1
    );

    let EventRuntimeStatus::Blocked { reason } = &spire_heart.options[0].status else {
        panic!("Spire Heart should be the remaining explicit blocked event");
    };
    assert!(
        reason.contains("Act 4 unlock flow"),
        "expected exact remaining primitive reason, got {reason}"
    );
}

#[test]
#[ignore = "Blocked on a shared final-act transition model (Act 4 entry plus terminal ending flow) proven by decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java"]
fn queued_spire_heart_requires_shared_final_act_transition_model() {}
