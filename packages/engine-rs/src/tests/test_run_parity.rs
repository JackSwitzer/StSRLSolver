// Java references:
// /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/neow/{NeowEvent.java,NeowReward.java,NeowRoom.java}
// /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/rooms/{EventRoom.java,MonsterRoom.java,MonsterRoomBoss.java,RestRoom.java,ShopRoom.java,TreasureRoom.java,TreasureRoomBoss.java}
// /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/rewards/{RewardItem.java}
// /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/rewards/chests/{SmallChest.java,MediumChest.java,LargeChest.java,BossChest.java}
// /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/shop/{Merchant.java,ShopScreen.java}

#[cfg(test)]
mod run_java_parity_tests {
    use crate::actions::Action;
    use crate::map::RoomType;
    use crate::run::{
        ActionStatus, BossSeenId, BossSeenSnapshot, GameAction, ProfileSnapshot, ProfileUpdate,
        RunEngine, RunPhase,
    };

    fn set_first_reachable_room(engine: &mut RunEngine, room_type: RoomType) {
        let start = engine.map.get_start_nodes()[0];
        let (x, y) = (start.x, start.y);
        engine.map.rows[y][x].room_type = room_type;
    }

    fn resolve_opening_neow(engine: &mut RunEngine) {
        if engine.current_phase() == RunPhase::Neow {
            let intro = engine.step_game(&GameAction::Proceed);
            assert!(intro.accepted());
            assert!(!intro.is_terminal());

            let outcome = engine.step_game(&GameAction::ChooseNeowOption(1));
            assert!(outcome.accepted());
            assert!(!outcome.is_terminal());
            while engine.current_phase() == RunPhase::CardReward {
                let actions = engine.get_legal_actions();
                let action = actions
                    .iter()
                    .find(|action| matches!(action, GameAction::SkipRewardItem(_)))
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::SelectRewardItem(_)))
                    })
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::ChooseRewardOption { .. }))
                    })
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|action| matches!(action, GameAction::LeaveRewards))
                    })
                    .cloned()
                    .expect("Neow follow-up must expose a reward action");
                engine.step_game(&action);
            }
            assert_eq!(engine.current_phase(), RunPhase::Neow);
            let exit = engine.step_game(&GameAction::Proceed);
            assert!(exit.accepted());
            assert!(!exit.is_terminal());
            assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        }
    }

    #[test]
    fn ascension_zero_watcher_run_starts_at_java_hp_and_gold() {
        let engine = RunEngine::new(42, 0);
        assert_eq!(engine.run_state.max_hp, 72);
        assert_eq!(engine.run_state.current_hp, 72);
        assert_eq!(engine.run_state.gold, 99);
        assert_eq!(engine.run_state.relics, vec!["PureWater".to_string()]);
    }

    #[test]
    fn ascension_twenty_run_uses_java_hp_floor_and_bane() {
        let engine = RunEngine::new(42, 20);
        assert_eq!(engine.run_state.max_hp, 68);
        assert_eq!(engine.run_state.current_hp, 68);
        assert!(engine.run_state.deck.contains(&"AscendersBane".to_string()));
    }

    #[test]
    fn first_path_choice_advances_to_floor_one() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        assert_eq!(engine.run_state.floor, 1);
    }

    #[test]
    fn treasure_room_entry_defers_open_callbacks_until_open_action() {
        // TreasureRoom.onPlayerEntry constructs the chest, consuming the size
        // and reward rolls. AbstractChest.open is a separate click that runs
        // relic callbacks, curse creation, gold, and relic reward generation.
        let mut engine = RunEngine::new(39, 0);
        engine
            .run_state
            .relics
            .extend(["Cursed Key".to_string(), "Matryoshka".to_string()]);
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine
            .run_state
            .relic_flags
            .init_relic_counter("Matryoshka");
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Treasure);

        let deck_before = engine.run_state.deck.len();
        engine.step_game(&engine.get_legal_actions()[0].clone());

        assert_eq!(engine.phase, RunPhase::Chest);
        assert_eq!(engine.rng_counters()["treasure"], 2);
        assert_eq!(engine.run_state.deck.len(), deck_before);
        assert_eq!(
            engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
            2
        );
        assert_eq!(
            engine.get_legal_actions(),
            vec![GameAction::OpenChest, GameAction::LeaveChest]
        );

        let result = engine.step_game(&GameAction::OpenChest);
        assert!(result.accepted());
        assert_eq!(engine.phase, RunPhase::CardReward);
        let screen = engine.current_reward_screen().expect("treasure rewards");
        assert_eq!(screen.source, crate::decision::RewardScreenSource::Treasure);
        assert!(screen
            .items
            .iter()
            .any(|item| item.kind == crate::decision::RewardItemKind::Relic));
        assert!(screen.items.iter().any(|item| matches!(
            item.kind,
            crate::decision::RewardItemKind::Key {
                color: crate::decision::RewardKeyColor::Sapphire,
                linked_item_index: Some(_),
            }
        )));
        assert_eq!(engine.rng_counters()["treasure"], 2);
        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(
            engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
            1
        );
    }

    #[test]
    fn leaving_an_unopened_chest_consumes_no_open_rng_or_callbacks() {
        // TreasureRoom's Proceed button permits leaving without opening.
        // Java: TreasureRoom.java::onPlayerEntry, AbstractChest.java::open.
        let mut engine = RunEngine::new(39, 0);
        engine
            .run_state
            .relics
            .extend(["Cursed Key".to_string(), "Matryoshka".to_string()]);
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine
            .run_state
            .relic_flags
            .init_relic_counter("Matryoshka");
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Treasure);

        let deck_before = engine.run_state.deck.clone();
        engine.step_game(&engine.get_legal_actions()[0].clone());
        let counters_after_entry = engine.rng_counters();
        let relic_pool_before = engine.debug_relic_pool_lengths();

        let result = engine.step_game(&GameAction::LeaveChest);
        assert!(result.accepted());
        assert_eq!(engine.phase, RunPhase::MapChoice);
        assert_eq!(engine.rng_counters(), counters_after_entry);
        assert_eq!(engine.debug_relic_pool_lengths(), relic_pool_before);
        assert_eq!(engine.run_state.deck, deck_before);
        assert_eq!(
            engine.run_state.relic_flags.counters[crate::relic_flags::counter::MATRYOSHKA_USES],
            2
        );
    }

    #[test]
    fn sapphire_key_and_chest_relic_are_mutually_exclusive_rewards() {
        // AbstractChest.open creates a SAPPHIRE_KEY RewardItem linked in both
        // directions with the chest's relic. Claiming either marks the other
        // ignored/done. RewardItem.java::RewardItem(RewardItem, RewardType).
        let mut take_key = RunEngine::new(39, 0);
        take_key.debug_build_treasure_reward_screen();
        let screen = take_key.current_reward_screen().expect("treasure rewards");
        let key_index = screen
            .items
            .iter()
            .position(|item| {
                matches!(
                    item.kind,
                    crate::decision::RewardItemKind::Key {
                        color: crate::decision::RewardKeyColor::Sapphire,
                        ..
                    }
                )
            })
            .expect("sapphire key reward");
        let linked_relic = match screen.items[key_index].kind {
            crate::decision::RewardItemKind::Key {
                linked_item_index: Some(index),
                ..
            } => index,
            _ => panic!("sapphire key must link its chest relic"),
        };

        assert!(take_key
            .step_game(&GameAction::SelectRewardItem(key_index))
            .accepted());
        assert!(take_key.run_state.has_sapphire_key);
        let screen = take_key
            .current_reward_screen()
            .expect("reward screen stays open");
        assert_eq!(
            screen.items[key_index].state,
            crate::decision::RewardItemState::Claimed
        );
        assert_eq!(
            screen.items[linked_relic].state,
            crate::decision::RewardItemState::Disabled
        );

        let mut take_relic = RunEngine::new(39, 0);
        take_relic.debug_build_treasure_reward_screen();
        let screen = take_relic
            .current_reward_screen()
            .expect("treasure rewards");
        let key_index = screen
            .items
            .iter()
            .position(|item| {
                matches!(
                    item.kind,
                    crate::decision::RewardItemKind::Key {
                        color: crate::decision::RewardKeyColor::Sapphire,
                        ..
                    }
                )
            })
            .expect("sapphire key reward");
        let linked_relic = match screen.items[key_index].kind {
            crate::decision::RewardItemKind::Key {
                linked_item_index: Some(index),
                ..
            } => index,
            _ => panic!("sapphire key must link its chest relic"),
        };
        assert!(take_relic
            .step_game(&GameAction::SelectRewardItem(linked_relic))
            .accepted());
        assert!(!take_relic.run_state.has_sapphire_key);
        let screen = take_relic
            .current_reward_screen()
            .expect("reward screen stays open");
        assert_eq!(
            screen.items[linked_relic].state,
            crate::decision::RewardItemState::Claimed
        );
        assert_eq!(
            screen.items[key_index].state,
            crate::decision::RewardItemState::Disabled
        );
    }

    #[test]
    fn combat_gold_is_an_independent_claim_and_global_proceed_discards_the_rest() {
        // AbstractRoom adds gold to the reward list first; CombatRewardScreen
        // lets every item update independently and exposes one Proceed button.
        // Java: AbstractRoom.java:286-331, CombatRewardScreen.java:111-140.
        let mut engine = RunEngine::new(42, 0);
        let gold_before = engine.run_state.gold;
        engine.debug_build_combat_reward_screen(RoomType::Monster);
        let screen = engine.current_reward_screen().expect("combat rewards");
        assert!(!screen.ordered);
        assert_eq!(screen.items[0].kind, crate::decision::RewardItemKind::Gold);
        let gold_amount = screen.items[0].label.parse::<i32>().expect("gold amount");
        assert_eq!(engine.run_state.gold, gold_before);
        assert!(engine
            .get_legal_actions()
            .contains(&GameAction::LeaveRewards));

        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert_eq!(engine.run_state.gold, gold_before + gold_amount);
        assert_eq!(engine.phase, RunPhase::CardReward);
        assert!(engine.step_game(&GameAction::LeaveRewards).accepted());
        assert_eq!(engine.phase, RunPhase::MapChoice);
    }

    #[test]
    fn boss_rewards_then_boss_chest_reopens_the_same_relic_choices_after_skip() {
        // Boss combat rewards precede TreasureRoomBoss. BossChest constructs
        // three choices on room entry; cancel closes the chest so those same
        // objects can be presented again without another pool draw.
        // Java: ProceedButton.java:111, BossChest.java:27-50,
        // BossRelicSelectScreen.java::relicSkipLogic.
        let mut engine = RunEngine::new(42, 0);
        engine.debug_build_combat_reward_screen(RoomType::Boss);
        assert_eq!(
            engine
                .current_reward_screen()
                .expect("boss combat rewards")
                .source,
            crate::decision::RewardScreenSource::BossCombat
        );

        assert!(engine.step_game(&GameAction::LeaveRewards).accepted());
        assert_eq!(engine.phase, RunPhase::Chest);
        assert_eq!(
            engine.get_legal_actions(),
            vec![GameAction::OpenChest, GameAction::LeaveChest]
        );

        assert!(engine.step_game(&GameAction::OpenChest).accepted());
        let first_choices = engine
            .current_reward_screen()
            .expect("boss relic screen")
            .items[0]
            .choices
            .clone();
        assert_eq!(first_choices.len(), 3);
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert!(engine.step_game(&GameAction::SkipRewardItem(0)).accepted());
        assert_eq!(engine.phase, RunPhase::Chest);

        assert!(engine.step_game(&GameAction::OpenChest).accepted());
        assert_eq!(
            engine
                .current_reward_screen()
                .expect("reopened boss relic screen")
                .items[0]
                .choices,
            first_choices
        );
    }

    #[test]
    fn final_act_profile_gates_burning_elites_and_campfire_recall() {
        // Settings.isFinalActAvailable gates map Emerald placement and the
        // Recall campfire option. CampfireRecallEffect grants the Ruby key
        // without consuming RNG.
        // Java: CampfireUI.java:84-102, AbstractDungeon.java::setEmeraldElite.
        let mut unavailable_profile = ProfileSnapshot::default();
        unavailable_profile.final_act_available = false;
        let mut unavailable = RunEngine::new_with_profile(42, 0, unavailable_profile);
        assert!(unavailable
            .map
            .rows
            .iter()
            .flatten()
            .all(|node| !node.has_emerald_key));
        unavailable.phase = RunPhase::Campfire;
        assert!(!unavailable
            .get_legal_actions()
            .contains(&GameAction::CampfireRecall));

        let mut available = RunEngine::new(42, 0);
        available.phase = RunPhase::Campfire;
        let rng_before = available.rng_counters();
        assert!(available.step_game(&GameAction::CampfireRecall).accepted());
        assert!(available.run_state.has_ruby_key);
        assert_eq!(available.rng_counters(), rng_before);
        assert_eq!(available.phase, RunPhase::MapChoice);
    }

    #[test]
    fn map_decision_context_exposes_the_visible_emerald_elite_marker() {
        // MapRoomNode renders the burning-elite flame before the path is
        // selected, so the canonical decision context must expose it.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/map/MapRoomNode.java:288.
        let mut engine = RunEngine::new(2024, 0);
        let (start_x, start_y) = {
            let start = engine.map.get_start_nodes()[0];
            (start.x, start.y)
        };
        engine.map.rows[start_y][start_x].has_emerald_key = true;
        engine.phase = RunPhase::MapChoice;

        let context = engine.current_decision_context();
        let map = context.map.expect("map decision context");
        assert!(map
            .paths
            .iter()
            .any(|path| {
                path.x == start_x as i32
                    && path.y == start_y as i32
                    && path.has_emerald_key
            }));
    }

    #[test]
    fn final_act_transition_installs_the_fixed_map_without_consuming_map_rng() {
        // AbstractDungeon.dungeonTransitionSetup advances cardRng to its next
        // bucket and heals, while TheEnding creates mapRng(seed + 1200) and a
        // fixed map without drawing it. The first room transition is Rest 52.
        // Java: AbstractDungeon.java:2552-2590, TheEnding.java:41-109.
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.floor = 51;
        engine.run_state.current_hp = 10;
        engine.run_state.has_ruby_key = true;
        engine.run_state.has_emerald_key = true;
        engine.run_state.has_sapphire_key = true;
        engine.debug_advance_persistent_card_rng_once();
        let before = engine.rng_counters();

        engine.debug_start_final_act();
        assert_eq!(engine.run_state.act, 4);
        assert_eq!(engine.run_state.floor, 51);
        assert_eq!(engine.run_state.current_hp, engine.run_state.max_hp);
        assert_eq!(engine.phase, RunPhase::MapChoice);
        assert_eq!(engine.rng_counters()["card"], 250);
        assert_eq!(engine.rng_counters()["map"], 0);
        for stream in [
            "monster", "event", "relic", "treasure", "potion", "merchant",
        ] {
            assert_eq!(engine.rng_counters()[stream], before[stream], "{stream}");
        }
        let starts = engine.map.get_start_nodes();
        assert_eq!(starts.len(), 1);
        assert_eq!(starts[0].room_type, RoomType::Rest);

        assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
        assert_eq!(engine.run_state.floor, 52);
        assert_eq!(engine.phase, RunPhase::Campfire);
        assert!(engine.step_game(&GameAction::CampfireRest).accepted());
        assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
        assert_eq!(engine.run_state.floor, 53);
        assert_eq!(engine.phase, RunPhase::Shop);
    }

    #[test]
    fn burning_elite_uses_one_map_draw_and_emerald_is_independent_of_relics() {
        // MonsterRoomElite applies one shared 0..3 mapRng buff to every enemy,
        // then adds an independent Emerald reward after elite/Black Star relics.
        // Java: MonsterRoomElite.java:34-83, RewardItem.java:290-298.
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        let start = engine.map.get_start_nodes()[0];
        let (x, y) = (start.x, start.y);
        engine.map.rows[y][x].room_type = RoomType::Elite;
        engine.map.rows[y][x].has_emerald_key = true;
        let expected_buff = engine.debug_peek_map_rng_int3();
        let map_counter_before = engine.rng_counters()["map"];

        assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
        assert_eq!(engine.rng_counters()["map"], map_counter_before + 1);
        let combat = engine.get_combat_engine().expect("burning elite combat");
        for enemy in &combat.state.enemies {
            match expected_buff {
                0 => assert_eq!(
                    enemy.entity.status(crate::status_ids::sid::STRENGTH),
                    engine.run_state.act + 1
                ),
                1 => assert_eq!(enemy.entity.hp, enemy.entity.max_hp),
                2 => assert_eq!(
                    enemy.entity.status(crate::status_ids::sid::METALLICIZE),
                    engine.run_state.act * 2 + 2
                ),
                _ => assert_eq!(
                    enemy.entity.status(crate::status_ids::sid::REGENERATION),
                    1 + engine.run_state.act * 2
                ),
            }
        }

        let mut rewards = RunEngine::new(42, 0);
        rewards.debug_set_active_emerald_elite(true);
        rewards.debug_build_combat_reward_screen(RoomType::Elite);
        let screen = rewards.current_reward_screen().expect("elite rewards");
        let relic_index = screen
            .items
            .iter()
            .position(|item| item.kind == crate::decision::RewardItemKind::Relic)
            .expect("elite relic");
        let key_index = screen
            .items
            .iter()
            .position(|item| {
                matches!(
                    item.kind,
                    crate::decision::RewardItemKind::Key {
                        color: crate::decision::RewardKeyColor::Emerald,
                        linked_item_index: None,
                    }
                )
            })
            .expect("emerald key");
        rewards.step_game(&GameAction::SelectRewardItem(relic_index));
        assert_eq!(
            rewards
                .current_reward_screen()
                .expect("elite rewards")
                .items[key_index]
                .state,
            crate::decision::RewardItemState::Available
        );
        rewards.step_game(&GameAction::SelectRewardItem(key_index));
        assert!(rewards.run_state.has_emerald_key);
    }

    #[test]
    fn nloths_mask_removes_a_linked_sapphire_key_with_its_chest_relic() {
        // AbstractChest.open adds the Sapphire key before onChestOpenAfter.
        // AbstractRoom.removeOneRelicFromRewards removes the first relic and
        // its immediately following link, so Nloth's Mask cannot leave an
        // orphaned key reward when the chest relic is the first relic.
        let mut engine = RunEngine::new(39, 0);
        engine.run_state.relics.push("NlothsMask".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.run_state.relic_flags.counters[crate::relic_flags::counter::NLOTHS_MASK] = 1;

        engine.debug_build_treasure_reward_screen();

        let screen = engine.current_reward_screen().expect("treasure rewards");
        assert!(screen.items.iter().all(|item| {
            item.kind != crate::decision::RewardItemKind::Relic
                && !matches!(
                    item.kind,
                    crate::decision::RewardItemKind::Key {
                        color: crate::decision::RewardKeyColor::Sapphire,
                        ..
                    }
                )
        }));
        assert_eq!(
            engine.run_state.relic_flags.counters[crate::relic_flags::counter::NLOTHS_MASK],
            -2
        );
    }

    #[test]
    fn shop_room_generates_seven_cards_three_potions_and_base_remove_price() {
        // Merchant.java creates five colored cards plus one uncommon and one
        // rare colorless card; ShopScreen.java::initPotions creates 3 potions.
        let mut engine = RunEngine::new(42, 0);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        let shop = engine.get_shop().expect("shop should exist");
        assert_eq!(shop.cards.len(), 7);
        assert_eq!(shop.potions.len(), 3);
        assert_eq!(shop.remove_price, 75);
    }

    #[test]
    fn shop_remove_price_is_not_derived_from_combats_won() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.combats_won = 99;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        let shop = engine.get_shop().expect("shop should exist");
        assert_eq!(shop.remove_price, 75);
    }

    #[test]
    fn shop_remove_price_persists_across_visits_and_applies_discounts() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        assert_eq!(
            engine.get_shop().expect("shop should exist").remove_price,
            75
        );
        engine.step_game(&GameAction::ShopRemoveCard(0));
        assert_eq!(engine.run_state.purge_cost, 100);
        engine.step_game(&GameAction::ShopLeave);

        engine.run_state.relics.push("The Courier".to_string());
        engine.run_state.relics.push("Membership Card".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.debug_enter_shop();

        let shop = engine.get_shop().expect("shop should exist");
        assert_eq!(shop.remove_price, 40);
    }

    #[test]
    fn shop_buy_card_spends_gold_and_removes_the_offer() {
        // ShopScreen.purchaseCard removes the bought card without Courier;
        // Merchant.java supplied seven offers, so six remain.
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let shop = engine.get_shop().expect("shop should exist");
        let (card, price) = shop.cards[0].clone();
        let deck_before = engine.run_state.deck.len();

        engine.step_game(&GameAction::ShopBuyCard(0));

        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last(), Some(&card));
        assert_eq!(engine.run_state.gold, 999 - price);
        assert_eq!(engine.get_shop().expect("shop stays open").cards.len(), 6);
        assert_eq!(engine.phase, RunPhase::Shop);
    }

    #[test]
    fn shop_remove_card_spends_gold_and_disables_future_removal() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck.push("Tantrum".to_string());
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let remove_price = engine.get_shop().expect("shop should exist").remove_price;
        let deck_before = engine.run_state.deck.len();

        engine.step_game(&GameAction::ShopRemoveCard(0));

        assert_eq!(engine.run_state.deck.len(), deck_before - 1);
        assert_eq!(engine.run_state.gold, 999 - remove_price);
        let shop = engine.get_shop().expect("shop stays open");
        assert!(shop.removal_used);
        assert!(!engine
            .get_legal_actions()
            .iter()
            .any(|action| matches!(action, GameAction::ShopRemoveCard(_))));
    }

    #[test]
    fn shop_remove_parasite_reduces_max_hp_and_clamps_current_hp() {
        // Parasite.java::onRemoveFromMasterDeck calls decreaseMaxHealth(3).
        // AbstractCreature.decreaseMaxHealth floors max HP at one and clamps
        // current HP down when it exceeds the new maximum.
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck = vec!["Parasite".to_string()];
        engine.run_state.max_hp = 40;
        engine.run_state.current_hp = 40;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let remove_price = engine.get_shop().expect("shop should exist").remove_price;

        engine.step_game(&GameAction::ShopRemoveCard(0));

        assert_eq!(engine.run_state.max_hp, 37);
        assert_eq!(engine.run_state.current_hp, 37);
        assert_eq!(engine.run_state.gold, 999 - remove_price);
        assert!(engine.run_state.deck.is_empty());
        assert!(!engine.run_state.deck.iter().any(|card| card == "Parasite"));
        assert!(engine.get_shop().expect("shop stays open").removal_used);
    }

    #[test]
    fn shop_remove_has_no_deck_size_floor_when_card_is_purgeable() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck = vec!["Strike".to_string()];
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let legal = engine.get_legal_actions();
        assert!(legal.contains(&GameAction::ShopRemoveCard(0)));

        engine.step_game(&GameAction::ShopRemoveCard(0));
        assert!(engine.run_state.deck.is_empty());
        assert!(engine.get_shop().expect("shop stays open").removal_used);
    }

    #[test]
    fn shop_remove_excludes_unremovable_curses_from_legal_actions() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck = vec!["AscendersBane".to_string(), "Strike".to_string()];
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let legal = engine.get_legal_actions();
        assert!(!legal.contains(&GameAction::ShopRemoveCard(0)));
        assert!(legal.contains(&GameAction::ShopRemoveCard(1)));

        engine.step_game(&GameAction::ShopRemoveCard(1));
        assert_eq!(engine.run_state.deck, vec!["AscendersBane".to_string()]);
    }

    #[test]
    fn event_room_enters_event_phase_with_choices() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Event);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        assert_eq!(engine.phase, RunPhase::Event);
        assert!(engine.event_option_count() >= 1);
    }

    #[test]
    fn event_choice_resolves_back_to_map_phase() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Event);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let before_hp = engine.run_state.current_hp;
        let before_gold = engine.run_state.gold;
        engine.step_game(&GameAction::EventChoice(0));

        assert_eq!(engine.phase, RunPhase::MapChoice);
        assert!(engine.run_state.current_hp >= 0);
        assert!(engine.run_state.gold >= 0);
        assert!(
            engine.run_state.current_hp != before_hp
                || engine.run_state.gold != before_gold
                || engine.phase == RunPhase::MapChoice
        );
    }

    #[test]
    fn campfire_rest_truncates_thirty_percent_like_java() {
        // Source: CampfireSleepEffect.java casts maxHealth * 0.3f to int.
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.max_hp = 72;
        engine.run_state.current_hp = 40;
        engine.step_game(&GameAction::CampfireRest);
        assert_eq!(engine.run_state.current_hp, 61);
    }

    #[test]
    fn regal_pillow_adds_exactly_fifteen_after_truncated_rest_healing() {
        // Sources: RegalPillow.java defines HEAL_AMT 15; the
        // CampfireSleepEffect constructor adds it after truncating base heal.
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.max_hp = 72;
        engine.run_state.current_hp = 20;
        engine.run_state.relics.push("Regal Pillow".to_string());
        engine
            .run_state
            .relic_flags
            .rebuild(&engine.run_state.relics);
        engine.step_game(&GameAction::CampfireRest);
        assert_eq!(engine.run_state.current_hp, 56);
    }

    #[test]
    fn campfire_upgrade_adds_plus_suffix() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.deck = vec!["Strike".to_string(), "Eruption".to_string()];
        engine.step_game(&GameAction::CampfireUpgrade(1));
        assert_eq!(engine.run_state.deck[1], "Eruption+");
    }

    #[test]
    fn shop_leave_returns_to_map_choice() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        engine.step_game(&GameAction::ShopLeave);
        assert_eq!(engine.phase, RunPhase::MapChoice);
    }

    #[test]
    fn monster_room_entry_creates_live_combat_engine() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Monster);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        assert_eq!(engine.phase, RunPhase::Combat);
        assert_eq!(engine.current_room_type(), "monster");
        assert!(engine.get_combat_engine().is_some());
    }

    #[test]
    fn boss_name_is_one_of_java_act_one_bosses() {
        let engine = RunEngine::new(42, 0);
        assert!(matches!(
            engine.boss_name(),
            "TheGuardian" | "Hexaghost" | "SlimeBoss"
        ));
    }

    #[test]
    fn current_room_type_tracks_forced_shop_room() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);
        assert_eq!(engine.current_room_type(), "shop");
    }

    #[test]
    fn java_neow_rewards_exist_and_rust_run_exposes_the_start_phase() {
        let mut engine = RunEngine::new(42, 0);
        assert_eq!(engine.phase, RunPhase::Neow);
        assert_eq!(engine.current_room_type(), "neow");
        assert_eq!(engine.current_choice_count(), 1);
        assert_eq!(engine.get_legal_actions(), vec![GameAction::Proceed]);
        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.current_choice_count(), 4);
        assert!(engine
            .get_legal_actions()
            .iter()
            .all(|action| matches!(action, GameAction::ChooseNeowOption(_))));
        assert_eq!(
            engine.run_state.floor, 0,
            "Rust run begins before the first map choice"
        );
    }

    #[test]
    fn rust_run_enters_act_one_map_choice_after_neow() {
        let mut engine = RunEngine::new(42, 0);
        assert_eq!(engine.current_phase(), RunPhase::Neow);
        resolve_opening_neow(&mut engine);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert_eq!(engine.run_state.act, 1);
    }

    #[test]
    fn wish_gold_branch_syncs_into_run_state_after_combat_resolution() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Wish+".to_string(); 10];
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Monster);

        let actions = engine.get_legal_actions();
        engine.step_game(&actions[0]);

        let gold_before = engine.run_state.gold;
        let combat = engine.get_combat_engine().expect("combat should be active");
        let wish_idx = combat
            .state
            .hand
            .iter()
            .position(|card| combat.card_registry.card_name(card.def_id) == "Wish+")
            .expect("opening hand should contain Wish+");
        engine.step_game(&GameAction::CombatAction(Action::PlayCard {
            card_idx: wish_idx,
            target_idx: -1,
        }));
        engine.step_game(&GameAction::CombatAction(Action::Choose(1)));
        assert_eq!(
            engine
                .get_combat_engine()
                .expect("combat should still be active")
                .state
                .pending_run_gold,
            30
        );
        assert_eq!(engine.run_state.gold, gold_before + 30);
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();

        assert!(
            engine.run_state.gold >= gold_before + 30,
            "Wish+ gold branch should sync its 30 gold into RunState on combat resolution, got {}",
            engine.run_state.gold - gold_before
        );
    }

    // LessonLearnedAction upgrades AbstractDungeon.player.masterDeck during
    // combat. The selected permanent upgrade must survive combat resolution.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java
    #[test]
    fn lesson_learned_master_deck_upgrade_syncs_into_run_state() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec![
            "LessonLearned+".to_string(),
            "Wallop".to_string(),
            "Strike+".to_string(),
        ];
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Monster);
        let action = engine.get_legal_actions()[0].clone();
        engine.step_game(&action);

        {
            let combat = engine.debug_combat_engine_mut();
            combat.state.enemies[0].entity.hp = 13;
            combat.state.enemies[0].entity.block = 0;
        }
        let lesson_idx = engine
            .get_combat_engine()
            .expect("combat active")
            .state
            .hand
            .iter()
            .position(|card| {
                engine
                    .get_combat_engine()
                    .expect("combat active")
                    .card_registry
                    .card_name(card.def_id)
                    == "LessonLearned+"
            })
            .expect("Lesson Learned+ drawn");
        engine.step_game(&GameAction::CombatAction(Action::PlayCard {
            card_idx: lesson_idx,
            target_idx: 0,
        }));

        assert!(engine.run_state.deck.iter().any(|card| card == "Wallop+"));
        assert!(!engine.run_state.deck.iter().any(|card| card == "Wallop"));
    }

    // IncreaseMiscAction updates the matching card in player.masterDeck, and
    // CardSave persists misc between combats.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardSave.java
    #[test]
    fn genetic_algorithm_misc_syncs_through_run_state_into_the_next_combat() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.deck = vec!["Genetic Algorithm".to_string()];
        engine.run_state.deck_card_states.clear();
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Monster);
        let path = engine.get_legal_actions()[0].clone();
        engine.step_game(&path);

        let genetic_idx = engine
            .get_combat_engine()
            .expect("combat active")
            .state
            .hand
            .iter()
            .position(|card| {
                engine
                    .get_combat_engine()
                    .expect("combat active")
                    .card_registry
                    .card_name(card.def_id)
                    == "Genetic Algorithm"
            })
            .expect("Genetic Algorithm drawn");
        engine.step_game(&GameAction::CombatAction(Action::PlayCard {
            card_idx: genetic_idx,
            target_idx: -1,
        }));
        assert_eq!(
            engine
                .get_combat_engine()
                .expect("combat active")
                .state
                .player
                .block,
            1,
        );

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.run_state.deck_card_states[0].misc, 3);

        engine.debug_enter_specific_combat(&["JawWorm"]);
        let next = engine.get_combat_engine().expect("next combat active");
        assert_eq!(next.state.master_deck[0].misc, 3);
    }

    fn profile_with_seen_bosses(seen: &[BossSeenId]) -> ProfileSnapshot {
        let mut profile = ProfileSnapshot::fresh();
        for &boss in seen {
            profile.apply_update(&ProfileUpdate::MarkBossSeen { boss });
        }
        profile
    }

    #[test]
    fn boss_unseen_priority_matrix_consumes_zero_monster_rng() {
        // Each standard dungeon selects the first unseen profile key, then
        // duplicates that sole boss without calling monsterRng.randomLong().
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:189-210,
        // decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheCity.java:155-175,
        // decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheBeyond.java:149-169.
        let cases: &[(i32, &[BossSeenId], &str)] = &[
            (1, &[], "TheGuardian"),
            (1, &[BossSeenId::Guardian], "Hexaghost"),
            (1, &[BossSeenId::Guardian, BossSeenId::Ghost], "SlimeBoss"),
            (2, &[], "TheChamp"),
            (2, &[BossSeenId::Champ], "BronzeAutomaton"),
            (
                2,
                &[BossSeenId::Champ, BossSeenId::Automaton],
                "TheCollector",
            ),
            (3, &[], "AwakenedOne"),
            (3, &[BossSeenId::Crow], "DonuAndDeca"),
            (3, &[BossSeenId::Crow, BossSeenId::Donut], "TimeEater"),
        ];

        for &(act, seen, expected) in cases {
            let mut engine = RunEngine::new_with_profile(42, 20, profile_with_seen_bosses(seen));
            let counter_before = engine.rng_counters()["monster"];

            engine.debug_roll_boss_sequence_for_act(act);

            assert_eq!(
                engine.debug_boss_sequence(),
                vec![expected.to_string(), expected.to_string()],
                "act {act}, seen {seen:?}",
            );
            assert_eq!(
                engine.rng_counters()["monster"],
                counter_before,
                "unseen selection must not draw monster RNG for act {act}",
            );
        }
    }

    #[test]
    fn boss_all_seen_shuffle_seed_42_consumes_one_outer_draw() {
        // The all-seen branch consumes one StS monsterRng.randomLong(), then
        // passes that signed long to java.util.Random for Collections.shuffle.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:202-207,
        // decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheCity.java:168-173,
        // decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheBeyond.java:162-167;
        // java.util.Collections::shuffle.
        let expected = [
            (1, ["TheGuardian", "SlimeBoss", "Hexaghost"]),
            (2, ["BronzeAutomaton", "TheChamp", "TheCollector"]),
            (3, ["AwakenedOne", "DonuAndDeca", "TimeEater"]),
        ];

        for (act, expected) in expected {
            let mut engine = RunEngine::new_with_profile(
                42,
                20,
                ProfileSnapshot {
                    bosses_seen: BossSeenSnapshot::all_seen(),
                    ..ProfileSnapshot::fresh()
                },
            );
            engine.debug_set_persistent_monster_rng(crate::seed::StsRandom::new(42));

            engine.debug_roll_boss_sequence_for_act(act);

            assert_eq!(
                engine.debug_boss_sequence(),
                expected.into_iter().map(str::to_string).collect::<Vec<_>>()
            );
            assert_eq!(engine.rng_counters()["monster"], 1, "act {act}");
        }
    }

    #[test]
    fn a20_unseen_act_three_boss_does_not_create_a_second_boss() {
        // The unseen branch creates [boss, boss]. MonsterRoomBoss removes the
        // first item on entry, leaving one; ProceedButton only starts an A20
        // second boss when two entries remain after that removal.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheBeyond.java:149-169,
        // decompiled/java-src/com/megacrit/cardcrawl/rooms/MonsterRoomBoss.java:25-35,
        // decompiled/java-src/com/megacrit/cardcrawl/ui/buttons/ProceedButton.java:100-105,210-219.
        let mut engine = RunEngine::new_with_profile(42, 20, ProfileSnapshot::fresh());
        engine.run_state.act = 3;
        engine.run_state.floor = 50;
        engine.debug_roll_boss_sequence_for_act(3);
        assert_eq!(
            engine.debug_boss_sequence(),
            vec!["AwakenedOne".to_string(), "AwakenedOne".to_string()]
        );

        engine.debug_enter_boss_room();
        assert_eq!(
            engine.debug_boss_sequence(),
            vec!["AwakenedOne".to_string()]
        );
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();

        assert_eq!(engine.current_phase(), RunPhase::Transition);
        assert!(engine.step_game(&GameAction::Proceed).accepted());
        assert_eq!(engine.current_phase(), RunPhase::Event);
        assert!(engine.debug_current_enemy_ids().is_empty());
        assert_eq!(engine.run_state.bosses_killed, 1);
    }

    #[test]
    fn secret_portal_consumes_the_same_boss_room_frontier() {
        // SecretPortal installs a MonsterRoomBoss and enters it through the
        // normal room transition. That room obtains the current boss and
        // removes exactly bossList[0]; the event must not bypass this frontier.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SecretPortal.java:43-76,
        // decompiled/java-src/com/megacrit/cardcrawl/rooms/MonsterRoomBoss.java:25-35.
        let mut engine = RunEngine::new_with_profile(
            42,
            20,
            ProfileSnapshot {
                bosses_seen: BossSeenSnapshot::all_seen(),
                ..ProfileSnapshot::fresh()
            },
        );
        engine.run_state.act = 3;
        engine.run_state.floor = 45;
        engine.debug_roll_boss_sequence_for_act(3);
        let frontier_before = engine.debug_boss_sequence();
        let monster_counter_before = engine.rng_counters()["monster"];
        let portal = crate::events::typed_events_for_act(3)
            .into_iter()
            .find(|event| event.name == "Secret Portal")
            .expect("Secret Portal must be registered");
        engine.debug_set_typed_event_state(portal);

        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());

        assert_eq!(engine.current_phase(), RunPhase::Combat);
        // The Awakened One encounter expands to two Cultists plus the boss;
        // the boss-list frontier itself still owns only the boss ID.
        assert_eq!(
            engine.debug_current_enemy_ids().last(),
            frontier_before.first()
        );
        assert_eq!(engine.debug_boss_sequence(), frontier_before[1..].to_vec());
        assert_eq!(engine.rng_counters()["monster"], monster_counter_before);
    }

    #[test]
    fn boss_seen_profile_update_occurs_during_prebattle_before_outcome() {
        // Boss constructors mark their profile key as seen during preBattleAction,
        // before the player can win or lose the combat.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/TheGuardian.java:123-131,
        // decompiled/java-src/com/megacrit/cardcrawl/unlock/UnlockTracker.java:623-632.
        let mut engine = RunEngine::new_with_profile(42, 0, ProfileSnapshot::fresh());
        assert!(engine.profile_updates().is_empty());

        engine.debug_enter_boss_room();

        assert_eq!(
            engine.profile_updates(),
            &[ProfileUpdate::MarkBossSeen {
                boss: BossSeenId::Guardian,
            }]
        );
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        assert!(
            !engine
                .get_combat_engine()
                .expect("boss combat")
                .state
                .combat_over
        );
    }

    #[test]
    fn act_four_boss_frontier_is_three_hearts_with_zero_monster_rng() {
        // TheEnding.initializeBoss appends The Heart three times and never
        // samples monsterRng.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/TheEnding.java:192-197.
        let mut engine = RunEngine::new_with_profile(42, 20, ProfileSnapshot::fresh());
        let counter_before = engine.rng_counters()["monster"];

        engine.debug_start_final_act();

        assert_eq!(
            engine.debug_boss_sequence(),
            vec![
                "CorruptHeart".to_string(),
                "CorruptHeart".to_string(),
                "CorruptHeart".to_string(),
            ]
        );
        assert_eq!(engine.boss_name(), "CorruptHeart");
        assert_eq!(engine.rng_counters()["monster"], counter_before);
    }

    #[test]
    fn game_action_v2_every_variant_serde_round_trips_and_has_a_stable_key() {
        let actions = vec![
            GameAction::ChooseNeowOption(3),
            GameAction::ChoosePath(2),
            GameAction::OpenChest,
            GameAction::LeaveChest,
            GameAction::SelectRewardItem(1),
            GameAction::ChooseRewardOption {
                item_index: 1,
                choice_index: 2,
            },
            GameAction::SkipRewardItem(1),
            GameAction::LeaveRewards,
            GameAction::Proceed,
            GameAction::CampfireRest,
            GameAction::CampfireUpgrade(4),
            GameAction::CampfireToke,
            GameAction::CampfireLift,
            GameAction::CampfireDig,
            GameAction::CampfireRecall,
            GameAction::ShopBuyCard(1),
            GameAction::ShopBuyRelic(2),
            GameAction::ShopBuyPotion(0),
            GameAction::ShopRemoveCard(3),
            GameAction::ShopLeave,
            GameAction::EventChoice(1),
            GameAction::CombatAction(Action::PlayCard {
                card_idx: 2,
                target_idx: 0,
            }),
            GameAction::CombatAction(Action::UsePotion {
                potion_idx: 1,
                target_idx: -1,
            }),
            GameAction::CombatAction(Action::Choose(2)),
            GameAction::CombatAction(Action::ConfirmSelection),
            GameAction::CombatAction(Action::EndTurn),
            GameAction::UsePotion(1),
            GameAction::DiscardPotion(2),
        ];

        for action in actions {
            let encoded = serde_json::to_string(&action).expect("serialize GameAction");
            let decoded: GameAction =
                serde_json::from_str(&encoded).expect("deserialize GameAction");
            assert_eq!(decoded, action);
            assert_eq!(decoded.canonical_sort_key(), action.canonical_sort_key());
        }
    }

    #[test]
    fn rejected_game_action_preserves_rng_state_events_and_run_state() {
        let mut engine = RunEngine::new_with_ambient_seed(888, 0, 999);
        let rng_before = engine.rng_counters();
        let ambient_before = engine.ambient_math_rng_state();
        let state_before = serde_json::to_value(&engine.run_state).expect("serialize run state");
        let phase_before = engine.current_phase();
        let legal_before = engine.get_legal_actions();

        let outcome = engine.step_game(&GameAction::ShopLeave);

        assert_eq!(outcome.status, ActionStatus::Rejected);
        assert!(outcome.events.is_empty());
        assert_eq!(engine.rng_counters(), rng_before);
        assert_eq!(engine.ambient_math_rng_state(), ambient_before);
        assert_eq!(
            serde_json::to_value(&engine.run_state).expect("serialize run state"),
            state_before
        );
        assert_eq!(engine.current_phase(), phase_before);
        assert_eq!(outcome.next_decision.legal_actions, legal_before);
    }
}
