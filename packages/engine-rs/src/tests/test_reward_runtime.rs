use crate::decision::{
    DecisionAction, RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen,
    RewardScreenSource,
};
use crate::events::{EventDef, EventEffect, EventOption};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine};

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
fn boss_reward_screen_requires_relic_choice_and_ends_run_after_resolution() {
    let mut engine = RunEngine::new(42, 20);
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
    assert!(engine.run_state.run_won);
    assert!(engine.run_state.run_over);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::GameOver);
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
