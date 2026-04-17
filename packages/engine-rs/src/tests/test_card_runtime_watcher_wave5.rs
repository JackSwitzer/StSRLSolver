#![cfg(test)]

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

fn two_enemy_engine(
    a: (&str, i32, i32),
    b: (&str, i32, i32),
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy(a.0, a.1, a.1, 1, a.2, 1),
            enemy(b.0, b.1, b.1, 1, b.2, 1),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave5_registry_exports_match_runtime_surface() {
    let conclude = global_registry().get("Conclude").expect("Conclude should be registered");
    assert_eq!(conclude.card_type, CardType::Attack);
    assert_eq!(conclude.target, CardTarget::AllEnemy);
    assert_eq!(
        conclude.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );
    assert!(conclude
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, crate::effects::types::CardRuntimeTrigger::PostPlay(crate::effects::types::PostPlayRule::EndTurn))));

    let consecrate = global_registry().get("Consecrate").expect("Consecrate should be registered");
    assert_eq!(consecrate.card_type, CardType::Attack);
    assert_eq!(consecrate.target, CardTarget::AllEnemy);
    assert_eq!(
        consecrate.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let crescendo = global_registry().get("Crescendo").expect("Crescendo should be registered");
    assert_eq!(
        crescendo.effect_data,
        &[E::Simple(SE::ChangeStance(Stance::Wrath))]
    );
    assert_eq!(crescendo.enter_stance, Some("Wrath"));
    assert!(crescendo.runtime_traits().retain);

    let establishment = global_registry()
        .get("Establishment+")
        .expect("Establishment+ should be registered");
    assert_eq!(
        establishment.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::ESTABLISHMENT, A::Magic))]
    );
    assert!(establishment.runtime_traits().innate);

    let fasting = global_registry().get("Fasting2").expect("Fasting should be registered");
    assert_eq!(
        fasting.effect_data,
        &[
            E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
            E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
            E::Simple(SE::ModifyMaxEnergy(A::Fixed(-1))),
        ]
    );
    assert!(fasting.complex_hook.is_none());

    let holy_water = global_registry().get("HolyWater").expect("Holy Water should be registered");
    assert_eq!(holy_water.base_block, 5);
    assert!(holy_water.runtime_traits().retain);
    assert_eq!(holy_water.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let judgement = global_registry().get("Judgement").expect("Judgement should be registered");
    assert_eq!(
        judgement.effect_data,
        &[E::Simple(SE::Judgement(A::Magic))]
    );
    assert!(judgement.complex_hook.is_none());
}

#[test]
fn watcher_wave5_conclude_and_consecrate_run_on_the_engine_path() {
    let mut conclude = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut conclude, "Conclude");
    let turn_before = conclude.state.turn;
    assert!(play_on_enemy(&mut conclude, "Conclude", 0));
    assert_eq!(conclude.state.turn, turn_before + 1);
    assert_eq!(conclude.state.enemies[0].entity.hp, 38);
    assert_eq!(conclude.state.enemies[1].entity.hp, 38);

    let mut consecrate = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut consecrate, "Consecrate+");
    assert!(play_on_enemy(&mut consecrate, "Consecrate+", 0));
    assert_eq!(consecrate.state.enemies[0].entity.hp, 42);
    assert_eq!(consecrate.state.enemies[1].entity.hp, 42);
}

#[test]
fn watcher_wave5_crescendo_changes_stance_through_declared_effects() {
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.discard_pile.push(engine.card_registry.make_card("FlurryOfBlows"));
    ensure_in_hand(&mut engine, "Crescendo+");
    assert!(play_self(&mut engine, "Crescendo+"));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert_eq!(hand_count(&engine, "FlurryOfBlows"), 1);
    assert_eq!(discard_prefix_count(&engine, "FlurryOfBlows"), 0);
}

#[test]
fn watcher_wave5_establishment_installs_and_reduces_retained_card_cost() {
    let mut engine = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut engine, "Establishment");
    ensure_in_hand(&mut engine, "Protect");

    assert!(play_self(&mut engine, "Establishment"));
    assert_eq!(engine.state.player.status(sid::ESTABLISHMENT), 1);

    end_turn(&mut engine);

    let protect = engine
        .state
        .hand
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Protect")
        .expect("Protect should be retained");
    assert_eq!(protect.cost, 1);

    end_turn(&mut engine);

    let protect = engine
        .state
        .hand
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Protect")
        .expect("Protect should still be retained on the next turn");
    assert_eq!(protect.cost, 0);
}

#[test]
fn watcher_wave5_fasting_uses_declared_buffs_and_hook_for_energy_penalty() {
    let mut engine = one_enemy_engine("JawWorm", 60, 0);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "Fasting2+");

    assert!(play_self(&mut engine, "Fasting2+"));
    assert_eq!(engine.state.player.status(sid::STRENGTH), 4);
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 4);
    assert_eq!(engine.state.max_energy, 2);
    assert_eq!(engine.state.energy, 1);
}

#[test]
fn watcher_wave5_holy_water_uses_block_preamble_and_retain_tag() {
    let mut engine = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut engine, "HolyWater");
    assert!(play_self(&mut engine, "HolyWater"));
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(exhaust_prefix_count(&engine, "HolyWater"), 1);
}

#[test]
fn watcher_wave5_judgement_kills_only_below_threshold() {
    let mut lethal = one_enemy_engine("JawWorm", 30, 0);
    ensure_in_hand(&mut lethal, "Judgement");
    assert!(play_on_enemy(&mut lethal, "Judgement", 0));
    assert_eq!(lethal.state.enemies[0].entity.hp, 0);

    let mut safe = one_enemy_engine("JawWorm", 31, 0);
    ensure_in_hand(&mut safe, "Judgement");
    assert!(play_on_enemy(&mut safe, "Judgement", 0));
    assert_eq!(safe.state.enemies[0].entity.hp, 31);
}
