#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DrawPileToHandAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, BulkAction, CardFilter, Effect as E, Pile as P, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::tests::support::{end_turn, enemy_no_intent, force_player_turn, make_deck, play_self, TEST_SEED};

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
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Violence", "Strike", "Strike", "Strike", "Defend", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Violence"]);
    state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Defend", "Defend"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;

    assert!(play_self(&mut engine, "Violence"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine
            .state
            .hand
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .count(),
        3,
    );
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike"))
            .count(),
        0,
    );
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Defend"))
            .count(),
        2,
    );
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
