#![cfg(test)]

use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::Trigger;
use crate::orbs::OrbType;
use crate::powers::defs::complex::{
    DEF_BURST, DEF_DOUBLE_TAP, DEF_ENVENOM, DEF_PANACHE, DEF_THOUSAND_CUTS,
};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, make_deck,
    make_deck_n, play_on_enemy,
};

fn invoke_hook(
    def: &crate::effects::entity_def::EntityDef,
    engine: &mut crate::engine::CombatEngine,
    owner: EffectOwner,
    event: GameEvent,
    state: &mut EffectState,
) {
    let hook = def.complex_hook.expect("complex hook should exist");
    hook(engine, owner, &event, state);
}

#[test]
fn thousand_cuts_runtime_hook_hits_all_living_enemies() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_without_start(make_deck_n("Strike_R", 5), enemies, 3);
    engine.state.player.set_status(sid::THOUSAND_CUTS, 2);

    let mut runtime_state = EffectState::default();
    invoke_hook(
        &DEF_THOUSAND_CUTS,
        &mut engine,
        EffectOwner::PlayerPower,
        GameEvent::empty(Trigger::OnAfterCardPlayed),
        &mut runtime_state,
    );

    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.enemies[1].entity.hp, 33);
}

#[test]
fn panache_runtime_hook_counts_cards_and_resets_after_five() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 50, 50),
        enemy_no_intent("Cultist", 45, 45),
    ];
    let mut engine = engine_without_start(make_deck_n("Strike_R", 5), enemies, 3);
    engine.state.player.set_status(sid::PANACHE, 10);

    let mut runtime_state = EffectState::default();
    for expected_count in 1..=4 {
        invoke_hook(
            &DEF_PANACHE,
            &mut engine,
            EffectOwner::PlayerPower,
            GameEvent::empty(Trigger::OnUseCard),
            &mut runtime_state,
        );
        assert_eq!(runtime_state.get(0), expected_count);
        assert_eq!(engine.state.enemies[0].entity.hp, 50);
        assert_eq!(engine.state.enemies[1].entity.hp, 45);
    }

    invoke_hook(
        &DEF_PANACHE,
        &mut engine,
        EffectOwner::PlayerPower,
        GameEvent::empty(Trigger::OnUseCard),
        &mut runtime_state,
    );

    assert_eq!(runtime_state.get(0), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 40);
    assert_eq!(engine.state.enemies[1].entity.hp, 35);
}

#[test]
fn double_tap_def_is_wired_to_attack_play_events() {
    assert_eq!(DEF_DOUBLE_TAP.triggers.len(), 1);
    assert_eq!(DEF_DOUBLE_TAP.triggers[0].trigger, Trigger::OnAttackPlayed);
    assert!(DEF_DOUBLE_TAP.complex_hook.is_some());
}

#[test]
fn burst_def_is_wired_to_skill_play_events() {
    assert_eq!(DEF_BURST.triggers.len(), 1);
    assert_eq!(DEF_BURST.triggers[0].trigger, Trigger::OnSkillPlayed);
    assert!(DEF_BURST.complex_hook.is_some());
}

#[test]
fn envenom_def_is_wired_to_damage_resolved_events() {
    assert_eq!(DEF_ENVENOM.triggers.len(), 1);
    assert_eq!(DEF_ENVENOM.triggers[0].trigger, Trigger::DamageResolved);
    assert!(DEF_ENVENOM.complex_hook.is_some());
}

#[test]
fn electrodynamics_lightning_passive_hits_all_enemies_instead_of_channeling_on_attack_play() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_without_start(make_deck_n("Strike_B", 5), enemies, 3);
    engine.init_defect_orbs(1);
    engine.state.player.set_status(sid::ELECTRODYNAMICS, 1);
    engine.channel_orb(OrbType::Lightning);

    engine.evoke_front_orb();

    assert_eq!(engine.state.enemies[0].entity.hp, 32);
    assert_eq!(engine.state.enemies[1].entity.hp, 27);
}

#[test]
fn electrodynamics_no_longer_channels_extra_lightning_when_attack_card_is_played() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_B"]),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 35, 35),
        ],
        3,
    ));
    engine.init_defect_orbs(2);
    engine.state.player.set_status(sid::ELECTRODYNAMICS, 1);
    engine.channel_orb(OrbType::Lightning);
    engine.state.hand = make_deck(&["Strike_B"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    let before_lightning = engine
        .state
        .orb_slots
        .slots
        .iter()
        .filter(|orb| orb.orb_type == OrbType::Lightning)
        .count();
    assert!(play_on_enemy(&mut engine, "Strike_B", 0));
    let after_lightning = engine
        .state
        .orb_slots
        .slots
        .iter()
        .filter(|orb| orb.orb_type == OrbType::Lightning)
        .count();

    assert_eq!(after_lightning, before_lightning);
}
