#![cfg(test)]

use crate::cards::{CardTarget, CardType};
use crate::effects::declarative::AmountSource as A;
use crate::gameplay::OrbCountHint;
use crate::orbs::OrbType;
use crate::status_ids::sid;
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
    let schema = def.card_schema().expect("card schema");
    assert_eq!(schema.card_type, Some(card_type), "{id} type");
    assert_eq!(schema.target, Some(target), "{id} target");
    assert_eq!(schema.cost, Some(cost), "{id} cost");
    assert_eq!(schema.exhausts, exhausts, "{id} exhaust");
    assert_eq!(schema.upgraded_from.as_deref(), upgraded_from, "{id} upgraded_from");
    (*schema).clone()
}

#[test]
fn test_card_runtime_defect_wave2_registry_exports_surface_declarative_hints() {
    let ball_lightning = assert_gameplay_card_export(
        "Ball Lightning",
        CardType::Attack,
        CardTarget::Enemy,
        1,
        false,
        None,
    );
    assert_eq!(
        ball_lightning.declared_channel_orbs,
        vec![OrbCountHint {
            orb_type: OrbType::Lightning,
            count: A::Fixed(1),
        }]
    );

    let beam_cell = assert_gameplay_card_export(
        "Beam Cell+",
        CardType::Attack,
        CardTarget::Enemy,
        0,
        false,
        Some("Beam Cell"),
    );
    assert_eq!(beam_cell.declared_effect_count, 1);

    let compile_driver = assert_gameplay_card_export(
        "Compile Driver",
        CardType::Attack,
        CardTarget::Enemy,
        1,
        false,
        None,
    );
    assert_eq!(compile_driver.declared_effect_count, 1);

    let coolheaded = assert_gameplay_card_export(
        "Coolheaded+",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        false,
        Some("Coolheaded"),
    );
    assert_eq!(
        coolheaded.declared_channel_orbs,
        vec![OrbCountHint {
            orb_type: OrbType::Frost,
            count: A::Fixed(1),
        }]
    );

    let darkness = assert_gameplay_card_export(
        "Darkness+",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        false,
        Some("Darkness"),
    );
    assert_eq!(
        darkness.declared_channel_orbs,
        vec![OrbCountHint {
            orb_type: OrbType::Dark,
            count: A::Fixed(1),
        }]
    );

    let fusion = assert_gameplay_card_export(
        "Fusion+",
        CardType::Skill,
        CardTarget::SelfTarget,
        1,
        false,
        Some("Fusion"),
    );
    assert_eq!(
        fusion.declared_channel_orbs,
        vec![OrbCountHint {
            orb_type: OrbType::Plasma,
            count: A::Fixed(1),
        }]
    );

    let rainbow = assert_gameplay_card_export(
        "Rainbow",
        CardType::Skill,
        CardTarget::SelfTarget,
        2,
        true,
        None,
    );
    assert_eq!(
        rainbow.declared_channel_orbs,
        vec![
            OrbCountHint {
                orb_type: OrbType::Lightning,
                count: A::Fixed(1),
            },
            OrbCountHint {
                orb_type: OrbType::Frost,
                count: A::Fixed(1),
            },
            OrbCountHint {
                orb_type: OrbType::Dark,
                count: A::Fixed(1),
            },
        ]
    );

    let rip_and_tear = assert_gameplay_card_export(
        "Rip and Tear+",
        CardType::Attack,
        CardTarget::AllEnemy,
        1,
        false,
        Some("Rip and Tear"),
    );
    assert_eq!(rip_and_tear.declared_effect_count, 2);
}

#[test]
fn test_card_runtime_defect_wave2_ball_lightning_beam_cell_and_compile_driver_resolve_on_engine_path() {
    let mut ball_lightning = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut ball_lightning);
    ball_lightning.init_defect_orbs(1);
    ball_lightning.state.hand = make_deck(&["Ball Lightning"]);
    assert!(play_on_enemy(&mut ball_lightning, "Ball Lightning", 0));
    assert_eq!(ball_lightning.state.enemies[0].entity.hp, 33);
    assert_eq!(ball_lightning.state.orb_slots.slots[0].orb_type, OrbType::Lightning);

    let mut beam_cell = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut beam_cell);
    beam_cell.state.hand = make_deck(&["Beam Cell+"]);
    assert!(play_on_enemy(&mut beam_cell, "Beam Cell+", 0));
    assert_eq!(beam_cell.state.enemies[0].entity.hp, 36);
    assert_eq!(beam_cell.state.enemies[0].entity.status(sid::VULNERABLE), 2);

    let mut compile_driver = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut compile_driver);
    compile_driver.init_defect_orbs(3);
    compile_driver.channel_orb(OrbType::Lightning);
    compile_driver.channel_orb(OrbType::Frost);
    compile_driver.channel_orb(OrbType::Dark);
    compile_driver.state.hand = make_deck(&["Compile Driver"]);
    compile_driver.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Dualcast"]);
    assert!(play_on_enemy(&mut compile_driver, "Compile Driver", 0));
    assert_eq!(compile_driver.state.enemies[0].entity.hp, 43);
    assert_eq!(compile_driver.state.hand.len(), 3);
}

#[test]
fn test_card_runtime_defect_wave2_coolheaded_fusion_darkness_and_rainbow_cover_channel_draw_and_exhaust_paths() {
    let mut coolheaded = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut coolheaded);
    coolheaded.init_defect_orbs(1);
    coolheaded.state.hand = make_deck(&["Coolheaded+"]);
    coolheaded.state.draw_pile = make_deck(&["Strike", "Defend", "Zap"]);
    assert!(play_self(&mut coolheaded, "Coolheaded+"));
    assert_eq!(coolheaded.state.orb_slots.slots[0].orb_type, OrbType::Frost);
    assert_eq!(coolheaded.state.hand.len(), 2);

    let mut darkness = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut darkness);
    darkness.init_defect_orbs(1);
    darkness.state.hand = make_deck(&["Darkness+"]);
    assert!(play_self(&mut darkness, "Darkness+"));
    assert_eq!(darkness.state.orb_slots.slots[0].orb_type, OrbType::Dark);
    assert_eq!(darkness.state.orb_slots.slots[0].evoke_amount, 12);

    let mut fusion = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut fusion);
    fusion.init_defect_orbs(1);
    fusion.state.hand = make_deck(&["Fusion+"]);
    assert!(play_self(&mut fusion, "Fusion+"));
    assert_eq!(fusion.state.orb_slots.slots[0].orb_type, OrbType::Plasma);

    let mut rainbow = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut rainbow);
    rainbow.init_defect_orbs(3);
    rainbow.state.hand = make_deck(&["Rainbow"]);
    assert!(play_self(&mut rainbow, "Rainbow"));
    assert_eq!(rainbow.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(rainbow.state.orb_slots.slots[1].orb_type, OrbType::Frost);
    assert_eq!(rainbow.state.orb_slots.slots[2].orb_type, OrbType::Dark);
    assert_eq!(rainbow.state.exhaust_pile.len(), 1);
    assert_eq!(
        rainbow.card_registry.card_name(rainbow.state.exhaust_pile[0].def_id),
        "Rainbow"
    );
}

#[test]
fn test_card_runtime_defect_wave2_rip_and_tear_hits_random_living_enemies_for_exact_total_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 30, 30),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Rip and Tear"]);

    let total_before = total_enemy_hp(&engine);
    assert!(play_on_enemy(&mut engine, "Rip and Tear", 0));
    let total_after = total_enemy_hp(&engine);

    assert_eq!(total_before - total_after, 14);
    assert!(engine.state.enemies.iter().all(|enemy| enemy.entity.hp >= 0));
}
