#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Calipers.java
//
// D49 parity fix: Calipers SUBTRACTS 15 block at end of round (Java's
// `loseBlock(15)` truncates at 0). Pre-fix the Rust engine capped at 15
// ("retain up to 15"), which silently gave Body Slam / Barricade decks
// extra block that Java would have lost. This test now asserts the
// Java-matching subtract-15 behavior.

use crate::tests::support::{combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck};

#[test]
fn calipers_subtracts_15_block_at_end_of_round() {
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec!["Calipers".to_string()];

    let mut engine = engine_with_state(state);
    engine.state.player.block = 22;

    end_turn(&mut engine);

    // Java: 22 - 15 = 7. Pre-fix Rust incorrectly returned 15 (cap-at-15).
    assert_eq!(engine.state.player.block, 7, "Calipers should subtract 15 block");
}

#[test]
fn calipers_truncates_at_zero_when_block_below_15() {
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec!["Calipers".to_string()];

    let mut engine = engine_with_state(state);
    engine.state.player.block = 8;

    end_turn(&mut engine);

    // 8 - 15 = -7 -> truncated to 0.
    assert_eq!(engine.state.player.block, 0);
}

#[test]
fn calipers_preserves_high_block_totals() {
    // Body Slam / Barricade scenarios: 50 block should keep 35, not cap at 15.
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec!["Calipers".to_string()];

    let mut engine = engine_with_state(state);
    engine.state.player.block = 50;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.block, 35);
}
