use crate::decision::{
    DecisionAction, RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardScreen,
    RewardScreenSource,
};
use crate::map::RoomType;
use crate::run::{RunAction, RunEngine, RunPhase, ShopState};

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

fn single_gold_reward_screen(amount: i32) -> RewardScreen {
    RewardScreen {
        source: RewardScreenSource::Event,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::Gold,
            state: RewardItemState::Available,
            label: amount.to_string(),
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
fn holy_water_is_offered_only_with_pure_water_and_replaces_it_when_chosen() {
    // Sources: HolyWater.java (`canSpawn` requires PureWater) and
    // BossRelicSelectScreen.java (instant-obtains HolyWater into relic slot 0).
    let offered_with_starter = (0..64).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| {
                    matches!(choice, RewardChoice::Named { label, .. } if label == "HolyWater")
                })
        })
    });
    assert!(offered_with_starter);

    for seed in 0..16 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.relics.clear();
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_boss_reward_screen();
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .all(|choice| {
                    !matches!(choice, RewardChoice::Named { label, .. } if label == "HolyWater")
                })
        }));
    }

    let mut engine = RunEngine::new(42, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["HolyWater"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert_eq!(engine.run_state.relics, vec!["HolyWater".to_string()]);
}

#[test]
fn violet_lotus_is_reachable_from_the_watcher_boss_relic_pool() {
    // Source: VioletLotus.java constructs the relic at BOSS tier. RunEngine is
    // currently Watcher-only, so its boss pool must be able to offer this ID.
    let offered = (0..64).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| {
                    matches!(choice, RewardChoice::Named { label, .. } if label == "VioletLotus")
                })
        })
    });
    assert!(offered);
}

#[test]
fn black_star_is_reachable_from_the_watcher_boss_relic_pool() {
    // Sources: RelicLibrary.java registers BlackStar and BlackStar.java
    // constructs it at BOSS tier with canonical ID "Black Star".
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| matches!(choice, RewardChoice::Named { label, .. } if label == "Black Star"))
        })
    });
    assert!(offered);
}

#[test]
fn busted_crown_is_reachable_and_subtracts_two_card_reward_choices() {
    // Source-derived (verify relic/Busted Crown): BustedCrown.java is BOSS
    // tier, adds one energy on equip, and changeNumberOfCardsInReward returns
    // numberOfCards - 2. Question Card remains an additive +1 callback.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| matches!(choice, RewardChoice::Named { label, .. } if label == "Busted Crown"))
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Busted Crown"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert!(engine
        .run_state
        .relic_flags
        .has(crate::relic_flags::flag::BUSTED_CROWN));

    engine.debug_build_combat_reward_screen(RoomType::Monster);
    let choices = engine
        .current_reward_screen()
        .and_then(|screen| {
            screen
                .items
                .iter()
                .find(|item| item.kind == RewardItemKind::CardChoice)
                .map(|item| item.choices.len())
        })
        .expect("card reward should exist");
    assert_eq!(choices, 1);

    engine.run_state.relics.push("QuestionCard".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);
    let choices = engine
        .current_reward_screen()
        .and_then(|screen| {
            screen
                .items
                .iter()
                .find(|item| item.kind == RewardItemKind::CardChoice)
                .map(|item| item.choices.len())
        })
        .expect("card reward should exist");
    assert_eq!(choices, 2);
}

#[test]
fn coffee_dripper_is_reachable_and_disables_only_campfire_rest() {
    // CoffeeDripper.java constructs a BOSS relic, increments energyMaster on
    // equip, and rejects the exact RestOption class in canUseCampfireOption.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].choices.iter().any(|choice| {
                matches!(choice, RewardChoice::Named { label, .. } if label == "Coffee Dripper")
            })
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Coffee Dripper"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert!(engine
        .run_state
        .relic_flags
        .has(crate::relic_flags::flag::COFFEE_DRIPPER));

    engine.debug_set_campfire_phase();
    assert!(!engine.get_legal_actions().contains(&RunAction::CampfireRest));
    assert!(!engine
        .current_decision_context()
        .campfire
        .expect("campfire context")
        .can_rest);
    assert!(engine
        .get_legal_actions()
        .iter()
        .any(|action| matches!(action, RunAction::CampfireUpgrade(_))));
}

#[test]
fn cursed_key_is_reachable_and_nonboss_chests_obtain_one_random_curse() {
    // CursedKey.java constructs a BOSS relic, increments energyMaster, and
    // onChestOpen obtains one random curse only when bossChest is false.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].choices.iter().any(|choice| {
                matches!(choice, RewardChoice::Named { label, .. } if label == "Cursed Key")
            })
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Cursed Key"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);

    let deck_before = engine.run_state.deck.len();
    engine.debug_build_boss_reward_screen();
    assert_eq!(engine.run_state.deck.len(), deck_before);
    engine.debug_build_treasure_reward_screen();
    assert_eq!(engine.run_state.deck.len(), deck_before + 1);
    assert!(matches!(
        engine.run_state.deck.last().map(String::as_str),
        Some(
            "Clumsy"
                | "Decay"
                | "Doubt"
                | "Injury"
                | "Normality"
                | "Pain"
                | "Parasite"
                | "Regret"
                | "Shame"
                | "Writhe"
        )
    ));

    // ShowCardAndObtainEffect.java consumes Omamori before adding the curse or
    // dispatching any onObtainCard hooks.
    let mut protected = RunEngine::new(43, 0);
    protected.run_state.relics.extend([
        "Cursed Key".to_string(),
        "Omamori".to_string(),
        "CeramicFish".to_string(),
    ]);
    protected
        .run_state
        .relic_flags
        .rebuild(&protected.run_state.relics);
    protected.run_state.relic_flags.counters
        [crate::relic_flags::counter::OMAMORI_USES] = 1;
    let deck_before = protected.run_state.deck.len();
    let gold_before = protected.run_state.gold;
    protected.debug_build_treasure_reward_screen();
    assert_eq!(protected.run_state.deck.len(), deck_before);
    assert_eq!(protected.run_state.gold, gold_before);
    assert_eq!(
        protected.run_state.relic_flags.counters
            [crate::relic_flags::counter::OMAMORI_USES],
        0
    );
}

#[test]
fn darkstone_periapt_is_reachable_and_an_obtained_curse_grants_six_max_hp() {
    // DarkstonePeriapt.java constructs an UNCOMMON relic, excludes floors
    // after 48, and calls increaseMaxHp(6, true) for an obtained CURSE card.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Darkstone Periapt"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Darkstone Periapt")
        }));
    }

    let mut engine = RunEngine::new(47, 0);
    engine.run_state.relics.push("Darkstone Periapt".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.current_hp = 40;
    engine.debug_set_card_reward_screen(vec!["Regret".to_string()]);
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert_eq!(engine.run_state.max_hp, 78);
    assert_eq!(engine.run_state.current_hp, 46);
    assert_eq!(engine.run_state.deck.last().map(String::as_str), Some("Regret"));

    let mut protected = RunEngine::new(49, 0);
    protected.run_state.relics.extend([
        "Darkstone Periapt".to_string(),
        "Omamori".to_string(),
    ]);
    protected
        .run_state
        .relic_flags
        .rebuild(&protected.run_state.relics);
    protected.run_state.relic_flags.counters
        [crate::relic_flags::counter::OMAMORI_USES] = 1;
    protected.debug_set_card_reward_screen(vec!["Regret".to_string()]);
    assert!(protected
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(protected
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert_eq!(protected.run_state.max_hp, 72);
    assert!(!protected.run_state.deck.iter().any(|card| card == "Regret"));
}

#[test]
fn dream_catcher_is_reachable_and_opens_a_card_reward_only_after_resting() {
    // DreamCatcher.java is COMMON with a floor-48 cutoff;
    // CampfireSleepEffect.java opens getRewardCards only after Rest resolves.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Dream Catcher"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Dream Catcher")
        }));
    }

    let mut rest = RunEngine::new(53, 0);
    rest.run_state.relics.push("Dream Catcher".to_string());
    rest.run_state.relic_flags.rebuild(&rest.run_state.relics);
    rest.debug_set_campfire_phase();
    assert!(rest
        .step_with_result(&RunAction::CampfireRest)
        .action_accepted);
    assert_eq!(rest.current_phase(), RunPhase::CardReward);
    let screen = rest.current_reward_screen().expect("Dream Catcher reward");
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].choices.len(), 3);

    let mut upgrade = RunEngine::new(59, 0);
    upgrade.run_state.relics.push("Dream Catcher".to_string());
    upgrade
        .run_state
        .relic_flags
        .rebuild(&upgrade.run_state.relics);
    upgrade.debug_set_campfire_phase();
    assert!(upgrade
        .step_with_result(&RunAction::CampfireUpgrade(0))
        .action_accepted);
    assert_eq!(upgrade.current_phase(), RunPhase::MapChoice);
    assert!(upgrade.current_reward_screen().is_none());
}

#[test]
fn ectoplasm_is_act_one_only_blocks_gold_gains_and_still_allows_spending() {
    // Ectoplasm.java constructs a BOSS relic, increments energyMaster, and can
    // spawn only in Act 1. AbstractPlayer.java::gainGold returns immediately
    // with Ectoplasm, while loseGold remains unchanged.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.act = 1;
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].choices.iter().any(|choice| {
                matches!(choice, RewardChoice::Named { label, .. } if label == "Ectoplasm")
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.act = 2;
        engine.debug_build_boss_reward_screen();
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].choices.iter().all(|choice| {
                !matches!(choice, RewardChoice::Named { label, .. } if label == "Ectoplasm")
            })
        }));
    }

    let mut engine = RunEngine::new(61, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Ectoplasm"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);

    engine.run_state.gold = 100;
    engine.debug_set_reward_screen(single_gold_reward_screen(75));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert_eq!(engine.run_state.gold, 100);

    engine.debug_set_shop_state(ShopState {
        cards: vec![("Strike".to_string(), 40)],
        relics: Vec::new(),
        remove_price: 75,
        removal_used: false,
    });
    assert!(engine
        .step_with_result(&RunAction::ShopBuyCard(0))
        .action_accepted);
    assert_eq!(engine.run_state.gold, 60);
}

#[test]
fn du_vu_doll_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers DuVuDoll; DuVuDoll.java constructs the
    // shared relic at RARE tier under canonical ID "Du-Vu Doll".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Du-Vu Doll"
            })
        })
    });
    assert!(offered);
}

#[test]
fn eternal_feather_is_reachable_and_heals_on_rest_room_entry_before_choice() {
    // EternalFeather.java constructs an UNCOMMON relic and onEnterRoom heals
    // floor(masterDeck.size / 5) * 3 when the room is a RestRoom.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Eternal Feather"
            })
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(67, 0);
    engine.run_state.relics.push("Eternal Feather".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.deck = vec!["Strike".to_string(); 14];
    engine.run_state.current_hp = 40;

    engine.debug_set_campfire_phase();
    assert_eq!(engine.run_state.current_hp, 46);
    assert!(engine
        .step_with_result(&RunAction::CampfireUpgrade(0))
        .action_accepted);
    assert_eq!(engine.run_state.current_hp, 46);
}

#[test]
fn fusion_hammer_is_reachable_and_disables_only_campfire_upgrades() {
    // FusionHammer.java constructs a BOSS relic, increments energyMaster, and
    // rejects the exact SmithOption class while allowing other campfire options.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].choices.iter().any(|choice| {
                matches!(choice, RewardChoice::Named { label, .. } if label == "Fusion Hammer")
            })
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(71, 0);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Fusion Hammer"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);

    engine.debug_set_campfire_phase();
    assert!(engine.get_legal_actions().contains(&RunAction::CampfireRest));
    assert!(!engine
        .get_legal_actions()
        .iter()
        .any(|action| matches!(action, RunAction::CampfireUpgrade(_))));
    let context = engine
        .current_decision_context()
        .campfire
        .expect("campfire context");
    assert!(context.can_rest);
    assert!(context.upgradable_cards.is_empty());
}

#[test]
fn ginger_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers Ginger; Ginger.java constructs the shared
    // relic at RARE tier under canonical ID "Ginger".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Ginger"
            })
        })
    });
    assert!(offered);
}

#[test]
fn fossilized_helix_is_reachable_under_its_canonical_java_id() {
    // RelicLibrary.java registers FossilizedHelix; FossilizedHelix.java
    // constructs the shared RARE relic under canonical ID "FossilizedHelix".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "FossilizedHelix"
            })
        })
    });
    assert!(offered);
}

#[test]
fn egg_relics_use_canonical_ids_floor_cutoffs_and_upgrade_all_obtain_paths() {
    // FrozenEgg2.java, MoltenEgg2.java, and ToxicEgg2.java are UNCOMMON, stop
    // spawning after floor 48, and upgrade Power/Attack/Skill cards in both
    // onPreviewObtainCard and onObtainCard.
    for relic_id in ["Frozen Egg 2", "Molten Egg 2", "Toxic Egg 2"] {
        let offered = (0..1024).any(|seed| {
            let mut engine = RunEngine::new(seed, 0);
            engine.run_state.floor = 48;
            engine.debug_build_combat_reward_screen(RoomType::Elite);
            engine.current_reward_screen().is_some_and(|screen| {
                screen.items.iter().any(|item| {
                    item.kind == RewardItemKind::Relic && item.label == relic_id
                })
            })
        });
        assert!(offered, "{relic_id} should be reachable through floor 48");

        for seed in 0..128 {
            let mut engine = RunEngine::new(seed, 0);
            engine.run_state.floor = 49;
            engine.debug_build_combat_reward_screen(RoomType::Elite);
            assert!(engine.current_reward_screen().is_some_and(|screen| {
                screen.items.iter().all(|item| item.label != relic_id)
            }));
        }
    }

    for (relic_id, card_id, upgraded_id) in [
        ("Frozen Egg 2", "Devotion", "Devotion+"),
        ("Molten Egg 2", "Wallop", "Wallop+"),
        ("Toxic Egg 2", "ThirdEye", "ThirdEye+"),
    ] {
        let mut engine = RunEngine::new(73, 0);
        engine.run_state.relics.push(relic_id.to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.run_state.gold = 100;
        engine.debug_set_shop_state(ShopState {
            cards: vec![(card_id.to_string(), 40)],
            relics: Vec::new(),
            remove_price: 75,
            removal_used: false,
        });
        assert!(engine
            .step_with_result(&RunAction::ShopBuyCard(0))
            .action_accepted);
        assert_eq!(
            engine.run_state.deck.last().map(String::as_str),
            Some(upgraded_id)
        );
    }

    let mut preview = RunEngine::new(79, 0);
    preview.run_state.relics.extend([
        "Frozen Egg 2".to_string(),
        "Molten Egg 2".to_string(),
        "Toxic Egg 2".to_string(),
    ]);
    preview
        .run_state
        .relic_flags
        .rebuild(&preview.run_state.relics);
    preview.debug_enter_shop();
    assert!(preview
        .get_shop()
        .expect("shop")
        .cards
        .iter()
        .all(|(card_id, _)| card_id.ends_with('+')));
}

#[test]
fn ice_cream_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers IceCream; IceCream.java constructs the
    // shared relic at RARE tier under canonical ID "Ice Cream".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Ice Cream"
            })
        })
    });
    assert!(offered);
}

#[test]
fn incense_burner_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers IncenseBurner; IncenseBurner.java constructs
    // the shared relic at RARE tier under canonical ID "Incense Burner".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Incense Burner"
            })
        })
    });
    assert!(offered);
}

#[test]
fn ink_bottle_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers InkBottle; InkBottle.java constructs the
    // shared relic at UNCOMMON tier under canonical ID "InkBottle".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "InkBottle"
            })
        })
    });
    assert!(offered);
}

#[test]
fn kunai_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers Kunai; Kunai.java constructs the shared
    // relic at UNCOMMON tier under canonical ID "Kunai".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Kunai"
            })
        })
    });
    assert!(offered);
}

#[test]
fn lantern_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers Lantern; Lantern.java constructs the shared
    // relic at COMMON tier under canonical ID "Lantern".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Lantern"
            })
        })
    });
    assert!(offered);
}

#[test]
fn letter_opener_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers LetterOpener; LetterOpener.java constructs
    // the shared relic at UNCOMMON tier under canonical ID "Letter Opener".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Letter Opener"
            })
        })
    });
    assert!(offered);
}

#[test]
fn lizard_tail_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers LizardTail; LizardTail.java constructs the
    // shared relic at RARE tier under canonical ID "Lizard Tail".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Lizard Tail"
            })
        })
    });
    assert!(offered);
}

#[test]
fn magic_flower_is_reachable_from_watcher_relic_rewards() {
    // RelicLibrary.java registers MagicFlower; MagicFlower.java constructs the
    // shared relic at RARE tier under canonical ID "Magic Flower".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Magic Flower"
            })
        })
    });
    assert!(offered);
}

#[test]
fn mango_is_reachable_and_increases_max_hp_by_fourteen_on_pickup() {
    // Mango.java constructs a RARE relic and onEquip calls
    // increaseMaxHp(14, true). Mark of the Bloom blocks the heal, not max HP.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Mango"
            })
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(41, 0);
    engine.run_state.current_hp = 40;
    engine.run_state.max_hp = 80;
    engine.debug_set_reward_screen(single_relic_reward_screen("Mango"));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert_eq!(engine.run_state.max_hp, 94);
    assert_eq!(engine.run_state.current_hp, 54);
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Mango"));

    let mut blocked = RunEngine::new(43, 0);
    blocked.run_state.current_hp = 40;
    blocked.run_state.max_hp = 80;
    blocked.run_state.relics.push("Mark of the Bloom".to_string());
    blocked
        .run_state
        .relic_flags
        .rebuild(&blocked.run_state.relics);
    blocked.debug_set_reward_screen(single_relic_reward_screen("Mango"));
    assert!(blocked
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert_eq!(blocked.run_state.max_hp, 94);
    assert_eq!(blocked.run_state.current_hp, 40);
}

#[test]
fn matryoshka_is_reachable_only_through_floor_forty() {
    // Matryoshka.java constructs an UNCOMMON relic and canSpawn allows
    // non-endless runs only while floorNum <= 40.
    let offered = (0..2048).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 40;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Matryoshka"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 41;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Matryoshka")
        }));
    }
}

#[test]
fn maw_bank_is_reachable_only_through_floor_forty_eight() {
    // MawBank.java constructs a COMMON relic and canSpawn excludes
    // non-endless runs after floor 48 (and rewards generated in a shop room).
    let offered = (0..2048).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "MawBank"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "MawBank")
        }));
    }
}

#[test]
fn meal_ticket_is_reachable_only_through_floor_forty_eight() {
    // MealTicket.java constructs a COMMON relic and canSpawn excludes
    // non-endless runs after floor 48.
    let offered = (0..2048).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "MealTicket"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "MealTicket")
        }));
    }
}

#[test]
fn meat_on_the_bone_is_reachable_only_through_floor_forty_eight() {
    // MeatOnTheBone.java constructs an UNCOMMON relic and canSpawn excludes
    // non-endless runs after floor 48.
    let offered = (0..2048).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Meat on the Bone"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Meat on the Bone")
        }));
    }
}

#[test]
fn happy_flower_is_reachable_from_common_watcher_relic_rewards() {
    // HappyFlower.java constructs a COMMON relic under canonical ID
    // "Happy Flower"; it must not be sourced from an UNCOMMON-only event roll.
    let offered = (0..2048).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Happy Flower"
            })
        })
    });
    assert!(offered);
}

#[test]
fn medical_kit_is_reachable_only_from_the_shop_relic_slot() {
    // MedicalKit.java constructs RelicTier.SHOP. ShopScreen.java::initRelics
    // makes its third relic slot SHOP-tier; ordinary combat rewards cannot offer it.
    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Medical Kit")
        }));
    }

    let offered_seed = (0..512).find(|seed| {
        let mut engine = RunEngine::new(*seed, 0);
        engine.debug_enter_shop();
        engine.get_shop().is_some_and(|shop| {
            shop.relics.iter().any(|(relic, _)| relic == "Medical Kit")
        })
    }).expect("Medical Kit should be reachable from the SHOP-tier slot");

    let mut engine = RunEngine::new(offered_seed, 0);
    engine.run_state.gold = 999;
    engine.debug_enter_shop();
    let idx = engine
        .get_shop()
        .expect("shop")
        .relics
        .iter()
        .position(|(relic, _)| relic == "Medical Kit")
        .expect("Medical Kit offer");
    assert!(engine
        .step_with_result(&RunAction::ShopBuyRelic(idx))
        .action_accepted);
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Medical Kit"));
}

#[test]
fn melange_is_reachable_only_from_the_shop_relic_slot() {
    // Melange.java constructs RelicTier.SHOP. ShopScreen.java::initRelics
    // reserves its third relic offer for that tier.
    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Melange")
        }));
    }

    let offered_seed = (0..512).find(|seed| {
        let mut engine = RunEngine::new(*seed, 0);
        engine.debug_enter_shop();
        engine.get_shop().is_some_and(|shop| {
            shop.relics.iter().any(|(relic, _)| relic == "Melange")
        })
    }).expect("Melange should be reachable from the SHOP-tier slot");

    let mut engine = RunEngine::new(offered_seed, 0);
    engine.run_state.gold = 999;
    engine.debug_enter_shop();
    let idx = engine
        .get_shop()
        .expect("shop")
        .relics
        .iter()
        .position(|(relic, _)| relic == "Melange")
        .expect("Melange offer");
    assert!(engine
        .step_with_result(&RunAction::ShopBuyRelic(idx))
        .action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Melange"));
}

#[test]
fn calling_bell_grants_mandatory_curse_then_one_relic_of_each_tier() {
    // Source-derived (verify relic/Calling Bell): CallingBell.java is BOSS tier,
    // confirms CurseOfTheBell, then opens COMMON, UNCOMMON, and RARE relic
    // rewards in that order.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| matches!(choice, RewardChoice::Named { label, .. } if label == "Calling Bell"))
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(77, 0);
    engine.run_state.deck.push("Wallop".to_string());
    engine.run_state.deck.push("ThirdEye".to_string());
    engine.run_state.deck.push("Devotion".to_string());
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Calling Bell"]));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);

    let screen = engine
        .current_reward_screen()
        .expect("Calling Bell rewards should replace boss choices");
    assert_eq!(screen.items.len(), 4);
    assert!(matches!(
        &screen.items[0].choices[0],
        RewardChoice::Card { card_id, .. } if card_id == "CurseOfTheBell"
    ));
    assert!(screen.items[1..]
        .iter()
        .all(|item| item.kind == RewardItemKind::Relic));

    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert!(engine
        .run_state
        .deck
        .iter()
        .any(|card| card == "CurseOfTheBell"));

    for item_index in 1..=3 {
        assert!(engine
            .step_with_result(&RunAction::SelectRewardItem(item_index))
            .action_accepted);
        if engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0].label.starts_with("deck_selection_bottled_")
        }) {
            assert!(engine
                .step_with_result(&RunAction::SelectRewardItem(0))
                .action_accepted);
            assert!(engine
                .step_with_result(&RunAction::ChooseRewardOption {
                    item_index: 0,
                    choice_index: 0,
                })
                .action_accepted);
        }
    }
    assert_eq!(engine.run_state.relics.len(), 5);
    assert!(engine.run_state.run_over);
}

#[test]
fn astrolabe_is_reachable_and_transforms_three_selected_cards_upgraded() {
    // Source-derived (verify relic/Astrolabe): Astrolabe.java is BOSS tier,
    // selects exactly three purgeable cards when more than three are eligible,
    // removes them, and transforms each with autoUpgrade=true.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_boss_reward_screen();
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items[0]
                .choices
                .iter()
                .any(|choice| matches!(choice, RewardChoice::Named { label, .. } if label == "Astrolabe"))
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    let original_len = engine.run_state.deck.len();
    engine.debug_set_reward_screen(relic_choice_reward_screen(&["Astrolabe"]));
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);

    for _ in 0..3 {
        assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
        assert!(engine
            .step_with_result(&RunAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            })
            .action_accepted);
    }

    assert_eq!(engine.run_state.deck.len(), original_len);
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.ends_with('+'))
            .count(),
        3
    );
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Astrolabe"));

    // Java skips the grid when at most three purgeable cards exist and gives
    // those transforms immediately; unpurgeable curses are not candidates.
    let mut automatic = RunEngine::new(7, 0);
    automatic.run_state.deck = vec![
        "Necronomicurse".to_string(),
        "Strike".to_string(),
        "Defend".to_string(),
        "Eruption".to_string(),
    ];
    automatic.debug_set_reward_screen(relic_choice_reward_screen(&["Astrolabe"]));
    assert!(automatic
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(automatic
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert_eq!(automatic.run_state.deck.len(), 4);
    assert!(automatic
        .run_state
        .deck
        .iter()
        .any(|card| card == "Necronomicurse"));
    assert_eq!(
        automatic
            .run_state
            .deck
            .iter()
            .filter(|card| card.ends_with('+'))
            .count(),
        3
    );
}

#[test]
fn akabeko_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers Akabeko and Akabeko.java constructs
    // it at COMMON tier; AbstractDungeon.java::populateRelicPool places common
    // relics into the run's common relic pool for the chosen character.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Akabeko")
        })
    });
    assert!(offered);
}

#[test]
fn anchor_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers Anchor and Anchor.java constructs it
    // at COMMON tier; AbstractDungeon.java::populateRelicPool places common
    // relics into the chosen character's common relic pool.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Anchor")
        })
    });
    assert!(offered);
}

#[test]
fn ancient_tea_set_reward_obeys_its_java_floor_cutoff() {
    // Sources: AncientTeaSet.java constructs a COMMON relic and canSpawn
    // returns true only through floor 48 outside Endless mode.
    let offered_before_cutoff = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Ancient Tea Set"
            })
        })
    });
    assert!(offered_before_cutoff);

    for seed in 0..1024 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| {
                item.kind != RewardItemKind::Relic || item.label != "Ancient Tea Set"
            })
        }));
    }
}

#[test]
fn art_of_war_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers Art of War and ArtOfWar.java
    // constructs it at COMMON tier, so Watcher common relic rewards can offer
    // this shared relic.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Art of War")
        })
    });
    assert!(offered);
}

#[test]
fn bag_of_marbles_is_reachable_under_its_canonical_java_id() {
    // Sources: RelicLibrary.java registers BagOfMarbles and BagOfMarbles.java
    // constructs the COMMON relic with ID "Bag of Marbles".
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Bag of Marbles"
            })
        })
    });
    assert!(offered);
}

#[test]
fn bag_of_preparation_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers BagOfPreparation and its constructor
    // assigns the shared relic to the COMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Bag of Preparation"
            })
        })
    });
    assert!(offered);
}

#[test]
fn bird_faced_urn_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers BirdFacedUrn and its constructor
    // assigns the shared relic to the RARE tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Bird Faced Urn"
            })
        })
    });
    assert!(offered);
}

#[test]
fn blood_vial_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers BloodVial and BloodVial.java
    // constructs the shared relic at COMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Blood Vial")
        })
    });
    assert!(offered);
}

#[test]
fn blue_candle_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers BlueCandle and BlueCandle.java
    // constructs the shared relic at UNCOMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Blue Candle")
        })
    });
    assert!(offered);
}

#[test]
fn boot_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers Boot and Boot.java constructs the
    // shared relic at COMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Boot")
        })
    });
    assert!(offered);
}

#[test]
fn bottled_flame_requires_a_nonbasic_attack_then_selects_any_purgeable_attack() {
    // Source-derived (verify relic/Bottled Flame): BottledFlame.java::canSpawn
    // requires at least one non-Basic Attack, while onEquip offers all
    // purgeable Attacks and marks the single selected card inBottleFlame.
    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Bottled Flame")
        }));
    }

    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.deck.push("Wallop".to_string());
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| item.label == "Bottled Flame")
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.run_state.deck.push("Wallop".to_string());
    engine.debug_set_reward_screen(single_relic_reward_screen("Bottled Flame"));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    let screen = engine
        .current_reward_screen()
        .expect("bottle selection should replace the relic reward");
    assert!(screen.items[0].choices.iter().all(|choice| {
        !matches!(choice, RewardChoice::Card { card_id, .. } if card_id == "Defend")
    }));
    let wallop_choice = screen.items[0]
        .choices
        .iter()
        .position(|choice| matches!(choice, RewardChoice::Card { card_id, .. } if card_id == "Wallop"))
        .expect("Wallop should be bottle-eligible");
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: wallop_choice,
        })
        .action_accepted);
    assert_eq!(engine.run_state.bottled_flame_card.as_deref(), Some("Wallop"));
}

#[test]
fn bottled_lightning_requires_a_nonbasic_skill_then_selects_any_purgeable_skill() {
    // Source-derived (verify relic/Bottled Lightning): canSpawn requires a
    // non-Basic Skill, while onEquip offers every purgeable Skill.
    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Bottled Lightning")
        }));
    }

    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.deck.push("ThirdEye".to_string());
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| item.label == "Bottled Lightning")
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.run_state.deck.push("ThirdEye".to_string());
    engine.debug_set_reward_screen(single_relic_reward_screen("Bottled Lightning"));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    let screen = engine.current_reward_screen().expect("skill selection should open");
    assert!(screen.items[0].choices.iter().all(|choice| {
        !matches!(choice, RewardChoice::Card { card_id, .. } if card_id == "Strike")
    }));
    let choice_index = screen.items[0]
        .choices
        .iter()
        .position(|choice| matches!(choice, RewardChoice::Card { card_id, .. } if card_id == "ThirdEye"))
        .expect("Third Eye should be bottle-eligible");
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index,
        })
        .action_accepted);
    assert_eq!(
        engine.run_state.bottled_lightning_card.as_deref(),
        Some("ThirdEye")
    );
}

#[test]
fn bottled_tornado_requires_and_selects_a_purgeable_power() {
    // Source-derived (verify relic/Bottled Tornado): canSpawn requires any
    // Power, and onEquip offers the purgeable Powers for one selection.
    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "Bottled Tornado")
        }));
    }

    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.deck.push("Devotion".to_string());
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| item.label == "Bottled Tornado")
        })
    });
    assert!(offered);

    let mut engine = RunEngine::new(42, 0);
    engine.run_state.deck.push("Devotion".to_string());
    engine.debug_set_reward_screen(single_relic_reward_screen("Bottled Tornado"));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    let screen = engine.current_reward_screen().expect("power selection should open");
    assert_eq!(screen.items[0].choices.len(), 1);
    assert!(matches!(
        &screen.items[0].choices[0],
        RewardChoice::Card { card_id, .. } if card_id == "Devotion"
    ));
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .action_accepted);
    assert_eq!(
        engine.run_state.bottled_tornado_card.as_deref(),
        Some("Devotion")
    );
}

#[test]
fn bronze_scales_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers BronzeScales and BronzeScales.java
    // constructs the shared relic at COMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Bronze Scales")
        })
    });
    assert!(offered);
}

#[test]
fn calipers_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers Calipers and Calipers.java
    // constructs the shared relic at RARE tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Relic && item.label == "Calipers")
        })
    });
    assert!(offered);
}

#[test]
fn centennial_puzzle_is_reachable_from_watcher_relic_rewards() {
    // Sources: RelicLibrary.java registers CentennialPuzzle and
    // CentennialPuzzle.java constructs the shared relic at COMMON tier.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "Centennial Puzzle"
            })
        })
    });
    assert!(offered);
}

#[test]
fn ceramic_fish_is_reachable_before_floor_49_and_pays_for_shop_card_obtains() {
    // CeramicFish.java constructs a COMMON relic, excludes floors after 48,
    // and onObtainCard gains exactly 9 gold. ShopScreen.java purchases through
    // FastCardObtainEffect.java, which dispatches onObtainCard to every relic.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 48;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Relic && item.label == "CeramicFish"
            })
        })
    });
    assert!(offered);

    for seed in 0..128 {
        let mut engine = RunEngine::new(seed, 0);
        engine.run_state.floor = 49;
        engine.debug_build_combat_reward_screen(RoomType::Elite);
        assert!(engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().all(|item| item.label != "CeramicFish")
        }));
    }

    let mut engine = RunEngine::new(42, 0);
    engine.run_state.relics.push("CeramicFish".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.gold = 100;
    engine.debug_set_shop_state(ShopState {
        cards: vec![("Strike".to_string(), 50)],
        relics: Vec::new(),
        remove_price: 75,
        removal_used: false,
    });

    let step = engine.step_with_result(&RunAction::ShopBuyCard(0));
    assert!(step.action_accepted);
    assert_eq!(engine.run_state.deck.last().map(String::as_str), Some("Strike"));
    assert_eq!(engine.run_state.gold, 59);
}

#[test]
fn ambrosia_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions(WATCHER, false) includes Ambrosia. White Beast
    // Statue guarantees a potion item here so the run reward path is sampled.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen
                .items
                .iter()
                .any(|item| item.kind == RewardItemKind::Potion && item.label == "Ambrosia")
        })
    });
    assert!(offered);
}

#[test]
fn bottled_miracle_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions(WATCHER, false) includes BottledMiracle.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "BottledMiracle"
            })
        })
    });
    assert!(offered);
}

#[test]
fn stance_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions(WATCHER, false) includes StancePotion.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "StancePotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn ancient_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper's shared potion list includes the canonical spaced ID.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Ancient Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn blessing_of_the_forge_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper's shared potion list includes BlessingOfTheForge.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "BlessingOfTheForge"
            })
        })
    });
    assert!(offered);
}

#[test]
fn colorless_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper's shared potion list includes ColorlessPotion.
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "ColorlessPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn cultist_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper's shared potion list includes CultistPotion.
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "CultistPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn distilled_chaos_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends DistilledChaos after the class-specific
    // switch, so it belongs to the Watcher's shared potion reward pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "DistilledChaos"
            })
        })
    });
    assert!(offered);
}

#[test]
fn duplication_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends DuplicationPotion outside the
    // class-specific switch, so it is in the Watcher's shared reward pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "DuplicationPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn fruit_juice_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Fruit Juice to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "FruitJuice"
            })
        })
    });
    assert!(offered);
}

#[test]
fn gamblers_brew_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends GamblersBrew to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "GamblersBrew"
            })
        })
    });
    assert!(offered);
}

#[test]
fn liquid_bronze_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends LiquidBronze to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "LiquidBronze"
            })
        })
    });
    assert!(offered);
}

#[test]
fn liquid_memories_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends LiquidMemories to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "LiquidMemories"
            })
        })
    });
    assert!(offered);
}

#[test]
fn regen_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Regen Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Regen Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn smoke_bomb_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends SmokeBomb to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "SmokeBomb"
            })
        })
    });
    assert!(offered);
}

#[test]
fn snecko_oil_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends SneckoOil to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "SneckoOil"
            })
        })
    });
    assert!(offered);
}

#[test]
fn speed_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends SpeedPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "SpeedPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn steroid_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends SteroidPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "SteroidPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn strength_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Strength Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Strength Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn swift_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Swift Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Swift Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn weak_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Weak Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Weak Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn power_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends PowerPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "PowerPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn skill_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends SkillPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..1024).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "SkillPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn energy_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends the shared Energy Potion after the
    // class-specific switch, so Watcher combat rewards can offer it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Energy Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn entropic_brew_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends EntropicBrew to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "EntropicBrew"
            })
        })
    });
    assert!(offered);
}

#[test]
fn essence_of_steel_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends EssenceOfSteel to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "EssenceOfSteel"
            })
        })
    });
    assert!(offered);
}

#[test]
fn explosive_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Explosive Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Explosive Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn fairy_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends FairyPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "FairyPotion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn fear_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends FearPotion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Fear Potion"
            })
        })
    });
    assert!(offered);
}

#[test]
fn fire_potion_is_reachable_from_watcher_potion_rewards() {
    // PotionHelper.getPotions appends Fire Potion to the shared pool.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
    let offered = (0..128).any(|seed| {
        let mut engine = RunEngine::new(seed, 0);
        engine
            .run_state
            .relics
            .push("White Beast Statue".to_string());
        engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        engine.current_reward_screen().is_some_and(|screen| {
            screen.items.iter().any(|item| {
                item.kind == RewardItemKind::Potion && item.label == "Fire Potion"
            })
        })
    });
    assert!(offered);
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
    // Source-derived (verify relic/Black Star): BlackStar.java is active for
    // elite rooms; AbstractRoom's elite victory rewards therefore include the
    // relic's additional relic roll.
    let mut engine = RunEngine::new(99, 20);
    engine.debug_set_reward_screen(relic_choice_reward_screen(&[
        "Black Star",
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
