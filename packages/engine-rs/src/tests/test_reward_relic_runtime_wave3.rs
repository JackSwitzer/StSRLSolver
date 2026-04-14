use crate::decision::{
    DecisionAction, RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen,
    RewardScreenSource,
};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine, RunPhase};

fn single_relic_reward_screen(label: &str) -> RewardScreen {
    RewardScreen {
        source: RewardScreenSource::Combat,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::Relic,
            state: RewardItemState::Available,
            label: label.to_string(),
            claimable: true,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: Vec::new(),
        }],
    }
}

fn relic_choice_reward_screen(labels: &[&str]) -> RewardScreen {
    RewardScreen {
        source: RewardScreenSource::Combat,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::Relic,
            state: RewardItemState::Available,
            label: "boss_relic_reward".to_string(),
            claimable: true,
            active: false,
            skip_allowed: false,
            skip_label: None,
            choices: labels
                .iter()
                .enumerate()
                .map(|(index, label)| RewardChoice::Named {
                    index,
                    label: (*label).to_string(),
                })
                .collect(),
        }],
    }
}

#[test]
fn claiming_question_card_expands_later_card_reward_choices() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Sozu".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_set_reward_screen(single_relic_reward_screen("QuestionCard"));

    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    engine.debug_build_combat_reward_screen(RoomType::Monster);
    let screen = engine
        .current_reward_screen()
        .expect("question card should mutate later combat rewards");
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].choices.len(), 4);
    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![
            DecisionAction::ClaimRewardItem { item_index: 0 },
            DecisionAction::SkipRewardItem { item_index: 0 },
        ]
    );
}

#[test]
fn claiming_prayer_wheel_adds_second_ordered_card_reward_item() {
    let mut engine = RunEngine::new(7, 20);
    engine.run_state.relics.push("Sozu".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_set_reward_screen(single_relic_reward_screen("PrayerWheel"));

    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);

    engine.debug_build_combat_reward_screen(RoomType::Monster);
    let screen = engine
        .current_reward_screen()
        .expect("prayer wheel should mutate later combat rewards");
    assert_eq!(screen.items.len(), 2);
    assert!(screen
        .items
        .iter()
        .all(|item| item.kind == RewardItemKind::CardChoice));
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);

    let open_first = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_first.action_accepted);
    let pick_first = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(pick_first.action_accepted);
    assert_eq!(
        pick_first.legal_decision_actions,
        vec![
            DecisionAction::ClaimRewardItem { item_index: 1 },
            DecisionAction::SkipRewardItem { item_index: 1 },
        ]
    );
}

#[test]
fn claiming_singing_bowl_turns_future_card_skip_into_max_hp() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_reward_screen(single_relic_reward_screen("SingingBowl"));
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);

    let max_hp_before = engine.run_state.max_hp;
    let hp_before = engine.run_state.current_hp;
    engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);
    let screen = engine
        .current_reward_screen()
        .expect("card reward screen should exist");
    assert_eq!(screen.items[0].skip_label.as_deref(), Some("+2 Max HP"));

    let skip = engine.step_with_result(&RunAction::SkipRewardItem(0));
    assert!(skip.action_accepted);
    assert_eq!(engine.run_state.max_hp, max_hp_before + 2);
    assert_eq!(engine.run_state.current_hp, hp_before + 2);
}

#[test]
fn choosing_black_star_from_relic_choice_doubles_future_elite_relic_rewards() {
    let mut engine = RunEngine::new(99, 20);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&[
        "BlackStar",
        "SacredBark",
        "Snecko Eye",
    ]));

    let open = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);
    assert_eq!(
        open.legal_decision_actions,
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

    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose.action_accepted);
    assert!(engine.run_state.relic_flags.has(crate::relic_flags::flag::BLACK_STAR));

    engine.debug_build_combat_reward_screen(RoomType::Elite);
    let screen = engine
        .current_reward_screen()
        .expect("black star should mutate future elite rewards");
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);
}

#[test]
fn white_beast_statue_flag_guarantees_potion_reward_on_ordered_screen() {
    let mut engine = RunEngine::new(5, 20);
    engine.debug_set_reward_screen(single_relic_reward_screen("White Beast Statue"));
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert!(engine.run_state.relic_flags.has(crate::relic_flags::flag::WHITE_BEAST));

    engine.debug_build_combat_reward_screen(RoomType::Monster);
    let screen = engine
        .current_reward_screen()
        .expect("white beast should guarantee potion rewards");
    assert_eq!(screen.items[0].kind, RewardItemKind::Potion);
    assert!(screen.items[0].claimable);
    assert_eq!(screen.items[1].kind, RewardItemKind::CardChoice);
    assert!(!screen.items[1].claimable);
}

#[test]
fn choosing_sacred_bark_uses_only_real_reward_choice_actions() {
    let mut engine = RunEngine::new(123, 20);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&[
        "BlackStar",
        "SacredBark",
        "Velvet Choker",
    ]));

    let open = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);
    assert_eq!(open.decision_context.reward_screen.as_ref().and_then(|s| s.active_item), Some(0));
    assert_eq!(open.legal_actions.len(), 3);

    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 1,
    });
    assert!(choose.action_accepted);
    assert!(engine.run_state.relic_flags.has(crate::relic_flags::flag::SACRED_BARK));
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
}

#[test]
fn claiming_matryoshka_mutates_next_two_chests_then_expires() {
    let mut engine = RunEngine::new(321, 20);
    engine.debug_set_reward_screen(single_relic_reward_screen("Matryoshka"));
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
        2
    );

    engine.debug_build_treasure_reward_screen();
    let first = engine
        .current_reward_screen()
        .expect("first treasure reward screen should exist");
    assert_eq!(first.items.len(), 3);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
        1
    );

    engine.debug_build_treasure_reward_screen();
    let second = engine
        .current_reward_screen()
        .expect("second treasure reward screen should exist");
    assert_eq!(second.items.len(), 3);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
        0
    );

    engine.debug_build_treasure_reward_screen();
    let third = engine
        .current_reward_screen()
        .expect("third treasure reward screen should exist");
    assert_eq!(third.items.len(), 2);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
        0
    );
}
