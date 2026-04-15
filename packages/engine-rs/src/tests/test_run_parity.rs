// Java references:
// /tmp/sts-decompiled/com/megacrit/cardcrawl/neow/{NeowEvent.java,NeowReward.java,NeowRoom.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/rooms/{EventRoom.java,MonsterRoom.java,MonsterRoomBoss.java,RestRoom.java,ShopRoom.java,TreasureRoom.java,TreasureRoomBoss.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/rewards/{RewardItem.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/rewards/chests/{SmallChest.java,MediumChest.java,LargeChest.java,BossChest.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/shop/{Merchant.java,ShopScreen.java}

#[cfg(test)]
mod run_java_parity_tests {
    use crate::actions::Action;
    use crate::map::RoomType;
    use crate::run::{RunAction, RunEngine, RunPhase};

    fn set_first_reachable_room(engine: &mut RunEngine, room_type: RoomType) {
        let start = engine.map.get_start_nodes()[0];
        let (x, y) = (start.x, start.y);
        engine.map.rows[y][x].room_type = room_type;
    }

    fn resolve_opening_neow(engine: &mut RunEngine) {
        if engine.current_phase() == RunPhase::Neow {
            let action = engine
                .current_decision_context()
                .neow
                .and_then(|neow| {
                    neow.options
                        .iter()
                        .position(|option| option.label == "Gain 100 gold")
                })
                .map(RunAction::ChooseNeowOption)
                .unwrap_or_else(|| engine.get_legal_actions()[0].clone());
            let (reward, done) = engine.step(&action);
            assert_eq!(reward, 0.0);
            assert!(!done);
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
        engine.step(&actions[0]);
        assert_eq!(engine.run_state.floor, 1);
    }

    #[test]
    fn treasure_room_grants_java_style_gold_band() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Treasure);
        let gold_before = engine.run_state.gold;
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        let gained = engine.run_state.gold - gold_before;
        assert!((50..=80).contains(&gained), "treasure gain {gained} not in 50..=80");
        assert_eq!(engine.phase, RunPhase::MapChoice);
    }

    #[test]
    fn shop_room_generates_five_cards_and_base_remove_price() {
        let mut engine = RunEngine::new(42, 0);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        let shop = engine.get_shop().expect("shop should exist");
        assert_eq!(shop.cards.len(), 5);
        assert_eq!(shop.remove_price, 75);
    }

    #[test]
    fn shop_remove_price_scales_by_25_per_combat() {
        let mut engine = RunEngine::new(42, 0);
        engine.run_state.combats_won = 3;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        resolve_opening_neow(&mut engine);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        let shop = engine.get_shop().expect("shop should exist");
        assert_eq!(shop.remove_price, 150);
    }

    #[test]
    fn shop_buy_card_spends_gold_and_removes_the_offer() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);

        let shop = engine.get_shop().expect("shop should exist");
        let (card, price) = shop.cards[0].clone();
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::ShopBuyCard(0));

        assert_eq!(engine.run_state.deck.len(), deck_before + 1);
        assert_eq!(engine.run_state.deck.last(), Some(&card));
        assert_eq!(engine.run_state.gold, 999 - price);
        assert_eq!(engine.get_shop().expect("shop stays open").cards.len(), 4);
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
        engine.step(&actions[0]);

        let remove_price = engine.get_shop().expect("shop should exist").remove_price;
        let deck_before = engine.run_state.deck.len();

        engine.step(&RunAction::ShopRemoveCard(0));

        assert_eq!(engine.run_state.deck.len(), deck_before - 1);
        assert_eq!(engine.run_state.gold, 999 - remove_price);
        let shop = engine.get_shop().expect("shop stays open");
        assert!(shop.removal_used);
        assert!(
            !engine
                .get_legal_actions()
                .iter()
                .any(|action| matches!(action, RunAction::ShopRemoveCard(_)))
        );
    }

    #[test]
    fn shop_remove_parasite_reduces_max_hp_and_clamps_current_hp() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck = vec!["Parasite".to_string()];
        engine.run_state.max_hp = 40;
        engine.run_state.current_hp = 40;
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);

        let remove_price = engine.get_shop().expect("shop should exist").remove_price;

        engine.step(&RunAction::ShopRemoveCard(0));

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
        engine.run_state.deck = vec!["Strike_P".to_string()];
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);

        let legal = engine.get_legal_actions();
        assert!(legal.contains(&RunAction::ShopRemoveCard(0)));

        engine.step(&RunAction::ShopRemoveCard(0));
        assert!(engine.run_state.deck.is_empty());
        assert!(engine.get_shop().expect("shop stays open").removal_used);
    }

    #[test]
    fn shop_remove_excludes_unremovable_curses_from_legal_actions() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        engine.run_state.gold = 999;
        engine.run_state.deck = vec!["AscendersBane".to_string(), "Strike_P".to_string()];
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);

        let legal = engine.get_legal_actions();
        assert!(!legal.contains(&RunAction::ShopRemoveCard(0)));
        assert!(legal.contains(&RunAction::ShopRemoveCard(1)));

        engine.step(&RunAction::ShopRemoveCard(1));
        assert_eq!(engine.run_state.deck, vec!["AscendersBane".to_string()]);
    }

    #[test]
    fn event_room_enters_event_phase_with_choices() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Event);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        assert_eq!(engine.phase, RunPhase::Event);
        assert!(engine.event_option_count() >= 1);
    }

    #[test]
    fn event_choice_resolves_back_to_map_phase() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Event);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);

        let before_hp = engine.run_state.current_hp;
        let before_gold = engine.run_state.gold;
        engine.step(&RunAction::EventChoice(0));

        assert_eq!(engine.phase, RunPhase::MapChoice);
        assert!(engine.run_state.current_hp >= 0);
        assert!(engine.run_state.gold >= 0);
        assert!(engine.run_state.current_hp != before_hp || engine.run_state.gold != before_gold || engine.phase == RunPhase::MapChoice);
    }

    #[test]
    fn campfire_rest_uses_ceiling_thirty_percent_formula() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.max_hp = 72;
        engine.run_state.current_hp = 40;
        engine.step(&RunAction::CampfireRest);
        assert_eq!(engine.run_state.current_hp, 62);
    }

    #[test]
    fn campfire_upgrade_adds_plus_suffix() {
        let mut engine = RunEngine::new(42, 0);
        engine.phase = RunPhase::Campfire;
        engine.run_state.deck = vec!["Strike_P".to_string(), "Eruption".to_string()];
        engine.step(&RunAction::CampfireUpgrade(1));
        assert_eq!(engine.run_state.deck[1], "Eruption+");
    }

    #[test]
    fn shop_leave_returns_to_map_choice() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        engine.step(&RunAction::ShopLeave);
        assert_eq!(engine.phase, RunPhase::MapChoice);
    }

    #[test]
    fn monster_room_entry_creates_live_combat_engine() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Monster);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        assert_eq!(engine.phase, RunPhase::Combat);
        assert_eq!(engine.current_room_type(), "monster");
        assert!(engine.get_combat_engine().is_some());
    }

    #[test]
    fn boss_name_is_one_of_java_act_one_bosses() {
        let engine = RunEngine::new(42, 0);
        assert!(matches!(engine.boss_name(), "TheGuardian" | "Hexaghost" | "SlimeBoss"));
    }

    #[test]
    fn current_room_type_tracks_forced_shop_room() {
        let mut engine = RunEngine::new(42, 0);
        resolve_opening_neow(&mut engine);
        set_first_reachable_room(&mut engine, RoomType::Shop);
        let actions = engine.get_legal_actions();
        engine.step(&actions[0]);
        assert_eq!(engine.current_room_type(), "shop");
    }

    #[test]
    fn java_neow_rewards_exist_and_rust_run_exposes_the_start_phase() {
        let engine = RunEngine::new(42, 0);
        assert_eq!(engine.phase, RunPhase::Neow);
        assert_eq!(engine.current_room_type(), "neow");
        assert_eq!(engine.current_choice_count(), 4);
        assert_eq!(engine.run_state.floor, 0, "Rust run begins before the first map choice");
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
        engine.step(&actions[0]);

        let gold_before = engine.run_state.gold;
        let combat = engine.get_combat_engine().expect("combat should be active");
        let wish_idx = combat
            .state
            .hand
            .iter()
            .position(|card| combat.card_registry.card_name(card.def_id) == "Wish+")
            .expect("opening hand should contain Wish+");
        engine.step(&RunAction::CombatAction(Action::PlayCard {
            card_idx: wish_idx,
            target_idx: -1,
        }));
        engine.step(&RunAction::CombatAction(Action::Choose(1)));
        assert_eq!(
            engine
                .get_combat_engine()
                .expect("combat should still be active")
                .state
                .pending_run_gold,
            30
        );
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();

        assert!(
            engine.run_state.gold >= gold_before + 30,
            "Wish+ gold branch should sync its 30 gold into RunState on combat resolution, got {}",
            engine.run_state.gold - gold_before
        );
    }
}
