#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/StrikeDummy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/RunicPyramid.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Calipers.java
//
// The remaining helpers in `relics/run.rs` are still live engine/runtime
// bridges, so this wave keeps the cleanup honest: prove the engine path still
// covers the bridge behavior, but do not delete helpers that are still called
// from production code.

use crate::tests::support::{combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck, play_on_enemy};

#[test]
fn relic_dead_helper_cleanup_wave18_live_bridge_helpers_still_work_on_engine_path() {
    let mut state = combat_state_with(
        make_deck(&["Strike_R", "Defend_R"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec![
        "StrikeDummy".to_string(),
        "Runic Pyramid".to_string(),
        "Calipers".to_string(),
    ];

    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.hand.push(engine.card_registry.make_card("Strike_R"));
    engine.state.hand.push(engine.card_registry.make_card("Defend_R"));
    engine.state.player.block = 20;

    let enemy_hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert_eq!(enemy_hp_before - engine.state.enemies[0].entity.hp, 9);

    end_turn(&mut engine);

    assert!(
        engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Defend_R"),
        "Runic Pyramid should retain the remaining hand card"
    );
    assert_eq!(engine.state.player.block, 15, "Calipers should retain up to 15 block");
}
