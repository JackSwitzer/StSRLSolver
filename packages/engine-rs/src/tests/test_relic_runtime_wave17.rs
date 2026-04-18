#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NeowsLament.java

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
fn relic_wave17_neows_blessing_decrements_counter_and_sets_enemy_hp_to_one() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.relics.push("NeowsBlessing".to_string());
    engine.state.player.set_status(sid::NEOWS_LAMENT_COUNTER, 3);

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.hp, 1);
    assert_eq!(engine.state.player.status(sid::NEOWS_LAMENT_COUNTER), 2);
}

#[test]
fn relic_wave17_neows_blessing_uses_up_on_final_charge() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.relics.push("NeowsBlessing".to_string());
    engine.state.player.set_status(sid::NEOWS_LAMENT_COUNTER, 1);

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.hp, 1);
    assert_eq!(engine.state.player.status(sid::NEOWS_LAMENT_COUNTER), -2);
}
