#![cfg(test)]

use super::DEF_STORM;
use crate::cards::CardType;
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::Trigger;
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_on_enemy, play_self,
};

#[test]
fn beat_of_death_engine_path_uses_java_thorns_damage_and_consumes_block() {
    // BeatOfDeathPower.onAfterUseCard queues DamageInfo.THORNS. AbstractPlayer.damage
    // applies block before HP loss for THORNS damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BeatOfDeathPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    let mut state = combat_state_with(
        make_deck(&["Miracle"]),
        vec![enemy_no_intent("CorruptHeart", 750, 750)],
        3,
    );
    state.enemies[0].entity.set_status(sid::BEAT_OF_DEATH, 1);
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Miracle"]);
    engine.state.player.hp = 70;
    engine.state.player.block = 3;

    assert!(play_self(&mut engine, "Miracle"));

    assert_eq!(engine.state.player.block, 2);
    assert_eq!(engine.state.player.hp, 70);
}

#[test]
fn lethal_card_still_receives_beat_of_death_after_use_callback() {
    // UseCardAction invokes every monster power's onAfterUseCard even when the
    // card's queued damage has already killed that monster.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BeatOfDeathPower.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("CorruptHeart", 1, 750)],
        3,
    );
    state.enemies[0].entity.set_status(sid::BEAT_OF_DEATH, 1);
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.player.hp = 70;
    engine.state.player.block = 3;

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert!(engine.state.enemies[0].entity.is_dead());
    assert_eq!(engine.state.player.block, 2);
    assert_eq!(engine.state.player.hp, 70);
}

#[test]
fn storm_hook_channels_one_lightning_per_stack_on_power_play_event() {
    // StormPower.onUseCard loops over its amount and queues one Lightning for
    // each stack whenever the used card is a Power.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StormPower.java
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::STORM, 2);
    engine.state.orb_slots.add_slot();
    engine.state.orb_slots.add_slot();

    let mut effect_state = EffectState::default();
    let hook = DEF_STORM
        .complex_hook
        .expect("Storm should provide a runtime hook");
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
    assert_eq!(engine.state.orb_slots.slots[1].orb_type, OrbType::Lightning);
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
    let hook = DEF_STORM
        .complex_hook
        .expect("Storm should provide a runtime hook");
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

    assert!(engine
        .state
        .orb_slots
        .slots
        .iter()
        .all(|orb| orb.is_empty()));
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
fn current_engine_path_storm_self_play_does_not_trigger_the_new_power() {
    // StormPower.onUseCard is invoked only on powers already owned when the
    // card is used; Storm's ApplyPowerAction has not resolved at that point.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StormPower.java
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
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Empty);
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
