#![cfg(test)]

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::gameplay::GameplayProgramSource;
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    end_turn, enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self,
};

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
    assert_eq!(def.program_source(), GameplayProgramSource::AdaptedLegacy, "{id} source");

    let schema = def.card_schema().expect("card schema");
    assert_eq!(schema.card_type, Some(card_type), "{id} type");
    assert_eq!(schema.target, Some(target), "{id} target");
    assert_eq!(schema.cost, Some(cost), "{id} cost");
    assert_eq!(schema.exhausts, exhausts, "{id} exhaust");
    assert_eq!(schema.upgraded_from.as_deref(), upgraded_from, "{id} upgraded_from");
    schema.clone()
}

#[test]
fn test_card_runtime_defect_wave5_registry_exports_match_runtime_progress() {
    let reg = global_registry();

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());

    let fission = reg.get("Fission").expect("Fission");
    assert!(fission.effect_data.is_empty());
    assert!(fission.complex_hook.is_some());

    let force_field = reg.get("Force Field").expect("Force Field");
    assert!(force_field.effect_data.is_empty());
    assert!(force_field.complex_hook.is_none());
    assert!(force_field.effects.contains(&"reduce_cost_per_power"));

    let heatsinks = reg.get("Heatsinks+").expect("Heatsinks+");
    assert_eq!(
        heatsinks.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::HEATSINK, A::Magic))]
    );
    assert!(heatsinks.complex_hook.is_none());

    let hello_world = reg.get("Hello World+").expect("Hello World+");
    assert_eq!(
        hello_world.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::HELLO_WORLD, A::Magic))]
    );
    assert!(hello_world.effects.contains(&"innate"));
    assert!(hello_world.complex_hook.is_none());

    let leap = reg.get("Leap+").expect("Leap+");
    assert!(leap.effect_data.is_empty());
    assert!(leap.complex_hook.is_none());

    let loop_card = reg.get("Loop+").expect("Loop+");
    assert_eq!(
        loop_card.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::LOOP, A::Magic))]
    );
    assert!(loop_card.complex_hook.is_none());

    let schema = assert_gameplay_card_export(
        "Force Field+",
        CardType::Skill,
        CardTarget::SelfTarget,
        4,
        false,
        Some("Force Field"),
    );
    assert_eq!(schema.declared_effect_count, 0);
}

#[test]
fn test_card_runtime_defect_wave5_double_energy_force_field_and_leap_follow_engine_path() {
    let mut double_energy = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut double_energy);
    double_energy.state.energy = 4;
    double_energy.state.hand = make_deck(&["Double Energy+"]);
    assert!(play_self(&mut double_energy, "Double Energy+"));
    assert_eq!(double_energy.state.energy, 8);
    assert!(double_energy
        .state
        .exhaust_pile
        .iter()
        .any(|card| double_energy.card_registry.card_name(card.def_id) == "Double Energy+"));

    let mut force_field = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        6,
    );
    force_player_turn(&mut force_field);
    force_field.state.hand = make_deck(&["Heatsinks", "Hello World", "Force Field+"]);
    assert!(play_self(&mut force_field, "Heatsinks"));
    assert!(play_self(&mut force_field, "Hello World"));
    force_field.state.energy = 2;
    assert!(play_self(&mut force_field, "Force Field+"));
    assert_eq!(force_field.state.player.block, 16);
    assert_eq!(force_field.state.energy, 0);

    let mut leap = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut leap);
    leap.state.hand = make_deck(&["Leap+"]);
    assert!(play_self(&mut leap, "Leap+"));
    assert_eq!(leap.state.player.block, 12);
}

#[test]
fn test_card_runtime_defect_wave5_fission_and_fission_plus_cover_remove_and_evoke_paths() {
    let mut fission = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut fission);
    fission.init_defect_orbs(3);
    fission.channel_orb(OrbType::Lightning);
    fission.channel_orb(OrbType::Frost);
    fission.channel_orb(OrbType::Dark);
    fission.state.hand = make_deck(&["Fission"]);
    fission.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    assert!(play_self(&mut fission, "Fission"));
    assert_eq!(fission.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission.state.energy, 6);
    assert_eq!(fission.state.hand.len(), 3);

    let mut fission_plus = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    force_player_turn(&mut fission_plus);
    fission_plus.init_defect_orbs(3);
    fission_plus.channel_orb(OrbType::Lightning);
    fission_plus.channel_orb(OrbType::Frost);
    fission_plus.channel_orb(OrbType::Dark);
    fission_plus.state.hand = make_deck(&["Fission+"]);
    fission_plus.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    let hp_before = fission_plus.state.enemies[0].entity.hp;
    let block_before = fission_plus.state.player.block;
    assert!(play_self(&mut fission_plus, "Fission+"));
    assert_eq!(fission_plus.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission_plus.state.energy, 6);
    assert_eq!(fission_plus.state.hand.len(), 3);
    assert_eq!(fission_plus.state.enemies[0].entity.hp, hp_before - 14);
    assert_eq!(fission_plus.state.player.block, block_before + 5);
}

#[test]
fn test_card_runtime_defect_wave5_heatsinks_hello_world_and_loop_install_runtime_statuses() {
    let mut heatsinks = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut heatsinks);
    heatsinks.state.hand = make_deck(&["Heatsinks+", "Hello World"]);
    heatsinks.state.draw_pile = make_deck(&["Strike_B", "Defend_B"]);
    assert!(play_self(&mut heatsinks, "Heatsinks+"));
    assert_eq!(heatsinks.state.player.status(sid::HEATSINK), 2);
    assert!(play_self(&mut heatsinks, "Hello World"));
    assert_eq!(heatsinks.state.hand.len(), 2);

    let mut hello_world = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut hello_world);
    hello_world.state.hand = make_deck(&["Hello World+"]);
    hello_world.state.draw_pile.clear();
    assert!(play_self(&mut hello_world, "Hello World+"));
    assert_eq!(hello_world.state.player.status(sid::HELLO_WORLD), 1);
    end_turn(&mut hello_world);
    assert_eq!(hello_world.state.hand.len(), 1);
    assert_eq!(
        hello_world.card_registry.card_name(hello_world.state.hand[0].def_id),
        "Strike_R"
    );

    let mut loop_card = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut loop_card);
    loop_card.init_defect_orbs(1);
    loop_card.channel_orb(OrbType::Lightning);
    loop_card.state.hand = make_deck(&["Loop+"]);
    assert!(play_self(&mut loop_card, "Loop+"));
    assert_eq!(loop_card.state.player.status(sid::LOOP), 2);
    end_turn(&mut loop_card);
    assert_eq!(loop_card.state.enemies[0].entity.hp, 54);
}
