use crate::decision::{DecisionAction, RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus};
use crate::run::{RunAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

fn typed_shrine_event(name: &str) -> crate::events::TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

#[test]
fn addict_pay_opens_canonical_relic_reward_screen() {
    let mut engine = RunEngine::new(42, 20);
    let gold_before = engine.run_state.gold;
    let addict = typed_event(2, "Addict");
    assert!(matches!(
        addict.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(addict);

    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.run_state.gold, gold_before - 85);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine
        .current_reward_screen()
        .expect("event reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);
    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![DecisionAction::ClaimRewardItem { item_index: 0 }]
    );
}

#[test]
fn library_read_uses_nested_card_reward_choice_flow() {
    let mut engine = RunEngine::new(7, 20);
    let library = typed_event(2, "The Library");
    assert!(matches!(
        library.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(library);

    let open_screen = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(open_screen.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine
        .current_reward_screen()
        .expect("library reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert!(screen.items[0].claimable);
    assert_eq!(screen.items[0].choices.len(), 3);

    let open_choice = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_choice.action_accepted);
    assert_eq!(
        open_choice.legal_decision_actions,
        vec![
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 0,
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 1,
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 2,
            },
        ]
    );

    let card_count_before = engine.run_state.deck.len();
    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 1,
    });
    assert!(choose.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck.len(), card_count_before + 1);
}

#[test]
fn winding_halls_supported_branches_apply_heal_curse_and_max_hp_changes() {
    let mut heal_engine = RunEngine::new(11, 20);
    let halls = typed_event(3, "Winding Halls");
    assert!(matches!(
        halls.options[1].status,
        EventRuntimeStatus::Supported
    ));
    assert!(matches!(
        halls.options[2].status,
        EventRuntimeStatus::Supported
    ));

    heal_engine.run_state.current_hp = 12;
    heal_engine.debug_set_typed_event_state(halls.clone());
    let heal_step = heal_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(heal_step.action_accepted);
    assert_eq!(heal_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(heal_engine.run_state.current_hp, heal_engine.run_state.max_hp);
    assert!(heal_engine.run_state.deck.iter().any(|card| card == "Writhe"));

    let mut max_hp_engine = RunEngine::new(11, 20);
    let max_hp_before = max_hp_engine.run_state.max_hp;
    let hp_before = max_hp_engine.run_state.current_hp;
    max_hp_engine.debug_set_typed_event_state(halls);
    let max_hp_step = max_hp_engine.step_with_result(&RunAction::EventChoice(2));
    assert!(max_hp_step.action_accepted);
    assert_eq!(max_hp_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(max_hp_engine.run_state.max_hp, max_hp_before - 3);
    assert_eq!(max_hp_engine.run_state.current_hp, hp_before - 3);
}

#[test]
fn woman_in_blue_routes_potion_rewards_through_ordered_event_screen() {
    let mut engine = RunEngine::new(19, 20);
    let gold_before = engine.run_state.gold;
    engine.debug_set_typed_event_state(typed_shrine_event("The Woman in Blue"));

    let step = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(step.action_accepted);
    assert_eq!(engine.run_state.gold, gold_before - 30);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine
        .current_reward_screen()
        .expect("potion reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 2);
    assert!(screen.items.iter().all(|item| item.kind == RewardItemKind::Potion));
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);
}

#[test]
fn blocked_map_or_combat_event_branches_no_op_and_return_to_map() {
    let mut portal_engine = RunEngine::new(23, 20);
    let portal = typed_event(3, "Secret Portal");
    assert!(matches!(
        portal.options[0].status,
        EventRuntimeStatus::Blocked { .. }
    ));
    let floor_before = portal_engine.run_state.floor;
    let hp_before = portal_engine.run_state.current_hp;
    portal_engine.debug_set_typed_event_state(portal);

    let step = portal_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(portal_engine.current_phase(), RunPhase::MapChoice);
    assert!(portal_engine.current_reward_screen().is_none());
    assert_eq!(portal_engine.run_state.floor, floor_before);
    assert_eq!(portal_engine.run_state.current_hp, hp_before);

    let mut sphere_engine = RunEngine::new(29, 20);
    let sphere = typed_event(3, "Mysterious Sphere");
    assert!(matches!(
        sphere.options[0].status,
        EventRuntimeStatus::Blocked { .. }
    ));
    let relic_count_before = sphere_engine.run_state.relics.len();
    sphere_engine.debug_set_typed_event_state(sphere);

    let sphere_step = sphere_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(sphere_step.action_accepted);
    assert_eq!(sphere_engine.current_phase(), RunPhase::MapChoice);
    assert!(sphere_engine.current_reward_screen().is_none());
    assert_eq!(sphere_engine.run_state.relics.len(), relic_count_before);
}
