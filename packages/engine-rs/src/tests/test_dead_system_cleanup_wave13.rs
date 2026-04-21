#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NeowsLament.java

use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
fn relic_wave13_neows_blessing_without_counter_leaves_enemy_unchanged() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.relics.push("NeowsBlessing".to_string());

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.hp, 50);
}
