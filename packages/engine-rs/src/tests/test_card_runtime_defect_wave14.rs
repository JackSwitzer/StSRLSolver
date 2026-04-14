#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Seek.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Chaos.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Reboot.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Redo.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Scrape.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::tests::support::{enemy_no_intent, force_player_turn, make_deck, play_self, TEST_SEED};

#[test]
fn defect_wave14_registry_exports_seek_on_the_typed_search_surface() {
    let seek = global_registry().get("Seek").expect("Seek");
    assert_eq!(
        seek.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::MoveToHand,
            min_picks: A::Fixed(1),
            max_picks: A::Magic,
        }]
    );
    assert!(seek.complex_hook.is_none());

    let seek_plus = global_registry().get("Seek+").expect("Seek+");
    assert_eq!(
        seek_plus.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::MoveToHand,
            min_picks: A::Fixed(1),
            max_picks: A::Magic,
        }]
    );
    assert!(seek_plus.complex_hook.is_none());

    let chaos = global_registry().get("Chaos").expect("Chaos");
    assert!(chaos.effect_data.is_empty());
    assert!(chaos.complex_hook.is_some());

    let fission = global_registry().get("Fission").expect("Fission");
    assert!(fission.effect_data.is_empty());
    assert!(fission.complex_hook.is_some());

    let reboot = global_registry().get("Reboot").expect("Reboot");
    assert!(reboot.effect_data.is_empty());
    assert!(reboot.complex_hook.is_some());

    let redo = global_registry().get("Redo").expect("Redo");
    assert!(redo.effect_data.is_empty());
    assert!(redo.complex_hook.is_some());

    let scrape = global_registry().get("Scrape").expect("Scrape");
    assert!(scrape.effect_data.is_empty());
    assert!(scrape.complex_hook.is_some());
}

#[test]
fn seek_plus_searches_the_draw_pile_with_the_declarative_choice_surface() {
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Strike_B", "Defend_B", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Seek+"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;

    assert!(play_self(&mut engine, "Seek+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(
        engine.choice.as_ref().map(|choice| choice.reason.clone()),
        Some(ChoiceReason::SearchDrawPile),
    );

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.draw_pile.len(), 2);
}

#[test]
#[ignore = "Chaos still needs a random-orb selection primitive; Java Chaos picks a fresh random orb type on use."]
fn chaos_still_needs_random_orb_selection() {}

#[test]
#[ignore = "Fission still needs a remove-all-orbs primitive before the energy/draw payload can be typed; Java FissionAction removes or evokes all orbs first."]
fn fission_still_needs_remove_all_orbs_before_payload() {}

#[test]
#[ignore = "Reboot still needs a shuffle-hand-and-discard-into-draw primitive; Java ShuffleAllAction moves the whole hand and discard before drawing."]
fn reboot_still_needs_shuffle_hand_and_discard_into_draw() {}

#[test]
#[ignore = "Redo still needs a typed front-orb reuse primitive; Java RecursionAction evokes the front orb and channels the same orb type back."]
fn redo_still_needs_front_orb_reuse_primitive() {}

#[test]
#[ignore = "Scrape still needs a draw-then-discard-non-zero-cost follow-up primitive; Java ScrapeFollowUpAction discards the drawn non-zero-cost cards after the draw resolves."]
fn scrape_still_needs_draw_then_discard_non_zero_cost_follow_up() {}
