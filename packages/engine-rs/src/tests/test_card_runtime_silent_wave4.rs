#![cfg(test)]

#[cfg(test)]
mod silent_wave4 {
    use crate::actions::Action;
    use crate::cards::{global_registry, CardTarget, CardType};
    use crate::effects::declarative::{
        AmountSource as A, BulkAction, CardFilter, ChoiceAction, Effect as E, Pile as P,
        SimpleEffect as SE, Target as T,
    };
    use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, enemy, enemy_no_intent, force_player_turn, make_deck, play_on_enemy,
        play_self, TEST_SEED,
    };

    fn engine_for(
        hand: &[&str],
        draw: &[&str],
        enemies: Vec<crate::state::EnemyCombatState>,
        energy: i32,
    ) -> CombatEngine {
        let mut state = combat_state_with(make_deck(draw), enemies, energy);
        state.hand = make_deck(hand);
        let mut engine = CombatEngine::new(state, TEST_SEED);
        force_player_turn(&mut engine);
        engine.state.turn = 1;
        engine
    }

    fn hand_names(engine: &CombatEngine) -> Vec<String> {
        engine
            .state
            .hand
            .iter()
            .map(|card| engine.card_registry.card_name(card.def_id).to_string())
            .collect()
    }

    #[test]
    fn silent_wave4_registry_exports_show_runtime_progress_honestly() {
        let reg = global_registry();

        let bouncing_flask = reg.get("Bouncing Flask").expect("Bouncing Flask");
        assert_eq!(bouncing_flask.card_type, CardType::Skill);
        assert_eq!(bouncing_flask.target, CardTarget::AllEnemy);
        assert_eq!(
            bouncing_flask.effect_data,
            &[
                E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
                E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
                E::Simple(SE::AddStatus(T::RandomEnemy, sid::POISON, A::Fixed(3))),
            ]
        );
        assert!(bouncing_flask.complex_hook.is_none());

        let bouncing_flask_plus = reg.get("Bouncing Flask+").expect("Bouncing Flask+");
        assert_eq!(bouncing_flask_plus.effect_data.len(), 4);
        assert!(bouncing_flask_plus.complex_hook.is_none());

        let calculated_gamble = reg.get("Calculated Gamble").expect("Calculated Gamble");
        assert_eq!(
            calculated_gamble.effect_data,
            &[
                E::ForEachInPile {
                    pile: P::Hand,
                    filter: CardFilter::All,
                    action: BulkAction::Discard,
                },
                E::Simple(SE::DrawCards(A::HandSizeAtPlay)),
            ]
        );
        assert!(calculated_gamble.complex_hook.is_none());

    let concentrate = reg.get("Concentrate").expect("Concentrate");
    assert_eq!(
        concentrate.effect_data,
        &[
            E::ChooseCards {
                source: P::Hand,
                filter: CardFilter::All,
                action: ChoiceAction::DiscardForEffect,
                min_picks: A::Magic,
                max_picks: A::Magic,
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            }
        ]
    );
    assert!(concentrate.complex_hook.is_none());

        let dash = reg.get("Dash").expect("Dash");
        assert_eq!(dash.card_type, CardType::Attack);
        assert_eq!(dash.target, CardTarget::Enemy);
        assert_eq!(
            dash.effect_data,
            &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ]
        );
        assert!(dash.complex_hook.is_none());

        let defend = reg.get("Defend_G").expect("Defend_G");
        assert_eq!(defend.card_type, CardType::Skill);
        assert_eq!(defend.target, CardTarget::SelfTarget);

        let deflect = reg.get("Deflect").expect("Deflect");
        assert_eq!(deflect.cost, 0);
        assert_eq!(deflect.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

        let escape_plan = reg.get("Escape Plan").expect("Escape Plan");
        assert_eq!(
            escape_plan.effect_data,
            &[
                E::Simple(SE::DrawCards(A::Fixed(1))),
                E::Simple(SE::GainBlockIfLastHandCardType(CardType::Skill, A::Block)),
            ]
        );
        assert!(escape_plan.complex_hook.is_none());
    }

    #[test]
    fn silent_wave4_bouncing_flask_applies_the_expected_total_poison() {
        let mut engine = engine_for(
            &["Bouncing Flask"],
            &[],
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            3,
        );

        assert!(play_self(&mut engine, "Bouncing Flask"));

        let total_poison: i32 = engine
            .state
            .enemies
            .iter()
            .map(|enemy| enemy.entity.status(sid::POISON))
            .sum();
        assert_eq!(total_poison, 9);
    }

    #[test]
    fn silent_wave4_calculated_gamble_discards_the_hand_then_draws_that_many() {
        let mut engine = engine_for(
            &["Calculated Gamble", "Strike_G", "Defend_G"],
            &["Neutralize", "Survivor", "Deflect"],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );

        assert!(play_self(&mut engine, "Calculated Gamble"));

        let names = hand_names(&engine);
        assert_eq!(engine.state.hand.len(), 2);
        assert!(names.iter().any(|name| name == "Deflect"));
        assert!(names.iter().any(|name| name == "Survivor"));
        assert!(!names.iter().any(|name| name == "Strike"));
        assert!(!names.iter().any(|name| name == "Defend"));
        assert!(engine
            .state
            .discard_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Strike_G"));
        assert!(engine
            .state
            .discard_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Defend_G"));
        assert!(engine
            .state
            .exhaust_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Calculated Gamble"));
    }

    #[test]
    fn silent_wave4_concentrate_uses_the_canonical_discard_choice_surface() {
        let mut engine = engine_for(
            &["Concentrate", "Defend_G", "Deflect", "Neutralize"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            1,
        );

        assert!(play_self(&mut engine, "Concentrate"));
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(
            engine.choice.as_ref().map(|choice| choice.reason.clone()),
            Some(ChoiceReason::DiscardForEffect),
        );
        assert_eq!(engine.choice.as_ref().unwrap().min_picks, 3);
        assert_eq!(engine.choice.as_ref().unwrap().max_picks, 3);

        engine.execute_action(&Action::Choose(0));
        engine.execute_action(&Action::Choose(1));
        engine.execute_action(&Action::Choose(2));
        engine.execute_action(&Action::ConfirmSelection);

        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
        assert_eq!(engine.state.energy, 3);
        assert_eq!(engine.state.hand.len(), 0);
        assert_eq!(engine.state.discard_pile.len(), 3);
    }

    #[test]
    fn silent_wave4_dash_combines_damage_and_block_on_the_engine_path() {
        let mut engine = engine_for(
            &["Dash"],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        let hp_before = engine.state.enemies[0].entity.hp;
        let block_before = engine.state.player.block;

        assert!(play_on_enemy(&mut engine, "Dash", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 10);
        assert!(engine.state.player.block >= block_before + 10);
    }

    #[test]
    fn silent_wave4_defend_and_deflect_grant_their_expected_block_values() {
        let mut defend = engine_for(
            &["Defend_G"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut defend, "Defend_G"));
        assert!(defend.state.player.block >= 5);

        let mut deflect = engine_for(
            &["Deflect"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut deflect, "Deflect"));
        assert!(deflect.state.player.block >= 4);
        assert_eq!(deflect.state.energy, 3);
    }

    #[test]
    fn silent_wave4_escape_plan_only_grants_block_when_the_drawn_card_is_a_skill() {
        let mut skill_draw = engine_for(
            &["Escape Plan"],
            &["Defend_G"],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut skill_draw, "Escape Plan"));
        assert!(skill_draw.state.player.block >= 3);
        assert_eq!(skill_draw.state.hand.len(), 1);
        assert_eq!(skill_draw.state.draw_pile.len(), 0);

        let mut attack_draw = engine_for(
            &["Escape Plan"],
            &["Strike_G"],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut attack_draw, "Escape Plan"));
        assert_eq!(attack_draw.state.player.block, 0);
    }
}
