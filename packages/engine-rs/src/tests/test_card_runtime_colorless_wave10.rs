#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DrawPileToHandAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, BulkAction, CardFilter, Effect as E, Pile as P, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::tests::support::{
    discard_prefix_count, end_turn, enemy_no_intent, engine_without_start,
    exhaust_prefix_count, force_player_turn, make_deck, make_deck_n, play_self, TEST_SEED,
};

#[test]
fn colorless_wave10_registry_exports_enlightenment_and_violence_typed_surfaces() {
    let enlightenment = global_registry().get("Enlightenment").expect("Enlightenment");
    assert_eq!(
        enlightenment.effect_data,
        &[E::ForEachInPile {
            pile: P::Hand,
            filter: CardFilter::All,
            action: BulkAction::SetCostForTurn(1),
        }]
    );
    assert!(enlightenment.complex_hook.is_none());

    let enlightenment_plus = global_registry().get("Enlightenment+").expect("Enlightenment+");
    assert_eq!(
        enlightenment_plus.effect_data,
        &[E::ForEachInPile {
            pile: P::Hand,
            filter: CardFilter::All,
            action: BulkAction::SetCost(1),
        }]
    );
    assert!(enlightenment_plus.complex_hook.is_none());

    let violence = global_registry().get("Violence").expect("Violence");
    assert_eq!(
        violence.effect_data,
        &[E::Simple(SE::DrawRandomCardsFromPileToHand(P::Draw, CardFilter::Attacks, A::Magic))]
    );
    assert!(violence.complex_hook.is_none());

    let violence_plus = global_registry().get("Violence+").expect("Violence+");
    assert_eq!(
        violence_plus.effect_data,
        &[E::Simple(SE::DrawRandomCardsFromPileToHand(P::Draw, CardFilter::Attacks, A::Magic))]
    );
    assert!(violence_plus.complex_hook.is_none());
}

#[test]
fn violence_moves_random_attacks_from_draw_to_hand() {
    // DrawPileToHandAction copies Attacks through addToRandomSpot, consuming
    // cardRandomRng for every eligible card after the first, then shuffles the
    // temporary group once per selection through shuffleRng.randomLong(). For
    // cardRandom seed 42 and shuffle seed 99, the Java selections below are
    // Bash, Strike, Bludgeon, then Uppercut. A full hand sends later selections
    // to discard without changing RNG consumption.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DrawPileToHandAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    let draw = [
        "Strike", "Defend", "Bash", "Neutralize", "Uppercut", "Survivor", "Bludgeon",
    ];

    let mut base = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut base);
    base.card_random_rng = crate::seed::StsRandom::new(42);
    base.rng = crate::seed::StsRandom::new(99);
    base.state.hand = make_deck(&["Violence"]);
    base.state.draw_pile = make_deck(&draw);

    assert!(play_self(&mut base, "Violence"));
    assert_eq!(base.phase, CombatPhase::PlayerTurn);
    let base_hand: Vec<_> = base
        .state
        .hand
        .iter()
        .map(|card| base.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(base_hand, ["Bash", "Strike", "Bludgeon"]);
    assert_eq!(base.card_random_rng.counter, 4);
    assert_eq!(base.rng.counter, 3);
    assert_eq!(exhaust_prefix_count(&base, "Violence"), 1);

    let mut upgraded = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    force_player_turn(&mut upgraded);
    upgraded.card_random_rng = crate::seed::StsRandom::new(42);
    upgraded.rng = crate::seed::StsRandom::new(99);
    upgraded.state.hand = make_deck(&["Violence+"]);
    upgraded.state.hand.extend(make_deck_n("Defend", 8));
    upgraded.state.draw_pile = make_deck(&draw);

    assert!(play_self(&mut upgraded, "Violence+"));
    assert_eq!(upgraded.state.hand.len(), 10);
    assert_eq!(
        upgraded
            .state
            .hand
            .iter()
            .filter(|card| matches!(upgraded.card_registry.card_name(card.def_id), "Bash" | "Strike"))
            .count(),
        2,
    );
    assert_eq!(discard_prefix_count(&upgraded, "Uppercut"), 1);
    assert_eq!(discard_prefix_count(&upgraded, "Bludgeon"), 1);
    assert_eq!(upgraded.card_random_rng.counter, 4);
    assert_eq!(upgraded.rng.counter, 4);
    assert_eq!(exhaust_prefix_count(&upgraded, "Violence"), 1);
}

#[test]
fn enlightenment_base_reduces_hand_cards_above_one_for_this_turn() {
    let mut engine = crate::engine::CombatEngine::new(
        crate::tests::support::combat_state_with(
            make_deck(&["Enlightenment", "Mind Blast", "Mind Blast"]),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        ),
        TEST_SEED,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Enlightenment", "Mind Blast", "Mind Blast"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Enlightenment"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(engine.state.hand[0].cost, 1);
    assert_eq!(engine.state.hand[1].cost, 1);

    end_turn(&mut engine);
    assert!(engine
        .state
        .hand
        .iter()
        .all(|card| {
            let def = engine.card_registry.card_def_by_id(card.def_id);
            i32::from(card.cost) == def.cost
        }));
}

#[test]
fn enlightenment_preserves_zero_one_and_x_costs_and_upgrade_preserves_a_lower_turn_cost() {
    // EnlightenmentAction.java checks costForTurn > 1 and, when upgraded,
    // separately checks base cost > 1. Zero-, one-, and X-cost cards are not
    // raised, and a temporary 0 remains 0 while the upgraded base becomes 1.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
    let mut base = crate::engine::CombatEngine::new(
        crate::tests::support::combat_state_with(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            0,
        ),
        TEST_SEED,
    );
    force_player_turn(&mut base);
    base.state.hand = make_deck(&[
        "Enlightenment", "Mind Blast", "Flash of Steel", "Transmutation", "Strike",
    ]);

    assert!(play_self(&mut base, "Enlightenment"));
    let costs: Vec<(&str, i8, i8)> = base.state.hand.iter().map(|card| {
        (base.card_registry.card_name(card.def_id), card.cost, card.base_cost)
    }).collect();
    assert_eq!(costs, vec![
        ("Mind Blast", 1, 2),
        ("Flash of Steel", -1, 0),
        ("Transmutation", -1, -1),
        ("Strike", -1, 1),
    ]);

    let mut upgraded = crate::engine::CombatEngine::new(
        crate::tests::support::combat_state_with(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            0,
        ),
        TEST_SEED,
    );
    force_player_turn(&mut upgraded);
    upgraded.state.hand = make_deck(&[
        "Enlightenment+", "Mind Blast", "Flash of Steel", "Transmutation", "Strike",
    ]);
    upgraded.state.hand[1].set_cost_for_turn(0);

    assert!(play_self(&mut upgraded, "Enlightenment+"));
    assert_eq!(upgraded.state.hand[0].cost, 0);
    assert_eq!(upgraded.state.hand[0].base_cost, 1);
    upgraded.state.hand[0].reset_cost_for_turn();
    assert_eq!(upgraded.state.hand[0].cost, 1);
    let unchanged: Vec<(&str, i8, i8)> = upgraded.state.hand[1..].iter().map(|card| {
        (upgraded.card_registry.card_name(card.def_id), card.cost, card.base_cost)
    }).collect();
    assert_eq!(unchanged, vec![
        ("Flash of Steel", -1, 0),
        ("Transmutation", -1, -1),
        ("Strike", -1, 1),
    ]);
}
