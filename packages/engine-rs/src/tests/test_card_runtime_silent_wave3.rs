#![cfg(test)]

#[cfg(test)]
mod silent_wave3 {
    use crate::actions::Action;
    use crate::cards::global_registry;
    use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
    use crate::engine::CombatEngine;
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, enemy, enemy_no_intent, ensure_in_hand, force_player_turn, make_deck,
        play_on_enemy, play_self, TEST_SEED,
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

    #[test]
    fn silent_wave3_registry_exports_show_runtime_shape_progress() {
        let reg = global_registry();

        let finisher = reg.get("Finisher").expect("Finisher");
        assert_eq!(finisher.effect_data, &[E::ExtraHits(A::AttacksThisTurn)]);
        assert!(finisher.complex_hook.is_none());

        let finisher_plus = reg.get("Finisher+").expect("Finisher+");
        assert_eq!(finisher_plus.effect_data, &[E::ExtraHits(A::AttacksThisTurn)]);
        assert!(finisher_plus.complex_hook.is_none());

        let malaise = reg.get("Malaise").expect("Malaise");
        assert_eq!(
            malaise.effect_data,
            &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::WEAKENED,
                A::MagicPlusX,
            ))]
        );
        assert!(malaise.complex_hook.is_some());

        let bane = reg.get("Bane").expect("Bane");
        assert!(bane.complex_hook.is_some(), "Bane still needs a hook until conditional hit-count is runtime-native");

        let backstab = reg.get("Backstab").expect("Backstab");
        assert!(backstab.effects.contains(&"innate"));

        let die_die_die = reg.get("Die Die Die").expect("Die Die Die");
        assert_eq!(die_die_die.target, crate::cards::CardTarget::AllEnemy);

        let masterful = reg.get("Masterful Stab").expect("Masterful Stab");
        assert!(masterful.effects.contains(&"cost_increase_on_hp_loss"));
    }

    #[test]
    fn silent_wave3_bane_and_finisher_follow_poison_and_attack_count_engine_rules() {
        let mut bane_engine = engine_for(
            &["Deadly Poison", "Bane"],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        assert!(play_on_enemy(&mut bane_engine, "Deadly Poison", 0));
        let hp_before = bane_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut bane_engine, "Bane", 0));
        assert_eq!(bane_engine.state.enemies[0].entity.hp, hp_before - 14);

        let mut finisher_engine = engine_for(
            &["Backstab", "Strike_G", "Finisher"],
            &[],
            vec![enemy("JawWorm", 60, 60, 1, 0, 1)],
            3,
        );
        assert!(play_on_enemy(&mut finisher_engine, "Backstab", 0));
        assert!(play_on_enemy(&mut finisher_engine, "Strike_G", 0));
        let hp_before = finisher_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut finisher_engine, "Finisher", 0));
        assert_eq!(finisher_engine.state.attacks_played_this_turn, 3);
        assert_eq!(finisher_engine.state.enemies[0].entity.hp, hp_before - 18);
    }

    #[test]
    fn silent_wave3_all_enemy_cards_damage_everyone_and_keep_card_side_effects() {
        let mut die_die_die = engine_for(
            &["Die Die Die"],
            &[],
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            3,
        );
        assert!(play_self(&mut die_die_die, "Die Die Die"));
        assert_eq!(die_die_die.state.enemies[0].entity.hp, 27);
        assert_eq!(die_die_die.state.enemies[1].entity.hp, 22);
        assert!(die_die_die
            .state
            .exhaust_pile
            .iter()
            .any(|card| die_die_die.card_registry.card_name(card.def_id) == "Die Die Die"));

        let mut all_out = engine_for(
            &["All-Out Attack", "Strike_G", "Defend_G"],
            &[],
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            3,
        );
        let before_hand = all_out.state.hand.len();
        assert!(play_self(&mut all_out, "All-Out Attack"));
        assert_eq!(all_out.state.enemies[0].entity.hp, 30);
        assert_eq!(all_out.state.enemies[1].entity.hp, 25);
        assert_eq!(all_out.state.hand.len(), before_hand - 2);
        assert_eq!(all_out.state.discard_pile.len(), 2);
    }

    #[test]
    fn silent_wave3_malaise_scales_from_x_cost_and_upgrade_bonus() {
        let mut malaise = engine_for(
            &["Malaise"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        malaise.state.enemies[0].entity.set_status(sid::STRENGTH, 4);
        assert!(play_on_enemy(&mut malaise, "Malaise", 0));
        assert_eq!(malaise.state.energy, 0);
        assert_eq!(malaise.state.enemies[0].entity.status(sid::WEAKENED), 3);
        assert_eq!(malaise.state.enemies[0].entity.strength(), 1);

        let mut malaise_plus = engine_for(
            &["Malaise+"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        malaise_plus.state.enemies[0].entity.set_status(sid::STRENGTH, 7);
        assert!(play_on_enemy(&mut malaise_plus, "Malaise+", 0));
        assert_eq!(malaise_plus.state.enemies[0].entity.status(sid::WEAKENED), 4);
        assert_eq!(malaise_plus.state.enemies[0].entity.strength(), 3);
    }

    #[test]
    fn silent_wave3_masterful_stab_cost_tracks_damage_taken_in_legal_actions() {
        let mut engine = engine_for(
            &["Masterful Stab"],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            0,
        );
        ensure_in_hand(&mut engine, "Masterful Stab");
        let card_idx = engine
            .state
            .hand
            .iter()
            .position(|card| engine.card_registry.card_name(card.def_id) == "Masterful Stab")
            .expect("Masterful Stab should be in hand");
        assert!(engine.get_legal_actions().iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx: idx, .. } if *idx == card_idx
        )));

        engine.player_lose_hp(2);
        assert!(!engine.get_legal_actions().iter().any(|action| matches!(
            action,
            Action::PlayCard { card_idx: idx, .. } if *idx == card_idx
        )));
    }

    #[test]
    fn silent_wave3_backstab_is_innate_and_exhausts_after_use() {
        let reg = global_registry();
        let backstab = reg.get("Backstab").expect("Backstab");
        assert!(backstab.effects.contains(&"innate"));

        let mut engine = engine_for(
            &["Backstab"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );

        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Backstab", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 11);
        assert!(engine
            .state
            .exhaust_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Backstab"));
    }
}
