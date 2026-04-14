#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Reflex.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java

use crate::tests::support::*;

#[test]
fn nonplay_triggers_alchemize_obtains_a_random_potion_and_exhausts() {
    let mut engine = engine_with(make_deck(&["Alchemize"]), 50, 0);

    assert!(play_self(&mut engine, "Alchemize"));
    assert_eq!(exhaust_prefix_count(&engine, "Alchemize"), 1);
    assert!(
        engine.state.potions.iter().any(|p| !p.is_empty()),
        "Alchemize should obtain a potion into the first empty slot"
    );
}

#[test]
fn nonplay_triggers_reflex_draws_on_manual_discard() {
    let mut engine = engine_without_start(
        make_deck(&["Reflex"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Reflex"]);
    engine.state.draw_pile = make_deck(&["Strike_R", "Strike_R"]);

    let reflex = engine.state.hand.remove(0);
    engine.state.discard_pile.push(reflex);
    engine.on_card_discarded(reflex);

    assert_eq!(hand_count(&engine, "Strike_R"), 2);
    assert_eq!(discard_prefix_count(&engine, "Reflex"), 1);
}

#[test]
fn nonplay_triggers_tactician_gains_energy_on_manual_discard() {
    let mut engine = engine_without_start(
        make_deck(&["Tactician"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        0,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Tactician"]);
    let tactician = engine.state.hand.remove(0);
    engine.state.discard_pile.push(tactician);
    engine.on_card_discarded(tactician);

    assert_eq!(engine.state.energy, 1);
    assert_eq!(discard_prefix_count(&engine, "Tactician"), 1);
}

#[test]
fn nonplay_triggers_deus_ex_machina_draws_miracles_on_draw() {
    let engine = engine_with(make_deck(&["DeusExMachina+"]), 50, 0);

    assert_eq!(hand_count(&engine, "Miracle"), 3);
    assert_eq!(hand_count(&engine, "DeusExMachina+"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina+"), 1);
}
