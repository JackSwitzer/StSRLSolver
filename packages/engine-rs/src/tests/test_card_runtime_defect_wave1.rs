#![cfg(test)]

use crate::cards::{CardTarget, CardType};
use crate::gameplay::{EffectOp, GameplayDomain, GameplayProgramSource};
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
) {
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

    let program = def.program();
    assert!(matches!(
        program.steps.first(),
        Some(EffectOp::DeclareDefinition { domain: GameplayDomain::Card, .. })
    ));
    assert!(program.steps.iter().any(|step| matches!(step, EffectOp::PlayCard { .. })), "{id} missing play op");
}

#[test]
fn test_card_runtime_defect_wave1_registry_exports_are_canonical() {
    let cases = [
        ("BootSequence", CardType::Skill, CardTarget::SelfTarget, 0, true, None),
        ("BootSequence+", CardType::Skill, CardTarget::SelfTarget, 0, true, Some("BootSequence")),
        ("Leap", CardType::Skill, CardTarget::SelfTarget, 1, false, None),
        ("Leap+", CardType::Skill, CardTarget::SelfTarget, 1, false, Some("Leap")),
        ("Reinforced Body", CardType::Skill, CardTarget::SelfTarget, -1, false, None),
        ("Reinforced Body+", CardType::Skill, CardTarget::SelfTarget, -1, false, Some("Reinforced Body")),
        ("Storm", CardType::Power, CardTarget::SelfTarget, 1, false, None),
        ("Storm+", CardType::Power, CardTarget::SelfTarget, 1, false, Some("Storm")),
        ("Loop", CardType::Power, CardTarget::SelfTarget, 1, false, None),
        ("Loop+", CardType::Power, CardTarget::SelfTarget, 1, false, Some("Loop")),
        ("Machine Learning", CardType::Power, CardTarget::SelfTarget, 1, false, None),
        ("Machine Learning+", CardType::Power, CardTarget::SelfTarget, 1, false, Some("Machine Learning")),
        ("Buffer", CardType::Power, CardTarget::SelfTarget, 2, false, None),
        ("Buffer+", CardType::Power, CardTarget::SelfTarget, 2, false, Some("Buffer")),
        ("Hello World", CardType::Power, CardTarget::SelfTarget, 1, false, None),
        ("Hello World+", CardType::Power, CardTarget::SelfTarget, 1, false, Some("Hello World")),
    ];

    for (id, card_type, target, cost, exhausts, upgraded_from) in cases {
        assert_gameplay_card_export(id, card_type, target, cost, exhausts, upgraded_from);
    }
}

#[test]
fn test_card_runtime_defect_wave1_boot_sequence_leap_and_reinforced_body_play_through_engine() {
    let mut boot = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut boot);
    boot.state.hand = make_deck(&["BootSequence"]);
    assert!(play_self(&mut boot, "BootSequence"));
    assert_eq!(boot.state.player.block, 10);
    assert_eq!(boot.state.hand.len(), 0);
    assert_eq!(boot.state.exhaust_pile.len(), 1);
    assert_eq!(boot.card_registry.card_name(boot.state.exhaust_pile[0].def_id), "BootSequence");

    let mut leap = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut leap);
    leap.state.hand = make_deck(&["Leap"]);
    assert!(play_self(&mut leap, "Leap"));
    assert_eq!(leap.state.player.block, 9);

    let mut reinforced_body = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut reinforced_body);
    reinforced_body.state.energy = 3;
    reinforced_body.state.hand = make_deck(&["Reinforced Body"]);
    assert!(play_self(&mut reinforced_body, "Reinforced Body"));
    assert_eq!(reinforced_body.state.player.block, 21);
    assert_eq!(reinforced_body.state.energy, 0);
}

#[test]
fn test_card_runtime_defect_wave1_storm_and_buffer_install_on_the_engine_path() {
    let mut storm = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut storm);
    storm.init_defect_orbs(1);
    storm.state.hand = make_deck(&["Storm", "Defragment"]);
    assert!(play_self(&mut storm, "Storm"));
    assert_eq!(storm.state.player.status(sid::STORM), 1);
    assert!(play_self(&mut storm, "Defragment"));
    assert_eq!(storm.state.orb_slots.slots[0].orb_type, OrbType::Lightning);

    let mut buffer = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut buffer);
    buffer.state.hand = make_deck(&["Buffer"]);
    assert!(play_self(&mut buffer, "Buffer"));
    assert_eq!(buffer.state.player.status(sid::BUFFER), 1);
}

#[test]
fn test_card_runtime_defect_wave1_machine_learning_and_hello_world_trigger_start_of_turn_draws() {
    let mut machine_learning = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut machine_learning);
    machine_learning.state.hand = make_deck(&["Machine Learning"]);
    machine_learning.state.draw_pile = make_deck(&[
        "Strike_R",
        "Strike_R",
        "Strike_R",
        "Strike_R",
        "Strike_R",
        "Strike_R",
    ]);
    assert!(play_self(&mut machine_learning, "Machine Learning"));
    end_turn(&mut machine_learning);
    assert_eq!(machine_learning.state.hand.len(), 6);

    let mut hello_world = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut hello_world);
    hello_world.state.hand = make_deck(&["Hello World"]);
    hello_world.state.draw_pile.clear();
    assert!(play_self(&mut hello_world, "Hello World"));
    end_turn(&mut hello_world);
    assert_eq!(hello_world.state.hand.len(), 1);
    assert_eq!(hello_world.card_registry.card_name(hello_world.state.hand[0].def_id), "Strike_R");
}

#[test]
fn test_card_runtime_defect_wave1_loop_repeats_the_front_orb_passive_on_turn_end() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut engine);
    engine.init_defect_orbs(1);
    engine.channel_orb(OrbType::Lightning);
    engine.state.hand = make_deck(&["Loop"]);
    assert!(play_self(&mut engine, "Loop"));
    assert_eq!(engine.state.player.status(sid::LOOP), 1);
    end_turn(&mut engine);
    assert_eq!(engine.state.enemies[0].entity.hp, 54);
}
