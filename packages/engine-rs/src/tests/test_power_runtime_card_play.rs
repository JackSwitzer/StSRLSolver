#![cfg(test)]

use super::DEF_STORM;
use crate::cards::CardType;
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::Trigger;
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, play_self};

#[test]
fn storm_hook_channels_one_lightning_on_power_play_event() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::STORM, 2);
    engine.state.orb_slots.add_slot();

    let mut effect_state = EffectState::default();
    let hook = DEF_STORM.complex_hook.expect("Storm should provide a runtime hook");
    hook(
        &mut engine,
        EffectOwner::PlayerPower,
        &GameEvent {
            kind: Trigger::OnPowerPlayed,
            card_type: Some(CardType::Power),
            card_inst: None,
            is_first_turn: false,
            target_idx: -1,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        },
        &mut effect_state,
    );

    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
}

#[test]
fn storm_hook_ignores_non_power_card_events() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::STORM, 1);
    engine.state.orb_slots.add_slot();

    let mut effect_state = EffectState::default();
    let hook = DEF_STORM.complex_hook.expect("Storm should provide a runtime hook");
    hook(
        &mut engine,
        EffectOwner::PlayerPower,
        &GameEvent {
            kind: Trigger::OnAttackPlayed,
            card_type: Some(CardType::Attack),
            card_inst: None,
            is_first_turn: false,
            target_idx: 0,
            enemy_idx: -1,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        },
        &mut effect_state,
    );

    assert!(engine.state.orb_slots.slots.iter().all(|orb| orb.is_empty()));
}

#[test]
fn current_engine_path_still_channels_when_playing_a_power_with_storm_active() {
    let mut state = combat_state_with(
        crate::tests::support::make_deck(&["Defragment", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.player.set_status(sid::STORM, 1);

    let mut engine = engine_with_state(state);
    engine.state.orb_slots.add_slot();
    engine.state.hand = crate::tests::support::make_deck(&["Defragment", "Zap"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Defragment"));

    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
}

#[test]
fn current_engine_path_storm_self_play_still_channels_lightning() {
    let state = combat_state_with(
        crate::tests::support::make_deck(&["Storm"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    let mut engine = engine_with_state(state);
    engine.state.orb_slots.add_slot();
    engine.state.hand = crate::tests::support::make_deck(&["Storm"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Storm"));

    assert_eq!(engine.state.player.status(sid::STORM), 1);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
}

#[test]
fn playing_storm_installs_visible_status_via_runtime_metadata_lookup() {
    let state = combat_state_with(
        crate::tests::support::make_deck(&["Storm"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    let mut engine = engine_with_state(state);
    engine.state.orb_slots.add_slot();
    engine.state.hand = crate::tests::support::make_deck(&["Storm"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert_eq!(engine.state.player.status(sid::STORM), 0);
    assert!(play_self(&mut engine, "Storm"));

    assert_eq!(engine.state.player.status(sid::STORM), 1);
    assert!(engine
        .effect_runtime
        .has_instance("storm", EffectOwner::PlayerPower));
}
