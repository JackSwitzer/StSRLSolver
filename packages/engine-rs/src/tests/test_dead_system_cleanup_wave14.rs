#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/FrozenCore.java

use crate::actions::Action;
use crate::orbs::OrbType;
use crate::tests::support::{enemy_no_intent, engine_without_start};

#[test]
fn relic_wave14_frozen_core_engine_path_replaces_helper_flag_test() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.init_defect_orbs(3);
    engine.state.relics.push("FrozenCore".to_string());
    engine.start_combat();

    engine.channel_orb(OrbType::Lightning);
    let occupied_before = engine.state.orb_slots.occupied_count();
    engine.execute_action(&Action::EndTurn);

    assert_eq!(engine.state.orb_slots.occupied_count(), occupied_before + 1);
    assert!(engine.state.orb_slots.slots.iter().any(|orb| orb.orb_type == OrbType::Frost));
}
