#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Strike_Green.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Defend_Green.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Slice.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Dash.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/DieDieDie.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Backstab.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

#[test]
fn silent_wave8_registry_exports_match_typed_primary_surface() {
    let registry = global_registry();

    let strike = registry.get("Strike").expect("Strike_G should exist");
    assert_eq!(strike.card_type, CardType::Attack);
    assert_eq!(strike.target, CardTarget::Enemy);
    assert_eq!(
        strike.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let defend = registry.get("Defend").expect("Defend_G should exist");
    assert_eq!(defend.card_type, CardType::Skill);
    assert_eq!(defend.target, CardTarget::SelfTarget);
    assert_eq!(defend.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let slice = registry.get("Slice").expect("Slice should exist");
    assert_eq!(
        slice.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let dash = registry.get("Dash").expect("Dash should exist");
    assert_eq!(
        dash.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );

    let die_die_die = registry.get("Die Die Die").expect("Die Die Die should exist");
    assert!(die_die_die.exhaust);
    assert_eq!(
        die_die_die.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let backstab = registry.get("Backstab").expect("Backstab should exist");
    assert!(backstab.exhaust);
    assert!(backstab.has_test_marker("innate"));
    assert_eq!(
        backstab.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
}

#[test]
fn silent_wave8_single_target_typed_attacks_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Strike");
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 54);

    ensure_in_hand(&mut engine, "Strike+");
    assert!(play_on_enemy(&mut engine, "Strike+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 45);

    ensure_in_hand(&mut engine, "Slice");
    assert!(play_on_enemy(&mut engine, "Slice", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 39);

    ensure_in_hand(&mut engine, "Slice+");
    assert!(play_on_enemy(&mut engine, "Slice+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 30);

    ensure_in_hand(&mut engine, "Backstab+");
    assert!(play_on_enemy(&mut engine, "Backstab+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 15);
    assert_eq!(exhaust_prefix_count(&engine, "Backstab"), 1);
}

#[test]
fn silent_wave8_block_and_aoe_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 40, 40)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Defend");
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 5);

    ensure_in_hand(&mut engine, "Defend+");
    assert!(play_self(&mut engine, "Defend+"));
    assert_eq!(engine.state.player.block, 13);

    ensure_in_hand(&mut engine, "Dash");
    assert!(play_on_enemy(&mut engine, "Dash", 0));
    assert_eq!(engine.state.player.block, 23);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);

    ensure_in_hand(&mut engine, "Dash+");
    assert!(play_on_enemy(&mut engine, "Dash+", 0));
    assert_eq!(engine.state.player.block, 36);
    assert_eq!(engine.state.enemies[0].entity.hp, 17);

    ensure_in_hand(&mut engine, "Die Die Die");
    assert!(play_on_enemy(&mut engine, "Die Die Die", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 4);
    assert_eq!(engine.state.enemies[1].entity.hp, 27);
    assert_eq!(exhaust_prefix_count(&engine, "Die Die Die"), 1);

    ensure_in_hand(&mut engine, "Die Die Die+");
    assert!(play_on_enemy(&mut engine, "Die Die Die+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 0);
    assert_eq!(engine.state.enemies[1].entity.hp, 10);
    assert_eq!(exhaust_prefix_count(&engine, "Die Die Die"), 2);
}
