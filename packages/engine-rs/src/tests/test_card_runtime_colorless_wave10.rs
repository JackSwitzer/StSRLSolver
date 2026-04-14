#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DrawPileToHandAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, BulkAction, CardFilter, Effect as E, Pile as P, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::tests::support::{enemy_no_intent, force_player_turn, make_deck, play_self, TEST_SEED};

#[test]
fn violence_uses_the_typed_random_attack_fetch_surface() {
    let enlightenment = global_registry().get("Enlightenment").expect("Enlightenment");
    assert!(enlightenment.effect_data.is_empty());
    assert!(enlightenment.complex_hook.is_some());

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
        make_deck(&["Violence", "Strike_B", "Strike_B", "Strike_B", "Defend_B", "Defend_B"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Violence"]);
    state.draw_pile = make_deck(&["Strike_B", "Strike_B", "Strike_B", "Defend_B", "Defend_B"]);
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
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike_"))
            .count(),
        3,
    );
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Strike_"))
            .count(),
        0,
    );
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|c| engine.card_registry.card_name(c.def_id).starts_with("Defend_"))
            .count(),
        2,
    );
}

#[test]
#[ignore = "Enlightenment base still needs a turn-only cost-reduction lifetime primitive; Java updates costForTurn for the turn and only permanently reduces upgraded cards. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java"]
fn enlightenment_still_needs_turn_only_cost_reduction_lifetime() {}
