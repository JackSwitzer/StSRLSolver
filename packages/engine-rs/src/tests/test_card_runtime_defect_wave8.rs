#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Strike_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Defend_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Storm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Capacitor.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ForceField.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Rebound.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self,
};

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave8_registry_exports_match_typed_runtime_progress() {
    let reg = global_registry();

    let strike_b = reg.get("Strike_B").expect("Strike_B");
    assert_eq!(strike_b.card_type, CardType::Attack);
    assert_eq!(strike_b.target, CardTarget::Enemy);
    assert_eq!(
        strike_b.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(strike_b.uses_typed_primary_preamble());

    let defend_b = reg.get("Defend_B").expect("Defend_B");
    assert_eq!(defend_b.card_type, CardType::Skill);
    assert_eq!(defend_b.target, CardTarget::SelfTarget);
    assert_eq!(defend_b.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(defend_b.uses_typed_primary_preamble());

    let storm = reg.get("Storm+").expect("Storm+");
    assert_eq!(storm.card_type, CardType::Power);
    assert_eq!(
        storm.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))]
    );
    assert_eq!(storm.test_markers(), vec!["innate"]);

    let force_field = reg.get("Force Field+").expect("Force Field+");
    assert_eq!(force_field.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(force_field.has_test_marker("reduce_cost_per_power"));
    assert!(force_field.uses_typed_primary_preamble());

    let rebound = reg.get("Rebound+").expect("Rebound+");
    assert_eq!(
        rebound.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(rebound.has_test_marker("next_card_to_top"));
    assert!(rebound.uses_typed_primary_preamble());
}

#[test]
fn defect_wave8_basic_attack_and_block_cards_follow_engine_path() {
    let mut engine = one_enemy_engine(40, 10);
    engine.state.hand = make_deck(&["Strike_B", "Strike_B+", "Defend_B", "Defend_B+"]);

    assert!(play_on_enemy(&mut engine, "Strike_B", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 34);

    assert!(play_on_enemy(&mut engine, "Strike_B+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);

    assert!(play_self(&mut engine, "Defend_B"));
    assert_eq!(engine.state.player.block, 5);

    assert!(play_self(&mut engine, "Defend_B+"));
    assert_eq!(engine.state.player.block, 13);
}

#[test]
fn defect_wave8_storm_force_field_and_rebound_follow_engine_path() {
    let mut storm = one_enemy_engine(40, 10);
    storm.init_defect_orbs(1);
    storm.state.hand = make_deck(&["Storm", "Defragment"]);
    assert!(play_self(&mut storm, "Storm"));
    assert_eq!(storm.state.player.status(sid::STORM), 1);
    assert!(play_self(&mut storm, "Defragment"));
    assert_eq!(storm.state.player.status(sid::STORM), 1);
    assert_eq!(storm.state.orb_slots.slots[0].orb_type, OrbType::Lightning);

    let mut force_field = one_enemy_engine(60, 6);
    force_field.state.hand = make_deck(&["Heatsinks", "Hello World", "Force Field+"]);
    assert!(play_self(&mut force_field, "Heatsinks"));
    assert!(play_self(&mut force_field, "Hello World"));
    force_field.state.energy = 2;
    assert!(play_self(&mut force_field, "Force Field+"));
    assert_eq!(force_field.state.player.block, 16);
    assert_eq!(force_field.state.energy, 0);

    let mut rebound = one_enemy_engine(40, 3);
    rebound.state.hand = make_deck(&["Rebound"]);
    assert!(play_on_enemy(&mut rebound, "Rebound", 0));
    assert_eq!(rebound.state.enemies[0].entity.hp, 31);
}
