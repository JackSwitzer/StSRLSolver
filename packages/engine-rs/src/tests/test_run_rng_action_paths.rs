use crate::decision::{
    RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen, RewardScreenSource,
};
use crate::run::{GameAction, RunEngine};

fn tiny_house_boss_relic_screen() -> RewardScreen {
    RewardScreen {
        source: RewardScreenSource::BossRelic,
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
            choices: vec![RewardChoice::Named {
                index: 0,
                label: "Tiny House".to_string(),
            }],
        }],
    }
}

#[test]
fn neow_three_potions_action_path_preserves_order_duplicates_and_rng_ownership() {
    // NeowReward.java::activate(THREE_SMALL_POTIONS) calls
    // PotionHelper.getRandomPotion() three times in insertion order. The helper
    // indexes the initialized Watcher potion list directly, so duplicates are
    // allowed and no stream other than AbstractDungeon.potionRng is consumed.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/neow/NeowReward.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let mut engine = RunEngine::new(0, 0);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    let neow = engine
        .current_decision_context()
        .neow
        .expect("Neow choice frame must expose options");
    assert_eq!(neow.options[1].reward_id, "THREE_SMALL_POTIONS");

    let before = engine.rng_counters();
    assert!(engine
        .step_game(&GameAction::ChooseNeowOption(1))
        .accepted());

    let screen = engine
        .current_reward_screen()
        .expect("three potions must open a reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(
        screen
            .items
            .iter()
            .map(|item| (item.kind, item.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (RewardItemKind::Potion, "StancePotion"),
            (RewardItemKind::Potion, "StancePotion"),
            (RewardItemKind::Potion, "Block Potion"),
        ]
    );
    for item_index in 0..3 {
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(item_index))
            .accepted());
    }
    assert_eq!(
        engine.run_state.potions,
        ["StancePotion", "StancePotion", "Block Potion"]
    );

    let after = engine.rng_counters();
    assert_eq!(after["potion"], before["potion"] + 3);
    let mut expected = before;
    *expected.get_mut("potion").expect("potion RNG counter") += 3;
    assert_eq!(after, expected);
}

#[test]
fn tiny_house_boss_choice_upgrades_exact_duplicate_instance_and_orders_rewards() {
    // TinyHouse.java::onEquip collects every upgradeable physical master-deck
    // card, seeds Collections.shuffle with one miscRng.randomLong(), upgrades
    // the first shuffled entry, then uses one more miscRng draw for the potion.
    // It adds 50 gold before that potion and never consumes potionRng.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TinyHouse.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let mut engine = RunEngine::new(1, 0);
    let registry = crate::cards::global_registry();
    engine.run_state.deck = vec![
        "Strike_P".to_string(),
        "Strike_P".to_string(),
        "Defend_P".to_string(),
        "Eruption+".to_string(),
    ];
    engine.run_state.deck_card_states = [
        ("Strike_P", 101, 11),
        ("Strike_P", 202, 22),
        ("Defend_P", 303, 33),
        ("Eruption+", 404, 44),
    ]
    .into_iter()
    .map(|(card_id, instance_id, misc)| {
        registry
            .make_card(card_id)
            .with_instance_id(instance_id)
            .with_misc(misc)
    })
    .collect();
    engine.run_state.next_card_instance_id = 405;
    let deck_states_before = engine.run_state.deck_card_states.clone();
    let gold_before = engine.run_state.gold;

    engine.debug_set_reward_screen(tiny_house_boss_relic_screen());
    let rng_before = engine.rng_counters();
    assert!(engine
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(engine
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());

    let rng_after_choice = engine.rng_counters();
    assert_eq!(rng_after_choice["misc"], rng_before["misc"] + 2);
    assert_eq!(rng_after_choice["potion"], rng_before["potion"]);
    let mut expected_rng = rng_before;
    *expected_rng.get_mut("misc").expect("misc RNG counter") += 2;
    assert_eq!(rng_after_choice, expected_rng);
    assert_eq!(
        engine.run_state.deck,
        ["Strike_P", "Strike_P+", "Defend_P", "Eruption+"]
    );
    assert_eq!(engine.run_state.deck_card_states[0], deck_states_before[0]);
    assert_eq!(engine.run_state.deck_card_states[2], deck_states_before[2]);
    assert_eq!(engine.run_state.deck_card_states[3], deck_states_before[3]);
    let upgraded = engine.run_state.deck_card_states[1];
    assert_eq!(upgraded.instance_id, 202);
    assert_eq!(upgraded.misc, 22);
    assert_eq!(registry.card_name(upgraded.def_id), "Strike_P+");
    assert!(upgraded.is_upgraded());

    let screen = engine
        .current_reward_screen()
        .expect("Tiny House must open its reward screen");
    assert_eq!(screen.source, RewardScreenSource::BossRelic);
    assert!(screen.ordered);
    assert_eq!(
        screen
            .items
            .iter()
            .map(|item| (item.kind, item.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (RewardItemKind::Gold, "50"),
            (RewardItemKind::Potion, "Energy Potion"),
        ]
    );
    assert!(screen.items[0].claimable);
    assert!(!screen.items[1].claimable);

    assert!(engine
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert_eq!(engine.run_state.gold, gold_before + 50);
    assert!(
        engine
            .current_reward_screen()
            .expect("potion reward remains")
            .items[1]
            .claimable
    );
    assert!(engine
        .step_game(&GameAction::SelectRewardItem(1))
        .accepted());
    assert_eq!(engine.run_state.potions[0], "Energy Potion");
    assert_eq!(engine.rng_counters(), expected_rng);
}

#[test]
fn ambient_monster_constructor_cursor_matches_java_and_survives_combat_transfer() {
    // Constructor order is encounter order. Cultist and Jaw Worm each consume
    // one MathUtils.random() animation-time draw; Fungi Beast consumes one for
    // time and one for time scale. These presentation draws share the global
    // cursor later used by gameplay-facing ambient selections.
    // Java: Cultist.java:73, JawWorm.java:106, FungiBeast.java:68-69.
    let initial = crate::seed::AmbientMathRng::new(42).state_tuple();
    let mut expected = crate::seed::AmbientMathRng::from_state(initial.0, initial.1);
    let _ = expected.random_f32();
    let _ = expected.random_f32();
    let _ = expected.random_f32();
    let _ = expected.random_f32_range(0.7, 1.0);

    let mut engine = RunEngine::new_with_ambient_states(877, 0, initial, 0x1234_5678_9ABC);
    engine.debug_enter_specific_combat(&["Cultist", "JawWorm", "FungiBeast"]);
    assert_eq!(engine.ambient_math_rng_state(), expected.state_tuple());

    // Restoration while combat owns the stream updates both sides of the
    // snapshot boundary. The next action absorbs that exact state back once.
    let restored = (0x1111_2222_3333_4444, 0x5555_6666_7777_8888);
    engine.restore_ambient_math_rng_state(restored);
    assert_eq!(engine.ambient_math_rng_state(), restored);
    assert!(engine
        .step_game(&GameAction::CombatAction(crate::actions::Action::EndTurn))
        .accepted());
    assert_eq!(engine.ambient_math_rng_state(), restored);
}
