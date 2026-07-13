#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MadnessAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{Effect as E, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self};

#[test]
fn colorless_wave9_registry_exports_match_typed_surface_for_madness() {
    let registry = global_registry();

    let madness = registry.get("Madness").expect("Madness should exist");
    assert_eq!(madness.card_type, CardType::Skill);
    assert_eq!(madness.target, CardTarget::SelfTarget);
    assert_eq!(madness.effect_data, &[E::Simple(SE::SetRandomHandCardCost(0))]);
    assert!(madness.complex_hook.is_none());

    let madness_plus = registry.get("Madness+").expect("Madness+ should exist");
    assert_eq!(madness_plus.card_type, CardType::Skill);
    assert_eq!(madness_plus.target, CardTarget::SelfTarget);
    assert_eq!(madness_plus.effect_data, &[E::Simple(SE::SetRandomHandCardCost(0))]);
    assert!(madness_plus.complex_hook.is_none());
}

#[test]
fn madness_sets_one_random_eligible_hand_card_to_zero_cost_and_exhausts() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Madness", "Strike", "Defend"]);

    assert!(play_self(&mut engine, "Madness"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.exhaust_pile[0].def_id), "Madness");
    assert_eq!(
        engine.state.hand.iter().filter(|card| card.cost == 0).count(),
        1
    );
}

#[test]
fn madness_source_retries_card_random_and_can_make_a_temporarily_free_card_permanent() {
    // MadnessAction.java samples the whole hand with cardRandomRng until it
    // finds costForTurn > 0. For seed 42 and the two-card post-play hand below,
    // indices are 1,1,1,1,0: four rejected temporary-zero Defends, then Strike.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Madness", "Strike", "Defend"]);
    engine.state.hand[2].set_cost_for_turn(0);
    let card_random_before = engine.card_random_rng.counter;

    assert!(play_self(&mut engine, "Madness"));
    assert_eq!(engine.card_random_rng.counter, card_random_before + 5);
    assert_eq!(engine.state.hand[0].base_cost, 0);
    assert_eq!(engine.state.hand[1].base_cost, 1);

    // With no positive costForTurn, MadnessAction falls back to permanent
    // `cost > 0`, so a temporarily free Strike still becomes permanently free.
    let mut fallback = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut fallback);
    fallback.state.hand = make_deck(&["Madness+", "Strike"]);
    fallback.state.hand[1].set_cost_for_turn(0);
    let fallback_rng_before = fallback.card_random_rng.counter;

    assert!(play_self(&mut fallback, "Madness+"));
    assert_eq!(fallback.card_random_rng.counter, fallback_rng_before + 1);
    assert_eq!(fallback.state.hand[0].cost, 0);
    assert_eq!(fallback.state.hand[0].base_cost, 0);
}
