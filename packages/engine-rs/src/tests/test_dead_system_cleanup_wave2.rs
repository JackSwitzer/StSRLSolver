#![cfg(test)]

use crate::actions::Action;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy, enemy_no_intent, engine_with_state, engine_without_start, make_deck,
    make_deck_n, play_on_enemy,
};

#[test]
fn dead_cleanup_wave2_simple_relic_bundle_is_engine_path_authoritative() {
    let mut state = combat_state_with(
        make_deck(&["Strike", "Strike", "Strike", "Eruption", "Defend"]),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        20,
    );
    state.relics.extend([
        "Vajra".to_string(),
        "Bag of Marbles".to_string(),
        "Anchor".to_string(),
        "Lantern".to_string(),
        "Ornamental Fan".to_string(),
    ]);
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike", "Strike", "Strike"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert_eq!(engine.state.player.strength(), 1);
    assert_eq!(engine.state.player.block, 10);
    assert_eq!(engine.state.energy, 21);
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 1);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 14);
}

#[test]
fn dead_cleanup_wave2_relic_and_potion_use_no_longer_need_helper_combat_start() {
    let mut enemy = enemy("JawWorm", 100, 100, 1, 5, 1);
    enemy.entity.block = 0;
    let mut engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy],
        3,
    );
    engine.state.relics.push("Vajra".to_string());

    engine.start_combat();
    engine.state.potions[0] = "Strength Potion".to_string();
    engine.execute_action(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    });

    assert_eq!(engine.state.player.strength(), 3);
}
