#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/RedMask.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PhilosopherStone.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
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
use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, make_deck,
    make_deck_n,
};

#[test]
fn relic_wave12_runtime_combat_start_buffs_and_debuffs_replace_helper_contracts() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("Cultist", 24, 24), enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec![
        "Oddly Smooth Stone".to_string(),
        "Red Mask".to_string(),
        "Philosopher's Stone".to_string(),
    ];

    let engine = engine_with_state(state);

    assert_eq!(engine.state.player.dexterity(), 1);
    assert!(engine.state.enemies.iter().all(|enemy| enemy.entity.is_weak()));
    assert!(engine.state.enemies.iter().all(|enemy| enemy.entity.strength() == 1));
}

#[test]
fn relic_wave12_runtime_combat_start_temp_cards_replace_helper_contracts() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics = vec!["PureWater".to_string()];

    let engine = engine_with_state(state);

    let names: Vec<_> = engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(names.iter().filter(|name| **name == "Miracle").count(), 1);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn relic_wave12_runtime_hp_loss_families_replace_helper_contracts() {
    let mut state = combat_state_with(
        make_deck(&["Strike_R"; 10]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics = vec![
        "Centennial Puzzle".to_string(),
        "Self Forming Clay".to_string(),
        "Runic Cube".to_string(),
        "Red Skull".to_string(),
    ];
    state.player.hp = 50;

    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&[
        "Strike_R",
        "Defend_R",
        "Bash",
        "Strike_R",
        "Defend_R",
        "Strike_R",
    ]);

    engine.player_lose_hp(11);

    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.player.status(sid::NEXT_TURN_BLOCK), 3);
    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        engine.hidden_effect_value("Centennial Puzzle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 3 }, 0),
        1
    );

    engine.player_lose_hp(4);

    assert_eq!(engine.state.hand.len(), 5);
}

#[test]
fn relic_wave12_runtime_shuffle_and_enemy_death_families_replace_helper_contracts() {
    let mut shuffle = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    shuffle.state.relics = vec!["Sundial".to_string(), "TheAbacus".to_string()];
    shuffle.rebuild_effect_runtime();

    for expected_shuffle in [1, 2, 0] {
        shuffle.state.hand.clear();
        shuffle.state.draw_pile.clear();
        shuffle.state.discard_pile = make_deck(&["Strike_R"]);
        shuffle.draw_cards(1);
        assert_eq!(
            shuffle.hidden_effect_value("Sundial", EffectOwner::PlayerRelic { slot: 0 }, 0),
            expected_shuffle
        );
    }
    assert_eq!(shuffle.state.energy, 5);
    assert_eq!(shuffle.state.player.block, 18);

    let mut state = combat_state_with(
        make_deck_n("Strike_G", 3),
        vec![enemy_no_intent("JawWorm", 10, 10), enemy_no_intent("Cultist", 30, 30)],
        3,
    );
    state.relics = vec!["Gremlin Horn".to_string(), "The Specimen".to_string()];
    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike_G"]);
    engine.state.enemies[0].entity.set_status(sid::POISON, 5);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);

    assert_eq!(engine.state.energy, 4);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 5);
}

#[test]
fn relic_wave12_runtime_victory_families_replace_helper_contracts() {
    let mut burn_state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    burn_state.relics = vec!["Burning Blood".to_string()];
    burn_state.player.hp = 60;
    let mut burn_engine = engine_with_state(burn_state);
    burn_engine.state.enemies[0].entity.hp = 0;

    burn_engine.finalize_enemy_death(0);
    burn_engine.check_combat_end();

    assert!(burn_engine.state.combat_over);
    assert!(burn_engine.state.player_won);
    assert_eq!(burn_engine.state.player.hp, 66);
    let mut black_state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    black_state.relics = vec!["Black Blood".to_string()];
    black_state.player.hp = 60;
    let mut black_engine = engine_with_state(black_state);
    black_engine.state.enemies[0].entity.hp = 0;

    black_engine.finalize_enemy_death(0);
    black_engine.check_combat_end();

    assert!(black_engine.state.combat_over);
    assert!(black_engine.state.player_won);
    assert_eq!(black_engine.state.player.hp, 72);
}
