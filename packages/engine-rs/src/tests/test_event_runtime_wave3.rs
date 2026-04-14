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
fn library_read_has_nested_choice_without_fake_skip() {
    let mut engine = RunEngine::new(101, 20);
    let library = typed_event(2, "The Library");
    assert!(matches!(
        library.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(library);

    let open_screen = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(open_screen.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine.current_reward_screen().expect("library reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[0].skip_allowed);

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
}

#[test]
fn drug_dealer_all_three_supported_branches_use_canonical_runtime_paths() {
    let dealer = typed_event(2, "Drug Dealer");
    assert!(dealer
        .options
        .iter()
        .all(|option| matches!(option.status, EventRuntimeStatus::Supported)));

    let mut jax_engine = RunEngine::new(17, 20);
    jax_engine.debug_set_typed_event_state(dealer.clone());
    let jax_step = jax_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(jax_step.action_accepted);
    assert_eq!(jax_engine.current_phase(), RunPhase::CardReward);
    let jax_screen = jax_engine.current_reward_screen().expect("jax reward screen");
    assert_eq!(jax_screen.items.len(), 1);
    assert_eq!(jax_screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(jax_screen.items[0].choices.len(), 1);
    let open_jax = jax_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_jax.action_accepted);
    let choose_jax = jax_engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose_jax.action_accepted);
    assert!(jax_engine.run_state.deck.iter().any(|card| card == "J.A.X."));

    let mut transform_engine = RunEngine::new(23, 20);
    let deck_before = transform_engine.run_state.deck.len();
    transform_engine.debug_set_typed_event_state(dealer.clone());
    let transform_step = transform_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(transform_step.action_accepted);
    assert_eq!(transform_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(transform_engine.run_state.deck.len(), deck_before);

    let mut relic_engine = RunEngine::new(31, 20);
    relic_engine.debug_set_typed_event_state(dealer);
    let relic_step = relic_engine.step_with_result(&RunAction::EventChoice(2));
    assert!(relic_step.action_accepted);
    assert_eq!(relic_engine.current_phase(), RunPhase::CardReward);
    let relic_screen = relic_engine.current_reward_screen().expect("relic screen");
    assert_eq!(relic_screen.items[0].kind, RewardItemKind::Relic);
    let relic_id = relic_screen.items[0].label.clone();
    let claim = relic_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert!(relic_engine.run_state.relics.iter().any(|relic| relic == &relic_id));
}

#[test]
fn nest_branches_cover_direct_gold_and_specific_card_reward() {
    let nest = typed_event(2, "Nest");
    assert!(nest
        .options
        .iter()
        .all(|option| matches!(option.status, EventRuntimeStatus::Supported)));

    let mut gold_engine = RunEngine::new(37, 20);
    let gold_before = gold_engine.run_state.gold;
    gold_engine.debug_set_typed_event_state(nest.clone());
    let steal = gold_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(steal.action_accepted);
    assert_eq!(gold_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(gold_engine.run_state.gold, gold_before + 99);

    let mut dagger_engine = RunEngine::new(41, 20);
    let hp_before = dagger_engine.run_state.current_hp;
    dagger_engine.debug_set_typed_event_state(nest);
    let join = dagger_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(join.action_accepted);
    assert_eq!(dagger_engine.current_phase(), RunPhase::CardReward);
    assert_eq!(dagger_engine.run_state.current_hp, hp_before - 6);
    let screen = dagger_engine.current_reward_screen().expect("ritual dagger screen");
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].choices.len(), 1);
    let open = dagger_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);
    let choose = dagger_engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose.action_accepted);
    assert!(dagger_engine
        .run_state
        .deck
        .iter()
        .any(|card| card == "RitualDagger"));
}

#[test]
fn sensory_stone_focus_and_tomb_of_lord_red_mask_flow_through_event_rewards() {
    let mut sensory_engine = RunEngine::new(43, 20);
    sensory_engine.debug_set_typed_event_state(typed_event(3, "Sensory Stone"));
    let sensory = sensory_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(sensory.action_accepted);
    assert_eq!(sensory_engine.current_phase(), RunPhase::CardReward);
    let sensory_screen = sensory_engine.current_reward_screen().expect("sensory screen");
    assert_eq!(sensory_screen.source, RewardScreenSource::Event);
    assert_eq!(sensory_screen.items[0].kind, RewardItemKind::CardChoice);

    let mut tomb_engine = RunEngine::new(47, 20);
    tomb_engine.debug_set_typed_event_state(typed_event(3, "Tomb of Lord Red Mask"));
    let tomb = tomb_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(tomb.action_accepted);
    assert_eq!(tomb_engine.current_phase(), RunPhase::CardReward);
    let tomb_screen = tomb_engine.current_reward_screen().expect("tomb screen");
    assert_eq!(tomb_screen.source, RewardScreenSource::Event);
    assert_eq!(tomb_screen.items.len(), 1);
    assert_eq!(tomb_screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(tomb_screen.items[0].label, "Red Mask");
}

#[test]
fn mushrooms_eat_branch_is_now_supported_heal_plus_curse() {
    let mut engine = RunEngine::new(53, 20);
    let mushrooms = typed_event(1, "Mushrooms");
    assert!(matches!(
        mushrooms.options[1].status,
        EventRuntimeStatus::Supported
    ));
    engine.run_state.current_hp = 25;
    engine.debug_set_typed_event_state(mushrooms);

    let step = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.run_state.current_hp > 25);
    assert!(engine.run_state.deck.iter().any(|card| card == "Parasite"));
}

#[test]
fn big_fish_banana_branch_applies_direct_max_hp_gain() {
    let mut engine = RunEngine::new(59, 20);
    let big_fish = typed_event(1, "Big Fish");
    let max_hp_before = engine.run_state.max_hp;
    let hp_before = engine.run_state.current_hp;
    engine.debug_set_typed_event_state(big_fish);

    let step = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.max_hp, max_hp_before + 2);
    assert_eq!(engine.run_state.current_hp, hp_before + 2);
}
