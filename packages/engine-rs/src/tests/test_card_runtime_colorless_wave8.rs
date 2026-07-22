#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MadnessAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Apparition.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, CardFilter, ChoiceAction, Condition, Effect as E, Pile as P,
    SimpleEffect as SE,
};
use crate::engine::CombatPhase;
use crate::status_ids::sid;
use crate::tests::support::{
    end_turn, enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy,
    play_self,
};

#[test]
fn colorless_wave8_registry_exports_match_typed_surface_for_forethought_and_impatience() {
    let registry = global_registry();

    let forethought = registry
        .get("Forethought")
        .expect("Forethought should exist");
    assert_eq!(forethought.card_type, CardType::Skill);
    assert_eq!(forethought.target, CardTarget::None);
    assert_eq!(
        forethought.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::PutOnBottomFreeIfCostly,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(forethought.complex_hook.is_none());

    let forethought_plus = registry
        .get("Forethought+")
        .expect("Forethought+ should exist");
    assert_eq!(
        forethought_plus.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::PutOnBottomFreeIfCostly,
            min_picks: A::Fixed(0),
            max_picks: A::Fixed(99),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
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

    let impatience_plus = registry
        .get("Impatience+")
        .expect("Impatience+ should exist");
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
fn forethought_marks_a_selected_positive_cost_card_free_once() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought", "Strike", "Defend"]);

    assert!(play_self(&mut engine, "Forethought"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile[0].def_id),
        "Strike"
    );
    assert_eq!(engine.state.draw_pile[0].cost, -1);
    assert!(engine.state.draw_pile[0].is_free());
    assert_eq!(
        engine.card_registry.card_name(engine.state.hand[0].def_id),
        "Defend"
    );
}

#[test]
fn forethought_auto_moves_a_singleton_and_free_flag_survives_turn_cost_reset() {
    // ForethoughtAction auto-moves a singleton for the base card, checks the
    // permanent `cost` (not costForTurn), and sets freeToPlayOnce. UseCardAction
    // clears that flag before the played card reaches its destination.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought", "Strike"]);

    assert!(play_self(&mut engine, "Forethought"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(engine.state.draw_pile[0].cost, -1);
    assert!(engine.state.draw_pile[0].is_free());

    engine.state.draw_pile[0].reset_cost_for_turn();
    engine.draw_cards(1);
    engine.state.energy = 0;
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.energy, 0);
    let played = engine
        .state
        .discard_pile
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Strike")
        .expect("played Strike should be discarded");
    assert!(!played.is_free());
}

#[test]
fn forethought_plus_preserves_selection_order_and_does_not_free_zero_or_x_cost_cards() {
    // Each selected card is moved to the bottom in selection order. Since
    // CardGroup.addToBottom inserts at index zero, the last selection becomes
    // the absolute bottom. ForethoughtAction only marks `cost > 0` cards free.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought+", "Neutralize", "Whirlwind", "Strike"]);

    assert!(play_self(&mut engine, "Forethought+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    engine.execute_action(&Action::Choose(2)); // Strike
    engine.execute_action(&Action::Choose(0)); // Neutralize (0)
    engine.execute_action(&Action::Choose(1)); // Whirlwind (X)
    engine.execute_action(&Action::ConfirmSelection);

    let names: Vec<&str> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(names, vec!["Whirlwind", "Neutralize", "Strike"]);
    assert!(!engine.state.draw_pile[0].is_free());
    assert!(!engine.state.draw_pile[1].is_free());
    assert!(engine.state.draw_pile[2].is_free());
    assert!(engine.state.draw_pile.iter().all(|card| card.cost == -1));
}

#[test]
fn impatience_draws_when_no_attacks_are_in_hand() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Impatience", "Defend", "Defend"]);
    engine.state.draw_pile = make_deck(&["Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Impatience"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 4);
}

#[test]
fn impatience_does_not_draw_when_an_attack_is_present() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Impatience", "Strike", "Defend"]);
    engine.state.draw_pile = make_deck(&["Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Impatience"));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
}

#[test]
fn impatience_plus_draws_three_after_the_played_skill_leaves_hand() {
    // Impatience.java upgrades only magicNumber from 2 to 3.
    // ConditionalDrawAction scans the remaining hand for ATTACK cards, so the
    // played Impatience+ itself cannot suppress its draw.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ConditionalDrawAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Impatience+", "Defend"]);
    engine.state.draw_pile = make_deck(&["Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Impatience+"));

    assert_eq!(engine.state.energy, 3);
    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.draw_pile.len(), 0);
}

#[test]
fn ghostly_applies_one_intangible_and_upgrade_only_removes_ethereal() {
    // Apparition.java sets exhaust and Ethereal in the constructor. upgrade()
    // only clears Ethereal; both versions apply IntangiblePlayerPower(1).
    // Java: reference/extracted/methods/card/Apparition.java
    let registry = global_registry();
    let base = registry.get("Ghostly").expect("Ghostly");
    let upgraded = registry.get("Ghostly+").expect("Ghostly+");
    assert_eq!(base.cost, 1);
    assert_eq!(upgraded.cost, 1);
    assert!(base.exhaust && upgraded.exhaust);
    assert!(base.runtime_traits().ethereal);
    assert!(!upgraded.runtime_traits().ethereal);
    assert_eq!(
        base.effect_data,
        &[E::Simple(SE::AddStatus(
            crate::effects::declarative::Target::Player,
            sid::INTANGIBLE,
            A::Fixed(1),
        ))],
    );

    let mut played = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 2);
    force_player_turn(&mut played);
    played.state.hand = make_deck(&["Ghostly", "Ghostly+"]);
    assert!(play_self(&mut played, "Ghostly"));
    assert!(play_self(&mut played, "Ghostly+"));
    assert_eq!(played.state.player.status(sid::INTANGIBLE), 2);
    assert_eq!(played.state.exhaust_pile.len(), 2);

    let mut unplayed =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut unplayed);
    unplayed.state.hand = make_deck(&["Ghostly", "Ghostly+"]);
    unplayed.state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike"]);
    end_turn(&mut unplayed);
    assert!(unplayed
        .state
        .exhaust_pile
        .iter()
        .any(|card| { unplayed.card_registry.card_name(card.def_id) == "Ghostly" }));
    assert!(unplayed
        .state
        .discard_pile
        .iter()
        .any(|card| { unplayed.card_registry.card_name(card.def_id) == "Ghostly+" }));
}
