#[cfg(test)]
mod ironclad_wave2_card_runtime_tests {
    use crate::actions::Action;
    use crate::cards::{CardDef, CardTarget, CardType};
    use crate::effects::declarative::{
        AmountSource, Effect, Pile, SimpleEffect, Target, BoolFlag,
    };
    use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, CombatPhase};
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, discard_prefix_count, draw_prefix_count, enemy, force_player_turn,
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

    fn pile_contains(
        engine: &CombatEngine,
        pile: &[crate::combat_types::CardInstance],
        name: &str,
    ) -> bool {
        pile.iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == name)
    }

    #[test]
    fn wave2_cards_export_declarative_effect_data() {
        let power_through = card("Power Through");
        assert_eq!(power_through.card_type, CardType::Skill);
        assert_eq!(power_through.target, CardTarget::SelfTarget);
        assert_eq!(
            power_through.effect_data,
            &[Effect::Simple(SimpleEffect::AddCard(
                "Wound",
                Pile::Hand,
                AmountSource::Fixed(2),
            ))],
        );

        let headbutt = card("Headbutt");
        assert!(headbutt.effect_data.is_empty());
        assert!(headbutt.complex_hook.is_some());

        let rampage = card("Rampage");
        assert!(rampage.effect_data.is_empty());
        assert!(rampage.complex_hook.is_some());

        let pommel_strike = card("Pommel Strike");
        assert_eq!(
            pommel_strike.effect_data,
            &[Effect::Simple(SimpleEffect::DrawCards(AmountSource::Magic))],
        );

        let battle_trance = card("Battle Trance");
        assert_eq!(
            battle_trance.effect_data,
            &[
                Effect::Simple(SimpleEffect::DrawCards(AmountSource::Magic)),
                Effect::Simple(SimpleEffect::SetFlag(BoolFlag::NoDraw)),
            ],
        );

        let thunderclap = card("Thunderclap");
        assert_eq!(
            thunderclap.effect_data,
            &[Effect::Simple(SimpleEffect::AddStatus(
                Target::AllEnemies,
                sid::VULNERABLE,
                AmountSource::Magic,
            ))],
        );

        let bloodletting = card("Bloodletting");
        assert_eq!(
            bloodletting.effect_data,
            &[
                Effect::Simple(SimpleEffect::ModifyHp(AmountSource::Fixed(-3))),
                Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Magic)),
            ],
        );

        let flex = card("Flex");
        assert_eq!(
            flex.effect_data,
            &[
                Effect::Simple(SimpleEffect::AddStatus(
                    Target::Player,
                    sid::STRENGTH,
                    AmountSource::Magic,
                )),
                Effect::Simple(SimpleEffect::AddStatus(
                    Target::Player,
                    sid::TEMP_STRENGTH,
                    AmountSource::Magic,
                )),
            ],
        );
    }

    #[test]
    fn power_through_flex_and_bloodletting_use_the_production_card_path() {
        let mut engine = engine_for(
            &["Power Through", "Flex", "Bloodletting"],
            &[],
            &[],
            50,
            3,
        );

        let block_before = engine.state.player.block;
        assert!(play_self(&mut engine, "Power Through"));
        assert_eq!(engine.state.player.block, block_before + 15);
        assert_eq!(engine.state.hand.iter().filter(|c| engine.card_registry.card_name(c.def_id) == "Wound").count(), 2);

        assert!(play_self(&mut engine, "Flex"));
        assert_eq!(engine.state.player.strength(), 2);
        assert_eq!(engine.state.player.status(sid::TEMP_STRENGTH), 2);

        let hp_before = engine.state.player.hp;
        let energy_before = engine.state.energy;
        assert!(play_self(&mut engine, "Bloodletting"));
        assert_eq!(engine.state.player.hp, hp_before - 3);
        assert_eq!(engine.state.energy, energy_before + 2);
    }

    #[test]
    fn headbutt_pommel_strike_and_thunderclap_cover_choice_draw_and_vulnerable_paths() {
        let mut headbutt_engine = engine_for(
            &["Headbutt"],
            &["Shrug It Off"],
            &["Strike_R", "Defend_R"],
            50,
            3,
        );

        let hp_before = headbutt_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut headbutt_engine, "Headbutt", 0));
        assert_eq!(headbutt_engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(
            headbutt_engine.choice.as_ref().map(|choice| choice.reason.clone()),
            Some(ChoiceReason::PickFromDiscard),
        );
        let selected_name = match headbutt_engine.choice.as_ref().unwrap().options[0] {
            ChoiceOption::DiscardCard(idx) => headbutt_engine
                .card_registry
                .card_name(headbutt_engine.state.discard_pile[idx].def_id)
                .to_string(),
            _ => panic!("Headbutt should offer discard cards"),
        };
        headbutt_engine.execute_action(&Action::Choose(0));
        assert_eq!(headbutt_engine.state.enemies[0].entity.hp, hp_before - 9);
        assert!(pile_contains(
            &headbutt_engine,
            &headbutt_engine.state.draw_pile,
            &selected_name,
        ));
        assert!(!pile_contains(
            &headbutt_engine,
            &headbutt_engine.state.discard_pile,
            &selected_name,
        ));

        let mut pommel_engine = engine_for(&["Pommel Strike"], &["Strike_P"], &[], 50, 3);
        let hand_before = pommel_engine.state.hand.len();
        let hp_before = pommel_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut pommel_engine, "Pommel Strike", 0));
        assert_eq!(pommel_engine.state.enemies[0].entity.hp, hp_before - 9);
        assert_eq!(pommel_engine.state.hand.len(), hand_before);
        assert_eq!(draw_prefix_count(&pommel_engine, "Strike_"), 0);
        assert_eq!(pommel_engine.state.hand.iter().filter(|c| pommel_engine.card_registry.card_name(c.def_id) == "Strike_P").count(), 1);

        let mut thunderclap_engine = engine_for(
            &["Thunderclap"],
            &[],
            &[],
            50,
            3,
        );
        thunderclap_engine.state.enemies.push(enemy("Cultist", 50, 50, 1, 0, 1));
        let hp0 = thunderclap_engine.state.enemies[0].entity.hp;
        let hp1 = thunderclap_engine.state.enemies[1].entity.hp;
        assert!(play_on_enemy(&mut thunderclap_engine, "Thunderclap", 0));
        assert_eq!(thunderclap_engine.state.enemies[0].entity.hp, hp0 - 4);
        assert_eq!(thunderclap_engine.state.enemies[1].entity.hp, hp1 - 4);
        assert_eq!(thunderclap_engine.state.enemies[0].entity.status(sid::VULNERABLE), 1);
        assert_eq!(thunderclap_engine.state.enemies[1].entity.status(sid::VULNERABLE), 1);
    }

    #[test]
    fn rampage_battle_trance_and_bloodletting_cover_scaling_draw_and_no_draw() {
        let mut rampage_engine = engine_for(&["Rampage"], &[], &[], 80, 3);
        let hp_before = rampage_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut rampage_engine, "Rampage", 0));
        let after_first = rampage_engine.state.enemies[0].entity.hp;
        let played_once = rampage_engine
            .state
            .discard_pile
            .pop()
            .expect("played Rampage should be in discard");
        assert_eq!(played_once.misc, 13);
        rampage_engine.state.hand = vec![played_once];
        assert!(play_on_enemy(&mut rampage_engine, "Rampage", 0));
        let after_second = rampage_engine.state.enemies[0].entity.hp;
        assert_eq!(hp_before - after_first, 8);
        assert_eq!(after_first - after_second, 13);
        assert_eq!(
            rampage_engine
                .state
                .discard_pile
                .last()
                .expect("replayed Rampage should be in discard")
                .misc,
            18,
        );

        let mut battle_trance_engine = engine_for(
            &["Battle Trance"],
            &["Strike_R", "Defend_R", "Bash"],
            &[],
            50,
            3,
        );
        let hand_before = battle_trance_engine.state.hand.len();
        assert!(play_self(&mut battle_trance_engine, "Battle Trance"));
        assert_eq!(battle_trance_engine.state.hand.len(), hand_before + 2);
        assert_eq!(battle_trance_engine.state.player.status(sid::NO_DRAW), 1);
        assert_eq!(draw_prefix_count(&battle_trance_engine, "Strike_"), 0);

        let mut bloodletting_engine = engine_for(&["Bloodletting"], &[], &[], 50, 3);
        let hp_before = bloodletting_engine.state.player.hp;
        let energy_before = bloodletting_engine.state.energy;
        assert!(play_self(&mut bloodletting_engine, "Bloodletting"));
        assert_eq!(bloodletting_engine.state.player.hp, hp_before - 3);
        assert_eq!(bloodletting_engine.state.energy, energy_before + 2);
        assert_eq!(discard_prefix_count(&bloodletting_engine, "Bloodletting"), 1);
    }
}
