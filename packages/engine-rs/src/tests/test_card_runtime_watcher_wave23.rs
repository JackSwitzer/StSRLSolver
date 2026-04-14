#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java

use crate::tests::support::*;

#[test]
fn watcher_wave23_deus_ex_machina_draw_trigger_matches_java_oracle() {
    let engine = engine_with(make_deck(&["DeusExMachina+"]), 50, 0);

    assert_eq!(hand_count(&engine, "Miracle"), 3);
    assert_eq!(hand_count(&engine, "DeusExMachina+"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina+"), 1);
}
