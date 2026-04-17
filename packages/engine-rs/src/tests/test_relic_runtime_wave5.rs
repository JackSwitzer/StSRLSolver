#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OrangePellets.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Pocketwatch.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/PenNib.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/VelvetChoker.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Nunchaku.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/InkBottle.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/HappyFlower.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/IncenseBurner.java

use crate::actions::Action;
use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with_state, engine_without_start,
    make_deck, make_deck_n, play_on_enemy, play_self,
};

fn relic_engine(relic_id: &str, energy: i32) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck_n("Strike_R", 20),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        energy,
    );
    state.relics.push(relic_id.to_string());
    let mut engine = engine_with_state(state);
    engine.state.energy = energy;
    engine.state.hand.clear();
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine
}

#[test]
fn orange_pellets_clears_debuffs_after_all_three_card_types_in_one_turn() {
    let mut engine = relic_engine("OrangePellets", 20);
    engine.state.hand = make_deck(&["Strike_R", "Defend_R", "Inflame"]);
    engine.state.player.set_status(sid::WEAKENED, 2);
    engine.state.player.set_status(sid::VULNERABLE, 2);

    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_self(&mut engine, "Defend_R"));

    assert!(play_self(&mut engine, "Inflame"));
    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
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
        0
    );
}

#[test]
fn pocketwatch_grants_draw_after_short_previous_turn_and_skips_first_turn_bonus() {
    let mut engine = relic_engine("Pocketwatch", 3);
    engine.state.hand = make_deck(&["Strike_R", "Strike_R", "Strike_R", "Strike_R"]);
    engine.state.draw_pile = make_deck_n("Defend_R", 8);

    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));

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
fn pen_nib_triggers_on_tenth_attack_and_resets() {
    let mut engine = relic_engine("Pen Nib", 20);
    engine.state.hand = make_deck_n("Strike_R", 10);

    for expected_counter in 1..=9 {
        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Strike_R", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 6);
        assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), expected_counter);
    }

    let hp_before_tenth = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before_tenth - 12);
    assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), 0);
}

#[test]
fn velvet_choker_blocks_the_seventh_card_and_resets_next_turn() {
    let mut engine = relic_engine("Velvet Choker", 20);
    engine.state.hand = make_deck_n("Defend_R", 7);

    for expected in 1..=6 {
        let hand_before = engine.state.hand.len();
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        });
        assert_eq!(engine.state.hand.len(), hand_before - 1);
        assert_eq!(engine.state.cards_played_this_turn, expected);
        assert_eq!(
            engine.hidden_effect_value("Velvet Choker", EffectOwner::PlayerRelic { slot: 0 }, 0),
            expected
        );
    }

    let hand_before = engine.state.hand.len();
    let energy_before = engine.state.energy;
    engine.execute_action(&Action::PlayCard {
        card_idx: 0,
        target_idx: -1,
    });
    assert_eq!(engine.state.hand.len(), hand_before);
    assert_eq!(engine.state.energy, energy_before);
    assert_eq!(engine.state.cards_played_this_turn, 6);
    assert_eq!(
        engine.hidden_effect_value("Velvet Choker", EffectOwner::PlayerRelic { slot: 0 }, 0),
        6
    );

    end_turn(&mut engine);
    assert_eq!(
        engine.hidden_effect_value("Velvet Choker", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn nunchaku_persists_across_combats_and_grants_energy_on_the_tenth_attack() {
    let mut first_engine = relic_engine("Nunchaku", 20);
    first_engine.state.hand = make_deck_n("Strike_R", 10);

    for _ in 0..9 {
        assert!(play_on_enemy(&mut first_engine, "Strike_R", 0));
    }

    assert_eq!(
        first_engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        9
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike_R", 1),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        20,
    );
    next_engine.state.relics.push("Nunchaku".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();
    next_engine.state.hand = make_deck_n("Strike_R", 1);

    let energy_before = next_engine.state.energy;
    assert!(play_on_enemy(&mut next_engine, "Strike_R", 0));
    assert_eq!(next_engine.state.energy, energy_before);
    assert_eq!(
        next_engine.hidden_effect_value("Nunchaku", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn ink_bottle_persists_across_combats_and_draws_on_the_tenth_card() {
    let mut first_engine = relic_engine("InkBottle", 20);
    first_engine.state.hand = make_deck_n("Strike_R", 10);

    for _ in 0..9 {
        assert!(play_on_enemy(&mut first_engine, "Strike_R", 0));
    }

    assert_eq!(
        first_engine.hidden_effect_value("InkBottle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        9
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike_R", 1),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        20,
    );
    next_engine.state.relics.push("InkBottle".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();
    next_engine.state.hand = make_deck_n("Strike_R", 1);
    next_engine.state.draw_pile = make_deck(&["Defend_R"]);

    assert!(play_on_enemy(&mut next_engine, "Strike_R", 0));
    assert!(next_engine
        .state
        .hand
        .iter()
        .any(|card| next_engine.card_registry.card_name(card.def_id) == "Defend_R"));
    assert_eq!(
        next_engine.hidden_effect_value("InkBottle", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn happy_flower_persists_turn_progress_across_combats() {
    let mut first_engine = relic_engine("Happy Flower", 3);
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
        make_deck_n("Strike_R", 8),
        vec![enemy_no_intent("JawWorm", 160, 160)],
        3,
    );
    next_engine.state.relics.push("Happy Flower".to_string());
    next_engine.load_persisted_effects(persisted);
    next_engine.start_combat();

    assert_eq!(next_engine.state.energy, 4);
    assert_eq!(
        next_engine.hidden_effect_value("Happy Flower", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn incense_burner_persists_turn_progress_across_combats() {
    let mut first_engine = relic_engine("Incense Burner", 3);
    for _ in 0..4 {
        end_turn(&mut first_engine);
    }

    assert_eq!(
        first_engine.hidden_effect_value("Incense Burner", EffectOwner::PlayerRelic { slot: 0 }, 0),
        5
    );

    let persisted = first_engine.export_persisted_effects();
    let mut next_engine = engine_without_start(
        make_deck_n("Strike_R", 8),
        vec![enemy_no_intent("JawWorm", 160, 160)],
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
