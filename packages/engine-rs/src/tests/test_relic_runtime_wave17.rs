#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NeowsLament.java

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
#[ignore = "blocked on NeowsBlessing combat-start HP reduction; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NeowsLament.java"]
fn relic_wave17_neows_blessing_sets_enemy_hp_to_one() {
    let mut engine = engine_without_start(
        make_deck_n("Strike_R", 10),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.relics.push("NeowsBlessing".to_string());
    engine.state.player.set_status(sid::NEOWS_LAMENT_COUNTER, 3);

    engine.start_combat();

    assert_eq!(engine.state.enemies[0].entity.hp, 1);
    assert_eq!(engine.state.player.status(sid::NEOWS_LAMENT_COUNTER), 2);
}
