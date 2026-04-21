#![cfg(test)]

use crate::effects::runtime::EffectOwner;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with, engine_with_state, engine_without_start,
    ensure_in_hand, end_turn, force_player_turn, make_deck, make_deck_n, play_on_enemy, play_self,
};

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/TimeWarpPower.java
fn time_warp_ends_the_turn_before_the_twelfth_card_can_continue_playing() {
    let mut engine = engine_with(make_deck_n("Strike", 12), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.state.enemies[0].entity.set_status(sid::TIME_WARP, 11);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.enemies[0].entity.strength(), 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 44);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/PanachePower.java
fn panache_bursts_on_the_fifth_real_card_play_and_resets_its_counter() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 60, 60),
        enemy_no_intent("Cultist", 55, 55),
    ];
    let mut engine = engine_with_state(combat_state_with(Vec::new(), enemies, 10));
    engine.state.player.set_status(sid::PANACHE, 10);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&[
        "Defend",
        "Defend",
        "Defend",
        "Defend",
        "Defend",
    ]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    for expected in 1..=4 {
        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0), expected);
        assert_eq!(engine.state.enemies[0].entity.hp, 60);
        assert_eq!(engine.state.enemies[1].entity.hp, 55);
    }

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 50);
    assert_eq!(engine.state.enemies[1].entity.hp, 45);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/ThousandCutsPower.java
fn thousand_cuts_hits_all_enemies_after_a_real_card_play() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_with_state(combat_state_with(Vec::new(), enemies, 3));
    engine.state.player.set_status(sid::THOUSAND_CUTS, 2);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.enemies[1].entity.hp, 33);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/relics/OrangePellets.java
fn orange_pellets_clears_all_debuffs_after_attack_skill_and_power_have_all_been_played() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("OrangePellets".to_string());
    engine.state.player.set_status(sid::WEAKENED, 2);
    engine.state.player.set_status(sid::VULNERABLE, 1);
    engine.state.player.set_status(sid::FRAIL, 2);
    engine.state.player.set_status(sid::NO_DRAW, 1);
    engine.state.hand = make_deck(&["Strike", "Defend", "Inflame"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.rebuild_effect_runtime();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_self(&mut engine, "Defend"));
    assert!(play_self(&mut engine, "Inflame"));

    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
    assert_eq!(engine.state.player.status(sid::FRAIL), 0);
    assert_eq!(engine.state.player.status(sid::NO_DRAW), 0);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/relics/Pocketwatch.java
fn pocketwatch_draws_three_extra_cards_after_a_short_previous_turn() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics.push("Pocketwatch".to_string());
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.state.hand = make_deck(&["Strike", "Strike"]);
    engine.state.draw_pile = make_deck_n("Defend", 12);
    engine.state.discard_pile.clear();
    engine.rebuild_effect_runtime();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    end_turn(&mut engine);

    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.hand.len(), 8);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/EchoPower.java
fn echo_form_replays_the_next_card_during_the_card_use_phase() {
    let mut engine = engine_with(make_deck_n("Strike", 6), 50, 0);
    engine.state.player.set_status(sid::ECHO_FORM, 1);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.hand.len(), 0);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/DoubleTapPower.java
fn double_tap_replays_attack_cards_during_the_card_use_phase() {
    let mut engine = engine_with(make_deck_n("Strike", 6), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 38);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/BurstPower.java
fn burst_replays_skill_cards_during_the_card_use_phase() {
    let mut engine = engine_with(make_deck_n("Strike", 6), 50, 0);
    engine.state.player.set_status(sid::BURST, 1);
    engine.rebuild_effect_runtime();
    engine.state.hand = make_deck(&["Backflip"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Backflip"));
    assert!(engine.state.player.block >= 10);
}
