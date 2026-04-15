#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Scrawl.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java

use crate::tests::support::*;

fn watcher_engine() -> crate::engine::CombatEngine {
    engine_with_state(combat_state_with(
        make_deck(&["Strike_P"; 12]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ))
}

#[test]
fn watcher_wave26_scrawl_plus_draws_to_ten_for_hand_sizes_three_through_eight() {
    for initial_hand_size in 3usize..=8 {
        let mut engine = watcher_engine();
        force_player_turn(&mut engine);

        let mut hand_names = vec!["Scrawl+"];
        hand_names.extend(std::iter::repeat_n("Strike_P", initial_hand_size - 1));
        engine.state.hand = make_deck(&hand_names);
        engine.state.draw_pile = make_deck(&["Defend_P"; 10]);
        engine.state.exhaust_pile.clear();

        assert!(play_self(&mut engine, "Scrawl+"));

        let expected_draws = 11 - initial_hand_size;
        assert_eq!(engine.state.hand.len(), 10, "initial hand size {initial_hand_size}");
        assert_eq!(
            draw_prefix_count(&engine, "Defend_P"),
            10 - expected_draws,
            "initial hand size {initial_hand_size}"
        );
        assert_eq!(exhaust_prefix_count(&engine, "Scrawl+"), 1);
    }
}

#[test]
fn watcher_wave26_scrawl_plus_handles_deus_ex_machina_plus_as_next_draw() {
    for initial_hand_size in 3usize..=8 {
        let mut engine = watcher_engine();
        force_player_turn(&mut engine);

        let mut hand_names = vec!["Scrawl+"];
        hand_names.extend(std::iter::repeat_n("Strike_P", initial_hand_size - 1));
        engine.state.hand = make_deck(&hand_names);

        let filler_count = 10usize;
        let mut draw_names = vec!["Defend_P"; filler_count];
        draw_names.push("DeusExMachina+");
        engine.state.draw_pile = make_deck(&draw_names);
        engine.state.exhaust_pile.clear();

        assert!(play_self(&mut engine, "Scrawl+"));

        let expected_followup_draws = 8 - initial_hand_size;
        assert_eq!(engine.state.hand.len(), 10, "initial hand size {initial_hand_size}");
        assert_eq!(hand_count(&engine, "Miracle"), 3, "initial hand size {initial_hand_size}");
        assert_eq!(hand_count(&engine, "DeusExMachina+"), 0, "initial hand size {initial_hand_size}");
        assert_eq!(
            draw_prefix_count(&engine, "Defend_P"),
            filler_count - expected_followup_draws,
            "initial hand size {initial_hand_size}"
        );
        assert_eq!(exhaust_prefix_count(&engine, "Scrawl+"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina+"), 1);
    }
}

#[test]
fn watcher_wave26_deus_ex_machina_plus_respects_hand_limit_when_drawn_late() {
    for (starting_hand_size, expected_miracles) in [(8usize, 2usize), (9usize, 1usize)] {
        let mut engine = watcher_engine();
        force_player_turn(&mut engine);

        engine.state.hand = make_deck(&vec!["Strike_P"; starting_hand_size]);
        engine.state.draw_pile = make_deck(&["DeusExMachina+"]);
        engine.state.exhaust_pile.clear();

        engine.draw_cards(1);

        assert_eq!(engine.state.hand.len(), 10, "starting hand size {starting_hand_size}");
        assert_eq!(
            hand_count(&engine, "Miracle"),
            expected_miracles,
            "starting hand size {starting_hand_size}"
        );
        assert_eq!(hand_count(&engine, "DeusExMachina+"), 0, "starting hand size {starting_hand_size}");
        assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina+"), 1);
    }
}
