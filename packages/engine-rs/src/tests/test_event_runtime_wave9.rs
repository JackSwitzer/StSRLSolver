use crate::decision::{
    RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen, RewardScreenSource,
};
use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus};
use crate::run::{GameAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn secret_portal_requires_java_accept_confirmation_before_boss_combat() {
    let mut engine = RunEngine::new(73, 0);
    engine.run_state.act = 3;
    engine.run_state.floor = 42;
    let secret_portal = typed_event(3, "Secret Portal");
    assert!(matches!(
        secret_portal.options[0].status,
        EventRuntimeStatus::Supported
    ));

    engine.debug_set_typed_event_state(secret_portal);
    // SecretPortal.java changes INTRO to ACCEPT on the first click and only
    // enters the boss room when its single confirmation option is clicked.
    let confirm = engine.step_game(&GameAction::EventChoice(0));
    assert!(confirm.accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.floor, 42);
    assert!(engine.get_combat_engine().is_none());
    assert_eq!(engine.get_legal_actions(), vec![GameAction::EventChoice(0)]);

    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    assert_eq!(engine.run_state.floor, 43);
    assert!(engine.get_combat_engine().is_some());

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::Transition);
    assert_eq!(engine.run_state.floor, 43);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.floor, 44);
    assert_eq!(
        engine
            .debug_current_event()
            .as_ref()
            .map(|event| event.name.as_str()),
        Some("Spire Heart")
    );
    assert!(engine.current_reward_screen().is_none());
}

#[test]
fn secret_portal_leave_requires_java_confirmation_without_changing_run_state() {
    let mut engine = RunEngine::new(73, 0);
    engine.run_state.act = 3;
    engine.run_state.floor = 42;
    engine.debug_set_typed_event_state(typed_event(3, "Secret Portal"));
    let before_rng = engine.rng_counters();

    assert!(engine.step_game(&GameAction::EventChoice(1)).accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.floor, 42);
    assert_eq!(engine.rng_counters(), before_rng);
    assert_eq!(engine.get_legal_actions(), vec![GameAction::EventChoice(0)]);

    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.floor, 42);
    assert_eq!(engine.rng_counters(), before_rng);
}

#[test]
fn deck_selection_reward_screen_removes_the_chosen_card() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.deck = vec![
        "Strike".to_string(),
        "Wallop".to_string(),
        "Vigilance".to_string(),
    ];
    engine.debug_set_reward_screen(RewardScreen {
        source: RewardScreenSource::Event,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::CardChoice,
            state: RewardItemState::Available,
            label: "deck_selection_purge".to_string(),
            claimable: true,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: vec![
                RewardChoice::Card {
                    index: 0,
                    card_id: "Strike".to_string(),
                },
                RewardChoice::Card {
                    index: 1,
                    card_id: "Wallop".to_string(),
                },
                RewardChoice::Card {
                    index: 2,
                    card_id: "Vigilance".to_string(),
                },
            ],
        }],
    });

    let open = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(open.accepted());
    assert_eq!(
        open.next_decision
            .context
            .reward_screen
            .as_ref()
            .and_then(|screen| screen.active_item),
        Some(0)
    );

    let choose = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 1,
    });
    assert!(choose.accepted());
    assert!(!engine.run_state.deck.iter().any(|card| card == "Wallop"));
    assert_eq!(engine.run_state.deck.len(), 2);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
}

#[test]
fn wheel_of_change_uses_the_shared_random_outcome_table() {
    let wheel = crate::events::typed_shrine_events()
        .into_iter()
        .find(|event| event.name == "Wheel of Change")
        .expect("missing wheel of change");
    assert!(matches!(
        wheel.options[0].status,
        EventRuntimeStatus::Supported
    ));
    match wheel.options[0].program.ops.as_slice() {
        [EventProgramOp::RandomOutcomeTable { outcomes }] => {
            assert_eq!(outcomes.len(), 6);
            assert!(outcomes.iter().any(|program| {
                program
                    .ops
                    .iter()
                    .any(|op| matches!(op, EventProgramOp::DeckSelection { .. }))
            }));
        }
        other => panic!("expected random outcome table, found {other:?}"),
    }
}
