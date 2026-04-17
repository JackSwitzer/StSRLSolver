#[cfg(test)]
mod ironclad_wave1_card_runtime_tests {
    use crate::cards::{CardDef, CardTarget, CardType};
    use crate::effects::declarative::{
        AmountSource, ChoiceAction, Effect, Pile, SimpleEffect, Target,
    };
    use crate::engine::CombatEngine;
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, discard_prefix_count, enemy, exhaust_prefix_count, force_player_turn,
        make_deck, play_on_enemy, play_self, TEST_SEED,
    };

    fn card(id: &str) -> CardDef {
        crate::cards::global_registry()
            .get(id)
            .expect("card should exist")
            .clone()
    }

    fn engine_for(
        hand: &[&str],
        draw: &[&str],
        discard: &[&str],
        enemy_hp: i32,
        energy: i32,
    ) -> CombatEngine {
        let mut state = combat_state_with(
            make_deck(draw),
            vec![enemy("JawWorm", enemy_hp, enemy_hp, 1, 0, 1)],
            energy,
        );
        state.hand = make_deck(hand);
        state.discard_pile = make_deck(discard);
        let mut engine = CombatEngine::new(state, TEST_SEED);
        force_player_turn(&mut engine);
        engine.state.turn = 1;
        engine
    }

    #[test]
    fn body_slam_is_declared_through_effect_data_and_hits_for_current_block() {
        let body_slam = card("Body Slam");
        assert_eq!(body_slam.card_type, CardType::Attack);
        assert_eq!(body_slam.target, CardTarget::Enemy);
        assert!(body_slam.test_markers().is_empty());
        assert_eq!(
            body_slam.effect_data,
            &[Effect::Simple(SimpleEffect::DealDamage(
                Target::SelectedEnemy,
                AmountSource::PlayerBlock,
            ))],
        );

        let mut engine = engine_for(&["Body Slam"], &[], &[], 40, 3);
        engine.state.player.block = 13;
        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Body Slam", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 13);
    }

    #[test]
    fn true_grit_base_uses_the_typed_random_exhaust_surface_and_upgrade_uses_declarative_choice_data() {
        let true_grit = card("True Grit");
        assert_eq!(true_grit.card_type, CardType::Skill);
        assert_eq!(
            true_grit.effect_data,
            &[
                Effect::Simple(SimpleEffect::GainBlock(AmountSource::Block)),
                Effect::Simple(SimpleEffect::ExhaustRandomCardFromHand),
            ]
        );
        assert!(true_grit.complex_hook.is_none());

        let true_grit_plus = card("True Grit+");
        assert_eq!(
            true_grit_plus.effect_data,
            &[Effect::ChooseCards {
                source: Pile::Hand,
                filter: crate::effects::declarative::CardFilter::All,
                action: ChoiceAction::Exhaust,
                min_picks: AmountSource::Fixed(1),
                max_picks: AmountSource::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            }],
        );

        let mut engine = engine_for(&["True Grit", "Strike_P"], &[], &[], 40, 3);
        let hp_before = engine.state.player.hp;
        assert!(play_self(&mut engine, "True Grit"));
        assert_eq!(engine.state.player.hp, hp_before);
        assert_eq!(exhaust_prefix_count(&engine, "Strike_"), 1);
        assert_eq!(discard_prefix_count(&engine, "True Grit"), 1);
    }

    #[test]
    fn multi_hit_ironclad_cards_export_declarative_hit_metadata() {
        let pummel = card("Pummel");
        assert_eq!(pummel.effect_data, &[Effect::ExtraHits(AmountSource::Magic)]);

        let twin_strike = card("Twin Strike");
        assert_eq!(twin_strike.effect_data, &[Effect::ExtraHits(AmountSource::Magic)]);
    }

    #[test]
    fn grouped_ironclad_attacks_keep_the_expected_engine_path_behavior() {
        let mut bludgeon = engine_for(&["Bludgeon"], &[], &[], 60, 3);
        let hp_before = bludgeon.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut bludgeon, "Bludgeon", 0));
        assert_eq!(bludgeon.state.enemies[0].entity.hp, hp_before - 32);

        let mut pummel = engine_for(&["Pummel"], &[], &[], 60, 3);
        let hp_before = pummel.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut pummel, "Pummel", 0));
        assert_eq!(pummel.state.enemies[0].entity.hp, hp_before - 8);
        assert_eq!(exhaust_prefix_count(&pummel, "Pummel"), 1);

        let mut twin_strike = engine_for(&["Twin Strike"], &[], &[], 60, 3);
        let hp_before = twin_strike.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut twin_strike, "Twin Strike", 0));
        assert_eq!(twin_strike.state.enemies[0].entity.hp, hp_before - 10);
    }

    #[test]
    fn scaling_and_block_ironclad_cards_keep_the_expected_engine_path_behavior() {
        let mut heavy_blade = engine_for(&["Heavy Blade"], &[], &[], 80, 3);
        heavy_blade.state.player.set_status(sid::STRENGTH, 2);
        let hp_before = heavy_blade.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut heavy_blade, "Heavy Blade", 0));
        assert_eq!(heavy_blade.state.enemies[0].entity.hp, hp_before - 20);

        let mut perfected_strike = engine_for(
            &["Perfected Strike"],
            &["Strike_P", "Strike_P"],
            &["Strike_P"],
            80,
            3,
        );
        let hp_before = perfected_strike.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut perfected_strike, "Perfected Strike", 0));
        assert_eq!(perfected_strike.state.enemies[0].entity.hp, hp_before - 12);

        let mut ghostly_armor = engine_for(&["Ghostly Armor"], &[], &[], 40, 3);
        let block_before = ghostly_armor.state.player.block;
        assert!(play_self(&mut ghostly_armor, "Ghostly Armor"));
        assert!(ghostly_armor.state.player.block >= block_before + 10);
        assert_eq!(discard_prefix_count(&ghostly_armor, "Ghostly Armor"), 1);
    }
}
