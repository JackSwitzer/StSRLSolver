#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Strike_Purple.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Prostrate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Pray.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyBody.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyMind.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyFist.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Evaluate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FollowUp.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FearNoEvil.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, Pile, SimpleEffect as SE, Target as T};
use crate::state::Stance;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave8_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let strike = registry.get("Strike").expect("Strike_P should be registered");
    assert_eq!(strike.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);

    let prostrate = registry.get("Prostrate").expect("Prostrate should be registered");
    assert_eq!(
        prostrate.effect_data,
        &[E::Simple(SE::GainMantra(A::Magic)), E::Simple(SE::GainBlock(A::Block))]
    );

    let pray = registry.get("Pray").expect("Pray should be registered");
    assert_eq!(
        pray.effect_data,
        &[
            E::Simple(SE::GainMantra(A::Magic)),
            E::Simple(SE::AddCard("Insight", Pile::Draw, A::Fixed(1))),
        ]
    );

    let empty_body = registry.get("EmptyBody").expect("EmptyBody should be registered");
    assert_eq!(
        empty_body.effect_data,
        &[E::Simple(SE::GainBlock(A::Block)), E::Simple(SE::ChangeStance(Stance::Neutral))]
    );

    let evaluate = registry.get("Evaluate").expect("Evaluate should be registered");
    assert_eq!(
        evaluate.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::AddCard("Insight", Pile::Draw, A::Fixed(1))),
        ]
    );

    let follow_up = registry.get("FollowUp").expect("FollowUp should be registered");
    assert_eq!(follow_up.effect_data[0], E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)));

    let fear_no_evil = registry.get("FearNoEvil").expect("FearNoEvil should be registered");
    assert_eq!(fear_no_evil.effect_data[0], E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)));
}

#[test]
fn watcher_wave8_strike_empty_fist_follow_up_and_fear_no_evil_follow_java_behavior() {
    let mut strike = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut strike, "Strike+");
    assert!(play_on_enemy(&mut strike, "Strike+", 0));
    assert_eq!(strike.state.enemies[0].entity.hp, 41);

    let mut empty_fist = one_enemy_engine("JawWorm", 60, 0);
    set_stance(&mut empty_fist, Stance::Wrath);
    ensure_in_hand(&mut empty_fist, "EmptyFist");
    assert!(play_on_enemy(&mut empty_fist, "EmptyFist", 0));
    assert_eq!(empty_fist.state.enemies[0].entity.hp, 42);
    assert_eq!(empty_fist.state.stance, Stance::Neutral);

    let mut follow_up = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut follow_up, "Strike");
    ensure_in_hand(&mut follow_up, "FollowUp+");
    assert!(play_on_enemy(&mut follow_up, "Strike", 0));
    let energy_before = follow_up.state.energy;
    assert!(play_on_enemy(&mut follow_up, "FollowUp+", 0));
    assert_eq!(follow_up.state.enemies[0].entity.hp, 33);
    assert_eq!(follow_up.state.energy, energy_before);

    let mut fear_no_evil = one_enemy_engine("JawWorm", 50, 10);
    ensure_in_hand(&mut fear_no_evil, "FearNoEvil+");
    assert!(play_on_enemy(&mut fear_no_evil, "FearNoEvil+", 0));
    assert_eq!(fear_no_evil.state.enemies[0].entity.hp, 39);
    assert_eq!(fear_no_evil.state.stance, Stance::Calm);
}

#[test]
fn watcher_wave8_prostrate_pray_and_evaluate_run_through_declared_effects() {
    let mut prostrate = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut prostrate, "Prostrate+");
    assert!(play_self(&mut prostrate, "Prostrate+"));
    assert_eq!(prostrate.state.mantra_gained, 3);
    assert_eq!(prostrate.state.player.block, 4);

    let mut pray = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut pray, "Pray");
    assert!(play_self(&mut pray, "Pray"));
    assert_eq!(pray.state.mantra_gained, 3);
    assert_eq!(draw_prefix_count(&pray, "Insight"), 1);

    let mut evaluate = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut evaluate, "Evaluate+");
    assert!(play_self(&mut evaluate, "Evaluate+"));
    assert_eq!(evaluate.state.player.block, 10);
    assert_eq!(draw_prefix_count(&evaluate, "Insight"), 1);
}

#[test]
fn watcher_wave8_empty_body_and_empty_mind_exit_stance_after_primary_effects() {
    let mut empty_body = one_enemy_engine("JawWorm", 60, 0);
    set_stance(&mut empty_body, Stance::Wrath);
    ensure_in_hand(&mut empty_body, "EmptyBody+");
    assert!(play_self(&mut empty_body, "EmptyBody+"));
    assert_eq!(empty_body.state.player.block, 10);
    assert_eq!(empty_body.state.stance, Stance::Neutral);

    let mut empty_mind = one_enemy_engine("JawWorm", 60, 0);
    set_stance(&mut empty_mind, Stance::Calm);
    empty_mind.state.draw_pile = make_deck(&["Strike", "Defend", "Vigilance"]);
    ensure_in_hand(&mut empty_mind, "EmptyMind+");
    let hand_before = empty_mind.state.hand.len();
    assert!(play_self(&mut empty_mind, "EmptyMind+"));
    assert_eq!(empty_mind.state.hand.len(), hand_before - 1 + 3);
    assert_eq!(empty_mind.state.stance, Stance::Neutral);
}
