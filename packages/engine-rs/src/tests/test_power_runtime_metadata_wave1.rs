#![cfg(test)]

use crate::combat_types::mfx;
use crate::effects::runtime::EffectOwner;
use crate::enemies::create_enemy;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, engine_with_state, make_deck, play_self,
};

#[test]
fn power_card_install_rebuilds_runtime_without_legacy_tag_lookup() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![create_enemy("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Demon Form"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Demon Form"));

    assert!(engine.state.player.status(sid::DEMON_FORM) > 0);
    assert!(engine
        .effect_runtime
        .has_instance("demon_form", EffectOwner::PlayerPower));
}

#[test]
fn force_field_cost_scales_from_runtime_owned_player_powers() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![create_enemy("JawWorm", 50, 50)],
        6,
    ));
    engine.state.hand = make_deck(&["Demon Form", "Noxious Fumes", "Force Field"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Demon Form"));
    assert!(play_self(&mut engine, "Noxious Fumes"));

    assert!(engine
        .effect_runtime
        .has_instance("demon_form", EffectOwner::PlayerPower));
    assert!(engine
        .effect_runtime
        .has_instance("noxious_fumes", EffectOwner::PlayerPower));
    assert_eq!(engine.state.energy, 2);

    assert!(play_self(&mut engine, "Force Field"));

    assert_eq!(engine.state.player.block, 12);
    assert_eq!(engine.state.energy, 0);
}

#[test]
fn time_eater_haste_clears_only_debuffs_via_enemy_turn_path() {
    let mut time_eater = create_enemy("TimeEater", 150, 456);
    time_eater.set_move(999, 0, 0, 0);
    time_eater.add_effect(mfx::REMOVE_DEBUFFS, 1);
    time_eater.add_effect(mfx::HEAL_TO_HALF, 1);
    time_eater.entity.set_status(sid::VULNERABLE, 2);
    time_eater.entity.set_status(sid::POISON, 5);
    time_eater.entity.set_status(sid::STRENGTH, 4);
    time_eater.entity.set_status(sid::ARTIFACT, 1);

    let mut engine = engine_with_state(combat_state_with(Vec::new(), vec![time_eater], 3));

    end_turn(&mut engine);

    let enemy = &engine.state.enemies[0].entity;
    assert_eq!(enemy.status(sid::VULNERABLE), 0);
    assert_eq!(enemy.status(sid::POISON), 0);
    assert_eq!(enemy.status(sid::STRENGTH), 4);
    assert_eq!(enemy.status(sid::ARTIFACT), 1);
    assert_eq!(enemy.hp, 228);
}

#[test]
fn awakened_one_rebirth_clears_only_debuffs_via_enemy_turn_path() {
    let mut awakened_one = create_enemy("AwakenedOne", 50, 300);
    awakened_one.entity.set_status(sid::REBIRTH_PENDING, 1);
    awakened_one.entity.set_status(sid::VULNERABLE, 2);
    awakened_one.entity.set_status(sid::WEAKENED, 1);
    awakened_one.entity.set_status(sid::STRENGTH, 3);

    let mut engine = engine_with_state(combat_state_with(Vec::new(), vec![awakened_one], 3));

    end_turn(&mut engine);

    let enemy = &engine.state.enemies[0].entity;
    assert_eq!(enemy.status(sid::PHASE), 2);
    assert_eq!(enemy.status(sid::VULNERABLE), 0);
    assert_eq!(enemy.status(sid::WEAKENED), 0);
    assert_eq!(enemy.status(sid::STRENGTH), 3);
    assert_eq!(enemy.hp, enemy.max_hp);
}
