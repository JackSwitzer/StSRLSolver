#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RedMask.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PhilosopherStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/SelfformingClay.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RunicCube.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RedSkull.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Sundial.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TheAbacus.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/GremlinHorn.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TheSpecimen.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BurningBlood.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BlackBlood.java
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state};

#[test]
fn dead_cleanup_wave8_runtime_tests_cover_deleted_helper_start_shuffle_and_death_families() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 10, 10)], 3);
    state.relics = vec!["Oddly Smooth Stone".to_string(), "PureWater".to_string()];

    let engine = engine_with_state(state);

    assert_eq!(engine.state.player.dexterity(), 1);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn dead_cleanup_wave8_runtime_tests_cover_deleted_helper_victory_family() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.relics = vec!["Burning Blood".to_string()];
    state.player.hp = 60;
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert!(engine.state.player_won);
    assert_eq!(engine.state.player.hp, 66);
}
