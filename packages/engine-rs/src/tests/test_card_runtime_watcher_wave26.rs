#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java

use crate::tests::support::{exhaust_prefix_count, hand_count, make_deck, engine_without_start};

#[test]
fn deus_ex_machina_draw_dispatch_exhausts_self_before_creating_miracles() {
    let mut engine = engine_without_start(make_deck(&["DeusExMachina"]), vec![], 3);

    engine.draw_cards(1);

    assert_eq!(hand_count(&engine, "DeusExMachina"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina"), 1);
    assert_eq!(hand_count(&engine, "Miracle"), 2);
}
