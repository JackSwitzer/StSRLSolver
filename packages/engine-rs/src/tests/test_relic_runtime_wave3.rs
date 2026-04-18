#![cfg(test)]

use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;

use super::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, engine_without_start,
    make_deck, make_deck_n, play_on_enemy, play_self,
};

fn engine_with_relic_and_attacks(relic_id: &str, attack_count: usize) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck_n("Strike", attack_count.max(12)),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    state.relics.push(relic_id.to_string());
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck_n("Strike", attack_count);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine
}

#[test]
fn ornamental_fan_triggers_on_third_attack_and_resets_each_turn() {
    let mut engine = engine_with_relic_and_attacks("Ornamental Fan", 5);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 0);
    assert_eq!(
        engine.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 0 }, 0),
        2
    );

    end_turn(&mut engine);
    engine.state.hand = make_deck_n("Strike", 1);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 0);
    assert_eq!(
        engine.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );

    engine.state.hand = make_deck_n("Strike", 2);
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 4);
    assert_eq!(
        engine.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn kunai_triggers_on_third_attack_and_resets_each_turn() {
    let mut engine = engine_with_relic_and_attacks("Kunai", 5);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.dexterity(), 0);

    end_turn(&mut engine);
    engine.state.hand = make_deck_n("Strike", 1);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.dexterity(), 0);
    assert_eq!(
        engine.hidden_effect_value("Kunai", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );

    engine.state.hand = make_deck_n("Strike", 2);
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.dexterity(), 1);
    assert_eq!(
        engine.hidden_effect_value("Kunai", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn shuriken_triggers_on_third_attack_and_resets_each_turn() {
    let mut engine = engine_with_relic_and_attacks("Shuriken", 5);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.strength(), 0);

    end_turn(&mut engine);
    engine.state.hand = make_deck_n("Strike", 1);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.strength(), 0);
    assert_eq!(
        engine.hidden_effect_value("Shuriken", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );

    engine.state.hand = make_deck_n("Strike", 2);
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.strength(), 1);
    assert_eq!(
        engine.hidden_effect_value("Shuriken", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn nunchaku_persists_nine_attacks_and_grants_energy_on_tenth_attack() {
    let mut first_engine = engine_with_relic_and_attacks("Nunchaku", 9);
    let starting_energy = first_engine.state.energy;

    for _ in 0..9 {
        assert!(play_on_enemy(&mut first_engine, "Strike", 0));
    }

    assert_eq!(first_engine.state.energy, starting_energy - 9);
    assert_eq!(
        first_engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        9
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike", 1),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    next_engine.state.relics.push("Nunchaku".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();
    next_engine.state.hand = make_deck_n("Strike", 1);
    next_engine.state.draw_pile.clear();
    next_engine.state.discard_pile.clear();
    let energy_before = next_engine.state.energy;

    assert!(play_on_enemy(&mut next_engine, "Strike", 0));

    assert_eq!(next_engine.state.energy, energy_before);
    assert_eq!(
        next_engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn ink_bottle_persists_nine_cards_and_draws_on_tenth_card() {
    let mut first_engine = engine_with_relic_and_attacks("InkBottle", 9);

    for _ in 0..9 {
        assert!(play_on_enemy(&mut first_engine, "Strike", 0));
    }

    assert_eq!(
        first_engine.hidden_effect_value("InkBottle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        9
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike", 4),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    next_engine.state.relics.push("InkBottle".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();
    next_engine.state.hand = make_deck_n("Strike", 1);
    next_engine.state.draw_pile = make_deck(&["Defend"]);
    next_engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut next_engine, "Strike", 0));

    assert!(next_engine
        .state
        .hand
        .iter()
        .any(|card| next_engine.card_registry.card_name(card.def_id) == "Defend"));
    assert_eq!(
        next_engine.hidden_effect_value("InkBottle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn happy_flower_persists_turn_counter_across_combats() {
    let mut state = combat_state_with(
        make_deck_n("Strike", 20),
        vec![enemy("JawWorm", 120, 120, 1, 0, 1)],
        3,
    );
    state.relics.push("Happy Flower".to_string());

    let mut first_engine = engine_with_state(state);
    assert_eq!(
        first_engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 0 }, 0),
        1
    );
    end_turn(&mut first_engine);
    assert_eq!(
        first_engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 0 }, 0),
        2
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike", 20),
        vec![enemy("JawWorm", 120, 120, 1, 0, 1)],
        3,
    );
    next_engine.state.relics.push("Happy Flower".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();

    assert_eq!(next_engine.state.turn, 1);
    assert_eq!(next_engine.state.energy, 4);
    assert_eq!(
        next_engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn incense_burner_persists_turn_counter_across_combats() {
    let mut state = combat_state_with(
        make_deck_n("Strike", 40),
        vec![enemy("JawWorm", 120, 120, 1, 0, 1)],
        3,
    );
    state.relics.push("Incense Burner".to_string());

    let mut first_engine = engine_with_state(state);
    for _ in 0..4 {
        end_turn(&mut first_engine);
    }
    assert_eq!(
        first_engine.hidden_effect_value("Incense Burner", EffectOwner::PlayerRelic { slot: 0 }, 0),
        5
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike", 20),
        vec![enemy("JawWorm", 120, 120, 1, 0, 1)],
        3,
    );
    next_engine.state.relics.push("Incense Burner".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();

    assert_eq!(next_engine.state.player.status(sid::INTANGIBLE), 1);
    assert_eq!(
        next_engine.hidden_effect_value("Incense Burner", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn sundial_persists_two_shuffles_and_triggers_on_third_shuffle() {
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
fn orange_pellets_turn_reset_requires_all_three_card_types_in_same_turn() {
    let mut state = combat_state_with(
        make_deck(&["Strike", "Defend", "Inflame", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        5,
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
