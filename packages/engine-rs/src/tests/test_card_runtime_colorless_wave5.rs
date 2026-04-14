#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MindBlastAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RitualDaggerAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, ChoiceAction, CardFilter, Effect as E, Pile as P, BulkAction};
use crate::actions::Action;
use crate::engine::CombatPhase;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self};

#[test]
fn colorless_wave5_registry_exports_match_typed_surface_for_supported_plus_cards() {
    let registry = global_registry();

    let forethought_plus = registry.get("Forethought+").expect("Forethought+ should exist");
    assert_eq!(forethought_plus.card_type, CardType::Skill);
    assert_eq!(forethought_plus.target, CardTarget::None);
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

    let enlightenment_plus = registry.get("Enlightenment+").expect("Enlightenment+ should exist");
    assert_eq!(enlightenment_plus.card_type, CardType::Skill);
    assert_eq!(enlightenment_plus.target, CardTarget::SelfTarget);
    assert_eq!(
        enlightenment_plus.effect_data,
        &[E::ForEachInPile {
            pile: P::Hand,
            filter: CardFilter::All,
            action: BulkAction::SetCost(1),
        }]
    );
    assert!(enlightenment_plus.complex_hook.is_none());
}

#[test]
fn forethought_plus_keeps_selected_cards_on_bottom_at_zero_cost() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought+", "Strike_R", "Defend_R"]);

    assert!(play_self(&mut engine, "Forethought+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.draw_pile[0].def_id), "Strike_R");
    assert_eq!(engine.state.draw_pile[0].cost, 0);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Defend_R");
}

#[test]
fn enlightenment_plus_sets_costs_in_hand_to_one() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Enlightenment+", "Mind Blast", "Strike_R"]);

    assert!(play_self(&mut engine, "Enlightenment+"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(engine.state.hand[0].cost, 1);
    assert_eq!(engine.state.hand[1].cost, 1);
}

#[test]
#[ignore = "Forethought still needs the single-card auto-resolve primitive; Java moves the only card directly without opening the hand-select screen."]
fn forethought_still_needs_single_card_auto_resolve_primitive() {}

#[test]
#[ignore = "Impatience still needs a no-attacks-in-hand primitive; Java checks the current hand contents before drawing."]
fn impatience_still_needs_no_attacks_in_hand_primitive() {}

#[test]
#[ignore = "Mind Blast still needs a draw-pile-size attack scaling primitive on the typed primary attack path; Java resolves damage from the current draw pile size."]
fn mind_blast_still_needs_draw_pile_size_attack_scaling() {}

#[test]
#[ignore = "Ritual Dagger still needs kill-context and card-owned misc scaling propagation; Java updates the played copy after a kill and carries the dagger's misc state forward."]
fn ritual_dagger_still_needs_kill_context_and_misc_scaling() {}

#[test]
#[ignore = "Madness still needs a random-hand-card zero-cost primitive; Java repeatedly samples the hand until it finds a card that can be reduced."]
fn madness_still_needs_random_hand_card_zero_cost_primitive() {}

#[test]
#[ignore = "Violence still needs a typed draw-pile attack fetch primitive; Java pulls random attacks from the draw pile into hand."]
fn violence_still_needs_draw_pile_attack_fetch_primitive() {}
