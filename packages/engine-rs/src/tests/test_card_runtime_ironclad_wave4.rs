#[cfg(test)]
mod ironclad_wave4_card_runtime_tests {
    use crate::actions::Action;
    use crate::cards::{CardDef, CardTarget, CardType};
    use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, CombatPhase};
    use crate::tests::support::{
        combat_state_with, discard_prefix_count, enemy, enemy_no_intent, engine_without_start,
        exhaust_prefix_count, force_player_turn, make_deck, play_on_enemy, play_self, TEST_SEED,
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

    fn total_enemy_hp(engine: &CombatEngine) -> i32 {
        engine
            .state
            .enemies
            .iter()
            .map(|enemy| enemy.entity.hp.max(0))
            .sum()
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
    fn ironclad_wave4_registry_exports_honest_runtime_metadata() {
        let blood_for_blood = card("Blood for Blood");
        assert_eq!(blood_for_blood.card_type, CardType::Attack);
        assert_eq!(blood_for_blood.target, CardTarget::Enemy);
        assert!(blood_for_blood.effects.contains(&"cost_reduce_on_hp_loss"));
        assert!(blood_for_blood.effect_data.is_empty());

        let bludgeon = card("Bludgeon");
        assert_eq!(bludgeon.card_type, CardType::Attack);
        assert_eq!(bludgeon.target, CardTarget::Enemy);
        assert!(bludgeon.effect_data.is_empty());
        assert!(bludgeon.complex_hook.is_none());

        let burning_pact = card("Burning Pact");
        assert_eq!(
            burning_pact.effect_data,
            &[crate::effects::declarative::Effect::ChooseCards {
                source: crate::effects::declarative::Pile::Hand,
                filter: crate::effects::declarative::CardFilter::All,
                action: crate::effects::declarative::ChoiceAction::Exhaust,
                min_picks: crate::effects::declarative::AmountSource::Fixed(1),
                max_picks: crate::effects::declarative::AmountSource::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Magic,
            }]
        );
        assert!(burning_pact.complex_hook.is_none());

        let carnage = card("Carnage");
        assert!(carnage.effects.contains(&"ethereal"));

        let cleave = card("Cleave");
        assert_eq!(cleave.target, CardTarget::AllEnemy);
        assert!(cleave.effect_data.is_empty());

        let ghostly_armor = card("Ghostly Armor");
        assert_eq!(ghostly_armor.card_type, CardType::Skill);
        assert!(ghostly_armor.effects.contains(&"ethereal"));

        let headbutt = card("Headbutt");
        assert!(headbutt.effect_data.is_empty());
        assert!(headbutt.complex_hook.is_some());
    }

    #[test]
    fn blood_for_blood_uses_the_hp_loss_cost_path_on_engine_play() {
        let mut engine = engine_for(&["Blood for Blood"], &[], &[], 60, 2);
        engine.player_lose_hp(2);
        let hp_before = engine.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut engine, "Blood for Blood", 0));

        assert_eq!(engine.state.energy, 0);
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 18);
    }

    #[test]
    fn bludgeon_keeps_the_expected_single_hit_engine_behavior() {
        let mut engine = engine_for(&["Bludgeon"], &[], &[], 70, 3);
        let hp_before = engine.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut engine, "Bludgeon", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 32);
    }

    #[test]
    fn burning_pact_exhausts_a_selected_card_and_draws_after_choice_resolution() {
        let mut engine = engine_for(
            &["Burning Pact", "Strike_R"],
            &["Defend_R", "Bash"],
            &[],
            50,
            3,
        );

        assert!(play_self(&mut engine, "Burning Pact"));
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(
            engine.choice.as_ref().map(|choice| choice.reason.clone()),
            Some(ChoiceReason::ExhaustFromHand),
        );

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
        assert_eq!(exhaust_prefix_count(&engine, "Strike_"), 1);
        assert_eq!(engine.state.hand.len(), 2);
        assert!(engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Defend_R"));
        assert!(engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Bash"));
    }

    #[test]
    fn carnage_ethereal_exhausts_when_left_unplayed() {
        let mut engine = engine_for(&["Carnage"], &["Strike_R"], &[], 50, 3);

        engine.execute_action(&Action::EndTurn);

        assert_eq!(exhaust_prefix_count(&engine, "Carnage"), 1);
        assert_eq!(discard_prefix_count(&engine, "Carnage"), 0);
    }

    #[test]
    fn cleave_hits_every_enemy_on_the_production_path() {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 35, 35),
            ],
            3,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Cleave"]);
        let hp_before = total_enemy_hp(&engine);

        assert!(play_on_enemy(&mut engine, "Cleave", 0));

        assert_eq!(hp_before - total_enemy_hp(&engine), 16);
    }

    #[test]
    fn ghostly_armor_grants_block_and_exhausts_if_unplayed() {
        let mut played = engine_for(&["Ghostly Armor"], &[], &[], 50, 3);
        let block_before = played.state.player.block;

        assert!(play_self(&mut played, "Ghostly Armor"));

        assert!(played.state.player.block >= block_before + 10);
        assert_eq!(discard_prefix_count(&played, "Ghostly Armor"), 1);

        let mut held = engine_for(&["Ghostly Armor"], &["Strike_R"], &[], 50, 3);
        held.execute_action(&Action::EndTurn);

        assert_eq!(exhaust_prefix_count(&held, "Ghostly Armor"), 1);
        assert_eq!(discard_prefix_count(&held, "Ghostly Armor"), 0);
    }

    #[test]
    fn headbutt_keeps_the_discard_to_draw_choice_on_the_engine_path() {
        let mut engine = engine_for(
            &["Headbutt"],
            &["Shrug It Off"],
            &["Strike_R", "Defend_R"],
            50,
            3,
        );
        let hp_before = engine.state.enemies[0].entity.hp;

        assert!(play_on_enemy(&mut engine, "Headbutt", 0));
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(
            engine.choice.as_ref().map(|choice| choice.reason.clone()),
            Some(ChoiceReason::PickFromDiscard),
        );

        let selected_name = match engine.choice.as_ref().unwrap().options[0] {
            ChoiceOption::DiscardCard(idx) => engine
                .card_registry
                .card_name(engine.state.discard_pile[idx].def_id)
                .to_string(),
            _ => panic!("Headbutt should offer discard cards"),
        };

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 9);
        assert!(pile_contains(&engine, &engine.state.draw_pile, &selected_name));
        assert!(!pile_contains(
            &engine,
            &engine.state.discard_pile,
            &selected_name,
        ));
    }
}
