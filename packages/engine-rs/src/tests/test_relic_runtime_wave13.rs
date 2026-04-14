#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NinjaScroll.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/FaceOfCleric.java

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
}

#[test]
#[ignore = "blocked on combat-start temp-card materialization timing; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java"]
fn relic_wave13_holy_water_remains_queued_until_temp_card_runtime_is_authoritative() {}

#[test]
#[ignore = "blocked on combat-start temp-card materialization timing; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NinjaScroll.java"]
fn relic_wave13_ninja_scroll_remains_queued_until_temp_card_runtime_is_authoritative() {}
