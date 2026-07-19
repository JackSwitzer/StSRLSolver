use crate::decision::{
    DecisionAction, RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen,
    RewardScreenSource,
};
use crate::events::{EventDef, EventEffect, EventOption};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine};

fn assert_floor_rngs(engine: &RunEngine, seed: u64, floor: i32, counters: [i32; 5]) {
    const NAMES: [&str; 5] = ["monsterHp", "ai", "shuffle", "cardRandom", "misc"];
    let actual_counters = engine.rng_counters();
    for (name, expected) in NAMES.into_iter().zip(counters) {
        assert_eq!(actual_counters[name], i64::from(expected), "{name} counter");
    }

    let floor_seed = seed.wrapping_add(floor as u64);
    let expected_states = counters
        .map(|counter| crate::seed::StsRandom::with_counter(floor_seed, counter).state_tuple());
    assert_eq!(engine.debug_floor_rng_states(), expected_states);
}

#[test]
fn elite_reward_screen_orders_relic_before_card_choice() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_build_combat_reward_screen(RoomType::Elite);

    let screen = engine
        .current_reward_screen()
        .expect("elite reward screen should be present");
    assert!(screen.ordered);
    assert_eq!(screen.items.len(), 3);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);
    assert_eq!(screen.items[1].kind, RewardItemKind::Potion);
    assert!(!screen.items[1].claimable);
    assert!(screen.items[1].skip_allowed);
    assert_eq!(screen.items[2].kind, RewardItemKind::CardChoice);
    assert!(!screen.items[2].claimable);
    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![DecisionAction::ClaimRewardItem { item_index: 0 }]
    );
}

#[test]
fn reward_screen_requires_claim_before_card_choice() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_card_reward_screen(vec![
        "TalkToTheHand".to_string(),
        "Wallop".to_string(),
        "Scrawl".to_string(),
    ]);

    assert_eq!(
        engine.get_legal_decision_actions(),
        vec![
            DecisionAction::ClaimRewardItem { item_index: 0 },
            DecisionAction::SkipRewardItem { item_index: 0 },
        ]
    );

    let step = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(step.action_accepted);
    assert_eq!(
        step.legal_decision_actions,
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
            DecisionAction::SkipRewardItem { item_index: 0 },
        ]
    );
}

#[test]
fn prayer_wheel_and_question_card_expand_reward_structure() {
    // Sources: PrayerWheel.java and CombatRewardScreen.java::setupItemReward.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Prayer Wheel".to_string());
    engine.run_state.relics.push("Question Card".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("combat reward screen should exist");
    assert_eq!(screen.items.iter().filter(|item| item.kind == RewardItemKind::CardChoice).count(), 2);
    assert!(screen
        .items
        .iter()
        .filter(|item| item.kind == RewardItemKind::CardChoice)
        .all(|item| item.choices.len() == 4));
}

#[test]
fn claiming_egg_relic_upgrades_later_card_reward_choice() {
    // Source: MoltenEgg2.java uses canonical ID "Molten Egg 2" and onEquip
    // upgrades already-visible Attack cards through onPreviewObtainCard.
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_reward_screen(RewardScreen {
        source: RewardScreenSource::Combat,
        ordered: true,
        active_item: None,
        items: vec![
            RewardItem {
                index: 0,
                kind: RewardItemKind::Relic,
                state: RewardItemState::Available,
                label: "Molten Egg 2".to_string(),
                claimable: true,
                active: false,
                skip_allowed: false,
                skip_label: None,
                choices: Vec::new(),
            },
            RewardItem {
                index: 1,
                kind: RewardItemKind::CardChoice,
                state: RewardItemState::Available,
                label: "card_reward".to_string(),
                claimable: false,
                active: false,
                skip_allowed: true,
                skip_label: Some("Skip".to_string()),
                choices: vec![
                    RewardChoice::Card {
                        index: 0,
                        card_id: "Wallop".to_string(),
                    },
                    RewardChoice::Card {
                        index: 1,
                        card_id: "Scrawl".to_string(),
                    },
                ],
            },
        ],
    });

    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert!(engine.run_state.relic_flags.has(crate::relic_flags::flag::MOLTEN_EGG));
    let screen = engine.current_reward_screen().expect("reward screen remains open");
    assert!(matches!(
        &screen.items[1].choices[0],
        RewardChoice::Card { card_id, .. } if card_id == "Wallop+"
    ));
    assert!(matches!(
        &screen.items[1].choices[1],
        RewardChoice::Card { card_id, .. } if card_id == "Scrawl"
    ));
    assert_eq!(
        claim.legal_decision_actions,
        vec![
            DecisionAction::ClaimRewardItem { item_index: 1 },
            DecisionAction::SkipRewardItem { item_index: 1 },
        ]
    );

    let open = engine.step_with_result(&RunAction::SelectRewardItem(1));
    assert!(open.action_accepted);
    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 1,
        choice_index: 0,
    });
    assert!(choose.action_accepted);
    assert_eq!(engine.run_state.deck.last().map(String::as_str), Some("Wallop+"));
}

#[test]
fn singing_bowl_keeps_skip_separate_from_its_max_hp_choice() {
    // Sources: CardRewardScreen.java shows both buttons, while
    // SingingBowlButton.java::onClick alone grants 2 max HP.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Singing Bowl".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    let max_hp_before = engine.run_state.max_hp;
    let hp_before = engine.run_state.current_hp;
    engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

    let step = engine.step_with_result(&RunAction::SkipRewardItem(0));
    assert!(step.action_accepted);
    assert_eq!(engine.run_state.max_hp, max_hp_before);
    assert_eq!(engine.run_state.current_hp, hp_before);
}

#[test]
fn white_beast_adds_potion_reward_item_before_card_choice() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("White Beast Statue".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("reward screen should exist");
    assert_eq!(screen.items.len(), 2);
    assert_eq!(screen.items[0].kind, RewardItemKind::Potion);
    assert!(screen.items[0].claimable);
    assert!(screen.items[0].skip_allowed);
    assert_eq!(screen.items[1].kind, RewardItemKind::CardChoice);
    assert!(!screen.items[1].claimable);

    let offered_potion = screen.items[0].label.clone();
    let claim_potion = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim_potion.action_accepted);
    assert!(engine.run_state.potions.iter().any(|p| p == &offered_potion));
    let mut expected_actions = vec![
        DecisionAction::ClaimRewardItem { item_index: 1 },
        DecisionAction::SkipRewardItem { item_index: 1 },
    ];
    // FruitJuice.canUse permits use on non-combat reward screens.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
    if matches!(offered_potion.as_str(), "FruitJuice" | "Fruit Juice") {
        expected_actions.push(DecisionAction::UsePotion(0));
    }
    assert_eq!(
        claim_potion.legal_decision_actions,
        expected_actions
    );
}

#[test]
fn sozu_keeps_potion_reward_claimable_but_obtains_nothing() {
    // AbstractRoom.addPotionToRewards does not check Sozu. RewardItem checks
    // it only on claim, returns true, and does not call obtainPotion.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Sozu".to_string());
    engine.run_state.relics.push("White Beast Statue".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("reward screen should exist");
    assert_eq!(screen.items.len(), 2);
    assert_eq!(screen.items[0].kind, RewardItemKind::Potion);
    assert!(screen.items[0].claimable);
    let before = engine.run_state.potions.clone();
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
    assert_eq!(engine.run_state.potions, before);
    assert_eq!(engine.get_legal_decision_actions(), vec![
        DecisionAction::ClaimRewardItem { item_index: 1 },
        DecisionAction::SkipRewardItem { item_index: 1 },
    ]);
}

#[test]
fn full_inventory_leaves_a_potion_reward_unclaimed_without_sozu() {
    // RewardItem.claimReward returns false when obtainPotion cannot find a
    // PotionSlot, so the reward remains available instead of disappearing.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rewards/RewardItem.java
    let mut engine = RunEngine::new(43, 0);
    engine.run_state.potions = vec![
        "BlockPotion".to_string(),
        "FirePotion".to_string(),
        "SwiftPotion".to_string(),
    ];
    engine.debug_set_reward_screen(RewardScreen {
        source: RewardScreenSource::Combat,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::Potion,
            state: RewardItemState::Available,
            label: "EnergyPotion".to_string(),
            claimable: true,
            active: false,
            skip_allowed: true,
            skip_label: Some("Skip".to_string()),
            choices: Vec::new(),
        }],
    });
    let before = engine.run_state.potions.clone();
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
    assert_eq!(engine.run_state.potions, before);
    assert_eq!(engine.current_reward_screen().expect("reward").items[0].state,
        RewardItemState::Available);
}

#[test]
fn boss_reward_screen_requires_relic_choice_and_transitions_to_act_two() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.floor = 16;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    engine.run_state.current_hp = 20;
    engine.debug_build_boss_reward_screen();

    let screen = engine
        .current_reward_screen()
        .expect("boss reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::BossCombat);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[0].choices.len(), 3);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[0].skip_allowed);

    // Astrolabe.java::onEquip and CallingBell.java::onEquip/update open
    // mandatory nested decisions, so this generic immediate-completion test
    // deliberately chooses a boss relic without a nested decision.
    let choice_index = screen.items[0]
        .choices
        .iter()
        .position(|choice| {
            matches!(choice, RewardChoice::Named { label, .. }
                if label != "Astrolabe" && label != "Calling Bell")
        })
        .expect("boss screen should include a non-nested relic choice");
    let chosen_relic = match &screen.items[0].choices[choice_index] {
        RewardChoice::Named { label, .. } => label.clone(),
        other => panic!("expected named boss relic choice, got {other:?}"),
    };

    let open = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);
    assert_eq!(open.decision_context.reward_screen.as_ref().and_then(|s| s.active_item), Some(0));
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
        choice_index,
    });
    assert!(choose.action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == &chosen_relic));
    assert_eq!(engine.run_state.act, 2);
    assert_eq!(engine.run_state.floor, 17);
    assert_eq!(engine.run_state.current_hp, 56);
    assert_eq!(engine.run_state.map_x, -1);
    assert_eq!(engine.run_state.map_y, -1);
    assert!(!engine.run_state.run_won);
    assert!(!engine.run_state.run_over);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
    assert!(!engine.get_legal_actions().is_empty());
}

#[test]
fn act_one_and_two_boss_chests_reset_floor_rngs_before_astrolabe_effects() {
    // ProceedButton.goToTreasureRoom invokes nextRoomTransition before
    // TreasureRoomBoss.onPlayerEntry constructs and exposes its BossChest.
    // Astrolabe.giveCards then transforms with that new floor's miscRng.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java:179-186
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1731-1741
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Astrolabe.java:59-72
    for (act, boss_floor, boss_id) in [(1, 16, "SlimeBoss"), (2, 33, "TheChamp")] {
        let seed = (0..512)
            .find(|&seed| {
                let mut probe = RunEngine::new(seed, 0);
                probe.run_state.act = act;
                probe.debug_build_boss_reward_screen();
                probe.current_reward_screen().is_some_and(|screen| {
                    screen.items[0].choices.iter().any(|choice| {
                        matches!(choice, RewardChoice::Named { label, .. } if label == "Astrolabe")
                    })
                })
            })
            .expect("Astrolabe must be reachable from each boss pool");

        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.act = act;
        engine.run_state.floor = boss_floor;
        engine.debug_enter_specific_combat(&[boss_id]);
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();

        let chest_floor = boss_floor + 1;
        assert_eq!(engine.run_state.floor, chest_floor);
        assert_eq!(engine.current_phase(), crate::run::RunPhase::CardReward);
        assert_floor_rngs(&engine, seed, chest_floor, [0, 0, 0, 0, 0]);

        let astrolabe_index = engine
            .current_reward_screen()
            .expect("boss relic screen")
            .items[0]
            .choices
            .iter()
            .position(|choice| {
                matches!(choice, RewardChoice::Named { label, .. } if label == "Astrolabe")
            })
            .expect("seeded boss chest should expose Astrolabe");
        assert!(
            engine
                .step_with_result(&RunAction::SelectRewardItem(0))
                .action_accepted
        );
        assert!(
            engine
                .step_with_result(&RunAction::ChooseRewardOption {
                    item_index: 0,
                    choice_index: astrolabe_index,
                })
                .action_accepted
        );
        assert_floor_rngs(&engine, seed, chest_floor, [0, 0, 0, 0, 0]);

        for _ in 0..3 {
            assert!(
                engine
                    .step_with_result(&RunAction::SelectRewardItem(0))
                    .action_accepted
            );
            assert!(
                engine
                    .step_with_result(&RunAction::ChooseRewardOption {
                        item_index: 0,
                        choice_index: 0,
                    })
                    .action_accepted
            );
        }

        assert_eq!(engine.run_state.act, act + 1);
        assert_eq!(engine.run_state.floor, chest_floor);
        assert_floor_rngs(&engine, seed, chest_floor, [0, 0, 0, 0, 4]);
    }
}

#[test]
fn act_two_hallway_combat_after_floor_sixteen_is_not_a_boss() {
    let mut engine = RunEngine::new(43, 20);
    engine.run_state.act = 2;
    engine.run_state.floor = 18;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 0;
    engine.debug_enter_specific_combat(&["JawWorm"]);
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    let screen = engine
        .current_reward_screen()
        .expect("ordinary Act 2 hallway should open combat rewards");
    assert_eq!(screen.source, RewardScreenSource::Combat);
    assert!(!engine.run_state.run_over);
}

#[test]
fn second_boss_relic_transitions_to_act_three_with_a_fresh_map() {
    let seed = 45;
    let mut engine = RunEngine::new(seed, 0);
    engine.run_state.act = 2;
    engine.run_state.floor = 33;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    engine.run_state.current_hp = 1;
    engine.debug_build_boss_reward_screen();

    engine.step(&RunAction::SelectRewardItem(0));
    let screen = engine
        .current_reward_screen()
        .expect("boss relic choices should remain active");
    let choice_index = screen.items[0]
        .choices
        .iter()
        .position(|choice| {
            matches!(choice, RewardChoice::Named { label, .. }
                if label != "Astrolabe" && label != "Calling Bell")
        })
        .expect("boss screen should include a direct relic choice");
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index,
    });

    assert_eq!(engine.run_state.act, 3);
    assert_eq!(engine.run_state.floor, 34);
    assert_eq!(engine.run_state.current_hp, engine.run_state.max_hp);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
    let expected = crate::map::generate_map(seed + 600, 0);
    for y in 0..expected.height {
        for x in 0..expected.width {
            assert_eq!(engine.map.rows[y][x].room_type, expected.rows[y][x].room_type);
            assert_eq!(engine.map.rows[y][x].edges, expected.rows[y][x].edges);
        }
    }
}

#[test]
fn act_three_boss_routes_to_spire_heart_without_a_boss_relic() {
    let mut engine = RunEngine::new(44, 0);
    engine.run_state.act = 3;
    engine.run_state.floor = 50;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    let gold_before_boss = engine.run_state.gold;
    engine.debug_enter_specific_combat(&["TimeEater"]);
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    assert_eq!(engine.run_state.floor, 51);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Event);
    assert_eq!(
        engine.debug_current_event().as_ref().map(|event| event.name.as_str()),
        Some("Spire Heart")
    );
    assert!(engine.current_reward_screen().is_none());
    assert_eq!(engine.run_state.gold, gold_before_boss);
    assert!(!engine.run_state.run_over);
}

#[test]
fn ascension_twenty_runs_the_second_act_three_boss_before_spire_heart() {
    // TheBeyond.initializeBoss shuffles all three bosses once.
    // MonsterRoomBoss removes the current boss from bossList, and
    // ProceedButton.goToDoubleBoss selects the next entry at A20 without
    // opening or claiming the first boss's reward screen.
    let mut engine = RunEngine::new(46, 20);
    engine.run_state.act = 2;
    engine.run_state.floor = 33;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    engine.debug_build_boss_reward_screen();

    engine.step(&RunAction::SelectRewardItem(0));
    let choice_index = engine
        .current_reward_screen()
        .expect("Act 2 boss relic choices should remain active")
        .items[0]
        .choices
        .iter()
        .position(|choice| {
            matches!(choice, RewardChoice::Named { label, .. }
                if label != "Astrolabe" && label != "Calling Bell")
        })
        .expect("boss screen should include a direct relic choice");
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index,
    });

    assert_eq!(engine.run_state.act, 3);
    let first_boss = engine.boss_name().to_string();
    let gold_before_bosses = engine.run_state.gold;
    engine.run_state.floor = 50;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    engine.debug_enter_specific_combat(&[first_boss.as_str()]);
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    let second_boss = engine.boss_name().to_string();
    assert_ne!(second_boss, first_boss);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Combat);
    assert_eq!(engine.run_state.floor, 51);
    assert_eq!(engine.run_state.gold, gold_before_bosses);
    assert_eq!(engine.run_state.bosses_killed, 1);
    assert!(engine.current_reward_screen().is_none());

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    assert_eq!(engine.run_state.floor, 52);
    assert_eq!(engine.run_state.bosses_killed, 2);
    assert_eq!(engine.run_state.gold, gold_before_bosses);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Event);
    assert_eq!(
        engine.debug_current_event().as_ref().map(|event| event.name.as_str()),
        Some("Spire Heart")
    );
    assert!(engine.current_reward_screen().is_none());
    assert!(!engine.run_state.run_over);
}

#[test]
fn event_reward_items_flow_through_ordered_reward_screen() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_event_state(EventDef {
        name: "Golden Shrine".to_string(),
        options: vec![
            EventOption {
                text: "Take relic".to_string(),
                effect: EventEffect::GainRelic,
            },
            EventOption {
                text: "Leave".to_string(),
                effect: EventEffect::Nothing,
            },
        ],
    });

    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::CardReward);
    let screen = engine
        .current_reward_screen()
        .expect("event reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);

    let relic_id = screen.items[0].label.clone();
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
    assert!(engine.run_state.relics.iter().any(|relic| relic == &relic_id));
}

#[test]
fn deck_selection_purge_reward_removes_the_selected_card() {
    let mut engine = RunEngine::new(42, 20);
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

    let open = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);

    let choose = engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 1,
    });
    assert!(choose.action_accepted);
    assert!(!engine.run_state.deck.iter().any(|card| card == "Wallop"));
    assert_eq!(engine.run_state.deck.len(), 2);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
}
