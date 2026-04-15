#![cfg(test)]

use crate::cards::{CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, BulkAction, CardFilter, Effect as E, Pile as P, SimpleEffect as SE};
use crate::gameplay::GameplayProgramSource;
use crate::orbs::OrbType;
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self,
};

fn total_enemy_hp(engine: &crate::engine::CombatEngine) -> i32 {
    engine
        .state
        .enemies
        .iter()
        .map(|enemy| enemy.entity.hp.max(0))
        .sum()
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
    assert_eq!(def.program_source(), GameplayProgramSource::Canonical, "{id} source");

    let schema = def.card_schema().expect("card schema");
    assert_eq!(schema.card_type, Some(card_type), "{id} type");
    assert_eq!(schema.target, Some(target), "{id} target");
    assert_eq!(schema.cost, Some(cost), "{id} cost");
    assert_eq!(schema.exhausts, exhausts, "{id} exhaust");
    assert_eq!(schema.upgraded_from.as_deref(), upgraded_from, "{id} upgraded_from");
    schema.clone()
}

#[test]
fn test_card_runtime_defect_wave3_registry_exports_surface_x_cost_and_exhaust_hints() {
    let multi_cast = assert_gameplay_card_export(
        "Multi-Cast",
        CardType::Skill,
        CardTarget::None,
        -1,
        false,
        None,
    );
    assert_eq!(multi_cast.declared_evoke_count, Some(A::XCost));
    assert_eq!(multi_cast.declared_x_cost_amounts, vec![A::XCost]);

    let multi_cast_plus = assert_gameplay_card_export(
        "Multi-Cast+",
        CardType::Skill,
        CardTarget::None,
        -1,
        false,
        Some("Multi-Cast"),
    );
    assert_eq!(multi_cast_plus.declared_evoke_count, Some(A::MagicPlusX));
    assert_eq!(multi_cast_plus.declared_x_cost_amounts, vec![A::MagicPlusX]);

    let reboot = assert_gameplay_card_export(
        "Reboot+",
        CardType::Skill,
        CardTarget::SelfTarget,
        0,
        true,
        Some("Reboot"),
    );
    assert_eq!(
        reboot.declared_effect_count,
        3,
        "Reboot should expose discard/shuffle/draw effects"
    );
    let reboot_def = crate::cards::global_registry().get("Reboot").expect("Reboot");
    assert_eq!(
        reboot_def.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::ShuffleDiscardIntoDraw),
            E::Simple(SE::DrawCards(A::Magic)),
        ]
    );
    assert!(reboot_def.complex_hook.is_none());

    let fission_def = crate::cards::global_registry().get("Fission").expect("Fission");
    assert_eq!(
        fission_def.effect_data,
        &[E::Simple(SE::ResolveFission { evoke: false })]
    );
    assert!(fission_def.complex_hook.is_none());

    let fission_plus_def = crate::cards::global_registry().get("Fission+").expect("Fission+");
    assert_eq!(
        fission_plus_def.effect_data,
        &[E::Simple(SE::ResolveFission { evoke: true })]
    );
    assert!(fission_plus_def.complex_hook.is_none());

    let force_field = assert_gameplay_card_export(
        "Force Field+",
        CardType::Skill,
        CardTarget::SelfTarget,
        4,
        false,
        Some("Force Field"),
    );
    assert_eq!(force_field.declared_effect_count, 1);
}

#[test]
fn test_card_runtime_defect_wave3_barrage_and_blizzard_scale_on_engine_path() {
    let mut barrage = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut barrage);
    barrage.init_defect_orbs(3);
    barrage.channel_orb(OrbType::Lightning);
    barrage.channel_orb(OrbType::Frost);
    barrage.channel_orb(OrbType::Dark);
    barrage.state.hand = make_deck(&["Barrage+"]);
    assert!(play_on_enemy(&mut barrage, "Barrage+", 0));
    assert_eq!(barrage.state.enemies[0].entity.hp, 42);

    let mut blizzard = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 35, 35),
        ],
        3,
    );
    force_player_turn(&mut blizzard);
    blizzard.init_defect_orbs(3);
    blizzard.channel_orb(OrbType::Frost);
    blizzard.channel_orb(OrbType::Frost);
    blizzard.state.hand = make_deck(&["Blizzard+"]);
    let hp_before = total_enemy_hp(&blizzard);
    assert!(play_self(&mut blizzard, "Blizzard+"));
    assert_eq!(hp_before - total_enemy_hp(&blizzard), 12);
}

#[test]
fn test_card_runtime_defect_wave3_double_energy_and_force_field_use_runtime_cost_paths() {
    let mut double_energy = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut double_energy);
    double_energy.state.energy = 4;
    double_energy.state.hand = make_deck(&["Double Energy"]);
    assert!(play_self(&mut double_energy, "Double Energy"));
    assert_eq!(double_energy.state.energy, 6);
    assert_eq!(double_energy.state.exhaust_pile.len(), 1);
    assert_eq!(
        double_energy
            .card_registry
            .card_name(double_energy.state.exhaust_pile[0].def_id),
        "Double Energy"
    );

    let mut force_field = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        6,
    );
    force_player_turn(&mut force_field);
    force_field.state.hand = make_deck(&["Demon Form", "Noxious Fumes", "Force Field"]);
    assert!(play_self(&mut force_field, "Demon Form"));
    assert!(play_self(&mut force_field, "Noxious Fumes"));
    force_field.state.energy = 2;
    assert!(play_self(&mut force_field, "Force Field"));
    assert_eq!(force_field.state.player.block, 12);
    assert_eq!(force_field.state.energy, 0);
}

#[test]
fn test_card_runtime_defect_wave3_fission_and_multicast_cover_orb_evoke_and_x_cost_paths() {
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

    let mut multi_cast = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    force_player_turn(&mut multi_cast);
    multi_cast.init_defect_orbs(3);
    multi_cast.channel_orb(OrbType::Lightning);
    multi_cast.channel_orb(OrbType::Frost);
    multi_cast.channel_orb(OrbType::Dark);
    multi_cast.state.energy = 2;
    multi_cast.state.hand = make_deck(&["Multi-Cast+"]);
    let hp_before = multi_cast.state.enemies[0].entity.hp;
    let block_before = multi_cast.state.player.block;
    assert!(play_self(&mut multi_cast, "Multi-Cast+"));
    assert_eq!(multi_cast.state.energy, 0);
    assert_eq!(multi_cast.state.enemies[0].entity.hp, hp_before - 14);
    assert_eq!(multi_cast.state.player.block, block_before + 5);
}

#[test]
fn test_card_runtime_defect_wave3_reboot_exhausts_and_refills_from_reset_piles() {
    let mut reboot = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut reboot);
    reboot.state.hand = make_deck(&["Reboot", "Strike_B", "Defend_B"]);
    reboot.state.draw_pile.clear();
    reboot.state.discard_pile = make_deck(&["Zap", "Dualcast", "Cold Snap"]);

    assert!(play_self(&mut reboot, "Reboot"));

    assert_eq!(reboot.state.hand.len(), 4);
    assert_eq!(reboot.state.exhaust_pile.len(), 1);
    assert_eq!(
        reboot.card_registry.card_name(reboot.state.exhaust_pile[0].def_id),
        "Reboot"
    );
    assert_eq!(reboot.state.discard_pile.len(), 0);
}
