#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Calipers.java
//
// `calipers_block_retention` was deleted from `relics/run.rs`; this test proves
// the canonical engine path still retains at most 15 block between turns.

use crate::tests::support::{combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck};

#[test]
fn relic_dead_helper_cleanup_wave20_calipers_retention_is_inline_on_engine_path() {
    let mut state = combat_state_with(
        make_deck(&["Strike_R"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec!["Calipers".to_string()];

    let mut engine = engine_with_state(state);
    engine.state.player.block = 22;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.block, 15, "Calipers should retain up to 15 block");
}
