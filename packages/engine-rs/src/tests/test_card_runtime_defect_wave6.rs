#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Strike_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Defend_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Leap.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SelfRepair.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/BootSequence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/MachineLearning.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ReinforcedBody.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/StaticDischarge.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::effects::types::{CardBlockHint, CardRuntimeTraits};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave6_registry_exports_honest_runtime_surface() {
    let strike = global_registry().get("Strike_B+").expect("Strike_B+ should exist");
    assert_eq!(strike.card_type, CardType::Attack);
    assert_eq!(strike.target, CardTarget::Enemy);
    assert_eq!(strike.base_damage, 9);
    assert_eq!(
        strike.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let defend = global_registry().get("Defend_B+").expect("Defend_B+ should exist");
    assert_eq!(defend.card_type, CardType::Skill);
    assert_eq!(defend.target, CardTarget::SelfTarget);
    assert_eq!(defend.base_block, 8);
    assert_eq!(defend.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let leap = global_registry().get("Leap+").expect("Leap+ should exist");
    assert_eq!(leap.base_block, 12);
    assert_eq!(leap.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let self_repair = global_registry().get("Self Repair+").expect("Self Repair+ should exist");
    assert_eq!(
        self_repair.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::SELF_REPAIR, A::Magic))]
    );
    assert_eq!(self_repair.runtime_traits(), CardRuntimeTraits::default());

    let boot_sequence = global_registry().get("BootSequence+").expect("BootSequence+ should exist");
    assert!(boot_sequence.runtime_traits().innate);
    assert!(boot_sequence.exhaust);
    assert_eq!(boot_sequence.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let machine_learning = global_registry()
        .get("Machine Learning+")
        .expect("Machine Learning+ should exist");
    assert_eq!(
        machine_learning.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::DRAW, A::Magic))]
    );
    assert!(machine_learning.runtime_traits().innate);

    let reinforced_body = global_registry()
        .get("Reinforced Body+")
        .expect("Reinforced Body+ should exist");
    assert_eq!(reinforced_body.cost, -1);
    assert_eq!(reinforced_body.base_block, 9);
    assert_eq!(
        reinforced_body.play_hints().block_hint,
        Some(CardBlockHint::XTimes)
    );
    assert_eq!(reinforced_body.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let static_discharge = global_registry()
        .get("Static Discharge+")
        .expect("Static Discharge+ should exist");
    assert_eq!(
        static_discharge.effect_data,
        &[E::Simple(SE::AddStatus(
            T::Player,
            sid::STATIC_DISCHARGE,
            A::Magic,
        ))]
    );
    assert_eq!(static_discharge.runtime_traits(), CardRuntimeTraits::default());
}

#[test]
fn defect_wave6_strike_defend_leap_boot_sequence_and_reinforced_body_follow_engine_path() {
    let mut strike = one_enemy_engine(20);
    ensure_in_hand(&mut strike, "Strike_B+");
    assert!(play_on_enemy(&mut strike, "Strike_B+", 0));
    assert_eq!(strike.state.enemies[0].entity.hp, 11);

    let mut defend = one_enemy_engine(20);
    ensure_in_hand(&mut defend, "Defend_B+");
    assert!(play_self(&mut defend, "Defend_B+"));
    assert_eq!(defend.state.player.block, 8);

    let mut leap = one_enemy_engine(20);
    ensure_in_hand(&mut leap, "Leap+");
    assert!(play_self(&mut leap, "Leap+"));
    assert_eq!(leap.state.player.block, 12);

    let mut boot_sequence = one_enemy_engine(20);
    boot_sequence.state.hand = make_deck(&["BootSequence+"]);
    assert!(play_self(&mut boot_sequence, "BootSequence+"));
    assert_eq!(boot_sequence.state.player.block, 13);
    assert_eq!(exhaust_prefix_count(&boot_sequence, "BootSequence"), 1);

    let mut reinforced_body = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 20, 20)], 5);
    force_player_turn(&mut reinforced_body);
    reinforced_body.state.energy = 4;
    reinforced_body.state.hand = make_deck(&["Reinforced Body+"]);
    assert!(play_self(&mut reinforced_body, "Reinforced Body+"));
    assert_eq!(reinforced_body.state.player.block, 36);
    assert_eq!(reinforced_body.state.energy, 0);
}

#[test]
fn defect_wave6_self_repair_machine_learning_and_static_discharge_install_runtime_statuses() {
    let mut self_repair = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 6, 6)],
        3,
    );
    force_player_turn(&mut self_repair);
    self_repair.state.player.hp = 40;
    self_repair.state.hand = make_deck(&["Self Repair+", "Strike_B"]);
    assert!(play_self(&mut self_repair, "Self Repair+"));
    assert_eq!(self_repair.state.player.status(sid::SELF_REPAIR), 10);
    assert!(play_on_enemy(&mut self_repair, "Strike_B", 0));
    assert!(self_repair.state.player_won);
    assert_eq!(self_repair.state.player.hp, 40);

    let mut machine_learning = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 30, 30)],
        3,
    );
    force_player_turn(&mut machine_learning);
    machine_learning.state.hand = make_deck(&["Machine Learning+"]);
    machine_learning.state.draw_pile = make_deck(&[
        "Strike_B",
        "Defend_B",
        "Zap",
        "Dualcast",
        "Strike_B",
        "Defend_B",
    ]);
    assert!(play_self(&mut machine_learning, "Machine Learning+"));
    assert_eq!(machine_learning.state.player.status(sid::DRAW), 1);
    end_turn(&mut machine_learning);
    assert_eq!(machine_learning.state.hand.len(), 6);

    let mut static_discharge = engine_without_start(
        Vec::new(),
        vec![enemy("JawWorm", 40, 40, 1, 5, 1)],
        3,
    );
    force_player_turn(&mut static_discharge);
    static_discharge.init_defect_orbs(2);
    static_discharge.state.hand = make_deck(&["Static Discharge+"]);
    assert!(play_self(&mut static_discharge, "Static Discharge+"));
    assert_eq!(static_discharge.state.player.status(sid::STATIC_DISCHARGE), 2);
    end_turn(&mut static_discharge);
    assert_eq!(static_discharge.state.player.hp, 75);
    assert_eq!(static_discharge.state.orb_slots.occupied_count(), 2);
    assert_eq!(static_discharge.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(static_discharge.state.orb_slots.slots[1].orb_type, OrbType::Lightning);
}
