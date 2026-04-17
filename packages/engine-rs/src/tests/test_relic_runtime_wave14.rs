#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MeatOnTheBone.java

use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state};

#[test]
fn relic_wave14_meat_on_the_bone_heals_on_victory_at_half_or_below() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics.push("Meat on the Bone".to_string());
    state.player.hp = 40;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.hp, 52);
}

#[test]
fn relic_wave14_meat_on_the_bone_does_not_heal_above_half() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics.push("Meat on the Bone".to_string());
    state.player.hp = 60;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.hp, 60);
}
