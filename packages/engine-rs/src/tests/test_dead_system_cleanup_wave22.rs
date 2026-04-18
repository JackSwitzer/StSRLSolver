#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/RunicPyramid.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/UnceasingTop.java
//
// The final hand-lifecycle bridge helpers were deleted from `relics/run.rs`.
// This test proves the canonical engine path still retains Runic Pyramid hand
// cards and auto-draws for Unceasing Top when the hand is empty.

use crate::tests::support::{combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck, play_on_enemy};

#[test]
fn relic_dead_helper_cleanup_wave22_runic_pyramid_and_unceasing_top_are_inline_on_engine_path() {
    let mut state = combat_state_with(
        make_deck(&["Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec![
        "Runic Pyramid".to_string(),
        "Unceasing Top".to_string(),
    ];

    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.hand.push(engine.card_registry.make_card("Strike"));
    engine.state.hand.push(engine.card_registry.make_card("Defend"));

    end_turn(&mut engine);

    assert!(
        engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Strike"),
        "Runic Pyramid should retain the hand card that was not exhausted"
    );
    assert!(
        engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Defend"),
        "Runic Pyramid should retain the second hand card"
    );

    let mut top_engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    top_engine.state.relics = vec!["Unceasing Top".to_string()];
    top_engine.state.hand.clear();
    top_engine.state.draw_pile.clear();
    top_engine.state.discard_pile.clear();
    top_engine.state.hand.push(top_engine.card_registry.make_card("Strike"));
    top_engine.state.draw_pile.push(top_engine.card_registry.make_card("Defend"));

    let enemy_hp_before = top_engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut top_engine, "Strike", 0));
    assert!(enemy_hp_before > top_engine.state.enemies[0].entity.hp);
    assert!(
        top_engine
            .state
            .hand
            .iter()
            .any(|card| top_engine.card_registry.card_name(card.def_id) == "Defend"),
        "Unceasing Top should draw when the hand becomes empty after a play"
    );
}
