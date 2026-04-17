#![cfg(test)]

use crate::powers::registry::{active_player_power_count, status_is_debuff};
use crate::status_ids::sid;
use crate::tests::support::{engine_with, make_deck_n};

#[test]
fn power_registry_helper_surface_is_reduced_to_live_production_queries() {
    let mut engine = engine_with(make_deck_n("Strike_R", 5), 40, 0);
    engine.state.player.set_status(sid::METALLICIZE, 1);
    engine.state.player.set_status(sid::PANACHE, 1);
    engine.state.player.set_status(sid::STRENGTH, 5);
    engine.state.player.set_status(sid::RITUAL, 1);

    assert_eq!(
        active_player_power_count(&engine.state.player),
        2,
        "Force Field style power counting should include live player powers but skip non-power statuses"
    );
}

#[test]
fn power_registry_debuff_classification_matches_runtime_needs() {
    assert!(status_is_debuff(sid::WEAKENED));
    assert!(status_is_debuff(sid::SLOW));
    assert!(status_is_debuff(sid::NO_BLOCK));
    assert!(!status_is_debuff(sid::STRENGTH));
    assert!(!status_is_debuff(sid::ARTIFACT));
}
