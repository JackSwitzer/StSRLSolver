#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Vajra.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BagOfMarbles.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/ThreadAndNeedle.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Anchor.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Akabeko.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BronzeScales.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/BloodVial.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/ClockworkSouvenir.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/FossilizedHelix.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/DataDisk.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/MarkOfPain.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Lantern.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/OrnamentalFan.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/VioletLotus.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Torii.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/TungstenRod.java

use crate::effects::runtime::EffectOwner;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, engine_without_start,
    make_deck, make_deck_n, play_on_enemy,
};

fn engine_without_start_with_relics(
    relics: &[&str],
    deck: &[&str],
    enemies: Vec<crate::state::EnemyCombatState>,
    energy: i32,
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(make_deck(deck), enemies, energy);
    engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine
}

#[test]
fn combat_start_bundle_applies_simple_java_relic_effects_on_runtime_path() {
    let mut engine = engine_without_start_with_relics(
        &[
            "Vajra",
            "Bag of Marbles",
            "Thread and Needle",
            "Anchor",
            "Akabeko",
            "Bronze Scales",
            "Clockwork Souvenir",
            "Fossilized Helix",
            "Data Disk",
        ],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![
            enemy_no_intent("JawWorm", 50, 50),
            enemy_no_intent("Cultist", 48, 48),
        ],
        3,
    );

    engine.start_combat();

    assert_eq!(engine.state.player.strength(), 1);
    assert_eq!(engine.state.player.status(sid::PLATED_ARMOR), 4);
    assert_eq!(engine.state.player.block, 10);
    assert_eq!(engine.state.player.status(sid::VIGOR), 8);
    assert_eq!(engine.state.player.status(sid::THORNS), 3);
    assert_eq!(engine.state.player.status(sid::ARTIFACT), 1);
    assert_eq!(engine.state.player.status(sid::BUFFER), 1);
    assert_eq!(engine.state.player.status(sid::FOCUS), 1);
    assert!(engine
        .state
        .enemies
        .iter()
        .all(|enemy| enemy.entity.status(sid::VULNERABLE) == 1));
}

#[test]
fn blood_vial_and_mark_of_pain_apply_at_real_combat_start() {
    let mut engine = engine_without_start_with_relics(
        &["Blood Vial", "Mark of Pain"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.player.hp = 79;

    engine.start_combat();

    assert_eq!(engine.state.player.hp, 80);
    let wound_count = engine
        .state
        .draw_pile
        .iter()
        .chain(engine.state.hand.iter())
        .chain(engine.state.discard_pile.iter())
        .filter(|card| engine.card_registry.card_name(card.def_id) == "Wound")
        .count();
    assert_eq!(wound_count, 2);
}

#[test]
fn lantern_grants_bonus_energy_only_on_turn_one_runtime_path() {
    let mut engine = engine_without_start_with_relics(
        &["Lantern"],
        &["Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy("JawWorm", 60, 60, 1, 0, 1)],
        3,
    );

    engine.start_combat();

    assert_eq!(engine.state.turn, 1);
    assert_eq!(engine.state.energy, 4);
    assert_eq!(
        engine.hidden_effect_value("Lantern", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );

    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.energy, 3);
}

#[test]
fn ornamental_fan_grants_block_every_third_attack_on_runtime_path() {
    let mut state = combat_state_with(
        make_deck_n("Strike_R", 12),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    state.relics.push("Ornamental Fan".to_string());
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck_n("Strike_R", 3);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert_eq!(engine.state.player.block, 0);
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));

    assert_eq!(engine.state.player.block, 4);
    assert_eq!(
        engine.hidden_effect_value("Ornamental Fan", EffectOwner::PlayerRelic { slot: 0 }, 0),
        0
    );
}

#[test]
fn violet_lotus_grants_extra_energy_when_exiting_calm() {
    let mut engine = engine_without_start_with_relics(
        &["Violet Lotus"],
        &["Eruption", "Strike_R", "Strike_R", "Strike_R", "Strike_R"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    engine.start_combat();
    engine.state.stance = Stance::Calm;
    engine.state.energy = 3;
    engine.state.hand = make_deck(&["Eruption"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Eruption", 0));

    assert_eq!(engine.state.stance, Stance::Wrath);
    assert_eq!(engine.state.energy, 4);
}

#[test]
fn torii_and_tungsten_rod_reduce_real_hp_loss_from_enemy_attacks() {
    let mut torii = engine_without_start_with_relics(
        &["Torii"],
        &["Defend_R", "Defend_R", "Defend_R", "Defend_R", "Defend_R"],
        vec![enemy("JawWorm", 100, 100, 1, 4, 1)],
        3,
    );
    torii.start_combat();
    let hp_before_torii = torii.state.player.hp;
    end_turn(&mut torii);
    assert_eq!(torii.state.player.hp, hp_before_torii - 1);

    let mut tungsten = engine_without_start_with_relics(
        &["Tungsten Rod"],
        &["Defend_R", "Defend_R", "Defend_R", "Defend_R", "Defend_R"],
        vec![enemy("JawWorm", 100, 100, 1, 10, 1)],
        3,
    );
    tungsten.start_combat();
    let hp_before_tungsten = tungsten.state.player.hp;
    end_turn(&mut tungsten);
    assert_eq!(tungsten.state.player.hp, hp_before_tungsten - 9);
}
