#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/DataDisk.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/ClockworkSouvenir.java

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
fn relic_wave18_combat_start_data_disk_and_clockwork_souvenir_replace_helper_parity() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics = vec!["Data Disk".to_string(), "Clockwork Souvenir".to_string()];

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::FOCUS), 1);
    assert_eq!(engine.state.player.status(sid::ARTIFACT), 1);
}
