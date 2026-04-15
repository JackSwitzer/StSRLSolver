#![cfg(test)]

use crate::cards::{CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

fn assert_gameplay_card_export(
    id: &str,
    card_type: CardType,
    target: CardTarget,
    cost: i32,
    exhausts: bool,
    upgraded_from: Option<&str>,
) -> crate::gameplay::CardSchema {
    let def = crate::gameplay::global_registry()
        .card(id)
        .unwrap_or_else(|| panic!("missing gameplay card export for {id}"));
    let schema = def.card_schema().expect("card schema");
    assert_eq!(schema.card_type, Some(card_type), "{id} type");
    assert_eq!(schema.target, Some(target), "{id} target");
    assert_eq!(schema.cost, Some(cost), "{id} cost");
    assert_eq!(schema.exhausts, exhausts, "{id} exhaust");
    assert_eq!(schema.upgraded_from.as_deref(), upgraded_from, "{id} upgraded_from");
    schema.clone()
}

#[test]
fn watcher_wave4_registry_exports_surface_declared_block_stance_and_power_installs() {
    let rushdown = crate::cards::global_registry()
        .get("Adaptation")
        .expect("Rushdown should be registered");
    assert_eq!(
        rushdown.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::RUSHDOWN, A::Magic))]
    );

    let battle_hymn = crate::cards::global_registry()
        .get("BattleHymn")
        .expect("Battle Hymn should be registered");
    assert_eq!(
        battle_hymn.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BATTLE_HYMN, A::Magic))]
    );

    let defend = assert_gameplay_card_export(
        "Defend_P+",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        false,
        Some("Defend_P"),
    );
    assert_eq!(defend.declared_effect_count, 1);

    let eruption = crate::cards::global_registry()
        .get("Eruption")
        .expect("Eruption should be registered");
    assert_eq!(eruption.enter_stance, Some("Wrath"));
    assert_eq!(
        eruption.effect_data,
        &[E::Simple(SE::ChangeStance(Stance::Wrath))]
    );

    let devotion = crate::cards::global_registry()
        .get("Devotion+")
        .expect("Devotion+ should be registered");
    assert_eq!(
        devotion.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::DEVOTION, A::Magic))]
    );

    let deva_form = crate::cards::global_registry()
        .get("DevaForm")
        .expect("Deva Form should be registered");
    assert_eq!(
        deva_form.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::DEVA_FORM, A::Magic))]
    );
    assert!(deva_form.runtime_traits().ethereal);
}

#[test]
fn watcher_wave4_rushdown_and_eruption_run_on_the_engine_path() {
    let mut engine = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut engine, "Adaptation");
    ensure_in_hand(&mut engine, "Eruption");
    engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Vigilance"]);

    assert!(play_self(&mut engine, "Adaptation"));
    assert_eq!(engine.state.player.status(sid::RUSHDOWN), 2);

    let hand_before = engine.state.hand.len();
    assert!(play_on_enemy(&mut engine, "Eruption", 0));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert_eq!(engine.state.enemies[0].entity.hp, 51);
    assert!(engine.state.hand.len() >= hand_before + 1);
}

#[test]
fn watcher_wave4_battle_hymn_devotion_and_deva_form_trigger_after_install() {
    let mut battle_hymn = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut battle_hymn, "BattleHymn");
    assert!(play_self(&mut battle_hymn, "BattleHymn"));
    assert_eq!(battle_hymn.state.player.status(sid::BATTLE_HYMN), 1);
    end_turn(&mut battle_hymn);
    assert_eq!(hand_count(&battle_hymn, "Smite"), 1);

    let mut devotion = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut devotion, "Devotion+");
    assert!(play_self(&mut devotion, "Devotion+"));
    assert_eq!(devotion.state.player.status(sid::DEVOTION), 3);
    end_turn(&mut devotion);
    assert_eq!(devotion.state.mantra, 3);

    let mut deva_form = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut deva_form, "DevaForm");
    assert!(play_self(&mut deva_form, "DevaForm"));
    assert_eq!(deva_form.state.player.status(sid::DEVA_FORM), 1);
    end_turn(&mut deva_form);
    assert_eq!(deva_form.state.energy, 4);
    assert_eq!(deva_form.state.player.status(sid::DEVA_FORM), 2);
}

#[test]
fn watcher_wave4_defend_plus_uses_declared_block() {
    let mut engine = one_enemy_engine("JawWorm", 40, 0);
    ensure_in_hand(&mut engine, "Defend_P+");
    assert!(play_self(&mut engine, "Defend_P+"));
    assert_eq!(engine.state.player.block, 8);
}
