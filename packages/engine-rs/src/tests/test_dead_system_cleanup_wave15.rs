#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/FossilizedHelix.java

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
fn relic_wave15_fossilized_helix_engine_path_replaces_helper_tail() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics.push("Fossilized Helix".to_string());

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::BUFFER), 1);
}
