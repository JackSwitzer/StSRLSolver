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
    make_deck, make_deck_n, play_on_enemy, play_self,
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
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
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
fn akabeko_vigor_survives_skills_then_buffs_every_hit_of_the_first_attack() {
    // Source-derived (verify relic/Akabeko): Akabeko.java::atBattleStart
    // applies 8 Vigor. VigorPower.java adds that amount to NORMAL damage and
    // removes itself only when an Attack card is used.
    let mut engine = engine_without_start_with_relics(
        &["Akabeko"],
        &["Defend", "FlyingSleeves", "Strike", "Strike", "Strike"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    engine.start_combat();
    engine.state.hand = make_deck(&["Defend", "FlyingSleeves"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert_eq!(engine.state.player.status(sid::VIGOR), 8);
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.status(sid::VIGOR), 8);

    assert!(play_on_enemy(&mut engine, "FlyingSleeves", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 36);
    assert_eq!(engine.state.player.status(sid::VIGOR), 0);
}

#[test]
fn anchor_applies_ten_block_only_at_combat_start() {
    // Source-derived (verify relic/Anchor): Anchor.java::atBattleStart queues a
    // GainBlockAction for exactly 10 block and has no per-turn block hook.
    let mut engine = engine_without_start_with_relics(
        &["Anchor"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );

    engine.start_combat();
    assert_eq!(engine.state.player.block, 10);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.block, 0);
}

#[test]
fn ancient_tea_set_charge_only_grants_energy_on_the_first_turn() {
    // Source-derived (verify relic/Ancient Tea Set): AncientTeaSet.java checks
    // its armed counter only during the first atTurnStart call, grants 2
    // energy, and immediately resets the counter.
    let mut engine = engine_without_start_with_relics(
        &["Ancient Tea Set"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    engine.state.relic_counters[crate::relic_flags::counter::ANCIENT_TEA_SET] = 1;

    engine.start_combat();
    assert_eq!(engine.state.energy, 5);
    assert_eq!(
        engine.state.relic_counters[crate::relic_flags::counter::ANCIENT_TEA_SET],
        0
    );

    end_turn(&mut engine);
    assert_eq!(engine.state.energy, 3);
}

#[test]
fn art_of_war_tracks_whether_the_previous_turn_used_an_attack() {
    // Source-derived (verify relic/Art of War): ArtOfWar.java starts ready but
    // skips turn one, clears readiness from onUseCard only for Attacks, and
    // restores readiness at every turn start.
    let mut engine = engine_without_start_with_relics(
        &["Art of War"],
        &["Strike", "Defend", "Strike", "Defend", "Strike"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    engine.start_combat();
    assert_eq!(engine.state.energy, 3);

    engine.state.hand = make_deck(&["Strike"]);
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    end_turn(&mut engine);
    assert_eq!(engine.state.energy, 3);

    engine.state.hand = make_deck(&["Defend"]);
    assert!(play_self(&mut engine, "Defend"));
    end_turn(&mut engine);
    assert_eq!(engine.state.energy, 4);
}

#[test]
fn bag_of_marbles_applies_one_vulnerable_to_every_enemy_at_combat_start() {
    // Source-derived (verify relic/Bag of Marbles): BagOfMarbles.java loops
    // over the room's complete monster list and queues VulnerablePower(1) for
    // each monster during atBattleStart.
    let mut engine = engine_without_start_with_relics(
        &["Bag of Marbles"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
        vec![
            enemy_no_intent("JawWorm", 60, 60),
            enemy_no_intent("Cultist", 60, 60),
            enemy_no_intent("FungiBeast", 60, 60),
        ],
        3,
    );

    engine.start_combat();
    assert!(engine
        .state
        .enemies
        .iter()
        .all(|enemy| enemy.entity.status(sid::VULNERABLE) == 1));
}

#[test]
fn bag_of_preparation_draws_two_extra_cards_only_in_the_opening_hand() {
    // Source-derived (verify relic/Bag of Preparation):
    // BagOfPreparation.java::atBattleStart queues exactly one DrawCardAction(2)
    // and defines no later turn-start hook.
    let mut engine = engine_without_start_with_relics(
        &["Bag of Preparation"],
        &[
            "Strike", "Strike", "Strike", "Strike", "Strike", "Defend", "Defend", "Defend",
            "Defend", "Defend", "Defend", "Defend",
        ],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );

    engine.start_combat();
    assert_eq!(engine.state.hand.len(), 7);

    end_turn(&mut engine);
    assert_eq!(engine.state.hand.len(), 5);
}

#[test]
fn blood_vial_and_mark_of_pain_apply_at_real_combat_start() {
    let mut engine = engine_without_start_with_relics(
        &["Blood Vial", "Mark of Pain"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
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
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
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
        make_deck_n("Strike", 12),
        vec![enemy_no_intent("JawWorm", 120, 120)],
        20,
    );
    state.relics.push("Ornamental Fan".to_string());
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck_n("Strike", 3);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 0);
    assert!(play_on_enemy(&mut engine, "Strike", 0));

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
        &["Eruption", "Strike", "Strike", "Strike", "Strike"],
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
fn violet_lotus_uses_its_java_id_and_only_triggers_when_leaving_calm() {
    // Source-derived (verify relic/VioletLotus): VioletLotus.java has no
    // combat-start hook and grants one energy only when the previous stance is
    // Calm and differs from the new stance.
    let mut engine = engine_without_start_with_relics(
        &["VioletLotus"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
        vec![enemy_no_intent("JawWorm", 60, 60)],
        0,
    );
    engine.start_combat();
    assert_eq!(engine.state.player.status(sid::VIOLET_LOTUS), 0);

    engine.state.energy = 0;
    engine.state.stance = Stance::Calm;
    engine.change_stance(Stance::Wrath);
    assert_eq!(engine.state.energy, 3);

    engine.change_stance(Stance::Neutral);
    engine.change_stance(Stance::Calm);
    engine.change_stance(Stance::Calm);
    assert_eq!(engine.state.energy, 3);

    engine.change_stance(Stance::Wrath);
    assert_eq!(engine.state.energy, 6);
}

#[test]
fn torii_and_tungsten_rod_reduce_real_hp_loss_from_enemy_attacks() {
    let mut torii = engine_without_start_with_relics(
        &["Torii"],
        &["Defend", "Defend", "Defend", "Defend", "Defend"],
        vec![enemy("JawWorm", 100, 100, 1, 4, 1)],
        3,
    );
    torii.start_combat();
    let hp_before_torii = torii.state.player.hp;
    end_turn(&mut torii);
    assert_eq!(torii.state.player.hp, hp_before_torii - 1);

    let mut tungsten = engine_without_start_with_relics(
        &["Tungsten Rod"],
        &["Defend", "Defend", "Defend", "Defend", "Defend"],
        vec![enemy("JawWorm", 100, 100, 1, 10, 1)],
        3,
    );
    tungsten.start_combat();
    let hp_before_tungsten = tungsten.state.player.hp;
    end_turn(&mut tungsten);
    assert_eq!(tungsten.state.player.hp, hp_before_tungsten - 9);
}
