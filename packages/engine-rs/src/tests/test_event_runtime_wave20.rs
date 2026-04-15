use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn scrap_ooze_is_supported_in_the_typed_catalog() {
    let scrap_ooze = typed_event(1, "Scrap Ooze");
    assert!(matches!(
        scrap_ooze.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(scrap_ooze.options[0]
        .text
        .contains("25% relic chance"));
}

#[test]
fn scrap_ooze_retries_with_escalating_damage_and_relic_chance_before_rewarding_a_relic() {
    let mut engine = RunEngine::new(77, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    engine.debug_set_typed_event_state(typed_event(1, "Scrap Ooze"));
    engine.debug_force_event_rolls(&[0, 99]);

    let first = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(first.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 75);

    let ctx = engine.current_decision_context();
    let event = ctx.event.expect("scrap ooze retry event");
    assert!(event.options[0].label.contains("6 dmg"));
    assert!(event.options[0].label.contains("35%"));

    let second = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(second.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.current_hp, 69);

    let screen = engine
        .current_reward_screen()
        .expect("scrap ooze reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);

    let relic_id = screen.items[0].label.clone();
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.run_state.relics.iter().any(|relic| relic == &relic_id));
}

#[test]
fn scrap_ooze_leave_exits_without_damage_or_reward() {
    let mut engine = RunEngine::new(79, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    engine.debug_set_typed_event_state(typed_event(1, "Scrap Ooze"));

    let leave = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(leave.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.current_reward_screen().is_none());
    assert_eq!(engine.run_state.current_hp, 80);
}
