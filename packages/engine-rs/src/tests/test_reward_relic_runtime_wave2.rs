use crate::decision::{DecisionAction, RewardItemKind, RewardScreenSource};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine, RunPhase};
use crate::tests::support::resolve_opening_neow;

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
fn maw_bank_pays_on_shop_entry_then_is_used_up_by_the_first_purchase() {
    // MawBank.java::onEnterRoom gains 12 in every room while active, including
    // a ShopRoom; onSpendGold then permanently sets its counter to used-up.
    let mut engine = RunEngine::new(42, 0);
    engine.run_state.relics.push("MawBank".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.gold = 999;
    resolve_opening_neow(&mut engine);
    set_first_reachable_room(&mut engine, RoomType::Shop);
    let gold_before = engine.run_state.gold;

    let enter = engine.get_legal_actions()[0].clone();
    assert!(engine.step_with_result(&enter).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Shop);
    assert_eq!(engine.run_state.gold, gold_before + 12);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD],
        0
    );

    assert!(engine
        .step_with_result(&RunAction::ShopBuyCard(0))
        .action_accepted);
    assert_eq!(
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::MAW_BANK_GOLD],
        -2
    );
}

#[test]
fn meal_ticket_heals_exactly_fifteen_on_shop_entry() {
    // MealTicket.java::justEnteredRoom heals 15 only for ShopRoom. Because the
    // room is not in COMBAT, MagicFlower.java does not multiply this heal.
    let mut engine = RunEngine::new(44, 0);
    engine.run_state.relics.extend([
        "MealTicket".to_string(),
        "Magic Flower".to_string(),
    ]);
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.current_hp = 40;
    resolve_opening_neow(&mut engine);
    set_first_reachable_room(&mut engine, RoomType::Shop);

    let enter = engine.get_legal_actions()[0].clone();
    assert!(engine.step_with_result(&enter).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Shop);
    assert_eq!(engine.run_state.current_hp, 55);
}

#[test]
fn juzu_bracelet_converts_a_mystery_monster_roll_into_an_event() {
    // EventHelper.java rolls MONSTER first in the 100-slot mystery table,
    // resets MONSTER_CHANCE, then Juzu Bracelet converts MONSTER to EVENT.
    let mut baseline = RunEngine::new(51, 0);
    resolve_opening_neow(&mut baseline);
    set_first_reachable_room(&mut baseline, RoomType::Event);
    baseline.debug_force_event_rolls(&[0]);
    let enter = baseline.get_legal_actions()[0].clone();
    assert!(baseline.step_with_result(&enter).action_accepted);
    assert_eq!(baseline.current_phase(), RunPhase::Combat);
    assert_eq!(baseline.run_state.event_monster_chance, 10);

    let mut protected = RunEngine::new(51, 0);
    protected.run_state.relics.push("Juzu Bracelet".to_string());
    protected
        .run_state
        .relic_flags
        .rebuild(&protected.run_state.relics);
    resolve_opening_neow(&mut protected);
    set_first_reachable_room(&mut protected, RoomType::Event);
    protected.debug_force_event_rolls(&[0]);
    let enter = protected.get_legal_actions()[0].clone();
    assert!(protected.step_with_result(&enter).action_accepted);
    assert_eq!(protected.current_phase(), RunPhase::Event);
    assert_eq!(protected.run_state.event_monster_chance, 10);
}

#[test]
fn matryoshka_treasure_room_builds_ordered_chest_reward_screen() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Matryoshka".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    resolve_opening_neow(&mut engine);
    set_first_reachable_room(&mut engine, RoomType::Treasure);

    let actions = engine.get_legal_actions();
    let step = engine.step_with_result(&actions[0]);
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    let screen = engine
        .current_reward_screen()
        .expect("treasure reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Treasure);
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
    resolve_opening_neow(&mut engine);
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

#[test]
fn matryoshka_extra_chest_reward_uses_only_common_or_uncommon_tiers() {
    // Matryoshka.java::onChestOpen rolls COMMON at 75%, otherwise UNCOMMON;
    // it never adds a RARE relic. Exercise both branches on treasure screens.
    const COMMON: &[&str] = &[
        "Akabeko", "Anchor", "Ancient Tea Set", "Art of War", "Bag of Marbles",
        "Bag of Preparation", "Blood Vial", "Boot", "Bronze Scales", "CeramicFish",
        "Dream Catcher", "Lantern", "Omamori", "Vajra",
    ];
    const UNCOMMON: &[&str] = &[
        "Blue Candle", "Bottled Flame", "Bottled Lightning", "Bottled Tornado",
        "Darkstone Periapt", "Eternal Feather", "Frozen Egg 2", "InkBottle", "Kunai",
        "Letter Opener", "Matryoshka", "Molten Egg 2", "Ornamental Fan",
        "Toxic Egg 2", "White Beast Statue",
    ];
    let mut saw_common = false;
    let mut saw_uncommon = false;
    for seed in 0..256 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.relics.push("Matryoshka".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.run_state.relic_flags.init_relic_counter("Matryoshka");
        engine.debug_build_treasure_reward_screen();
        let extra = &engine.current_reward_screen().expect("treasure").items[2].label;
        saw_common |= COMMON.contains(&extra.as_str());
        saw_uncommon |= UNCOMMON.contains(&extra.as_str());
        assert!(COMMON.contains(&extra.as_str()) || UNCOMMON.contains(&extra.as_str()));
    }
    assert!(saw_common && saw_uncommon);
}
