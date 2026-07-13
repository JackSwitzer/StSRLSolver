#[cfg(test)]
mod silent_wave1 {
    use crate::actions::Action;
    use crate::cards::global_registry;
    use crate::engine::CombatEngine;
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
        // Eviscerate.java queues exactly three DamageActions and upgradeDamage(2).
        // Java: reference/extracted/methods/card/Eviscerate.java
        let reg = global_registry();
        let eviscerate = reg.get("Eviscerate").expect("Eviscerate should exist");
        assert_eq!(eviscerate.effect_data, &[Effect::ExtraHits(A::Fixed(3))]);
        // RiddleWithHoles.java declares canonical ID "Riddle With Holes".
        let riddle = reg.get("Riddle With Holes").expect("Riddle With Holes should exist");
        assert_eq!(riddle.effect_data, &[Effect::ExtraHits(A::Magic)]);

        let mut eviscerate_engine = engine_with(make_deck_n("Eviscerate", 8), 100, 0);
        ensure_in_hand(&mut eviscerate_engine, "Eviscerate");
        let eviscerate_hp = eviscerate_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut eviscerate_engine, "Eviscerate", 0));
        assert_eq!(eviscerate_engine.state.enemies[0].entity.hp, eviscerate_hp - 21);

        let mut upgraded_engine = engine_with(make_deck_n("Eviscerate+", 8), 100, 0);
        ensure_in_hand(&mut upgraded_engine, "Eviscerate+");
        let upgraded_hp = upgraded_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut upgraded_engine, "Eviscerate+", 0));
        assert_eq!(upgraded_engine.state.enemies[0].entity.hp, upgraded_hp - 27);

        let mut riddle_engine = engine_with(make_deck_n("Riddle With Holes", 8), 100, 0);
        ensure_in_hand(&mut riddle_engine, "Riddle With Holes");
        let riddle_hp = riddle_engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut riddle_engine, "Riddle With Holes", 0));
        assert_eq!(riddle_engine.state.enemies[0].entity.hp, riddle_hp - 15);
    }

    #[test]
    fn silent_runtime_cost_scaling_uses_the_production_engine_path() {
        // Eviscerate.triggerWhenDrawn sets costForTurn to card.cost minus
        // totalDiscardedThisTurn. Each later incrementDiscard calls
        // Eviscerate.didDiscard and removes one more cost.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Eviscerate.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
        let mut eviscerate_engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        force_player_turn(&mut eviscerate_engine);
        eviscerate_engine.state.draw_pile = make_deck(&["Eviscerate"]);
        eviscerate_engine.state.discard_pile.clear();
        eviscerate_engine.state.player.add_status(sid::DISCARDED_THIS_TURN, 2);
        eviscerate_engine.draw_cards(1);
        assert_eq!(eviscerate_engine.state.hand[0].cost, 1);
        assert!(eviscerate_engine.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        }));
        assert!(play_on_enemy(&mut eviscerate_engine, "Eviscerate", 0));
        assert_eq!(eviscerate_engine.state.energy, 0);

        let mut discard_engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            2,
        );
        force_player_turn(&mut discard_engine);
        discard_engine.state.hand = make_deck(&["Eviscerate"]);
        discard_engine.state.discard_pile = make_deck(&["Defend"]);
        discard_engine.on_card_discarded(discard_engine.state.discard_pile[0]);
        assert_eq!(discard_engine.state.hand[0].cost, 2);

        // triggerWhenDrawn runs before ConfusionPower.onCardDraw. Confusion
        // therefore overwrites the discount from earlier discards; only later
        // discards decrement the randomized cost.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ConfusionPower.java
        let confusion_seed = (0u64..100)
            .find(|seed| {
                let mut rng = crate::seed::StsRandom::new(*seed);
                rng.random(3) == 3
            })
            .expect("a seed whose first Confusion roll is three");
        let mut confusion_engine = CombatEngine::new(
            combat_state_with(
                Vec::new(),
                vec![enemy_no_intent("JawWorm", 40, 40)],
                2,
            ),
            confusion_seed,
        );
        force_player_turn(&mut confusion_engine);
        confusion_engine.state.draw_pile = make_deck(&["Eviscerate"]);
        confusion_engine.state.discard_pile = make_deck(&["Defend"]);
        confusion_engine.state.player.set_status(sid::DISCARDED_THIS_TURN, 2);
        confusion_engine.state.player.set_status(sid::CONFUSION, 1);
        confusion_engine.draw_cards(1);

        assert_eq!(confusion_engine.state.hand[0].cost, 3);
        assert!(!confusion_engine.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        }));

        confusion_engine.on_card_discarded(confusion_engine.state.discard_pile[0]);
        assert_eq!(confusion_engine.state.hand[0].cost, 2);
        assert!(confusion_engine.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        }));

        let mut stab_engine = engine_with(
            make_deck(&["Masterful Stab"]),
            40,
            0,
        );
        stab_engine.state.hand = make_deck(&["Masterful Stab"]);
        stab_engine.state.draw_pile.clear();
        stab_engine.state.discard_pile.clear();
        stab_engine.state.energy = 0;
        stab_engine.player_lose_hp(7);
        assert!(
            !stab_engine
                .get_legal_actions()
                .iter()
                .any(|action| matches!(action, Action::PlayCard { .. })),
            "Masterful Stab should be illegal when runtime cost exceeds available energy"
        );

        stab_engine.state.energy = 1;
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
