#![cfg(test)]

use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with, engine_with_state, ensure_in_hand,
    make_deck, play_on_enemy,
};

#[test]
fn envenom_applies_poison_after_positive_attack_damage_on_engine_path() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/powers/EnvenomPower.java
    let state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    let mut engine = engine_with_state(state);
    engine.state.player.set_status(sid::ENVENOM, 1);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 1);
}

#[test]
fn sadistic_nature_deals_damage_when_player_applies_a_debuff() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SadisticNature.java
    let mut engine = engine_with(make_deck(&["Strike"; 10]), 30, 0);
    engine.state.player.set_status(sid::SADISTIC, 5);
    engine.rebuild_effect_runtime();
    ensure_in_hand(&mut engine, "Trip");

    assert!(play_on_enemy(&mut engine, "Trip", 0));
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 25);
}

#[test]
fn the_specimen_uses_combat_rng_instead_of_always_picking_first_alive_enemy() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/TheSpecimen.java
    let mut hit_left = false;
    let mut hit_right = false;

    for seed in 0..32 {
        let mut state = combat_state_with(
            make_deck(&["Strike"]),
            vec![
                enemy_no_intent("JawWorm", 20, 20),
                enemy_no_intent("Cultist", 20, 20),
                enemy_no_intent("Louse", 20, 20),
            ],
            3,
        );
        state.relics.push("The Specimen".to_string());
        let mut engine = crate::engine::CombatEngine::new(state, seed);

        engine.state.enemies[1].entity.set_status(sid::POISON, 7);
        engine.state.enemies[1].entity.hp = 0;
        engine.finalize_enemy_death(1);

        hit_left |= engine.state.enemies[0].entity.status(sid::POISON) == 7;
        hit_right |= engine.state.enemies[2].entity.status(sid::POISON) == 7;

        if hit_left && hit_right {
            break;
        }
    }

    assert!(hit_left, "expected at least one seed to target the left enemy");
    assert!(hit_right, "expected at least one seed to target the right enemy");
}

#[test]
fn preserved_insect_reduces_all_elite_enemies_to_seventy_five_percent_of_max_hp() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/PreservedInsect.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![
            enemy_no_intent("Sentry", 40, 40),
            enemy_no_intent("Sentry", 36, 36),
        ],
        3,
    );
    state.relics.push("PreservedInsect".to_string());

    let engine = engine_with_state(state);

    assert_eq!(engine.state.enemies[0].entity.hp, 30);
    assert_eq!(engine.state.enemies[1].entity.hp, 27);
}

#[test]
fn centennial_puzzle_draws_three_cards_only_on_first_hp_loss() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
    let mut state = combat_state_with(
        make_deck(&["Strike"; 10]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Centennial Puzzle".to_string());
    let mut engine = engine_with_state(state);
    assert_eq!(
        engine.hidden_effect_value(
            "Centennial Puzzle",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        1
    );
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&[
        "Strike", "Defend", "Bash", "Strike", "Defend", "Strike",
    ]);

    engine.player_lose_hp(4);
    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(
        engine.hidden_effect_value(
            "Centennial Puzzle",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        0
    );

    engine.player_lose_hp(4);
    assert_eq!(engine.state.hand.len(), 3);
}

#[test]
fn red_skull_gains_and_loses_strength_when_crossing_bloodied_threshold() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/RedSkull.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Red Skull".to_string());
    state.player.hp = 50;
    let mut engine = engine_with_state(state);

    assert_eq!(
        engine.hidden_effect_value(
            "Red Skull",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        0
    );
    engine.player_lose_hp(11);
    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        engine.hidden_effect_value(
            "Red Skull",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        1
    );

    engine.heal_player(5);
    assert_eq!(engine.state.player.strength(), 0);
}

#[test]
fn du_vu_doll_applies_strength_from_current_curse_count_on_combat_start() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/DuVuDoll.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Du-Vu Doll".to_string());
    state.player.set_status(sid::DU_VU_DOLL_CURSES, 3);

    let mut engine = engine_with_state(state);

    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        engine.hidden_effect_value(
            "Du-Vu Doll",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        3
    );
    engine.state.player.set_status(sid::DU_VU_DOLL_CURSES, 0);
    engine.rebuild_effect_runtime();
    assert_eq!(
        engine.hidden_effect_value(
            "Du-Vu Doll",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        3
    );
}

#[test]
fn girya_applies_strength_from_lift_counter_on_combat_start() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/Girya.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Girya".to_string());
    state.player.set_status(sid::GIRYA_COUNTER, 2);

    let mut engine = engine_with_state(state);

    assert_eq!(engine.state.player.strength(), 2);
    assert_eq!(
        engine.hidden_effect_value(
            "Girya",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        2
    );
    engine.state.player.set_status(sid::GIRYA_COUNTER, 0);
    engine.rebuild_effect_runtime();
    assert_eq!(
        engine.hidden_effect_value(
            "Girya",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        2
    );
}

#[test]
fn slavers_collar_grants_one_extra_energy_on_boss_turn_start() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/SlaversCollar.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("Hexaghost", 30, 30)],
        3,
    );
    state.relics.push("SlaversCollar".to_string());

    let engine = engine_with_state(state);

    assert_eq!(engine.state.energy, 4);
}

#[test]
fn red_skull_hidden_state_is_runtime_owned() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/RedSkull.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Red Skull".to_string());
    state.player.hp = 50;
    let mut engine = engine_with_state(state);

    engine.player_lose_hp(11);
    assert_eq!(
        engine.hidden_effect_value(
            "Red Skull",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        1
    );
    engine.heal_player(5);
    assert_eq!(
        engine.hidden_effect_value(
            "Red Skull",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        0
    );
}

#[test]
fn centennial_puzzle_hidden_state_is_runtime_owned() {
    // Java oracle: decompiled/java-src/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
    let mut state = combat_state_with(
        make_deck(&["Strike"; 10]),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    state.relics.push("Centennial Puzzle".to_string());
    let mut engine = engine_with_state(state);

    assert_eq!(
        engine.hidden_effect_value(
            "Centennial Puzzle",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        1
    );
    engine.player_lose_hp(4);
    assert_eq!(
        engine.hidden_effect_value(
            "Centennial Puzzle",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 0 },
            0,
        ),
        0
    );
}
