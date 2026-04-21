#[cfg(test)]
mod silent_wave1 {
    use crate::actions::Action;
    use crate::cards::global_registry;
    use crate::effects::declarative::{AmountSource as A, Effect};
    use crate::status_ids::sid;
    use crate::tests::support::*;

    #[test]
    fn silent_basic_attack_cards_still_play_on_the_engine_path() {
        let enemies = vec![
            enemy("A", 50, 50, 1, 0, 1),
            enemy("B", 50, 50, 1, 0, 1),
        ];
        let mut die_die_die = engine_with_enemies(make_deck_n("Die Die Die", 8), enemies, 3);
        ensure_in_hand(&mut die_die_die, "Die Die Die");
        assert!(play_self(&mut die_die_die, "Die Die Die"));
        assert_eq!(die_die_die.state.enemies[0].entity.hp, 37);
        assert_eq!(die_die_die.state.enemies[1].entity.hp, 37);
        assert!(die_die_die
            .state
            .exhaust_pile
            .iter()
            .any(|card| die_die_die.card_registry.card_name(card.def_id) == "Die Die Die"));

        let mut slice = engine_with(make_deck_n("Slice", 8), 40, 0);
        ensure_in_hand(&mut slice, "Slice");
        let slice_hp = slice.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut slice, "Slice", 0));
        assert_eq!(slice.state.enemies[0].entity.hp, slice_hp - 6);

        let mut dash = engine_with(make_deck_n("Dash", 8), 50, 0);
        ensure_in_hand(&mut dash, "Dash");
        let dash_hp = dash.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut dash, "Dash", 0));
        assert_eq!(dash.state.enemies[0].entity.hp, dash_hp - 10);
        assert_eq!(dash.state.player.block, 10);
    }

    #[test]
    fn silent_multi_hit_cards_export_declarative_extra_hits_and_play_correctly() {
        let reg = global_registry();
        let eviscerate = reg.get("Eviscerate").expect("Eviscerate should exist");
        assert_eq!(eviscerate.effect_data, &[Effect::ExtraHits(A::Magic)]);
        let riddle = reg.get("Riddle with Holes").expect("Riddle with Holes should exist");
        assert_eq!(riddle.effect_data, &[Effect::ExtraHits(A::Magic)]);

        let mut eviscerate_engine = engine_with(make_deck_n("Eviscerate", 8), 100, 0);
        ensure_in_hand(&mut eviscerate_engine, "Eviscerate");
        let eviscerate_hp = eviscerate_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut eviscerate_engine, "Eviscerate", 0));
        assert_eq!(eviscerate_engine.state.enemies[0].entity.hp, eviscerate_hp - 21);

        let mut riddle_engine = engine_with(make_deck_n("Riddle with Holes", 8), 100, 0);
        ensure_in_hand(&mut riddle_engine, "Riddle with Holes");
        let riddle_hp = riddle_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut riddle_engine, "Riddle with Holes", 0));
        assert_eq!(riddle_engine.state.enemies[0].entity.hp, riddle_hp - 15);
    }

    #[test]
    fn silent_runtime_cost_scaling_uses_the_production_engine_path() {
        let mut eviscerate_engine = engine_with(
            make_deck(&["Strike", "Eviscerate"]),
            40,
            0,
        );
        eviscerate_engine.state.hand = make_deck(&["Eviscerate"]);
        eviscerate_engine.state.draw_pile.clear();
        eviscerate_engine.state.discard_pile.clear();
        eviscerate_engine.state.energy = 1;
        eviscerate_engine.state.player.add_status(sid::DISCARDED_THIS_TURN, 2);
        assert!(eviscerate_engine.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        }));
        assert!(play_on_enemy(&mut eviscerate_engine, "Eviscerate", 0));
        assert_eq!(eviscerate_engine.state.energy, 0);

        let mut stab_engine = engine_with(
            make_deck(&["Masterful Stab"]),
            40,
            0,
        );
        stab_engine.state.hand = make_deck(&["Masterful Stab"]);
        stab_engine.state.draw_pile.clear();
        stab_engine.state.discard_pile.clear();
        stab_engine.state.energy = 6;
        stab_engine.state.total_damage_taken = 7;
        assert!(
            !stab_engine
                .get_legal_actions()
                .iter()
                .any(|action| matches!(action, Action::PlayCard { .. })),
            "Masterful Stab should be illegal when runtime cost exceeds available energy"
        );

        stab_engine.state.energy = 7;
        assert!(stab_engine.get_legal_actions().iter().any(|action| {
            matches!(action, Action::PlayCard { .. })
        }));
        assert!(play_on_enemy(&mut stab_engine, "Masterful Stab", 0));
        assert_eq!(stab_engine.state.energy, 0);
    }

    #[test]
    fn silent_draw_and_discard_hooks_still_fire_for_reflex_and_endless_agony() {
        let reg = global_registry();

        let mut reflex_engine = engine_without_start(
            make_deck_n("Strike", 6),
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        reflex_engine.state.draw_pile = make_deck_n("Strike", 6);
        reflex_engine.state.hand.clear();
        let reflex_card = reg.make_card("Reflex");
        reflex_engine.state.discard_pile.push(reflex_card);
        reflex_engine.on_card_discarded(reflex_card);
        assert_eq!(reflex_engine.state.hand.len(), 2);
        assert_eq!(reflex_engine.state.player.status(sid::DISCARDED_THIS_TURN), 1);

        let mut agony_engine = engine_without_start(
            make_deck_n("Endless Agony", 1),
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );
        agony_engine.state.draw_pile = make_deck_n("Endless Agony", 1);
        agony_engine.state.hand.clear();
        agony_engine.draw_cards(1);
        assert_eq!(hand_count(&agony_engine, "Endless Agony"), 2);
    }
}
