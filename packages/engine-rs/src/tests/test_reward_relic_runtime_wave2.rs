use crate::decision::{DecisionAction, RewardItemKind, RewardScreenSource};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine, RunPhase};

fn set_first_reachable_room(engine: &mut RunEngine, room_type: RoomType) {
    let start = engine.map.get_start_nodes()[0];
    let (x, y) = (start.x, start.y);
    engine.map.rows[y][x].room_type = room_type;
}

#[test]
fn black_star_elite_rewards_unlock_second_relic_before_other_rewards() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("BlackStar".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Elite);

    let screen = engine
        .current_reward_screen()
        .expect("elite reward screen should exist");
    assert_eq!(screen.items.len(), 4);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[2].kind, RewardItemKind::Potion);
    assert_eq!(screen.items[3].kind, RewardItemKind::CardChoice);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);
    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![DecisionAction::ClaimRewardItem { item_index: 0 }]
    );

    let second_relic = screen.items[1].label.clone();
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert!(
        engine.run_state.relics.iter().any(|relic| relic == &screen.items[0].label),
        "first elite relic should be added immediately"
    );
    assert_eq!(
        claim.legal_decision_actions,
        vec![DecisionAction::ClaimRewardItem { item_index: 1 }]
    );

    let claim_second = engine.step_with_result(&RunAction::SelectRewardItem(1));
    assert!(claim_second.action_accepted);
    assert!(
        engine.run_state.relics.iter().any(|relic| relic == &second_relic),
        "second elite relic should be added before potion or card rewards"
    );
    assert_eq!(
        claim_second.legal_decision_actions,
        vec![
            DecisionAction::ClaimRewardItem { item_index: 2 },
            DecisionAction::SkipRewardItem { item_index: 2 },
        ]
    );
}

#[test]
fn matryoshka_treasure_room_builds_ordered_chest_reward_screen() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Matryoshka".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    set_first_reachable_room(&mut engine, RoomType::Treasure);

    let actions = engine.get_legal_actions();
    let step = engine.step_with_result(&actions[0]);
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    let screen = engine
        .current_reward_screen()
        .expect("treasure reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Unknown);
    assert_eq!(screen.items.len(), 3);
    assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
    assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[2].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
        1,
        "opening the chest should consume one Matryoshka use"
    );
    assert_eq!(
        step.legal_decision_actions,
        vec![DecisionAction::ClaimRewardItem { item_index: 0 }]
    );
}

#[test]
fn matryoshka_chest_rewards_preserve_gold_then_relic_then_extra_relic_order() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Matryoshka".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    set_first_reachable_room(&mut engine, RoomType::Treasure);

    let first_action = engine.get_legal_actions()[0].clone();
    engine.step_with_result(&first_action);
    let screen = engine
        .current_reward_screen()
        .expect("treasure reward screen should exist");
    let gold_amount = screen.items[0]
        .label
        .parse::<i32>()
        .expect("gold reward should be numeric");
    let first_relic = screen.items[1].label.clone();
    let second_relic = screen.items[2].label.clone();
    let gold_before = engine.run_state.gold;

    let claim_gold = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim_gold.action_accepted);
    assert_eq!(engine.run_state.gold, gold_before + gold_amount);
    assert_eq!(
        claim_gold.legal_decision_actions,
        vec![DecisionAction::ClaimRewardItem { item_index: 1 }]
    );

    let claim_first_relic = engine.step_with_result(&RunAction::SelectRewardItem(1));
    assert!(claim_first_relic.action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == &first_relic));
    assert_eq!(
        claim_first_relic.legal_decision_actions,
        vec![DecisionAction::ClaimRewardItem { item_index: 2 }]
    );

    let claim_second_relic = engine.step_with_result(&RunAction::SelectRewardItem(2));
    assert!(claim_second_relic.action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == &second_relic));
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.current_reward_screen().is_none());
}
