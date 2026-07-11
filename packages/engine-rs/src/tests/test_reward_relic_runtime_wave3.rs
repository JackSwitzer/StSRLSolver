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
