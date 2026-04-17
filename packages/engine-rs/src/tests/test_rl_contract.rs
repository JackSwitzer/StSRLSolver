use crate::actions::Action;
use crate::decision::{DecisionAction, DecisionKind, RewardChoice, RewardItemKind, RewardItemState};
use crate::events::{EventDef, EventEffect, EventOption};
use crate::obs::{
    encode_combat_state_v2, get_observation, ACTION_FEAT_DIM, COMBAT_DIM, COMBAT_OBS_VERSION,
    RUN_DECISION_TAIL_OFFSET, STATE_DIM,
};
use crate::run::{RunAction, RunEngine, RunPhase, ShopState};
use crate::{PyRunEngine, COMBAT_BASE};
use std::sync::{Mutex, OnceLock};

fn python_bridge_guard() -> std::sync::MutexGuard<'static, ()> {
    static PYTHON_BRIDGE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    PYTHON_BRIDGE_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn enter_test_combat(engine: &mut RunEngine) {
    if engine.current_phase() == RunPhase::Neow {
        let neow = engine
            .get_legal_actions()
            .into_iter()
            .next()
            .expect("expected a Neow action");
        let result = engine.step_with_result(&neow);
        assert!(result.action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    }

    let action = engine
        .get_legal_actions()
        .into_iter()
        .next()
        .expect("expected a map action");
    let result = engine.step_with_result(&action);
    assert!(result.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
}

#[test]
fn combat_action_encoding_stays_non_overlapping_for_large_hand_indices() {
    let _guard = python_bridge_guard();
    let engine = PyRunEngine::new_py(42, 20);

    let play = RunAction::CombatAction(Action::PlayCard {
        card_idx: 16,
        target_idx: 0,
    });
    let potion = RunAction::CombatAction(Action::UsePotion {
        potion_idx: 0,
        target_idx: 0,
    });

    let play_id = engine.encode_action(&play);
    let potion_id = engine.encode_action(&potion);

    assert_ne!(play_id, potion_id, "card plays and potion uses must never collide");
    assert_eq!(engine.decode_action(play_id), Some(play));
    assert_eq!(engine.decode_action(potion_id), Some(potion));
}

#[test]
fn invalid_run_actions_are_rejected_by_step_result() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    enter_test_combat(&mut engine);

    let hp_before = engine
        .get_combat_engine()
        .expect("combat should be active")
        .state
        .player
        .hp;
    let result = engine.step_with_result(&RunAction::CombatAction(Action::PlayCard {
        card_idx: 99,
        target_idx: -1,
    }));

    assert!(!result.action_accepted, "illegal actions should be surfaced, not silently accepted");
    assert_eq!(result.reward, 0.0);
    assert_eq!(
        engine
            .get_combat_engine()
            .expect("combat should be active")
            .state
            .player
            .hp,
        hp_before
    );
    assert_eq!(result.phase, RunPhase::Combat);
}

#[test]
fn combat_obs_v3_exposes_potions_and_choice_context() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.deck = vec![
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Strike_P".to_string(),
        "ThirdEye".to_string(),
    ];
    engine.run_state.potions[0] = "Block Potion".to_string();
    enter_test_combat(&mut engine);

    let combat = engine
        .get_combat_engine()
        .expect("combat should be active");
    assert!(
        !combat.state.draw_pile.is_empty(),
        "Third Eye needs cards left in the draw pile to produce a scry choice"
    );
    let hand = &combat.state.hand;
    let registry = crate::cards::global_registry();
    let third_eye_idx = hand
        .iter()
        .position(|card| registry.card_name(card.def_id) == "ThirdEye")
        .expect("Third Eye should be in hand");

    let result = engine.step_with_result(&RunAction::CombatAction(Action::PlayCard {
        card_idx: third_eye_idx,
        target_idx: -1,
    }));

    assert!(result.action_accepted);
    assert_eq!(COMBAT_OBS_VERSION, 3);
    let context = result
        .combat_context
        .as_ref()
        .expect("combat context should be present while combat is active");
    assert_eq!(context.potions.len(), 3);
    assert!(context.potions[0].occupied);
    assert_eq!(context.potions[0].potion_id, "Block Potion");
    assert!(!context.potions[0].requires_target);
    assert!(context.choice.active);
    assert_eq!(context.choice.reason.as_deref(), Some("scry"));
    assert_eq!(context.choice.option_count, context.choice.options.len());
    assert!(!context.choice.options.is_empty());
    assert!(context
        .choice
        .options
        .iter()
        .all(|option| !option.label.is_empty()));
    assert!(result
        .combat_events
        .iter()
        .any(|record| record.event == crate::effects::trigger::Trigger::OnSkillPlayed
            && record.card_type == Some(crate::cards::CardType::Skill)));

    let obs = encode_combat_state_v2(&engine);
    let potion_offset = COMBAT_DIM + 18;
    assert_eq!(obs[potion_offset], 1.0, "slot 0 should report a potion");
    assert_eq!(obs[potion_offset + 3], 1.0, "Block Potion should mark the defensive bucket");

    let choice_offset = COMBAT_DIM + 18 + 12;
    assert_eq!(obs[choice_offset], 1.0, "choice state should be active after Third Eye");
    assert_eq!(obs[choice_offset + 1], 1.0, "Third Eye should expose the scry reason");
}

#[test]
fn end_turn_encoding_stays_stable() {
    let _guard = python_bridge_guard();
    let engine = PyRunEngine::new_py(42, 20);
    let action = RunAction::CombatAction(Action::EndTurn);
    let encoded = engine.encode_action(&action);
    assert_eq!(encoded, COMBAT_BASE);
    assert_eq!(engine.decode_action(encoded), Some(action));
}

#[test]
fn combat_decision_state_and_actions_are_exposed() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    enter_test_combat(&mut engine);

    let state = engine.current_decision_state();
    assert_eq!(state.phase, RunPhase::Combat);
    assert_eq!(state.kind, DecisionKind::CombatAction);

    let context = engine.current_decision_context();
    assert!(context.combat.is_some());
    assert!(context.reward_screen.is_none());

    let legal = engine.get_legal_decision_actions();
    assert!(!legal.is_empty());
    assert!(legal.iter().all(|action| matches!(action, DecisionAction::Combat(_))));

    for action in legal.iter().take(3) {
        let run_action = action.to_run_action();
        assert!(engine.get_legal_actions().contains(&run_action));
    }
}

#[test]
fn card_reward_decision_context_surfaces_structured_reward_screen() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_card_reward_screen(vec![
        "TalkToTheHand".to_string(),
        "Wallop".to_string(),
        "Scrawl".to_string(),
    ]);

    let state = engine.current_decision_state();
    assert_eq!(state.kind, DecisionKind::RewardScreen);

    let context = engine.current_decision_context();
    let screen = context
        .reward_screen
        .as_ref()
        .expect("reward screen should be present in card reward phase");
    assert!(screen.ordered);
    assert_eq!(screen.active_item, None);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].state, RewardItemState::Available);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[0].active);
    assert!(screen.items[0].skip_allowed);
    assert_eq!(screen.items[0].skip_label.as_deref(), Some("Skip"));
    assert_eq!(screen.items[0].choices.len(), 3);
    assert!(matches!(
        screen.items[0].choices[1],
        RewardChoice::Card {
            index: 1,
            ref card_id
        } if card_id == "Wallop"
    ));

    let legal = engine.get_legal_decision_actions();
    assert_eq!(
        legal,
        vec![
            DecisionAction::ClaimRewardItem { item_index: 0 },
            DecisionAction::SkipRewardItem { item_index: 0 },
        ]
    );

    let reward_obs = get_observation(&engine);
    assert_eq!(reward_obs[RUN_DECISION_TAIL_OFFSET + 1], 0.25);
    assert_eq!(reward_obs[RUN_DECISION_TAIL_OFFSET + 2], 0.0);
    assert_eq!(reward_obs[RUN_DECISION_TAIL_OFFSET + 3], 1.0);
    assert_eq!(reward_obs[RUN_DECISION_TAIL_OFFSET + 7], 0.0);
    assert_eq!(reward_obs[RUN_DECISION_TAIL_OFFSET + 8], 0.2);

    let step = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(step.action_accepted);
    assert_eq!(step.decision_state.kind, DecisionKind::RewardScreen);
    let active_screen = step
        .decision_context
        .reward_screen
        .as_ref()
        .expect("reward screen remains active while choosing");
    assert_eq!(active_screen.active_item, Some(0));
    assert!(active_screen.items[0].active);

    assert_eq!(
        step.legal_decision_actions,
        vec![
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 0
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 1
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 2
            },
            DecisionAction::SkipRewardItem { item_index: 0 }
        ]
    );

    let active_obs = get_observation(&engine);
    assert_eq!(active_obs[RUN_DECISION_TAIL_OFFSET + 1], 0.5);
    assert_eq!(active_obs[RUN_DECISION_TAIL_OFFSET + 2], 0.6);
    assert_eq!(active_obs[RUN_DECISION_TAIL_OFFSET + 3], 1.0);
    assert_eq!(active_obs[RUN_DECISION_TAIL_OFFSET + 7], 1.0);
    assert_eq!(active_obs[RUN_DECISION_TAIL_OFFSET + 8], 0.2);
}

#[test]
fn shop_and_event_decision_contexts_are_stable_and_bridged() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.gold = 120;
    engine.debug_set_shop_state(ShopState {
        cards: vec![("Wallop".to_string(), 80), ("Scrawl".to_string(), 150)],
        remove_price: 50,
        removal_used: false,
    });

    let shop_state = engine.current_decision_state();
    assert_eq!(shop_state.kind, DecisionKind::ShopAction);
    let shop_context = engine.current_decision_context();
    let shop = shop_context.shop.expect("shop context should be present");
    assert_eq!(shop.offers.len(), 2);
    assert!(shop.offers[0].affordable);
    assert!(!shop.offers[1].affordable);
    assert_eq!(shop.remove_price, 50);

    let shop_legal = engine.get_legal_decision_actions();
    assert!(shop_legal.contains(&DecisionAction::ShopBuyCard(0)));
    assert!(shop_legal.contains(&DecisionAction::ShopRemoveCard(0)));
    assert!(shop_legal.contains(&DecisionAction::ShopLeave));
    assert!(!shop_legal.contains(&DecisionAction::ShopBuyCard(1)));

    engine.debug_set_event_state(EventDef {
        name: "Golden Shrine".to_string(),
        options: vec![
            EventOption {
                text: "Pray".to_string(),
                effect: EventEffect::Gold(100),
            },
            EventOption {
                text: "Leave".to_string(),
                effect: EventEffect::Nothing,
            },
        ],
    });

    let event_state = engine.current_decision_state();
    assert_eq!(event_state.kind, DecisionKind::EventOption);
    let event_context = engine.current_decision_context();
    let event = event_context.event.expect("event context should be present");
    assert_eq!(event.name, "Golden Shrine");
    assert_eq!(event.options.len(), 2);
    assert_eq!(event.options[0].label, "Pray");

    let event_legal = engine.get_legal_decision_actions();
    assert_eq!(
        event_legal,
        vec![DecisionAction::EventChoice(0), DecisionAction::EventChoice(1)]
    );
}

#[test]
fn reward_action_features_distinguish_potion_and_boss_relic_states() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("White Beast Statue".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(crate::map::RoomType::Monster);

    let obs = get_observation(&engine);
    let reward_slot = STATE_DIM;
    assert_eq!(obs[reward_slot], 1.0, "reward actions should be marked");
    assert_eq!(obs[reward_slot + 1], 1.0, "first action should be reward item selection");
    assert_eq!(obs[reward_slot + 6], 1.0, "first reward item should encode as a potion");
    assert_eq!(obs[reward_slot + 10], 1.0, "first reward item should be claimable");

    let potion_claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(potion_claim.action_accepted);
    assert_eq!(
        potion_claim.legal_decision_actions,
        vec![
            DecisionAction::ClaimRewardItem { item_index: 1 },
            DecisionAction::SkipRewardItem { item_index: 1 },
        ]
    );

    let skip_slot = STATE_DIM + ACTION_FEAT_DIM;
    assert_eq!(obs[skip_slot + 3], 1.0, "second action should encode skip");
    assert_eq!(obs[skip_slot + 6], 1.0, "skip should still carry potion item kind");

    let py_engine = PyRunEngine::new_py(42, 20);
    let select = RunAction::SelectRewardItem(0);
    let select_id = py_engine.encode_action(&select);
    assert_eq!(py_engine.decode_action(select_id), Some(select));

    let mut boss_engine = RunEngine::new(7, 20);
    boss_engine.debug_build_boss_reward_screen();
    let claim_step = boss_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim_step.action_accepted);
    let boss_obs = get_observation(&boss_engine);
    let boss_slot = STATE_DIM;
    assert_eq!(boss_obs[boss_slot], 1.0);
    assert_eq!(boss_obs[boss_slot + 2], 1.0, "boss reward choice should encode option-pick action");
    assert_eq!(boss_obs[boss_slot + 5], 1.0, "boss reward should encode as relic item");
    assert_eq!(boss_obs[boss_slot + 9], 1.0, "active boss reward item should be visible");
    assert_eq!(boss_obs[RUN_DECISION_TAIL_OFFSET + 4], 1.0, "boss reward source should be visible");

    let mut treasure_engine = RunEngine::new(42, 20);
    treasure_engine.run_state.relics.push("Matryoshka".to_string());
    treasure_engine.run_state.relic_flags.rebuild(&treasure_engine.run_state.relics);
    treasure_engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    treasure_engine.debug_build_treasure_reward_screen();
    let treasure_obs = get_observation(&treasure_engine);
    assert_eq!(
        treasure_obs[RUN_DECISION_TAIL_OFFSET + 6],
        1.0,
        "treasure reward source should be visible in the RL tail"
    );
    assert_eq!(
        treasure_obs[RUN_DECISION_TAIL_OFFSET + 8],
        0.6,
        "Matryoshka should expand the treasure reward item count"
    );
}

#[test]
fn rl_surface_does_not_fabricate_blocked_campfire_or_empty_event_actions() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Coffee Dripper".to_string());
    engine.run_state.relics.push("Fusion Hammer".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.deck = vec!["Strike_P+".to_string(), "Defend_P+".to_string()];
    engine.debug_set_campfire_phase();

    let campfire_context = engine.current_decision_context();
    assert!(campfire_context.campfire.is_some());
    assert!(engine.get_legal_actions().is_empty());
    assert!(engine.get_legal_decision_actions().is_empty());

    engine.debug_clear_event_state();
    let event_state = engine.current_decision_state();
    assert_eq!(event_state.kind, DecisionKind::GameOver);
    assert!(engine.current_decision_context().event.is_none());
    assert!(engine.get_legal_actions().is_empty());
    assert!(engine.get_legal_decision_actions().is_empty());
}

#[test]
fn step_with_result_surfaces_illegal_actions_and_decision_context() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.deck = vec![
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Strike_P".to_string(),
        "ThirdEye".to_string(),
    ];
    engine.run_state.potions[0] = "Block Potion".to_string();
    let first_neow = engine
        .get_legal_actions()
        .into_iter()
        .next()
        .expect("expected a Neow action");
    let neow_step = engine.step_with_result(&first_neow);
    assert!(neow_step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    let first_map = engine
        .get_legal_actions()
        .into_iter()
        .next()
        .expect("expected a map action");
    let combat_step = engine.step_with_result(&first_map);
    assert!(combat_step.action_accepted);
    assert_eq!(combat_step.decision_state.kind, DecisionKind::CombatAction);
    assert_eq!(combat_step.decision_state.phase, RunPhase::Combat);
    assert!(combat_step.decision_context.combat.is_some());
    assert_eq!(
        combat_step
            .decision_context
            .combat
            .as_ref()
            .expect("combat context should exist")
            .potions
            .len(),
        3
    );

    let illegal_action = RunAction::CombatAction(Action::PlayCard {
        card_idx: 99,
        target_idx: -1,
    });
    let illegal_result = engine.step_with_result(&illegal_action);
    assert!(!illegal_result.action_accepted);
    assert_eq!(illegal_result.reward, 0.0);
    assert!(!illegal_result.legal_actions.is_empty());
}

#[test]
fn python_bridge_rejects_illegal_and_unknown_step_ids() {
    let _guard = python_bridge_guard();
    let mut engine = PyRunEngine::new_py(42, 20);
    let first_neow = engine
        .get_legal_actions()
        .into_iter()
        .next()
        .expect("expected a neow action");
    engine
        .step(first_neow)
        .expect("expected legal neow action to execute");

    let illegal = engine.step(COMBAT_BASE + (99 << 8));
    assert!(illegal.is_err(), "illegal actions should not be silent no-ops");

    let unknown = engine.step(-999);
    assert!(unknown.is_err(), "unknown action ids should raise immediately");
}

#[test]
fn decision_accessors_match_canonical_run_state() {
    let _guard = python_bridge_guard();
    let mut engine = RunEngine::new(7, 20);

    let state = engine.current_decision_state();
    assert_eq!(state.kind, DecisionKind::NeowChoice);
    assert_eq!(state.phase, RunPhase::Neow);

    let context = engine.current_decision_context();
    let neow = context
        .neow
        .as_ref()
        .expect("neow context should exist at the start of the run");
    assert_eq!(neow.options.len(), 4);
    assert!(engine
        .get_legal_decision_actions()
        .iter()
        .all(|action| matches!(action, DecisionAction::ChooseNeowOption(_))));

    let legal = engine.get_legal_decision_actions();
    assert_eq!(
        legal.iter().map(|action| action.to_run_action()).collect::<Vec<_>>(),
        engine.get_legal_actions()
    );

    let first_neow = engine.get_legal_actions()[0].clone();
    let result = engine.step_with_result(&first_neow);
    assert!(result.action_accepted);
    assert!(result.reward >= 0.0);

    let context = engine.current_decision_context();
    assert_eq!(context.kind, DecisionKind::MapPath);
    assert!(context.neow.is_none());

    let next_map = engine.get_legal_actions()[0].clone();
    let result = engine.step_with_result(&next_map);
    assert!(result.action_accepted);
    assert_eq!(result.decision_state.kind, DecisionKind::CombatAction);
    let combat = result
        .decision_context
        .combat
        .as_ref()
        .expect("combat context should exist");
    assert_eq!(combat.potions.len(), 3);

    engine.run_state.relics.push("Matryoshka".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    engine.debug_build_treasure_reward_screen();

    let treasure_state = engine.current_decision_state();
    assert_eq!(treasure_state.kind, DecisionKind::RewardScreen);
    let treasure_context = engine.current_decision_context();
    let treasure_screen = treasure_context
        .reward_screen
        .as_ref()
        .expect("treasure reward screen should be available");
    assert_eq!(treasure_screen.source, crate::decision::RewardScreenSource::Treasure);
    assert_eq!(treasure_screen.items.len(), 3);
}
