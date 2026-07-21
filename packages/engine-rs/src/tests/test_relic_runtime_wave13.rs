#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/FaceOfCleric.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java

use crate::effects::runtime::EventRecordPhase;
use crate::effects::trigger::Trigger;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state};

#[test]
fn relic_wave13_face_of_cleric_follow_engine_victory_path() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics.push("FaceOfCleric".to_string());
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.max_hp, 81);
    assert_eq!(engine.state.player.hp, 81);
}

#[test]
fn relic_wave13_combat_victory_dispatch_keeps_inventory_order() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.player.hp = 60;
    state.relics = vec!["Burning Blood".to_string(), "FaceOfCleric".to_string()];
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    let handled: Vec<_> = engine
        .event_log
        .iter()
        .filter(|record| {
            record.phase == EventRecordPhase::Handled && record.event == Trigger::CombatVictory
        })
        .filter_map(|record| record.def_id)
        .collect();

    assert_eq!(handled, vec!["Burning Blood", "FaceOfCleric"]);
    assert_eq!(engine.state.player.max_hp, 81);
    assert_eq!(engine.state.player.hp, 67);
}
