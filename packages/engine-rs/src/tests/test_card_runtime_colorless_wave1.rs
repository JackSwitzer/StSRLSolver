#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/DramaticEntrance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/GoodInstincts.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Magnetism.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Mayhem.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panache.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SadisticNature.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SwiftStrike.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn colorless_wave1_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let dramatic_entrance = registry
        .get("Dramatic Entrance")
        .expect("Dramatic Entrance should exist");
    assert_eq!(dramatic_entrance.card_type, CardType::Attack);
    assert_eq!(dramatic_entrance.target, CardTarget::AllEnemy);
    assert!(dramatic_entrance.exhaust);
    assert!(dramatic_entrance.has_test_marker("innate"));
    assert_eq!(
        dramatic_entrance.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let good_instincts = registry
        .get("Good Instincts")
        .expect("Good Instincts should exist");
    assert_eq!(good_instincts.card_type, CardType::Skill);
    assert_eq!(good_instincts.target, CardTarget::SelfTarget);
    assert_eq!(good_instincts.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let swift_strike = registry
        .get("Swift Strike")
        .expect("Swift Strike should exist");
    assert_eq!(swift_strike.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);

    let magnetism = registry.get("Magnetism").expect("Magnetism should exist");
    assert_eq!(
        magnetism.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::MAGNETISM, A::Magic))]
    );

    let mayhem = registry.get("Mayhem").expect("Mayhem should exist");
    assert_eq!(
        mayhem.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::MAYHEM, A::Magic))]
    );

    let panache = registry.get("Panache").expect("Panache should exist");
    assert_eq!(
        panache.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::PANACHE, A::Magic))]
    );

    let sadistic = registry
        .get("Sadistic Nature")
        .expect("Sadistic Nature should exist");
    assert_eq!(
        sadistic.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::SADISTIC, A::Magic))]
    );
}

#[test]
fn colorless_wave1_attack_and_block_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50), enemy_no_intent("Cultist", 40, 40)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Swift Strike");
    assert!(play_on_enemy(&mut engine, "Swift Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 43);

    ensure_in_hand(&mut engine, "Swift Strike+");
    assert!(play_on_enemy(&mut engine, "Swift Strike+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 33);

    ensure_in_hand(&mut engine, "Good Instincts");
    assert!(play_self(&mut engine, "Good Instincts"));
    assert_eq!(engine.state.player.block, 6);

    ensure_in_hand(&mut engine, "Good Instincts+");
    assert!(play_self(&mut engine, "Good Instincts+"));
    assert_eq!(engine.state.player.block, 15);

    ensure_in_hand(&mut engine, "Dramatic Entrance");
    assert!(play_on_enemy(&mut engine, "Dramatic Entrance", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);
    assert_eq!(engine.state.enemies[1].entity.hp, 32);
    assert_eq!(exhaust_prefix_count(&engine, "Dramatic Entrance"), 1);

    ensure_in_hand(&mut engine, "Dramatic Entrance+");
    assert!(play_on_enemy(&mut engine, "Dramatic Entrance+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 13);
    assert_eq!(engine.state.enemies[1].entity.hp, 20);
    assert_eq!(exhaust_prefix_count(&engine, "Dramatic Entrance"), 2);
}

#[test]
fn colorless_wave1_power_cards_install_runtime_owned_statuses() {
    let mut magnetism = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut magnetism);
    ensure_in_hand(&mut magnetism, "Magnetism");
    assert!(play_self(&mut magnetism, "Magnetism"));
    assert_eq!(magnetism.state.player.status(sid::MAGNETISM), 1);

    let mut mayhem = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut mayhem);
    ensure_in_hand(&mut mayhem, "Mayhem+");
    assert!(play_self(&mut mayhem, "Mayhem+"));
    assert_eq!(mayhem.state.player.status(sid::MAYHEM), 1);

    let mut panache = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut panache);
    ensure_in_hand(&mut panache, "Panache+");
    assert!(play_self(&mut panache, "Panache+"));
    assert_eq!(panache.state.player.status(sid::PANACHE), 14);

    let mut sadistic = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        10,
    );
    force_player_turn(&mut sadistic);
    ensure_in_hand(&mut sadistic, "Sadistic Nature");
    assert!(play_self(&mut sadistic, "Sadistic Nature"));
    assert_eq!(sadistic.state.player.status(sid::SADISTIC), 5);
}
