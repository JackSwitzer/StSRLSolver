#[cfg(test)]
mod ironclad_wave3_card_runtime_tests {
    use crate::actions::Action;
    use crate::cards::{CardDef, CardTarget, CardType};
    use crate::effects::declarative::{
        AmountSource as A, CardFilter, ChoiceAction, Effect, Pile, SimpleEffect as SE,
        Target as T,
    };
    use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
    use crate::status_ids::sid;
    use crate::tests::support::{
        discard_prefix_count, draw_prefix_count, engine_without_start, enemy_no_intent,
        exhaust_prefix_count, force_player_turn, make_deck, play_on_enemy, play_self,
    };

    fn card(id: &str) -> CardDef {
        crate::cards::global_registry()
            .get(id)
            .expect("card should exist")
            .clone()
    }

    fn total_enemy_hp(engine: &crate::engine::CombatEngine) -> i32 {
        engine
            .state
            .enemies
            .iter()
            .map(|enemy| enemy.entity.hp.max(0))
            .sum()
    }

    #[test]
    fn ironclad_wave3_registry_exports_show_declarative_progress() {
        let body_slam = card("Body Slam");
        assert_eq!(body_slam.card_type, CardType::Attack);
        assert_eq!(body_slam.target, CardTarget::Enemy);
        assert_eq!(
            body_slam.effect_data,
            &[Effect::Simple(SE::DealDamage(T::SelectedEnemy, A::PlayerBlock))]
        );

        let pummel = card("Pummel");
        assert_eq!(pummel.effect_data, &[Effect::ExtraHits(A::Magic)]);

        let twin_strike = card("Twin Strike");
        assert_eq!(twin_strike.effect_data, &[Effect::ExtraHits(A::Magic)]);

        let battle_trance = card("Battle Trance");
        assert_eq!(
            battle_trance.effect_data,
            &[
                Effect::Simple(SE::DrawCards(A::Magic)),
                Effect::Simple(SE::AddStatus(
                    crate::effects::declarative::Target::Player,
                    sid::NO_DRAW,
                    A::Fixed(1),
                )),
            ]
        );

        let whirlwind = card("Whirlwind");
        assert_eq!(whirlwind.card_type, CardType::Attack);
        assert_eq!(whirlwind.target, CardTarget::AllEnemy);
        assert_eq!(whirlwind.effect_data, &[Effect::ExtraHits(A::XCost)]);
        assert!(whirlwind.uses_declared_x_cost());
        assert_eq!(whirlwind.declared_x_cost_amounts(), vec![A::XCost]);
        assert_eq!(whirlwind.declared_all_enemy_damage(), Some(A::Damage));

        let true_grit = card("True Grit");
        assert_eq!(
            true_grit.effect_data,
            &[
                Effect::Simple(SE::GainBlock(A::Block)),
                Effect::Simple(SE::ExhaustRandomCardFromHand),
            ]
        );
        assert!(true_grit.complex_hook.is_none());

        let true_grit_plus = card("True Grit+");
        assert_eq!(
            true_grit_plus.effect_data,
            &[
                Effect::Simple(SE::GainBlock(A::Block)),
                Effect::ChooseCards {
                    source: Pile::Hand,
                    filter: CardFilter::All,
                    action: ChoiceAction::Exhaust,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: A::Fixed(0),
                },
            ]
        );

        let clash = card("Clash");
        assert_eq!(
            clash.effect_data,
            &[Effect::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
        );
        assert!(
            clash
                .runtime_triggers()
                .contains(&crate::effects::types::CardRuntimeTrigger::CanPlay(
                    crate::effects::types::CanPlayRule::OnlyAttacksInHand,
                ))
        );
    }

    #[test]
    fn body_slam_scales_from_current_block_on_the_engine_path() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Body Slam"]);
        engine.state.player.block = 13;
        let hp_before = engine.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut engine, "Body Slam", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 13);
    }

    #[test]
    fn pummel_and_twin_strike_keep_the_expected_multi_hit_engine_behavior() {
        let mut pummel = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 60, 60)],
            3,
        );
        force_player_turn(&mut pummel);
        pummel.state.hand = make_deck(&["Pummel"]);
        let hp_before = pummel.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut pummel, "Pummel", 0));

        assert_eq!(pummel.state.enemies[0].entity.hp, hp_before - 8);
        assert_eq!(exhaust_prefix_count(&pummel, "Pummel"), 1);

        let mut twin_strike = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 60, 60)],
            3,
        );
        force_player_turn(&mut twin_strike);
        twin_strike.state.hand = make_deck(&["Twin Strike"]);
        let hp_before = twin_strike.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut twin_strike, "Twin Strike", 0));

        assert_eq!(twin_strike.state.enemies[0].entity.hp, hp_before - 10);
    }

    #[test]
    fn pummel_source_uses_four_or_five_separate_two_damage_hits_then_exhausts() {
        // Pummel.java queues magicNumber damage actions: four at base and five
        // after upgradeMagicNumber(1). DamageInfo snapshots the Strength-adjusted
        // damage for each hit, so three Block absorbs exactly the first 3-damage hit.
        for (card_id, hits) in [("Pummel", 4), ("Pummel+", 5)] {
            let mut engine = engine_without_start(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 60, 60)],
                3,
            );
            force_player_turn(&mut engine);
            engine.state.player.set_status(sid::STRENGTH, 1);
            engine.state.enemies[0].entity.block = 3;
            engine.state.hand = make_deck(&[card_id]);

            assert!(play_on_enemy(&mut engine, card_id, 0));

            assert_eq!(engine.state.energy, 2, "{card_id}");
            assert_eq!(engine.state.enemies[0].entity.block, 0, "{card_id}");
            assert_eq!(
                engine.state.enemies[0].entity.hp,
                60 - (hits - 1) * 3,
                "{card_id}"
            );
            assert_eq!(exhaust_prefix_count(&engine, "Pummel"), 1, "{card_id}");
        }
    }

    #[test]
    fn battle_trance_draws_then_blocks_future_draws_on_the_engine_path() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Battle Trance"]);
        engine.state.draw_pile = make_deck(&["Strike", "Defend"]);

        assert!(play_self(&mut engine, "Battle Trance"));
        assert_eq!(engine.state.player.status(sid::NO_DRAW), 1);
        assert_eq!(engine.state.hand.len(), 2);
        assert_eq!(draw_prefix_count(&engine, "Strike_"), 0);

        let hand_before = engine.state.hand.len();
        engine.draw_cards(2);
        assert_eq!(engine.state.hand.len(), hand_before);
    }

    #[test]
    fn true_grit_base_uses_the_typed_random_exhaust_surface_while_plus_uses_declared_exhaust_choice_data() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["True Grit", "Strike", "Defend"]);

        assert!(play_self(&mut engine, "True Grit"));
        assert_eq!(engine.state.hand.len(), 1);
        assert_eq!(engine.state.exhaust_pile.len(), 1);
        assert_eq!(discard_prefix_count(&engine, "True Grit"), 1);
        assert_eq!(
            exhaust_prefix_count(&engine, "Strike") + exhaust_prefix_count(&engine, "Defend"),
            1,
        );

        let true_grit_plus = card("True Grit+");
        assert_eq!(
            true_grit_plus.effect_data,
            &[
                Effect::Simple(SE::GainBlock(A::Block)),
                Effect::ChooseCards {
                    source: Pile::Hand,
                    filter: CardFilter::All,
                    action: ChoiceAction::Exhaust,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: A::Fixed(0),
                },
            ]
        );

        let mut plus_engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut plus_engine);
        plus_engine.state.hand = make_deck(&["True Grit+", "Strike", "Defend"]);

        assert!(play_self(&mut plus_engine, "True Grit+"));
        assert_eq!(plus_engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(
            plus_engine.choice.as_ref().map(|choice| choice.reason.clone()),
            Some(ChoiceReason::ExhaustFromHand),
        );

        let selected_name = match plus_engine.choice.as_ref().unwrap().options[0] {
            ChoiceOption::HandCard(idx) => plus_engine
                .card_registry
                .card_name(plus_engine.state.hand[idx].def_id)
                .to_string(),
            _ => panic!("True Grit+ should exhaust from hand"),
        };

        plus_engine.execute_action(&Action::Choose(0));

        assert_eq!(plus_engine.state.exhaust_pile.len(), 1);
        assert_eq!(
            plus_engine
                .state
                .exhaust_pile
                .iter()
                .map(|card| plus_engine.card_registry.card_name(card.def_id))
                .collect::<Vec<_>>(),
            vec![selected_name.as_str()],
        );
        assert_ne!(plus_engine.phase, CombatPhase::AwaitingChoice);
    }

    #[test]
    fn true_grit_variants_gain_block_before_java_exhaust_paths_and_rng() {
        // TrueGrit.java queues GainBlockAction first. Base then uses random
        // ExhaustAction(1), which consumes cardRandomRng when multiple cards
        // remain but auto-exhausts a singleton without RNG. The upgrade gains
        // 9 Block and uses a non-random one-card selection instead.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
        let mut base = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        force_player_turn(&mut base);
        base.state.hand = make_deck(&["True Grit", "Strike", "Defend"]);
        let mut oracle = base.card_random_rng.clone();
        let expected = ["Strike", "Defend"][oracle.random_int(1) as usize];
        let generic_before = base.shuffle_rng.counter;

        assert!(play_self(&mut base, "True Grit"));

        assert_eq!(base.state.energy, 0);
        assert_eq!(base.state.player.block, 7);
        assert_eq!(
            base.card_registry
                .card_name(base.state.exhaust_pile.last().expect("randomly exhausted card").def_id),
            expected
        );
        assert_eq!(base.card_random_rng.counter, oracle.counter);
        assert_eq!(base.shuffle_rng.counter, generic_before);

        let mut singleton = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        force_player_turn(&mut singleton);
        singleton.state.hand = make_deck(&["True Grit", "Strike"]);
        let card_random_before = singleton.card_random_rng.counter;

        assert!(play_self(&mut singleton, "True Grit"));
        assert_eq!(singleton.state.player.block, 7);
        assert_eq!(exhaust_prefix_count(&singleton, "Strike"), 1);
        assert_eq!(singleton.card_random_rng.counter, card_random_before);

        let mut upgraded = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        force_player_turn(&mut upgraded);
        upgraded.state.hand = make_deck(&["True Grit+", "Strike", "Defend"]);
        let card_random_before = upgraded.card_random_rng.counter;

        assert!(play_self(&mut upgraded, "True Grit+"));
        assert_eq!(upgraded.state.energy, 0);
        assert_eq!(upgraded.state.player.block, 9);
        assert_eq!(upgraded.phase, CombatPhase::AwaitingChoice);
        upgraded.execute_action(&Action::Choose(1));
        assert_eq!(upgraded.phase, CombatPhase::PlayerTurn);
        assert_eq!(exhaust_prefix_count(&upgraded, "Defend"), 1);
        assert_eq!(upgraded.card_random_rng.counter, card_random_before);
    }

    #[test]
    fn whirlwind_variants_use_exact_energy_chemical_x_and_separate_aoe_hits() {
        // WhirlwindAction does nothing at zero effective Energy, adds two
        // iterations for Chemical X, and queues a distinct
        // DamageAllEnemiesAction per iteration. The upgrade changes only each
        // hit from five to eight.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Whirlwind.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/WhirlwindAction.java
        let mut zero = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("Cultist", 50, 50),
            ],
            0,
        );
        force_player_turn(&mut zero);
        zero.state.hand = make_deck(&["Whirlwind"]);
        assert!(play_on_enemy(&mut zero, "Whirlwind", 0));
        assert_eq!(total_enemy_hp(&zero), 100);
        assert_eq!(zero.state.energy, 0);

        let mut chemical_x = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("Cultist", 50, 50),
            ],
            0,
        );
        force_player_turn(&mut chemical_x);
        chemical_x.state.relics.push("Chemical X".to_string());
        chemical_x.state.hand = make_deck(&["Whirlwind"]);
        assert!(play_on_enemy(&mut chemical_x, "Whirlwind", 0));
        assert_eq!(chemical_x.state.enemies[0].entity.hp, 40);
        assert_eq!(chemical_x.state.enemies[1].entity.hp, 40);
        assert_eq!(chemical_x.state.energy, 0);

        let mut flying = enemy_no_intent("Byrd", 50, 50);
        flying.entity.set_status(sid::FLIGHT, 2);
        let mut upgraded = engine_without_start(
            Vec::new(),
            vec![flying, enemy_no_intent("Cultist", 50, 50)],
            2,
        );
        force_player_turn(&mut upgraded);
        upgraded.state.hand = make_deck(&["Whirlwind+"]);
        assert!(play_on_enemy(&mut upgraded, "Whirlwind+", 0));

        assert_eq!(upgraded.state.energy, 0);
        assert_eq!(upgraded.state.enemies[0].entity.hp, 42);
        assert_eq!(upgraded.state.enemies[0].entity.status(sid::FLIGHT), 0);
        assert_eq!(upgraded.state.enemies[1].entity.hp, 34);
    }

    #[test]
    fn clash_requires_an_attack_only_hand_to_be_legal() {
        let mut blocked = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut blocked);
        blocked.state.hand = make_deck(&["Clash", "Defend"]);
        let clash_idx = blocked
            .state
            .hand
            .iter()
            .position(|card| blocked.card_registry.card_name(card.def_id) == "Clash")
            .expect("Clash should be in hand");

        assert!(!blocked.get_legal_actions().iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx, .. } if *card_idx == clash_idx
        )));

        let mut allowed = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut allowed);
        allowed.state.hand = make_deck(&["Clash", "Strike"]);
        let clash_idx = allowed
            .state
            .hand
            .iter()
            .position(|card| allowed.card_registry.card_name(card.def_id) == "Clash")
            .expect("Clash should be in hand");

        assert!(allowed.get_legal_actions().iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx, .. } if *card_idx == clash_idx
        )));
    }
}
