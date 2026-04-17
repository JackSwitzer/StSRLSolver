use crate::events::{typed_events_for_act, EventProgramOp, TypedEventDef};
use crate::run::RunEngine;

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/exordium/DeadAdventurer.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

fn search_branch_counts(engine: &mut RunEngine) -> (usize, usize) {
    let event = engine
        .debug_current_event()
        .expect("expected a normalized Dead Adventurer event");
    let EventProgramOp::RandomOutcomeTable { outcomes } = &event.options[0].program.ops[0] else {
        panic!("expected top-level shuffled reward-order table");
    };
    let first_order = &outcomes[0];
    let Some(EventProgramOp::RandomOutcomeTable {
        outcomes: search_outcomes,
    }) = first_order.ops.first()
    else {
        panic!("expected search chance table inside Dead Adventurer order");
    };
    let fight_count = search_outcomes
        .iter()
        .filter(|program| {
            matches!(
                program.ops.as_slice(),
                [EventProgramOp::RandomOutcomeTable { .. }]
            )
        })
        .count();
    (fight_count, search_outcomes.len() - fight_count)
}

#[test]
fn dead_adventurer_uses_the_25_percent_initial_search_roll_before_asc15() {
    let mut engine = RunEngine::new(101, 14);
    engine.debug_set_typed_event_state(typed_event(1, "Dead Adventurer"));
    let (fight_count, reward_count) = search_branch_counts(&mut engine);
    assert_eq!(fight_count, 25);
    assert_eq!(reward_count, 75);
}
