#![cfg(test)]

use crate::actions::Action;
use crate::effects::runtime::{EffectOwner, GameEvent};
use crate::effects::trigger::{Trigger, TriggerContext};
use crate::orbs::OrbType;
use crate::obs;
use crate::run::{RunAction, RunEngine};
use crate::status_ids::sid;

use super::support::{
    combat_state_with, enemy, enemy_no_intent, end_turn, engine_with_state, engine_without_start,
    make_deck, play_on_enemy, play_self, resolve_opening_neow,
};

#[test]
fn enemy_runtime_handlers_mutate_enemy_owner_only() {
    let mut enemy = enemy_no_intent("AwakenedOne", 300, 300);
    enemy.entity.add_status(sid::CURIOSITY, 2);

    let mut engine = engine_without_start(Vec::new(), vec![enemy], 3);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    engine.emit_event(GameEvent::from_trigger(
        Trigger::OnPowerPlayed,
        &TriggerContext {
            card_type: Some(crate::cards::CardType::Power),
            is_first_turn: false,
            target_idx: -1,
        },
    ));

    assert_eq!(engine.state.player.strength(), 0);
    assert_eq!(engine.state.enemies[0].entity.strength(), 2);
    assert!(engine
        .take_event_log()
        .iter()
        .any(|record| record.def_id == Some("curiosity")));
}

#[test]
fn persisted_relic_counters_reload_into_new_combat_runtime() {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Nunchaku".to_string());

    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics = state.relics.clone();
    engine.rebuild_effect_runtime();

    let attack_ctx = TriggerContext {
        card_type: Some(crate::cards::CardType::Attack),
        is_first_turn: false,
        target_idx: 0,
    };
    for _ in 0..4 {
        engine.emit_event(GameEvent::from_trigger(Trigger::OnAttackPlayed, &attack_ctx));
    }

    let persisted = engine.export_persisted_effects();
    assert!(!persisted.is_empty());

    let mut next_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    next_engine.state.relics = vec!["Nunchaku".to_string()];
    next_engine.load_persisted_effects(persisted);

    assert_eq!(
        next_engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        4
    );
}

#[test]
fn potion_use_flows_through_owner_aware_runtime() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Toy Ornithopter".to_string());
    state.potions[0] = "Energy Potion".to_string();
    state.player.hp = 50;

    let mut engine = engine_with_state(state);
    engine.clear_event_log();
    engine.execute_action(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    });

    let events = engine.take_event_log();
    assert_eq!(engine.state.player.hp, 55);
    assert!(engine.state.potions[0].is_empty());
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::ManualActivation && record.def_id == Some("EnergyPotion")));
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::OnPotionUsed && record.potion_slot == 0));
    assert!(events
        .iter()
        .any(|record| record.owner == Some(EffectOwner::PotionSlot { slot: 0 })));
    assert!(events
        .iter()
        .any(|record| record.def_id == Some("Toy Ornithopter")));
}

#[test]
fn block_potion_runtime_keeps_flat_potion_block() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.potions[0] = "Block Potion".to_string();
    state.player.add_status(sid::DEXTERITY, 3);

    let mut engine = engine_with_state(state);
    engine.clear_event_log();
    engine.execute_action(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    });

    let events = engine.take_event_log();
    assert_eq!(engine.state.player.block, 12);
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::ManualActivation && record.def_id == Some("BlockPotion")));
}

#[test]
fn envenom_uses_runtime_damage_resolved_path_on_real_attack_hits() {
    let state = combat_state_with(
        make_deck(&["Strike", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    let mut engine = engine_with_state(state);
    engine.state.player.set_status(sid::ENVENOM, 2);
    engine.state.hand = make_deck(&["Strike", "Strike"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.enemies[0].entity.block = 20;
    engine.rebuild_effect_runtime();

    engine.clear_event_log();
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    let blocked_events = engine.take_event_log();

    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 0);
    assert!(!blocked_events.iter().any(|record| {
        record.event == Trigger::DamageResolved && record.def_id == Some("envenom")
    }));

    ensure_second_strike_for_envenom(&mut engine);
    engine.state.enemies[0].entity.block = 0;
    engine.clear_event_log();
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    let unblocked_events = engine.take_event_log();

    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2);
    assert!(unblocked_events.iter().any(|record| {
        record.event == Trigger::DamageResolved
            && record.def_id == Some("envenom")
            && record.amount > 0
    }));
}

#[test]
fn step_result_exposes_events_and_v2_observation() {
    let mut engine = RunEngine::new(42, 20);
    resolve_opening_neow(&mut engine);
    let map_action = engine.get_legal_actions()[0].clone();
    let _ = engine.step_with_result(&map_action);
    assert_eq!(engine.current_phase(), crate::run::RunPhase::Combat);

    let end_turn = engine
        .get_legal_actions()
        .into_iter()
        .find(|action| matches!(action, RunAction::CombatAction(Action::EndTurn)))
        .expect("combat should always offer EndTurn");

    let result = engine.step_with_result(&end_turn);

    assert_eq!(result.combat_obs_version, obs::COMBAT_OBS_VERSION);
    assert_eq!(result.combat_obs_v2.as_ref().map(Vec::len), Some(obs::COMBAT_V2_DIM));
    assert!(!result.legal_actions.is_empty());
    assert!(!result.combat_events.is_empty());
    assert_eq!(engine.last_combat_events(), result.combat_events.as_slice());
}

#[test]
fn orange_pellets_clears_debuffs_on_attack_skill_power_sequence() {
    let mut state = combat_state_with(make_deck(&["Strike", "Defend", "Inflame"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("OrangePellets".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike", "Defend", "Inflame"]);
    engine.state.player.set_status(sid::WEAKENED, 2);
    engine.state.player.set_status(sid::VULNERABLE, 2);
    engine.state.player.set_status(sid::FRAIL, 2);
    engine.state.player.set_status(sid::NO_DRAW, 1);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_self(&mut engine, "Defend"));
    assert!(play_self(&mut engine, "Inflame"));

    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
    assert_eq!(engine.state.player.status(sid::FRAIL), 0);
    assert_eq!(engine.state.player.status(sid::NO_DRAW), 0);
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn orange_pellets_turn_reset_requires_all_three_types_in_the_same_turn() {
    let mut state = combat_state_with(
        make_deck(&["Strike", "Defend", "Inflame", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics.push("OrangePellets".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike", "Defend", "Inflame"]);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 1),
        1
    );
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 2),
        0
    );

    end_turn(&mut engine);
    engine.state.player.set_status(sid::WEAKENED, 2);
    engine.state.player.set_status(sid::VULNERABLE, 2);
    engine.state.hand = make_deck(&["Inflame"]);

    assert!(play_self(&mut engine, "Inflame"));

    assert_eq!(engine.state.player.status(sid::WEAKENED), 2);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 2);
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 1),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("OrangePellets", EffectOwner::PlayerRelic { slot: 0 }, 2),
        1
    );
}

#[test]
fn sundial_persists_counter_across_combats_on_engine_path() {
    let mut first_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    first_engine.state.relics.push("Sundial".to_string());
    first_engine.rebuild_effect_runtime();

    for _ in 0..2 {
        first_engine.state.hand.clear();
        first_engine.state.draw_pile.clear();
        first_engine.state.discard_pile = make_deck(&["Strike"]);
        first_engine.draw_cards(1);
    }

    assert_eq!(
        first_engine.hidden_effect_value("Sundial", EffectOwner::PlayerRelic { slot: 0 }, 0),
        2
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    next_engine.state.relics.push("Sundial".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.state.hand.clear();
    next_engine.state.draw_pile.clear();
    next_engine.state.discard_pile = make_deck(&["Strike"]);

    next_engine.draw_cards(1);

    assert_eq!(next_engine.state.energy, 5);
    assert_eq!(
        next_engine.hidden_effect_value("Sundial", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn mercury_hourglass_hits_all_enemies_via_turn_start_engine_path() {
    let mut cultist = enemy_no_intent("Cultist", 24, 24);
    cultist.entity.block = 2;
    let jaw_worm = enemy_no_intent("JawWorm", 40, 40);

    let mut state = combat_state_with(Vec::new(), vec![cultist, jaw_worm], 3);
    state.relics.push("Mercury Hourglass".to_string());

    let engine = engine_with_state(state);

    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 23);
    assert_eq!(engine.state.enemies[1].entity.hp, 37);
}

#[test]
fn brimstone_buffs_player_and_all_enemies_on_turn_start() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("Cultist", 24, 24), enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics.push("Brimstone".to_string());

    let engine = engine_with_state(state);

    assert_eq!(engine.state.player.strength(), 2);
    assert_eq!(engine.state.enemies[0].entity.strength(), 1);
    assert_eq!(engine.state.enemies[1].entity.strength(), 1);
}

#[test]
fn philosophers_stone_buffs_all_enemies_at_combat_start() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("Cultist", 24, 24), enemy_no_intent("JawWorm", 40, 40)],
        4,
    );
    state.relics.push("Philosopher's Stone".to_string());

    let engine = engine_with_state(state);

    assert_eq!(engine.state.enemies[0].entity.strength(), 1);
    assert_eq!(engine.state.enemies[1].entity.strength(), 1);
}

#[test]
fn happy_flower_grants_energy_on_every_third_turn_via_engine_path() {
    let mut state = combat_state_with(make_deck(&["Strike"; 20]), vec![enemy("JawWorm", 80, 80, 1, 0, 1)], 3);
    state.relics.push("Happy Flower".to_string());

    let mut engine = engine_with_state(state);
    assert_eq!(engine.state.energy, 3);

    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.energy, 3);

    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 3);
    assert_eq!(engine.state.energy, 4);
    assert_eq!(
        engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn incense_burner_grants_intangible_on_sixth_turn_via_engine_path() {
    let mut state = combat_state_with(make_deck(&["Strike"; 30]), vec![enemy("JawWorm", 120, 120, 1, 0, 1)], 3);
    state.relics.push("Incense Burner".to_string());

    let mut engine = engine_with_state(state);
    for expected_turn in 2..=5 {
        end_turn(&mut engine);
        assert_eq!(engine.state.turn, expected_turn);
        assert_eq!(engine.state.player.status(sid::INTANGIBLE), 0);
    }

    end_turn(&mut engine);

    assert_eq!(engine.state.turn, 6);
    assert_eq!(engine.state.player.status(sid::INTANGIBLE), 1);
    assert_eq!(
        engine.hidden_effect_value("Incense Burner", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn pocketwatch_grants_extra_draw_on_short_previous_turn() {
    let mut state = combat_state_with(make_deck(&["Strike"; 12]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Pocketwatch".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile = make_deck(&[
        "Strike", "Strike", "Strike", "Strike", "Strike", "Strike", "Strike", "Strike",
    ]);
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    end_turn(&mut engine);

    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.hand.len(), 8);
    assert_eq!(
        engine.hidden_effect_value("Pocketwatch", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
    assert_eq!(
        engine.hidden_effect_value("Pocketwatch", EffectOwner::PlayerRelic { slot: 0 }, 1),
        0
    );
}

#[test]
fn mummified_hand_sets_a_remaining_hand_card_cost_to_zero() {
    let mut state = combat_state_with(make_deck(&["Inflame", "Strike", "Defend"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Mummified Hand".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Inflame", "Strike", "Defend"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Inflame"));

    assert_eq!(engine.state.energy, 2);
    assert!(engine.state.hand.iter().any(|card| card.cost == 0));
}

#[test]
fn centennial_puzzle_draws_only_on_first_hp_loss() {
    let mut state = combat_state_with(make_deck(&["Strike"; 10]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Centennial Puzzle".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&[
        "Strike", "Strike", "Strike", "Strike", "Strike", "Strike",
    ]);

    engine.player_lose_hp(4);
    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(engine.state.player.status(sid::CENTENNIAL_PUZZLE_READY), 0);

    engine.player_lose_hp(4);
    assert_eq!(engine.state.hand.len(), 3);
}

#[test]
fn emotion_chip_marks_a_next_turn_orb_pulse_on_hp_loss() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Emotion Chip".to_string());

    let mut engine = engine_with_state(state);
    engine.state.orb_slots.add_slot();
    engine.channel_orb(OrbType::Lightning);
    let hp_before = engine.state.enemies[0].entity.hp;

    engine.player_lose_hp(2);

    assert_eq!(engine.state.enemies[0].entity.hp, hp_before);
    assert_eq!(engine.state.player.status(sid::EMOTION_CHIP_TRIGGER), 1);
}

#[test]
fn red_skull_activates_on_mid_combat_hp_drop_and_clears_on_heal() {
    let mut state = combat_state_with(make_deck(&["Strike"; 5]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Red Skull".to_string());
    state.player.hp = 50;

    let mut engine = engine_with_state(state);
    assert_eq!(engine.state.player.strength(), 0);
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    engine.player_lose_hp(11);
    assert_eq!(engine.state.player.hp, 39);
    assert_eq!(engine.state.player.strength(), 3);
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );

    engine.heal_player(5);
    assert_eq!(engine.state.player.hp, 44);
    assert_eq!(engine.state.player.strength(), 0);
    assert_eq!(
        engine.hidden_effect_value("Red Skull", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn strike_dummy_buffs_strike_damage_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("StrikeDummy".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile.clear();
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 9);
}

#[test]
fn wrist_blade_buffs_zero_cost_attacks_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Shiv"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("WristBlade".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Shiv"]);
    engine.state.draw_pile.clear();
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Shiv", 0));

    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 8);
}

#[test]
fn snecko_skull_buffs_player_applied_poison_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Deadly Poison"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("SneckoSkull".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Deadly Poison"]);
    engine.state.draw_pile.clear();

    assert!(play_on_enemy(&mut engine, "Deadly Poison", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 6);
}

#[test]
fn champion_belt_adds_weak_when_player_applies_vulnerable() {
    let mut state = combat_state_with(make_deck(&["Bash"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Champion Belt".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Bash"]);
    engine.state.draw_pile.clear();

    assert!(play_on_enemy(&mut engine, "Bash", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
    assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
}

#[test]
fn boot_raises_small_unblocked_damage_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Shiv"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Boot".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Shiv"]);
    engine.state.draw_pile.clear();
    engine.state.enemies[0].entity.block = 2;
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Shiv", 0));

    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 3); // D26: Boot -> raw=5, 5-2 block = 3
}

#[test]
fn hand_drill_applies_vulnerable_when_block_breaks_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("HandDrill".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile.clear();
    engine.state.enemies[0].entity.block = 2;

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
}

#[test]
fn sword_boomerang_uses_boot_and_hand_drill_on_custom_multi_hit_path() {
    let mut state = combat_state_with(make_deck(&["Sword Boomerang"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Boot".to_string());
    state.relics.push("HandDrill".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Sword Boomerang"]);
    engine.state.draw_pile.clear();
    engine.state.enemies[0].entity.block = 2;
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Sword Boomerang", 0));

    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 13); // D26: Boot bumps first hit raw=5 (not post-block), later hits unchanged
}

#[test]
fn blade_dance_generated_shiv_respects_wrist_blade_on_engine_path() {
    let mut state = combat_state_with(make_deck(&["Blade Dance"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("WristBlade".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Blade Dance"]);
    engine.state.draw_pile.clear();

    assert!(play_self(&mut engine, "Blade Dance"));

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Shiv", 0));

    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 8);
}

#[test]
fn poison_tick_death_triggers_shared_enemy_death_effects() {
    let state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 1, 1)], 3);

    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.set_status(sid::POISON, 1);
    engine.state.enemies[0].entity.set_status(sid::SPORE_CLOUD, 2);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::VULNERABLE), 1);
    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
}

#[test]
fn thorns_kill_triggers_shared_enemy_death_effects() {
    let state = combat_state_with(make_deck(&["Strike"]), vec![enemy("JawWorm", 3, 3, 1, 5, 1)], 3);

    let mut engine = engine_with_state(state);
    engine.state.player.set_status(sid::THORNS, 3);
    engine.state.enemies[0].entity.set_status(sid::SPORE_CLOUD, 2);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::VULNERABLE), 1);
    assert!(engine.state.combat_over);
    assert!(engine.state.player_won);
}

#[test]
fn the_specimen_transfers_poison_on_engine_death_path() {
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 10, 10), enemy_no_intent("Cultist", 30, 30)],
        3,
    );
    state.relics.push("The Specimen".to_string());

    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.set_status(sid::POISON, 5);
    engine.state.enemies[0].entity.hp = 0;
    engine.clear_event_log();

    engine.finalize_enemy_death(0);

    let events = engine.take_event_log();
    assert_eq!(engine.state.enemies[1].entity.status(sid::POISON), 5);
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::OnEnemyDeath && record.def_id == Some("The Specimen")));
}

#[test]
fn frozen_core_uses_late_turn_hook_without_immediate_passive_block() {
    let mut state = combat_state_with(make_deck(&["Strike"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("FrozenCore".to_string());

    let mut engine = engine_with_state(state);
    engine.init_defect_orbs(3);
    engine.clear_event_log();

    end_turn(&mut engine);

    let events = engine.take_event_log();
    assert_eq!(engine.state.player.block, 0);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Frost);
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::TurnEndPostOrbs && record.def_id == Some("FrozenCore")));
}

#[test]
fn hovering_kite_triggers_once_on_manual_discard_and_resets_next_turn() {
    let mut state = combat_state_with(
        make_deck(&["Concentrate", "Defend", "Defend", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.relics.push("HoveringKite".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Concentrate", "Defend", "Defend", "Defend"]);
    engine.state.draw_pile.clear();
    engine.clear_event_log();
    let energy_before = engine.state.energy;

    assert!(play_self(&mut engine, "Concentrate"));
    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    let first_turn_events = engine.take_event_log();
    assert_eq!(engine.state.energy, energy_before + 3);
    assert!(first_turn_events
        .iter()
        .any(|record| record.event == Trigger::OnCardDiscard && record.def_id == Some("HoveringKite")));

    engine.state.energy = 0;
    engine.emit_event(GameEvent::empty(Trigger::TurnStart));
    engine.on_card_discarded(engine.card_registry.make_card("Defend"));

    assert_eq!(engine.state.energy, 1);
}

#[test]
fn warped_tongs_uses_runtime_turn_start_post_draw_hook() {
    let mut state = combat_state_with(make_deck(&["Defend"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("WarpedTongs".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Defend"]);
    engine.clear_event_log();

    engine.emit_event(GameEvent {
        kind: Trigger::TurnStartPostDrawLate,
        card_type: None,
        card_inst: None,
        is_first_turn: false,
        target_idx: -1,
        enemy_idx: -1,
        potion_slot: -1,
        status_id: None,
        amount: 0,
        replay_window: false,
    });

    let events = engine.take_event_log();
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Defend+");
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::TurnStartPostDrawLate && record.def_id == Some("WarpedTongs")));
}

#[test]
fn gambling_chip_uses_runtime_turn_start_post_draw_hook() {
    let mut state = combat_state_with(make_deck(&["Strike", "Defend"]), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Gambling Chip".to_string());

    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.clear_event_log();

    engine.emit_event(GameEvent {
        kind: Trigger::TurnStartPostDrawLate,
        card_type: None,
        card_inst: None,
        is_first_turn: true,
        target_idx: -1,
        enemy_idx: -1,
        potion_slot: -1,
        status_id: None,
        amount: 0,
        replay_window: false,
    });

    let events = engine.take_event_log();
    assert!(matches!(engine.phase, crate::engine::CombatPhase::AwaitingChoice));
    assert_eq!(engine.state.player.status(sid::GAMBLING_CHIP_ACTIVE), 1);
    assert!(events
        .iter()
        .any(|record| record.event == Trigger::TurnStartPostDrawLate && record.def_id == Some("Gambling Chip")));
}

fn ensure_second_strike_for_envenom(engine: &mut crate::engine::CombatEngine) {
    if !engine
        .state
        .hand
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Strike")
    {
        engine.state.hand = make_deck(&["Strike"]);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
    }
}
