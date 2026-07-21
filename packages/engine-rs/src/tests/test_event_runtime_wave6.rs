#[cfg(test)]
mod event_runtime_wave6_tests {
    use crate::decision::{RewardItemKind, RewardScreenSource};
    use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
    use crate::run::{GameAction, RunEngine, RunPhase, ShopState};
    use crate::status_ids::sid;

    fn typed_event(act: i32, name: &str) -> TypedEventDef {
        typed_events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
    }

    fn enter_mushrooms_combat(engine: &mut RunEngine, mushrooms: TypedEventDef) {
        engine.debug_set_typed_event_state(mushrooms);
        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::Event);
        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::Combat);
    }

    #[test]
    fn mushrooms_stomp_branch_enters_scripted_combat_and_continues_to_event_relic_reward() {
        let mut engine = RunEngine::new(61, 20);
        let gold_before = engine.run_state.gold;
        let mushrooms = typed_event(1, "Mushrooms");
        assert!(matches!(
            mushrooms.options[0].status,
            EventRuntimeStatus::Supported
        ));
        enter_mushrooms_combat(&mut engine, mushrooms);
        let combat = engine.get_combat_engine().expect("event combat");
        assert_eq!(combat.state.enemies.len(), 3);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "FungiBeast"));

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.current_phase(), RunPhase::CardReward);
        assert_eq!(engine.run_state.gold, gold_before);

        let screen = engine.current_reward_screen().expect("event reward screen");
        assert_eq!(screen.source, RewardScreenSource::Event);
        assert_eq!(screen.items.len(), 2);
        assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
        assert!((20..=30).contains(&screen.items[0].label.parse::<i32>().unwrap()));
        assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
        assert_eq!(screen.items[1].label, "Odd Mushroom");

        assert!(engine.step_game(&GameAction::SelectRewardItem(0)).accepted());
        assert!((gold_before + 20..=gold_before + 30).contains(&engine.run_state.gold));
        assert!(engine.step_game(&GameAction::SelectRewardItem(1)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert!(engine.run_state.relics.iter().any(|relic| relic == "Odd Mushroom"));
    }

    #[test]
    fn odd_mushroom_from_event_reduces_vulnerable_damage_and_duplicates_to_circlet() {
        // OddMushroom.java defines vulnerability effectiveness as 1.25.
        // VulnerablePower.java applies that multiplier to NORMAL damage against
        // the player, and Mushrooms.java substitutes Circlet on duplicate reward.
        let mushrooms = typed_event(1, "Mushrooms");
        let mut engine = RunEngine::new(71, 20);
        enter_mushrooms_combat(&mut engine, mushrooms.clone());
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(1))
            .accepted());

        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.state.player.set_status(sid::VULNERABLE, 1);
        let hp_before = combat.state.player.hp;
        assert_eq!(combat.state.enemies[0].move_damage(), 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, hp_before - 15);

        let mut duplicate = RunEngine::new(73, 20);
        duplicate.run_state.relics.push("Odd Mushroom".to_string());
        enter_mushrooms_combat(&mut duplicate, mushrooms);
        duplicate.debug_force_current_combat_outcome(true);
        duplicate.debug_resolve_current_combat_outcome();
        let screen = duplicate.current_reward_screen().expect("duplicate reward");
        assert_eq!(screen.items[1].label, "Circlet");
    }

    #[test]
    fn masked_bandits_fight_rolls_reward_before_combat_and_grants_it_after_victory() {
        // MaskedBandits.buttonEffect rolls miscRng 25..35, queues Red Mask (or
        // Circlet when owned), then enters the three-bandit combat.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/city/MaskedBandits.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java
        let mut engine = RunEngine::new(79, 0);
        let gold_before = engine.run_state.gold;
        let misc_before = engine.rng_counters()["misc"];
        engine.debug_set_typed_event_state(typed_event(2, "Masked Bandits"));

        assert!(engine.step_game(&GameAction::EventChoice(1)).accepted());
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        assert_eq!(engine.rng_counters()["misc"], misc_before + 1);
        assert_eq!(
            engine
                .get_combat_engine()
                .expect("bandit combat")
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.id.as_str())
                .collect::<Vec<_>>(),
            ["BanditChild", "BanditLeader", "BanditBear"],
        );

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.run_state.gold, gold_before);
        let screen = engine.current_reward_screen().expect("bandit reward screen");
        assert_eq!(screen.source, RewardScreenSource::Event);
        assert_eq!(screen.items.len(), 2);
        assert_eq!(screen.items[0].kind, RewardItemKind::Gold);
        assert!((25..=35).contains(&screen.items[0].label.parse::<i32>().unwrap()));
        assert_eq!(screen.items[1].kind, RewardItemKind::Relic);
        assert_eq!(screen.items[1].label, "Red Mask");
        assert!(engine.step_game(&GameAction::SelectRewardItem(0)).accepted());
        assert!((gold_before + 25..=gold_before + 35).contains(&engine.run_state.gold));

        let mut duplicate = RunEngine::new(81, 0);
        duplicate.run_state.relics.push("Red Mask".to_string());
        duplicate.debug_set_typed_event_state(typed_event(2, "Masked Bandits"));
        assert!(duplicate.step_game(&GameAction::EventChoice(1)).accepted());
        duplicate.debug_force_current_combat_outcome(true);
        duplicate.debug_resolve_current_combat_outcome();
        assert_eq!(
            duplicate.current_reward_screen().unwrap().items[1].label,
            "Circlet",
        );
    }

    #[test]
    fn masked_bandits_pay_advances_ambient_math_once_per_gold_before_courier_refill() {
        // stealGold creates one GainPennyEffect per current gold and selects
        // each source through MonsterGroup.getRandomMonster(), whose no-RNG
        // overload consumes MathUtils.random(0, 2). Shop card identity later
        // uses that same process-global stream.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/city/MaskedBandits.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java
        let mut observed_different_refill = false;

        for ambient_seed in 0..64 {
            let mut paid = RunEngine::new_with_ambient_seed(83, 0, ambient_seed);
            paid.run_state.gold = 3;
            let before = paid.ambient_math_rng_state();
            let mut oracle = crate::seed::AmbientMathRng::from_state(before.0, before.1);
            for _ in 0..3 {
                let _ = oracle.random_int(2);
            }
            paid.debug_set_typed_event_state(typed_event(2, "Masked Bandits"));
            assert!(paid.step_game(&GameAction::EventChoice(0)).accepted());
            assert_eq!(paid.run_state.gold, 0);
            assert_eq!(paid.ambient_math_rng_state(), oracle.state_tuple());

            let mut omitted = RunEngine::new_with_ambient_seed(83, 0, ambient_seed);
            omitted.run_state.gold = 0;
            for engine in [&mut paid, &mut omitted] {
                engine.run_state.gold = 10_000;
                engine.run_state.relics.push("The Courier".to_string());
                engine
                    .run_state
                    .relic_flags
                    .rebuild(&engine.run_state.relics);
                engine.debug_set_shop_state(ShopState {
                    cards: vec![("Strike_P".to_string(), 1)],
                    relics: Vec::new(),
                    potions: Vec::new(),
                    remove_price: 75,
                    removal_used: false,
                });
                assert!(engine.step_game(&GameAction::ShopBuyCard(0)).accepted());
            }

            let paid_card = &paid.get_shop().expect("paid shop").cards[0].0;
            let omitted_card = &omitted.get_shop().expect("control shop").cards[0].0;
            if paid_card != omitted_card {
                observed_different_refill = true;
                break;
            }
        }

        assert!(
            observed_different_refill,
            "omitting Masked Bandits ambient draws must alter a reachable Courier refill"
        );
    }

    #[test]
    fn mysterious_sphere_open_branch_enters_scripted_combat_and_keeps_event_owned_reward_flow() {
        let mut engine = RunEngine::new(67, 20);
        let sphere = typed_event(3, "Mysterious Sphere");
        assert!(matches!(
            sphere.options[0].status,
            EventRuntimeStatus::Supported
        ));
        engine.debug_set_typed_event_state(sphere);

        let start = engine.step_game(&GameAction::EventChoice(0));
        assert!(start.accepted());
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        let combat = engine.get_combat_engine().expect("event combat");
        assert_eq!(combat.state.enemies.len(), 2);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "Orb Walker"));

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.current_phase(), RunPhase::CardReward);

        let screen = engine.current_reward_screen().expect("event reward screen");
        assert_eq!(screen.source, RewardScreenSource::Event);
        assert_eq!(screen.items.len(), 1);
        assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
        assert!(screen.items[0].claimable);
    }
}
