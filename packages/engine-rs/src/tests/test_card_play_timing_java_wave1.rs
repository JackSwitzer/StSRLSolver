#![cfg(test)]

use crate::effects::runtime::EffectOwner;
use crate::engine::CombatEngine;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with, engine_with_state,
    engine_without_start, ensure_in_hand, force_player_turn, make_deck, make_deck_n, play_on_enemy,
    play_self,
};

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/TimeWarpPower.java
fn time_warp_ends_the_turn_before_the_twelfth_card_can_continue_playing() {
    let mut engine = engine_with(make_deck_n("Strike", 12), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    engine.state.enemies[0]
        .entity
        .set_status(sid::TIME_WARP_ACTIVE, 1);
    engine.state.enemies[0]
        .entity
        .set_status(sid::TIME_WARP, 11);
    engine.rebuild_effect_runtime();
    engine.clear_event_log();
    ensure_in_hand(&mut engine, "Strike");
    let cards_before = engine.state.hand.len()
        + engine.state.draw_pile.len()
        + engine.state.discard_pile.len()
        + engine.state.exhaust_pile.len();

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.state.enemies[0].entity.strength(), 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 44);
    assert_eq!(
        engine.state.hand.len()
            + engine.state.draw_pile.len()
            + engine.state.discard_pile.len()
            + engine.state.exhaust_pile.len(),
        cards_before,
        "UseCardAction must move the twelfth card before Time Warp shuffles the turn"
    );
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
    engine.state.hand = make_deck(&["Defend", "Defend", "Defend", "Defend", "Defend"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    for expected in 1..=4 {
        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(
            engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
            expected
        );
        assert_eq!(engine.state.enemies[0].entity.hp, 60);
        assert_eq!(engine.state.enemies[1].entity.hp, 55);
    }

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(
        engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
        0
    );
    assert_eq!(engine.state.enemies[0].entity.hp, 50);
    assert_eq!(engine.state.enemies[1].entity.hp, 45);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panache.java
// decompiled/java-src/com/megacrit/cardcrawl/powers/PanachePower.java
fn panache_counts_its_own_play_stacks_damage_and_bursts_as_thorns() {
    let mut grounded = enemy_no_intent("JawWorm", 100, 100);
    grounded.entity.block = 4;
    grounded.entity.set_status(sid::SLOW, 5);
    grounded.entity.set_status(sid::FLIGHT, 3);
    grounded.entity.set_status(sid::CURL_UP, 12);
    grounded.entity.set_status(sid::MALLEABLE, 3);
    let mut intangible = enemy_no_intent("Cultist", 100, 100);
    intangible.entity.set_status(sid::INTANGIBLE, 1);

    let mut engine =
        engine_with_state(combat_state_with(Vec::new(), vec![grounded, intangible], 3));
    engine.state.hand = make_deck(&["Panache", "Panache+", "Defend", "Defend", "Defend"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Panache"));
    assert_eq!(
        engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
        1
    );
    assert!(play_self(&mut engine, "Panache+"));
    assert_eq!(engine.state.player.status(sid::PANACHE), 24);
    assert_eq!(
        engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
        2
    );
    assert!(play_self(&mut engine, "Defend"));
    assert!(play_self(&mut engine, "Defend"));
    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(
        engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0),
        0
    );
    assert_eq!(engine.state.enemies[0].entity.hp, 80);
    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CURL_UP), 12);
    assert_eq!(engine.state.enemies[0].entity.status(sid::MALLEABLE), 3);
    assert_eq!(engine.state.enemies[1].entity.hp, 99);
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
// Java oracles:
// decompiled/java-src/com/megacrit/cardcrawl/powers/HexPower.java
// decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
// decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
fn hex_randomly_inserts_each_dazed_without_replacing_the_draw_top() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("Chosen", 95, 95)],
        3,
    ));
    engine.state.player.set_status(sid::HEX, 2);
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.draw_pile = make_deck(&["Strike", "Bash", "Inflame"]);
    engine.state.discard_pile.clear();
    engine.rebuild_effect_runtime();
    let card_random_before = engine.card_random_rng.counter;

    assert!(play_self(&mut engine, "Defend"));

    assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile.last().expect("draw top").def_id),
        "Inflame"
    );
    let existing: Vec<_> = engine
        .state
        .draw_pile
        .iter()
        .filter_map(|card| {
            let id = engine.card_registry.card_name(card.def_id);
            (id != "Dazed").then_some(id)
        })
        .collect();
    assert_eq!(existing, ["Strike", "Bash", "Inflame"]);
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Dazed")
            .count(),
        2
    );
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
    engine.state.player.set_status(sid::HEX, 1);
    engine.state.player.set_status(sid::CONFUSION, 1);
    engine.state.player.set_status(sid::SLOW, 1);
    engine.state.player.set_status(sid::NO_BLOCK, 1);
    engine.state.hand = make_deck(&["Strike", "Defend", "Inflame"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.rebuild_effect_runtime();
    // Confusion randomizes each cost to 0-3; ample energy keeps all three
    // source-required card types executable in this purge scenario.
    engine.state.energy = 20;

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert!(play_self(&mut engine, "Defend"));
    assert!(play_self(&mut engine, "Inflame"));

    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
    assert_eq!(engine.state.player.status(sid::VULNERABLE), 0);
    assert_eq!(engine.state.player.status(sid::FRAIL), 0);
    assert_eq!(engine.state.player.status(sid::NO_DRAW), 0);
    assert_eq!(engine.state.player.status(sid::HEX), 0);
    assert_eq!(engine.state.player.status(sid::CONFUSION), 0);
    assert_eq!(engine.state.player.status(sid::SLOW), 0);
    assert_eq!(engine.state.player.status(sid::NO_BLOCK), 0);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/relics/Pocketwatch.java
fn pocketwatch_draws_three_extra_cards_after_a_short_previous_turn() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
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
// decompiled/java-src/com/megacrit/cardcrawl/relics/Pocketwatch.java
fn pocketwatch_three_card_threshold_is_inclusive_and_four_cards_disable_bonus_draw() {
    for (cards_played, expected_hand) in [(3, 8), (4, 5)] {
        let mut engine =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 80, 80)], 10);
        engine.state.relics.push("Pocketwatch".to_string());
        force_player_turn(&mut engine);
        engine.state.turn = 1;
        engine.state.hand = make_deck_n("Defend", cards_played);
        engine.state.draw_pile = make_deck_n("Strike", 12);
        engine.state.discard_pile.clear();
        engine.rebuild_effect_runtime();

        for _ in 0..cards_played {
            assert!(play_self(&mut engine, "Defend"));
        }
        end_turn(&mut engine);

        assert_eq!(engine.state.turn, 2);
        assert_eq!(engine.state.hand.len(), expected_hand);
    }
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

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
fn strange_spoon_uses_one_boolean_roll_and_sends_saved_exhaust_to_discard() {
    let mut saw_discard = false;
    let mut saw_exhaust = false;

    for card_random_seed in 0..64 {
        let state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
        let mut engine = CombatEngine::new_with_card_random_seed(state, 0, card_random_seed);
        engine.state.relics.push("Strange Spoon".to_string());
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&["Miracle"]);

        let before = engine.rng_counters()["cardRandom"];
        assert!(play_self(&mut engine, "Miracle"));
        assert_eq!(engine.rng_counters()["cardRandom"], before + 1);
        assert!(engine.state.draw_pile.is_empty());

        let discarded = engine
            .state
            .discard_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Miracle");
        let exhausted = engine
            .state
            .exhaust_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Miracle");
        assert_ne!(discarded, exhausted);
        saw_discard |= discarded;
        saw_exhaust |= exhausted;
    }

    assert!(saw_discard, "sampled seeds must exercise the Spoon save");
    assert!(saw_exhaust, "sampled seeds must exercise normal exhaustion");
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Miracle.java
fn miracle_and_upgrade_self_retain_when_left_unplayed() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Miracle", "Miracle+"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    end_turn(&mut engine);

    let hand_names: Vec<_> = engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(hand_names, vec!["Miracle", "Miracle+"]);
    assert!(engine.state.discard_pile.is_empty());
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/relics/UnceasingTop.java
fn unceasing_top_does_not_draw_or_retry_while_no_draw_is_active() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("Unceasing Top".to_string());
    engine.state.player.set_status(sid::NO_DRAW, 1);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.draw_pile = make_deck_n("Strike", 3);
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Defend"));

    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.draw_pile.len(), 3);
    assert_eq!(engine.state.discard_pile.len(), 1);
}
