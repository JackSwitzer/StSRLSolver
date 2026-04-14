#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java
//
// Warning-only relic bridge helpers were deleted in this wave. The remaining
// helpers in `relics/run.rs` are live engine/runtime bridges only. If a future
// relic regression appears, add engine-path coverage instead of restoring
// helper-oracle parity.

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, ensure_in_hand, engine_without_start, make_deck, play_on_enemy};

#[test]
fn relic_dead_helper_cleanup_wave17_necronomicon_remains_on_live_bridge_path() {
    let mut engine = engine_without_start(
        make_deck(&["Bludgeon"]),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    engine.state.relics.push("Necronomicon".to_string());

    engine.start_combat();
    ensure_in_hand(&mut engine, "Bludgeon");

    assert!(play_on_enemy(&mut engine, "Bludgeon", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 16);
    assert_eq!(engine.state.total_damage_dealt, 64);
    assert_eq!(engine.state.player.status(sid::NECRONOMICON_USED), 1);
}

#[test]
#[ignore = "Warning-only relic bridge helpers were deleted in this wave; if any future relic regression appears, add engine-path coverage instead of restoring helper-oracle parity."]
fn relic_dead_helper_cleanup_wave17_no_warning_only_helpers_remain() {}
