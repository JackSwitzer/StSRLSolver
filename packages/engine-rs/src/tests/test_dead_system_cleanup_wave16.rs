#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/ChemicalX.java
//
// Legacy relic helper-path test modules and the relic combat stub were deleted
// in this wave. If any future relic regression appears, add an engine-path test
// for the specific relic behavior instead of restoring helper-path parity.

use crate::status_ids::sid;
use crate::tests::support::{engine_with, ensure_in_hand, play_self};

#[test]
fn relic_dead_surface_cleanup_wave16_chemical_x_is_covered_by_engine_path_collect() {
    let mut engine = engine_with(crate::tests::support::make_deck(&["Collect"]), 40, 0);
    engine.state.relics.push("Chemical X".to_string());
    ensure_in_hand(&mut engine, "Collect");

    assert!(play_self(&mut engine, "Collect"));
    assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 5);
}

#[test]
#[ignore = "Legacy relic helper-path parity has been deleted; if a relic regresses, add the engine-path assertion in a focused relic wave instead of restoring helper tests."]
fn relic_dead_surface_cleanup_wave16_no_helper_path_oracles_remain() {}
