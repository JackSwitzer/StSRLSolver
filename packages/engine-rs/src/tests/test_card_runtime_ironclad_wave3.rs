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
                Effect::Simple(SE::SetFlag(crate::effects::declarative::BoolFlag::NoDraw)),
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
            &[Effect::ChooseCards {
                source: Pile::Hand,
                filter: CardFilter::All,
                action: ChoiceAction::Exhaust,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
            }]
        );

        let clash = card("Clash");
        assert!(clash.effect_data.is_empty());
        assert!(clash.effects.contains(&"only_attacks_in_hand"));
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
    fn battle_trance_draws_then_blocks_future_draws_on_the_engine_path() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Battle Trance"]);
        engine.state.draw_pile = make_deck(&["Strike_R", "Defend_R"]);

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
        engine.state.hand = make_deck(&["True Grit", "Strike_R", "Defend_R"]);

        assert!(play_self(&mut engine, "True Grit"));
        assert_eq!(engine.state.hand.len(), 1);
        assert_eq!(engine.state.exhaust_pile.len(), 1);
        assert_eq!(discard_prefix_count(&engine, "True Grit"), 1);
        assert_eq!(
            exhaust_prefix_count(&engine, "Strike_") + exhaust_prefix_count(&engine, "Defend_"),
            1,
        );

        let true_grit_plus = card("True Grit+");
        assert_eq!(
            true_grit_plus.effect_data,
            &[Effect::ChooseCards {
                source: Pile::Hand,
                filter: CardFilter::All,
                action: ChoiceAction::Exhaust,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
            }]
        );

        let mut plus_engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut plus_engine);
        plus_engine.state.hand = make_deck(&["True Grit+", "Strike_R", "Defend_R"]);

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
    fn whirlwind_is_x_cost_aoe_and_consumes_all_energy() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 50, 50),
                enemy_no_intent("Cultist", 50, 50),
            ],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Whirlwind"]);

        let hp_before = total_enemy_hp(&engine);

        assert!(play_on_enemy(&mut engine, "Whirlwind", 0));

        assert_eq!(engine.state.energy, 0);
        assert_eq!(hp_before - total_enemy_hp(&engine), 30);
    }

    #[test]
    fn clash_requires_an_attack_only_hand_to_be_legal() {
        let mut blocked = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );
        force_player_turn(&mut blocked);
        blocked.state.hand = make_deck(&["Clash", "Defend_R"]);
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
        allowed.state.hand = make_deck(&["Clash", "Strike_R"]);
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
