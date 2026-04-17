#![cfg(test)]

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

#[test]
fn backend_wave3_registry_exports_use_typed_primary_preamble_ops() {
    let registry = global_registry();

    let strike_r = registry.get("Strike_R").expect("Strike_R should exist");
    assert_eq!(
        strike_r.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(strike_r.uses_typed_primary_preamble());

    let strike_r_plus = registry.get("Strike_R+").expect("Strike_R+ should exist");
    assert_eq!(
        strike_r_plus.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let defend_r = registry.get("Defend_R").expect("Defend_R should exist");
    assert_eq!(defend_r.card_type, CardType::Skill);
    assert_eq!(defend_r.target, CardTarget::SelfTarget);
    assert_eq!(defend_r.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let cleave = registry.get("Cleave").expect("Cleave should exist");
    assert_eq!(
        cleave.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let ghostly_armor = registry.get("Ghostly Armor").expect("Ghostly Armor should exist");
    assert_eq!(
        ghostly_armor.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );
    assert!(ghostly_armor.has_test_marker("ethereal"));

    let dagger_spray = registry.get("Dagger Spray").expect("Dagger Spray should exist");
    assert_eq!(
        dagger_spray.effect_data,
        &[
            E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
            E::ExtraHits(A::Magic),
        ]
    );

    let deflect = registry.get("Deflect").expect("Deflect should exist");
    assert_eq!(deflect.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let leap = registry.get("Leap").expect("Leap should exist");
    assert_eq!(leap.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let boot_sequence = registry.get("BootSequence").expect("BootSequence should exist");
    assert_eq!(boot_sequence.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(boot_sequence.has_test_marker("innate"));
    assert!(boot_sequence.exhaust);

    let defend_p = registry.get("Defend_P").expect("Defend_P should exist");
    assert_eq!(defend_p.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let consecrate = registry.get("Consecrate").expect("Consecrate should exist");
    assert_eq!(
        consecrate.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );
}

#[test]
fn backend_wave3_typed_attack_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 40, 40)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Strike_R");
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 34);

    ensure_in_hand(&mut engine, "Strike_R+");
    assert!(play_on_enemy(&mut engine, "Strike_R+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);

    ensure_in_hand(&mut engine, "Cleave");
    assert!(play_on_enemy(&mut engine, "Cleave", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 17);
    assert_eq!(engine.state.enemies[1].entity.hp, 32);

    ensure_in_hand(&mut engine, "Consecrate");
    assert!(play_on_enemy(&mut engine, "Consecrate", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 12);
    assert_eq!(engine.state.enemies[1].entity.hp, 27);

    ensure_in_hand(&mut engine, "Dagger Spray");
    assert!(play_on_enemy(&mut engine, "Dagger Spray", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 4);
    assert_eq!(engine.state.enemies[1].entity.hp, 19);
}

#[test]
fn backend_wave3_typed_block_and_ethereal_cards_follow_java_oracle_on_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Defend_R");
    assert!(play_self(&mut engine, "Defend_R"));
    assert_eq!(engine.state.player.block, 5);

    ensure_in_hand(&mut engine, "Defend_R+");
    assert!(play_self(&mut engine, "Defend_R+"));
    assert_eq!(engine.state.player.block, 13);

    ensure_in_hand(&mut engine, "Deflect");
    assert!(play_self(&mut engine, "Deflect"));
    assert_eq!(engine.state.player.block, 17);

    ensure_in_hand(&mut engine, "Leap");
    assert!(play_self(&mut engine, "Leap"));
    assert_eq!(engine.state.player.block, 26);

    ensure_in_hand(&mut engine, "Defend_P");
    assert!(play_self(&mut engine, "Defend_P"));
    assert_eq!(engine.state.player.block, 31);

    ensure_in_hand(&mut engine, "BootSequence");
    assert!(play_self(&mut engine, "BootSequence"));
    assert_eq!(engine.state.player.block, 41);
    assert_eq!(exhaust_prefix_count(&engine, "BootSequence"), 1);

    ensure_in_hand(&mut engine, "Ghostly Armor");
    assert!(play_self(&mut engine, "Ghostly Armor"));
    assert_eq!(engine.state.player.block, 51);
    // Ghostly Armor only exhausts if it is still in hand at end of turn.
    ensure_in_hand(&mut engine, "Ghostly Armor");
    end_turn(&mut engine);
    assert_eq!(exhaust_prefix_count(&engine, "Ghostly Armor"), 1);
}
