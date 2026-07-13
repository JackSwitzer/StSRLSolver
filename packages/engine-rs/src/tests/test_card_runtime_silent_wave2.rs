#![cfg(test)]

#[cfg(test)]
mod silent_wave2 {
    use crate::actions::Action;
    use crate::cards::{global_registry, CardType};
    use crate::effects::declarative::{AmountSource as A, Effect, SimpleEffect as SE, Target as T};
    use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, discard_prefix_count, enemy, enemy_no_intent, force_player_turn,
        hand_count, make_deck, play_on_enemy, play_self, TEST_SEED,
    };

    fn engine_for(
        hand: &[&str],
        draw: &[&str],
        discard: &[&str],
        enemies: Vec<crate::state::EnemyCombatState>,
        energy: i32,
    ) -> CombatEngine {
        let mut state = combat_state_with(make_deck(draw), enemies, energy);
        state.hand = make_deck(hand);
        state.discard_pile = make_deck(discard);
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
    fn silent_wave2_cards_export_declarative_runtime_data() {
        let reg = global_registry();

        let backflip = reg.get("Backflip").expect("Backflip should exist");
        assert_eq!(
            backflip.effect_data,
            &[Effect::Simple(crate::effects::declarative::SimpleEffect::DrawCards(
                A::Magic,
            ))]
        );

        let blade_dance = reg.get("Blade Dance").expect("Blade Dance should exist");
        assert_eq!(blade_dance.card_type, CardType::Skill);
        assert_eq!(blade_dance.base_magic, 3);

        let dagger_spray = reg.get("Dagger Spray").expect("Dagger Spray should exist");
        assert_eq!(
            dagger_spray.effect_data,
            &[
                Effect::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                Effect::ExtraHits(A::Magic),
            ]
        );

        let deadly_poison = reg.get("Deadly Poison").expect("Deadly Poison should exist");
        assert_eq!(deadly_poison.base_magic, 5);

        let flying_knee = reg.get("Flying Knee").expect("Flying Knee should exist");
        assert_eq!(flying_knee.base_magic, -1);
        assert_eq!(
            flying_knee.effect_data,
            &[Effect::Simple(SE::AddStatus(
                T::Player,
                sid::ENERGIZED,
                A::Fixed(1),
            ))]
        );

        let quick_slash = reg.get("Quick Slash").expect("Quick Slash should exist");
        assert_eq!(quick_slash.effect_data.len(), 1);

        let sneaky_strike = reg.get("Sneaky Strike").expect("Sneaky Strike should exist");
        assert_eq!(sneaky_strike.effect_data.len(), 1);
    }

    #[test]
    fn silent_wave2_draw_and_shiv_cards_use_the_engine_path() {
        let mut backflip = engine_for(
            &["Backflip"],
            &["Strike", "Defend", "Neutralize"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        let backflip_hand_before = backflip.state.hand.len();
        assert!(play_self(&mut backflip, "Backflip"));
        assert_eq!(backflip.state.player.block, 5);
        assert_eq!(backflip.state.hand.len(), backflip_hand_before + 1);

        let mut quick_slash = engine_for(
            &["Quick Slash"],
            &["Strike"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        let hp_before = quick_slash.state.enemies[0].entity.hp;
        let hand_before = quick_slash.state.hand.len();
        assert!(play_on_enemy(&mut quick_slash, "Quick Slash", 0));
        assert_eq!(quick_slash.state.enemies[0].entity.hp, hp_before - 8);
        assert_eq!(quick_slash.state.hand.len(), hand_before);
        assert!(hand_names(&quick_slash).contains(&"Strike".to_string()));

        let mut blade_dance = engine_for(
            &["Blade Dance"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut blade_dance, "Blade Dance"));
        assert_eq!(hand_count(&blade_dance, "Shiv"), 3);
    }

    #[test]
    fn quick_slash_source_resolves_damage_before_its_one_card_draw() {
        // QuickSlash.java queues DamageAction before DrawCardAction(1), and
        // upgradeDamage(4) changes eight damage to twelve. A final lethal hit
        // clears post-combat actions, while a kill with another enemy alive
        // still permits the queued draw.
        let mut nonterminal = engine_for(
            &["Quick Slash+"],
            &["Strike"],
            &[],
            vec![
                enemy_no_intent("JawWorm", 12, 12),
                enemy_no_intent("Cultist", 40, 40),
            ],
            3,
        );
        assert!(play_on_enemy(&mut nonterminal, "Quick Slash+", 0));
        assert!(nonterminal.state.enemies[0].entity.is_dead());
        assert_eq!(hand_count(&nonterminal, "Strike"), 1);
        assert!(nonterminal.state.draw_pile.is_empty());

        let mut terminal = engine_for(
            &["Quick Slash"],
            &["Strike"],
            &[],
            vec![enemy_no_intent("JawWorm", 8, 8)],
            3,
        );
        assert!(play_on_enemy(&mut terminal, "Quick Slash", 0));
        assert!(terminal.state.combat_over);
        assert_eq!(hand_count(&terminal, "Strike"), 0);
        assert_eq!(terminal.state.draw_pile.len(), 1);
    }

    #[test]
    fn backflip_uses_source_block_upgrade_and_draw_count() {
        // Source: Backflip.java queues GainBlockAction(this.block), followed by
        // DrawCardAction(p, 2); upgradeBlock(3) changes 5 block to 8.
        for (card_id, dexterity, expected_block) in
            [("Backflip", 0, 5), ("Backflip+", 2, 10)]
        {
            let mut engine = engine_for(
                &[card_id],
                &["Strike", "Defend", "Neutralize"],
                &[],
                vec![enemy_no_intent("JawWorm", 40, 40)],
                3,
            );
            engine.state.player.set_status(sid::DEXTERITY, dexterity);

            assert!(play_self(&mut engine, card_id));

            assert_eq!(engine.state.player.block, expected_block);
            assert_eq!(engine.state.energy, 2);
            assert_eq!(engine.state.hand.len(), 2);
            assert_eq!(engine.state.draw_pile.len(), 1);
            assert_eq!(hand_count(&engine, "Defend"), 1);
            assert_eq!(hand_count(&engine, "Neutralize"), 1);
        }
    }

    #[test]
    fn silent_wave2_dagger_throw_creates_real_discard_choice_and_enables_sneaky_strike_refund() {
        // Source: DaggerThrow.java queues damage, DrawCardAction(1), then
        // DiscardAction(1, false). The freshly drawn Defend must therefore be
        // available in the mandatory discard choice.
        let mut engine = engine_for(
            &["Dagger Throw", "Sneaky Strike", "Strike"],
            &["Defend"],
            &[],
            vec![enemy("JawWorm", 60, 60, 1, 0, 1)],
            3,
        );

        let throw_hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Dagger Throw", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, throw_hp - 9);
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("discard choice");
        assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
        assert_eq!(choice.min_picks, 1);
        assert_eq!(choice.max_picks, 1);
        assert_eq!(choice.options.len(), 3);
        assert_eq!(hand_count(&engine, "Defend"), 1);

        let discard_idx = engine
            .state
            .hand
            .iter()
            .position(|card| engine.card_registry.card_name(card.def_id) == "Strike")
            .expect("Strike_G should be in hand for the discard choice");
        engine.execute_action(&Action::Choose(discard_idx));
        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
        assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 1);

        let energy_before = engine.state.energy;
        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Sneaky Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 12);
        assert_eq!(engine.state.energy, energy_before);
    }

    #[test]
    fn silent_wave2_poison_multi_hit_and_next_turn_energy_work_through_runtime_cards() {
        let enemies = vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 35, 35),
        ];
        let mut dagger_spray = engine_for(&["Dagger Spray"], &[], &[], enemies, 3);
        let hp0 = dagger_spray.state.enemies[0].entity.hp;
        let hp1 = dagger_spray.state.enemies[1].entity.hp;
        assert!(play_on_enemy(&mut dagger_spray, "Dagger Spray", 0));
        assert_eq!(dagger_spray.state.enemies[0].entity.hp, hp0 - 8);
        assert_eq!(dagger_spray.state.enemies[1].entity.hp, hp1 - 8);

        let mut poison = engine_for(
            &["Deadly Poison"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_on_enemy(&mut poison, "Deadly Poison", 0));
        assert_eq!(poison.state.enemies[0].entity.status(sid::POISON), 5);

        let mut flying_knee = engine_for(
            &["Flying Knee"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        let hp_before = flying_knee.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut flying_knee, "Flying Knee", 0));
        assert_eq!(flying_knee.state.enemies[0].entity.hp, hp_before - 8);
        assert_eq!(flying_knee.state.player.status(sid::ENERGIZED), 1);
        flying_knee.execute_action(&Action::EndTurn);
        assert_eq!(flying_knee.state.energy, 4);
        assert_eq!(flying_knee.state.player.status(sid::ENERGIZED), 0);
    }

    #[test]
    fn flying_knee_lethal_damage_clears_the_queued_energized_power() {
        // FlyingKnee.java queues DamageAction before ApplyPowerAction.
        // DamageAction clears post-combat actions after the final monster dies,
        // and GameActionManager does not preserve ApplyPowerAction.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/FlyingKnee.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        let mut engine = engine_for(
            &["Flying Knee"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 8, 8)],
            1,
        );

        assert!(play_on_enemy(&mut engine, "Flying Knee", 0));
        assert!(engine.state.enemies[0].entity.is_dead());
        assert_eq!(engine.state.player.status(sid::ENERGIZED), 0);
    }

    #[test]
    fn outmaneuver_stacks_base_and_upgrade_for_one_next_turn_recharge() {
        // Outmaneuver.java applies EnergizedPower(2), or 3 upgraded.
        // EnergizedPower.onEnergyRecharge grants the stacked amount and queues
        // removal, so the bonus occurs on exactly one recharge.
        let mut engine = engine_for(
            &["Outmaneuver", "Outmaneuver+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );

        assert!(play_self(&mut engine, "Outmaneuver"));
        assert!(play_self(&mut engine, "Outmaneuver+"));
        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.status(sid::ENERGIZED), 5);

        engine.execute_action(&Action::EndTurn);
        assert_eq!(engine.state.energy, 8);
        assert_eq!(engine.state.player.status(sid::ENERGIZED), 0);

        engine.execute_action(&Action::EndTurn);
        assert_eq!(engine.state.energy, 3);
        assert_eq!(engine.state.player.status(sid::ENERGIZED), 0);
    }

    #[test]
    fn silent_wave2_sneaky_strike_condition_stays_dormant_without_a_discard() {
        let mut engine = engine_for(
            &["Sneaky Strike"],
            &[],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );

        let energy_before = engine.state.energy;
        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Sneaky Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 12);
        assert_eq!(engine.state.energy, energy_before - 2);
    }

    #[test]
    fn silent_wave2_upgraded_variants_keep_the_same_runtime_shape() {
        let reg = global_registry();

        assert_eq!(
            reg.get("Dagger Spray+").expect("Dagger Spray+").effect_data,
            &[
                Effect::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                Effect::ExtraHits(A::Magic),
            ]
        );
        assert_eq!(reg.get("Blade Dance+").expect("Blade Dance+").base_magic, 4);
        assert_eq!(reg.get("Deadly Poison+").expect("Deadly Poison+").base_magic, 7);
        assert_eq!(reg.get("Flying Knee+").expect("Flying Knee+").base_damage, 11);
        assert_eq!(reg.get("Quick Slash+").expect("Quick Slash+").base_damage, 12);
        assert_eq!(reg.get("Dagger Throw+").expect("Dagger Throw+").base_damage, 12);
        assert_eq!(reg.get("Sneaky Strike+").expect("Sneaky Strike+").base_damage, 16);

        let upgraded_backflip = reg.get("Backflip+").expect("Backflip+");
        assert_eq!(upgraded_backflip.base_block, 8);
        assert_eq!(upgraded_backflip.effect_data.len(), 1);
    }

    #[test]
    fn silent_wave2_engine_path_handles_full_blade_dance_output() {
        let mut engine = engine_for(
            &["Blade Dance"],
            &[],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut engine, "Blade Dance"));
        assert_eq!(hand_count(&engine, "Shiv"), 3);

        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Shiv", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 4);
        assert_eq!(hand_count(&engine, "Shiv"), 2);
    }

    #[test]
    fn silent_wave2_generated_shivs_do_not_need_a_seeded_draw_pile() {
        let mut engine = engine_for(
            &["Blade Dance"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut engine, "Blade Dance"));
        assert_eq!(hand_names(&engine), vec!["Shiv", "Shiv", "Shiv"]);
    }

    #[test]
    fn blade_dance_plus_spills_shivs_over_the_hand_cap_into_discard() {
        // Sources: BladeDance.java creates 4 Shivs when upgraded;
        // MakeTempCardInHandAction.java caps hand at 10 and discards overflow.
        let mut engine = engine_for(
            &[
                "Blade Dance+",
                "Defend",
                "Defend",
                "Defend",
                "Defend",
                "Defend",
                "Defend",
                "Defend",
                "Defend",
            ],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );

        assert!(play_self(&mut engine, "Blade Dance+"));

        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(hand_count(&engine, "Shiv"), 2);
        assert_eq!(discard_prefix_count(&engine, "Shiv"), 2);
        assert_eq!(discard_prefix_count(&engine, "Blade Dance"), 1);
        assert_eq!(engine.state.energy, 2);
    }
}
