use crate::decision::{
    RewardChoice, RewardItem, RewardItemKind, RewardItemState, RewardKeyColor, RewardScreen,
    RewardScreenSource,
};
use crate::events::{EventDef, EventEffect, EventOption};
use crate::map::RoomType;
use crate::run::{GameAction, RunEngine};
use crate::tests::support::resolve_opening_neow;

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

fn seed_for_next_map_buff(ascension: i32, expected_buff: i32) -> u64 {
    (0..512)
        .find(|&seed| {
            let engine = RunEngine::new(seed, ascension);
            engine.debug_peek_map_rng_int3() == expected_buff
        })
        .expect("fixture search should cover every inclusive mapRng 0..3 result")
}

fn enter_elite(
    seed: u64,
    ascension: i32,
    act: i32,
    burning: bool,
    preserved_insect: bool,
) -> (RunEngine, i64) {
    let mut engine = RunEngine::new(seed, ascension);
    resolve_opening_neow(&mut engine);
    engine.run_state.act = act;
    if preserved_insect {
        engine.run_state.relics.push("PreservedInsect".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
    }
    let start = engine.map.get_start_nodes()[0];
    let (start_x, start_y) = (start.x, start.y);
    engine.map.rows[start_y][start_x].room_type = RoomType::Elite;
    engine.map.rows[start_y][start_x].has_emerald_key = burning;
    let map_counter = engine.rng_counters()["map"];
    assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
    (engine, map_counter)
}

fn reward_relic_tier(relic_id: &str) -> &'static str {
    // RelicLibrary.java::populateRelicPool is the source for these all-unlocked
    // Watcher tier memberships. The production pools use the same Java-derived
    // vectors, but this independent classifier keeps the boundary assertion
    // from merely checking that some relic was returned.
    const COMMON: &[&str] = &[
        "Whetstone",
        "Boot",
        "Blood Vial",
        "MealTicket",
        "Pen Nib",
        "Akabeko",
        "Lantern",
        "Regal Pillow",
        "Bag of Preparation",
        "Ancient Tea Set",
        "Smiling Mask",
        "Potion Belt",
        "PreservedInsect",
        "Omamori",
        "MawBank",
        "Art of War",
        "Toy Ornithopter",
        "CeramicFish",
        "Vajra",
        "Centennial Puzzle",
        "Strawberry",
        "Happy Flower",
        "Oddly Smooth Stone",
        "War Paint",
        "Bronze Scales",
        "Juzu Bracelet",
        "Dream Catcher",
        "Nunchaku",
        "Tiny Chest",
        "Orichalcum",
        "Anchor",
        "Bag of Marbles",
        "Damaru",
    ];
    const UNCOMMON: &[&str] = &[
        "Bottled Tornado",
        "Sundial",
        "Kunai",
        "Pear",
        "Blue Candle",
        "Eternal Feather",
        "StrikeDummy",
        "Singing Bowl",
        "Matryoshka",
        "InkBottle",
        "The Courier",
        "Frozen Egg 2",
        "Ornamental Fan",
        "Bottled Lightning",
        "Gremlin Horn",
        "HornCleat",
        "Toxic Egg 2",
        "Letter Opener",
        "Question Card",
        "Bottled Flame",
        "Shuriken",
        "Molten Egg 2",
        "Meat on the Bone",
        "Darkstone Periapt",
        "Mummified Hand",
        "Pantograph",
        "White Beast Statue",
        "Mercury Hourglass",
        "Yang",
        "TeardropLocket",
    ];
    const RARE: &[&str] = &[
        "Ginger",
        "Old Coin",
        "Bird Faced Urn",
        "Unceasing Top",
        "Torii",
        "StoneCalendar",
        "Shovel",
        "WingedGreaves",
        "Thread and Needle",
        "Turnip",
        "Ice Cream",
        "Calipers",
        "Lizard Tail",
        "Prayer Wheel",
        "Girya",
        "Dead Branch",
        "Du-Vu Doll",
        "Pocketwatch",
        "Mango",
        "Incense Burner",
        "Gambling Chip",
        "Peace Pipe",
        "CaptainsWheel",
        "FossilizedHelix",
        "TungstenRod",
        "CloakClasp",
        "GoldenEye",
    ];
    if COMMON.contains(&relic_id) {
        "common"
    } else if UNCOMMON.contains(&relic_id) {
        "uncommon"
    } else if RARE.contains(&relic_id) {
        "rare"
    } else {
        panic!("unclassified Watcher reward relic {relic_id}")
    }
}

fn seed_for_next_relic_roll(expected_roll: i32) -> u64 {
    (0..10_000)
        .find(|&seed| {
            // AbstractDungeon.initializeRelicList consumes one outer randomLong
            // for each of the common, uncommon, rare, shop, and boss shuffles.
            let mut relic_rng = crate::seed::StsRandom::with_counter(seed, 5);
            relic_rng.random_int(99) == expected_roll
        })
        .expect("fixture search should cover every inclusive relicRng 0..99 result")
}

#[test]
fn emerald_elite_buff_matrix_covers_all_rolls_acts_and_ascensions() {
    // AbstractDungeon.java marks one elite node; AbstractPlayer.java then
    // consumes one shared mapRng.random(0, 3) for MonsterRoomElite. The buff
    // formula depends on act but not ascension, and every enemy receives the
    // same selected buff. Java: AbstractDungeon.java::setEmeraldElite,
    // AbstractPlayer.java::preBattlePrep, MonsterRoomElite.java::applyEmeraldEliteBuff.
    let mut saw_multi_enemy = false;
    let mut saw_fractional_hp_rounding = false;

    for ascension in [0, 20] {
        for buff in 0..=3 {
            let seed = seed_for_next_map_buff(ascension, buff);
            for act in 1..=3 {
                let (baseline, baseline_map_counter) =
                    enter_elite(seed, ascension, act, false, false);
                let (burning, burning_map_counter) = enter_elite(seed, ascension, act, true, false);
                assert_eq!(baseline_map_counter, burning_map_counter);
                assert_eq!(baseline.rng_counters()["map"], baseline_map_counter);
                assert_eq!(burning.rng_counters()["map"], burning_map_counter + 1);

                let baseline_combat = baseline.get_combat_engine().expect("baseline elite");
                let burning_combat = burning.get_combat_engine().expect("burning elite");
                assert_eq!(
                    baseline_combat.state.enemies.len(),
                    burning_combat.state.enemies.len()
                );
                saw_multi_enemy |= burning_combat.state.enemies.len() > 1;

                for (plain, emerald) in baseline_combat
                    .state
                    .enemies
                    .iter()
                    .zip(&burning_combat.state.enemies)
                {
                    assert_eq!(plain.id, emerald.id);
                    let expected_max_hp = if buff == 1 {
                        let increase = (plain.entity.max_hp + 2) / 4;
                        saw_fractional_hp_rounding |= plain.entity.max_hp % 4 != 0;
                        plain.entity.max_hp + increase
                    } else {
                        plain.entity.max_hp
                    };
                    assert_eq!(emerald.entity.max_hp, expected_max_hp);
                    assert_eq!(
                        emerald.entity.hp,
                        if buff == 1 {
                            plain.entity.hp + (plain.entity.max_hp + 2) / 4
                        } else {
                            plain.entity.hp
                        }
                    );
                    assert_eq!(
                        emerald.entity.status(crate::status_ids::sid::STRENGTH)
                            - plain.entity.status(crate::status_ids::sid::STRENGTH),
                        if buff == 0 { act + 1 } else { 0 }
                    );
                    assert_eq!(
                        emerald.entity.status(crate::status_ids::sid::METALLICIZE)
                            - plain.entity.status(crate::status_ids::sid::METALLICIZE),
                        if buff == 2 { act * 2 + 2 } else { 0 }
                    );
                    assert_eq!(
                        emerald.entity.status(crate::status_ids::sid::REGENERATION)
                            - plain.entity.status(crate::status_ids::sid::REGENERATION),
                        if buff == 3 { act * 2 + 1 } else { 0 }
                    );
                }
            }
        }
    }

    assert!(
        saw_multi_enemy,
        "matrix must prove one roll is shared by multiple enemies"
    );
    assert!(
        saw_fractional_hp_rounding,
        "matrix must include a non-multiple-of-four max HP fixture"
    );
}

#[test]
fn emerald_max_hp_buff_precedes_preserved_insect_and_uses_java_rounding() {
    // IncreaseMaxHpAction.java uses Math.round(maxHp * 0.25F), then
    // PreservedInsect.java runs during player relic setup and floors current HP
    // to 75% of the already-increased maximum. Java: AbstractPlayer.java,
    // IncreaseMaxHpAction.java, PreservedInsect.java.
    let seed = seed_for_next_map_buff(0, 1);
    let (plain, _) = enter_elite(seed, 0, 1, false, true);
    let (burning, _) = enter_elite(seed, 0, 1, true, true);
    let plain_enemies = &plain
        .get_combat_engine()
        .expect("plain elite")
        .state
        .enemies;
    let burning_enemies = &burning
        .get_combat_engine()
        .expect("burning elite")
        .state
        .enemies;
    assert_eq!(plain_enemies.len(), burning_enemies.len());

    for (plain_enemy, burning_enemy) in plain_enemies.iter().zip(burning_enemies) {
        assert_eq!(plain_enemy.id, burning_enemy.id);
        let base_max = plain_enemy.entity.max_hp;
        let increased_max = base_max + (base_max + 2) / 4;
        assert_eq!(plain_enemy.entity.hp, base_max * 3 / 4);
        assert_eq!(burning_enemy.entity.max_hp, increased_max);
        assert_eq!(burning_enemy.entity.hp, increased_max * 3 / 4);
    }
}

#[test]
fn black_star_emerald_reward_order_is_exact_and_the_key_is_independent() {
    // MonsterRoomElite.java adds gold, the normal relic, Black Star's relic,
    // then Emerald. RewardItem.java intentionally ignores Emerald's supplied
    // relic and creates no mutual-exclusion link. Four pre-potion rewards force
    // potion chance to zero while still consuming that roll.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Black Star".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.debug_set_active_emerald_elite(true);
    let potion_counter = engine.rng_counters()["potion"];
    engine.debug_build_combat_reward_screen(RoomType::Elite);

    let screen = engine
        .current_reward_screen()
        .expect("burning elite rewards");
    assert_eq!(screen.items.len(), 5);
    assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
    assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[2].kind, RewardItemKind::Relic);
    assert_eq!(
        screen.items[3].kind,
        RewardItemKind::Key {
            color: RewardKeyColor::Emerald,
            linked_item_index: None,
        }
    );
    assert_eq!(screen.items[4].kind, RewardItemKind::CardChoice);
    assert_eq!(engine.rng_counters()["potion"], potion_counter + 1);

    assert!(engine
        .step_game(&GameAction::SelectRewardItem(3))
        .accepted());
    let after_key = engine.current_reward_screen().expect("rewards remain open");
    assert_eq!(after_key.items[3].state, RewardItemState::Claimed);
    assert_eq!(after_key.items[1].state, RewardItemState::Available);
    assert_eq!(after_key.items[2].state, RewardItemState::Available);
}

#[test]
fn act_four_elite_relic_tier_boundaries_are_50_33_17_at_a0_and_a20() {
    // TheEnding.java installs the ordinary 50/33/17 relic tier chances, and
    // MonsterRoomElite.java uses AbstractDungeon.returnRandomRelicTier without
    // an Act 4 or ascension override. Boundary rolls are inclusive 0..99.
    for ascension in [0, 20] {
        for (roll, expected_tier) in [
            (49, "common"),
            (50, "uncommon"),
            (82, "uncommon"),
            (83, "rare"),
        ] {
            let seed = seed_for_next_relic_roll(roll);
            let mut engine = RunEngine::new(seed, ascension);
            assert_eq!(engine.rng_counters()["relic"], 5);
            engine.run_state.act = 4;
            engine.run_state.floor = 54;
            engine.debug_build_combat_reward_screen(RoomType::Elite);
            let screen = engine.current_reward_screen().expect("Act 4 elite rewards");
            assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
            assert_eq!(
                reward_relic_tier(&screen.items[1].label),
                expected_tier,
                "ascension {ascension}, boundary roll {roll}, relic {}",
                screen.items[1].label
            );
            assert_eq!(engine.rng_counters()["relic"], 6);
        }
    }
}

#[test]
fn elite_reward_screen_exposes_java_reward_order_without_forcing_claim_order() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_build_combat_reward_screen(RoomType::Elite);

    let screen = engine
        .current_reward_screen()
        .expect("elite reward screen should be present");
    assert!(!screen.ordered);
    assert_eq!(screen.items.len(), 4);
    assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
    assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[2].kind, RewardItemKind::Potion);
    assert_eq!(screen.items[3].kind, RewardItemKind::CardChoice);
    assert!(screen.items.iter().all(|item| item.claimable));
    assert!(engine
        .get_legal_actions()
        .contains(&GameAction::LeaveRewards));
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
        engine.get_legal_actions(),
        vec![GameAction::SelectRewardItem(0)]
    );

    let step = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(step.accepted());
    assert_eq!(
        step.next_decision.legal_actions,
        vec![
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            },
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 1,
            },
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 2,
            },
            GameAction::SkipRewardItem(0),
        ]
    );
}

#[test]
fn prayer_wheel_and_question_card_expand_reward_structure() {
    // Sources: PrayerWheel.java and CombatRewardScreen.java::setupItemReward.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Prayer Wheel".to_string());
    engine.run_state.relics.push("Question Card".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("combat reward screen should exist");
    assert_eq!(
        screen
            .items
            .iter()
            .filter(|item| item.kind == RewardItemKind::CardChoice)
            .count(),
        2
    );
    assert!(screen
        .items
        .iter()
        .filter(|item| item.kind == RewardItemKind::CardChoice)
        .all(|item| item.choices.len() == 4));
}

#[test]
fn mystery_monster_prayer_wheel_uses_resolved_monster_room_rewards() {
    // EventRoom.onPlayerEntry replaces the map's EventRoom with the concrete
    // MonsterRoom returned by EventHelper.generateRoom. CombatRewardScreen
    // therefore observes a MonsterRoom and PrayerWheel adds a second card
    // reward even though the map node still displays `?`.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/EventRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/EventHelper.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PrayerWheel.java
    let mut engine = RunEngine::new(42, 0);
    engine.run_state.relics.push("Prayer Wheel".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.debug_force_event_rolls(&[0]);

    engine.debug_enter_mystery_room();
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Combat);
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    let screen = engine
        .current_reward_screen()
        .expect("mystery monster combat should open normal rewards");
    assert_eq!(
        screen
            .items
            .iter()
            .filter(|item| item.kind == RewardItemKind::CardChoice)
            .count(),
        2,
    );
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

    let claim = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(claim.accepted());
    assert!(engine
        .run_state
        .relic_flags
        .has(crate::relic_flags::flag::MOLTEN_EGG));
    let screen = engine
        .current_reward_screen()
        .expect("reward screen remains open");
    assert!(matches!(
        &screen.items[1].choices[0],
        RewardChoice::Card { card_id, .. } if card_id == "Wallop+"
    ));
    assert!(matches!(
        &screen.items[1].choices[1],
        RewardChoice::Card { card_id, .. } if card_id == "Scrawl"
    ));
    assert_eq!(
        claim.next_decision.legal_actions,
        vec![GameAction::SelectRewardItem(1)]
    );

    let open = engine.step_game(&GameAction::SelectRewardItem(1));
    assert!(open.accepted());
    let choose = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 1,
        choice_index: 0,
    });
    assert!(choose.accepted());
    assert_eq!(
        engine.run_state.deck.last().map(String::as_str),
        Some("Wallop+")
    );
}

#[test]
fn singing_bowl_keeps_skip_separate_from_its_max_hp_choice() {
    // Sources: CardRewardScreen.java shows both buttons, while
    // SingingBowlButton.java::onClick alone grants 2 max HP.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Singing Bowl".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    let max_hp_before = engine.run_state.max_hp;
    let hp_before = engine.run_state.current_hp;
    engine.debug_set_card_reward_screen(vec!["Wallop".to_string(), "Scrawl".to_string()]);

    assert!(engine
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    let step = engine.step_game(&GameAction::SkipRewardItem(0));
    assert!(step.accepted());
    assert_eq!(engine.run_state.max_hp, max_hp_before);
    assert_eq!(engine.run_state.current_hp, hp_before);
}

#[test]
fn white_beast_adds_potion_reward_item_before_card_choice() {
    let mut engine = RunEngine::new(42, 20);
    engine
        .run_state
        .relics
        .push("White Beast Statue".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("reward screen should exist");
    assert_eq!(screen.items.len(), 3);
    assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
    assert_eq!(screen.items[1].kind, RewardItemKind::Potion);
    assert!(screen.items[1].claimable);
    assert_eq!(screen.items[2].kind, RewardItemKind::CardChoice);
    assert!(screen.items[2].claimable);

    let offered_potion = screen.items[1].label.clone();
    let claim_potion = engine.step_game(&GameAction::SelectRewardItem(1));
    assert!(claim_potion.accepted());
    assert!(engine
        .run_state
        .potions
        .iter()
        .any(|p| p == &offered_potion));
    let mut expected_actions = vec![
        GameAction::SelectRewardItem(0),
        GameAction::SelectRewardItem(2),
        GameAction::LeaveRewards,
    ];
    // FruitJuice.canUse permits use on non-combat reward screens.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
    if matches!(offered_potion.as_str(), "FruitJuice" | "Fruit Juice") {
        expected_actions.push(GameAction::UsePotion(0));
    }
    // AbstractPotion.canDiscard keeps the occupied top-panel slot available
    // while the reward screen is open.
    expected_actions.push(GameAction::DiscardPotion(0));
    assert_eq!(claim_potion.next_decision.legal_actions, expected_actions);
}

#[test]
fn sozu_keeps_potion_reward_claimable_but_obtains_nothing() {
    // AbstractRoom.addPotionToRewards does not check Sozu. RewardItem checks
    // it only on claim, returns true, and does not call obtainPotion.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Sozu".to_string());
    engine
        .run_state
        .relics
        .push("White Beast Statue".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.debug_build_combat_reward_screen(RoomType::Monster);

    let screen = engine
        .current_reward_screen()
        .expect("reward screen should exist");
    assert_eq!(screen.items.len(), 3);
    assert_eq!(screen.items[1].kind, RewardItemKind::Potion);
    assert!(screen.items[1].claimable);
    let before = engine.run_state.potions.clone();
    assert!(engine
        .step_game(&GameAction::SelectRewardItem(1))
        .accepted());
    assert_eq!(engine.run_state.potions, before);
    assert_eq!(
        engine.get_legal_actions(),
        vec![
            GameAction::SelectRewardItem(0),
            GameAction::SelectRewardItem(2),
            GameAction::LeaveRewards,
        ]
    );
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
    assert!(engine
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert_eq!(engine.run_state.potions, before);
    assert_eq!(
        engine.current_reward_screen().expect("reward").items[0].state,
        RewardItemState::Available
    );
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
    assert_eq!(screen.source, RewardScreenSource::BossRelic);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(screen.items[0].choices.len(), 3);
    assert!(screen.items[0].claimable);
    assert!(screen.items[0].skip_allowed);

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

    let open = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(open.accepted());
    assert_eq!(
        open.next_decision
            .context
            .reward_screen
            .as_ref()
            .and_then(|s| s.active_item),
        Some(0)
    );
    assert_eq!(
        open.next_decision.legal_actions,
        vec![
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            },
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 1,
            },
            GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 2,
            },
            GameAction::SkipRewardItem(0),
        ]
    );

    let choose = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index,
    });
    assert!(choose.accepted());
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == &chosen_relic));
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert_eq!(engine.run_state.act, 1);
    assert_eq!(engine.run_state.current_hp, 20);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.act, 2);
    assert_eq!(engine.run_state.floor, 17);
    assert_eq!(engine.run_state.current_hp, 56);
    assert_eq!(engine.run_state.map_x, 0);
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
        assert_eq!(engine.run_state.floor, boss_floor);
        assert_eq!(
            engine
                .current_reward_screen()
                .expect("boss combat rewards")
                .source,
            RewardScreenSource::BossCombat
        );
        assert!(engine.step_game(&GameAction::LeaveRewards).accepted());
        assert_eq!(engine.run_state.floor, chest_floor);
        assert_eq!(engine.current_phase(), crate::run::RunPhase::Chest);
        assert_floor_rngs(&engine, seed, chest_floor, [0, 0, 0, 0, 0]);
        assert!(engine.step_game(&GameAction::OpenChest).accepted());
        assert_eq!(engine.current_phase(), crate::run::RunPhase::CardReward);

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
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert!(engine
            .step_game(&GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: astrolabe_index,
            })
            .accepted());
        assert_floor_rngs(&engine, seed, chest_floor, [0, 0, 0, 0, 0]);

        for _ in 0..3 {
            assert!(engine
                .step_game(&GameAction::SelectRewardItem(0))
                .accepted());
            assert!(engine
                .step_game(&GameAction::ChooseRewardOption {
                    item_index: 0,
                    choice_index: 0,
                })
                .accepted());
        }

        assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
        assert_eq!(engine.run_state.act, act);
        assert!(engine.step_game(&GameAction::Proceed).accepted());
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

    engine.step_game(&GameAction::SelectRewardItem(0));
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
    engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index,
    });

    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert_eq!(engine.run_state.act, 2);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.act, 3);
    assert_eq!(engine.run_state.floor, 34);
    assert_eq!(engine.run_state.current_hp, engine.run_state.max_hp);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
    let expected = crate::map::generate_map(seed + 600, 0);
    for y in 0..expected.height {
        for x in 0..expected.width {
            assert_eq!(
                engine.map.rows[y][x].room_type,
                expected.rows[y][x].room_type
            );
            assert_eq!(engine.map.rows[y][x].edges, expected.rows[y][x].edges);
        }
    }
}

#[test]
fn act_three_boss_routes_to_spire_heart_without_a_boss_relic() {
    // Re-verified: AbstractRoom constructs a miscRng-based boss gold reward
    // before the Act 3 Proceed path suppresses the combat reward screen.
    // Java: AbstractRoom.java:286-331, ProceedButton.java:100-113.
    let mut engine = RunEngine::new(44, 0);
    engine.run_state.act = 3;
    engine.run_state.floor = 50;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    let gold_before_boss = engine.run_state.gold;
    engine.debug_enter_specific_combat(&["TimeEater"]);
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    assert_eq!(engine.run_state.floor, 50);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.floor, 51);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Event);
    assert_eq!(
        engine
            .debug_current_event()
            .as_ref()
            .map(|event| event.name.as_str()),
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

    engine.step_game(&GameAction::SelectRewardItem(0));
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
    engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index,
    });

    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.act, 3);
    let first_boss = engine.boss_name().to_string();
    let gold_before_bosses = engine.run_state.gold;
    engine.run_state.floor = 50;
    engine.run_state.map_x = 0;
    engine.run_state.map_y = 14;
    engine.debug_enter_current_boss_combat();
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert_eq!(engine.run_state.floor, 50);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    let second_boss = engine.boss_name().to_string();
    assert_ne!(second_boss, first_boss);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Combat);
    assert_eq!(engine.run_state.floor, 51);
    assert_eq!(engine.run_state.gold, gold_before_bosses);
    assert_eq!(engine.run_state.bosses_killed, 1);
    assert!(engine.current_reward_screen().is_none());

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();

    assert_eq!(engine.current_phase(), crate::run::RunPhase::Transition);
    assert_eq!(engine.run_state.floor, 51);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_eq!(engine.run_state.floor, 52);
    assert_eq!(engine.run_state.bosses_killed, 2);
    assert_eq!(engine.run_state.gold, gold_before_bosses);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Event);
    assert_eq!(
        engine
            .debug_current_event()
            .as_ref()
            .map(|event| event.name.as_str()),
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

    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    assert_eq!(engine.current_phase(), crate::run::RunPhase::CardReward);
    let screen = engine
        .current_reward_screen()
        .expect("event reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);

    let relic_id = screen.items[0].label.clone();
    let claim = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(claim.accepted());
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == &relic_id));
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

    let open = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(open.accepted());

    let choose = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 1,
    });
    assert!(choose.accepted());
    assert!(!engine.run_state.deck.iter().any(|card| card == "Wallop"));
    assert_eq!(engine.run_state.deck.len(), 2);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::MapChoice);
}
