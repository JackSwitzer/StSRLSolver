#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java
//
// The Necronomicon bridge trio was deleted from `relics/run.rs`; this test
// proves the canonical engine path still replays the first 2+-cost Attack once
// per turn and resets the used flag at turn start.

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, ensure_in_hand, engine_without_start, make_deck, play_on_enemy};

#[test]
fn relic_dead_helper_cleanup_wave21_necronomicon_replay_and_reset_are_inline_on_engine_path() {
    let mut engine = engine_without_start(
        make_deck(&["Bludgeon", "Bludgeon"]),
        vec![enemy_no_intent("JawWorm", 200, 200)],
        3,
    );
    engine.state.relics.push("Necronomicon".to_string());

    engine.start_combat();
    ensure_in_hand(&mut engine, "Bludgeon");

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Bludgeon", 0));
    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 64);
    assert_eq!(engine.state.player.status(sid::NECRONOMICON_USED), 1);

    crate::tests::support::end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::NECRONOMICON_USED), 0);

    ensure_in_hand(&mut engine, "Bludgeon");
    let hp_before_second = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Bludgeon", 0));
    assert_eq!(hp_before_second - engine.state.enemies[0].entity.hp, 64);
    assert_eq!(engine.state.player.status(sid::NECRONOMICON_USED), 1);
}
