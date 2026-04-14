#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MadnessAction.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Condition, Effect as E, Pile as P, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self};

#[test]
fn colorless_wave8_registry_exports_match_typed_surface_for_forethought_and_impatience() {
    let registry = global_registry();

    let forethought = registry.get("Forethought").expect("Forethought should exist");
    assert_eq!(forethought.card_type, CardType::Skill);
    assert_eq!(forethought.target, CardTarget::None);
    assert_eq!(
        forethought.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::PutOnBottomAtCostZero,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(forethought.complex_hook.is_none());

    let forethought_plus = registry.get("Forethought+").expect("Forethought+ should exist");
    assert_eq!(
        forethought_plus.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::PutOnBottomAtCostZero,
            min_picks: A::Fixed(0),
            max_picks: A::Fixed(99),
        }]
    );
    assert!(forethought_plus.complex_hook.is_none());

    let impatience = registry.get("Impatience").expect("Impatience should exist");
    assert_eq!(
        impatience.effect_data,
        &[E::Conditional(
            Condition::HandContainsType(CardType::Attack),
            &[],
            &[E::Simple(SE::DrawCards(A::Magic))],
        )]
    );
    assert!(impatience.complex_hook.is_none());

    let impatience_plus = registry.get("Impatience+").expect("Impatience+ should exist");
    assert_eq!(
        impatience_plus.effect_data,
        &[E::Conditional(
            Condition::HandContainsType(CardType::Attack),
            &[],
            &[E::Simple(SE::DrawCards(A::Magic))],
        )]
    );
    assert!(impatience_plus.complex_hook.is_none());
}

#[test]
fn forethought_puts_one_selected_card_on_bottom_at_zero_cost() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought", "Strike_R", "Defend_R"]);

    assert!(play_self(&mut engine, "Forethought"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.draw_pile[0].def_id), "Strike_R");
    assert_eq!(engine.state.draw_pile[0].cost, 0);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Defend_R");
}

#[test]
fn impatience_draws_when_no_attacks_are_in_hand() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Impatience", "Defend_R", "Defend_R"]);
    engine.state.draw_pile = make_deck(&["Strike_R", "Strike_R", "Strike_R"]);

    assert!(play_self(&mut engine, "Impatience"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 4);
}

#[test]
fn impatience_does_not_draw_when_an_attack_is_present() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Impatience", "Strike_R", "Defend_R"]);
    engine.state.draw_pile = make_deck(&["Strike_R", "Strike_R", "Strike_R"]);

    assert!(play_self(&mut engine, "Impatience"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
}

#[test]
#[ignore = "Enlightenment still needs the turn-only cost-reduction primitive; Java updates costForTurn for the turn and only permanently reduces upgraded cards. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java"]
fn enlightenment_still_needs_turn_only_cost_reduction_primitive() {}

#[test]
#[ignore = "Madness still needs a random-hand-card zero-cost primitive; Java repeatedly samples the hand until it finds a card that can be reduced. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MadnessAction.java"]
fn madness_still_needs_random_hand_card_zero_cost_primitive() {}
