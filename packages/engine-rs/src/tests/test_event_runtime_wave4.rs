use crate::decision::{DecisionAction, RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, EventRuntimeStatus};
use crate::run::{RunAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn test_event_runtime_wave4_golden_wing_feed_is_runtime_supported_and_removes_a_card() {
    let mut engine = RunEngine::new(13, 20);
    let hp_before = engine.run_state.current_hp;
    let deck_before = engine.run_state.deck.len();
    let golden_wing = typed_event(1, "Golden Wing");
    assert!(matches!(
        golden_wing.options[0].status,
        EventRuntimeStatus::Supported
    ));

    engine.debug_set_typed_event_state(golden_wing);
    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.current_hp, hp_before - 7);
    assert_eq!(engine.run_state.deck.len(), deck_before - 1);
}

#[test]
fn test_event_runtime_wave4_the_joust_resolves_bets_through_typed_runtime_rng() {
    let mut murderer_engine = RunEngine::new(5, 20);
    let mut owner_engine = RunEngine::new(5, 20);
    let gold_before = murderer_engine.run_state.gold;
    let joust = typed_event(2, "The Joust");
    assert!(matches!(joust.options[0].status, EventRuntimeStatus::Supported));
    assert!(matches!(joust.options[1].status, EventRuntimeStatus::Supported));

    murderer_engine.debug_set_typed_event_state(joust.clone());
    let murderer_step = murderer_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(murderer_step.action_accepted);
    assert_eq!(murderer_engine.current_phase(), RunPhase::MapChoice);

    owner_engine.debug_set_typed_event_state(joust);
    let owner_step = owner_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(owner_step.action_accepted);
    assert_eq!(owner_engine.current_phase(), RunPhase::MapChoice);

    assert_eq!(murderer_engine.run_state.gold, gold_before - 50);
    assert_eq!(owner_engine.run_state.gold, gold_before + 200);
}

#[test]
fn test_event_runtime_wave4_winding_halls_madness_uses_ordered_event_reward_flow() {
    let mut engine = RunEngine::new(11, 20);
    engine.run_state.current_hp = engine.run_state.max_hp;
    let deck_before = engine.run_state.deck.len();
    let halls = typed_event(3, "Winding Halls");
    assert!(matches!(halls.options[0].status, EventRuntimeStatus::Supported));

    engine.debug_set_typed_event_state(halls);
    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.current_hp, engine.run_state.max_hp - 12);

    let screen = engine
        .current_reward_screen()
        .expect("madness reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 2);
    assert!(screen.items.iter().all(|item| item.kind == RewardItemKind::CardChoice));
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);
    assert_eq!(screen.items[0].choices.len(), 1);
    assert_eq!(screen.items[1].choices.len(), 1);

    let open_first = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_first.action_accepted);
    assert_eq!(
        open_first.legal_decision_actions,
        vec![DecisionAction::PickRewardChoice {
            item_index: 0,
            choice_index: 0,
        }]
    );

    let choose_first = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose_first.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![DecisionAction::ClaimRewardItem { item_index: 1 }]
    );

    let open_second = engine.step_with_result(&RunAction::SelectRewardItem(1));
    assert!(open_second.action_accepted);
    let choose_second = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 1,
        choice_index: 0,
    });
    assert!(choose_second.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck.len(), deck_before + 2);
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Madness")
            .count(),
        2
    );
}

#[test]
fn test_event_runtime_wave4_winding_halls_retrace_and_press_on_use_percent_runtime_effects() {
    let mut retrace_engine = RunEngine::new(17, 20);
    retrace_engine.run_state.current_hp = 10;
    let halls = typed_event(3, "Winding Halls");
    retrace_engine.debug_set_typed_event_state(halls.clone());
    let retrace = retrace_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(retrace.action_accepted);
    assert_eq!(retrace_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(retrace_engine.run_state.current_hp, 23);
    assert!(retrace_engine.run_state.deck.iter().any(|card| card == "Writhe"));

    let mut press_engine = RunEngine::new(17, 20);
    let max_hp_before = press_engine.run_state.max_hp;
    press_engine.debug_set_typed_event_state(halls);
    let press = press_engine.step_with_result(&RunAction::EventChoice(2));
    assert!(press.action_accepted);
    assert_eq!(press_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(press_engine.run_state.max_hp, max_hp_before - 3);
    assert_eq!(press_engine.run_state.current_hp, press_engine.run_state.max_hp);
}
