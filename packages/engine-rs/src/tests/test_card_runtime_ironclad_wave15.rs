#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustSpecificCardAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/NewQueueCardAction.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn ironclad_wave15_registry_promotes_havoc_and_headbutt_to_the_typed_surface() {
    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.card_type, CardType::Skill);
    assert_eq!(havoc.target, CardTarget::None);
    assert_eq!(
        havoc.effect_data,
        &[E::Simple(SE::PlayTopCardOfDraw)]
    );
    assert!(havoc.complex_hook.is_none(), "Havoc should now be fully typed");

    let headbutt = global_registry().get("Headbutt").expect("Headbutt should exist");
    assert_eq!(headbutt.card_type, CardType::Attack);
    assert_eq!(headbutt.target, CardTarget::Enemy);
    assert_eq!(
        headbutt.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ChooseCards {
                source: P::Discard,
                filter: CardFilter::All,
                action: ChoiceAction::PutOnTopOfDraw,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ]
    );
    assert!(headbutt.complex_hook.is_none(), "Headbutt should now be fully typed");
}

#[test]
fn ironclad_wave15_havoc_plays_the_top_card_of_draw_pile_through_the_normal_free_path() {
    // PlayTopCardAction removes the top card, sets exhaustOnUseOnce, and queues
    // it as an autoplay without changing its cost fields.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
    let mut engine = engine_for(&["Havoc"], &["Defend", "Strike"], &[], 1);

    let hp_before = engine.state.enemies[0].entity.hp;
    let card_random_before = engine.card_random_rng.counter;
    assert!(play_self(&mut engine, "Havoc"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.card_random_rng.counter, card_random_before + 1);
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 6);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.draw_pile.last().expect("remaining draw card").def_id),
        "Defend"
    );
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.discard_pile.last().expect("top discard").def_id),
        "Havoc"
    );
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    let exhausted = engine.state.exhaust_pile[0];
    assert_eq!(engine.card_registry.card_name(exhausted.def_id), "Strike");
    assert_eq!(exhausted.cost, -1);
    assert_eq!(
        exhausted.flags & crate::combat_types::CardInstance::FLAG_EXHAUST_ON_USE,
        0
    );
}

#[test]
fn havoc_shuffles_discard_when_needed_and_still_rolls_a_target_with_no_cards() {
    // PlayTopCardAction first returns only when both piles are empty; when just
    // draw is empty it queues EmptyDeckShuffleAction and retries itself.
    // Havoc.java already consumed its cardRandom target roll in either case.
    // Java: reference/extracted/methods/card/Havoc.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/PlayTopCardAction.java
    let mut shuffled = engine_for(&["Havoc+"], &[], &["Strike"], 3);
    let hp_before = shuffled.state.enemies[0].entity.hp;
    let card_random_before = shuffled.card_random_rng.counter;

    assert!(play_self(&mut shuffled, "Havoc+"));
    assert_eq!(shuffled.state.energy, 3);
    assert_eq!(shuffled.card_random_rng.counter, card_random_before + 1);
    assert_eq!(shuffled.state.enemies[0].entity.hp, hp_before - 6);
    assert!(shuffled.state.draw_pile.is_empty());
    assert_eq!(shuffled.state.exhaust_pile.len(), 1);
    assert_eq!(
        shuffled.card_registry.card_name(shuffled.state.exhaust_pile[0].def_id),
        "Strike"
    );

    let mut empty = engine_for(&["Havoc+"], &[], &[], 3);
    let card_random_before = empty.card_random_rng.counter;
    assert!(play_self(&mut empty, "Havoc+"));
    assert_eq!(empty.card_random_rng.counter, card_random_before + 1);
    assert!(empty.state.exhaust_pile.is_empty());
    assert_eq!(empty.state.discard_pile.len(), 1);
}

#[test]
fn ironclad_wave15_headbutt_moves_a_discard_card_to_the_top_of_draw() {
    let mut engine = engine_for(&["Headbutt"], &[], &["Defend", "Strike"], 3);

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Headbutt", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 9);
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, crate::engine::ChoiceReason::PickFromDiscard);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, crate::engine::CombatPhase::PlayerTurn);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.draw_pile.last().expect("top draw card").def_id),
        "Strike"
    );
    let discard_names: Vec<&str> = engine
        .state
        .discard_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert!(discard_names.contains(&"Defend"));
    assert!(discard_names.contains(&"Headbutt"));
    assert!(!discard_names.contains(&"Strike"));
}

#[test]
fn headbutt_plus_auto_moves_a_singleton_and_skips_retrieval_after_a_final_kill() {
    // Headbutt upgrades damage by 3 (9 -> 12). DiscardPileToTopOfDeckAction
    // auto-moves a singleton, but returns immediately when battle is ending.
    // Java: reference/extracted/methods/card/Headbutt.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscardPileToTopOfDeckAction.java
    let mut singleton = engine_for(&["Headbutt+"], &["Shrug It Off"], &["Defend"], 3);
    let hp_before = singleton.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut singleton, "Headbutt+", 0));
    assert_eq!(singleton.state.enemies[0].entity.hp, hp_before - 12);
    assert_eq!(singleton.phase, crate::engine::CombatPhase::PlayerTurn);
    assert!(singleton.choice.is_none());
    assert_eq!(
        singleton.card_registry.card_name(singleton.state.draw_pile.last().expect("top draw").def_id),
        "Defend"
    );

    let mut lethal = engine_for(&["Headbutt+"], &[], &["Strike", "Defend"], 3);
    lethal.state.enemies[0].entity.hp = 12;
    lethal.state.enemies[0].entity.max_hp = 12;

    assert!(play_on_enemy(&mut lethal, "Headbutt+", 0));
    assert!(lethal.state.enemies[0].entity.is_dead());
    assert!(lethal.choice.is_none());
    assert!(lethal.state.draw_pile.is_empty());
    assert_eq!(lethal.state.discard_pile.len(), 3);
}
