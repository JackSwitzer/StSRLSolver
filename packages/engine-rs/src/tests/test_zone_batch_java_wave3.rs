#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Purity.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretTechnique.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, ChoiceAction, Effect, Pile};
use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
use crate::tests::support::{
    combat_state_with, enemy_no_intent, force_player_turn, make_deck, play_self, TEST_SEED,
};

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
    let mut engine = CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn purity_now_uses_declarative_hand_exhaust_selection() {
    let def = CombatEngine::new(combat_state_with(vec![], vec![], 3), TEST_SEED)
        .card_registry
        .get("Purity")
        .expect("Purity should be registered");

    assert!(def.complex_hook.is_none());
    assert_eq!(
        def.effect_data,
        &[Effect::ChooseCards {
            source: Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: crate::effects::declarative::AmountSource::Fixed(0),
            max_picks: crate::effects::declarative::AmountSource::Magic,
        }]
    );
}

#[test]
fn purity_still_uses_zero_to_many_exhaust_selection_up_to_its_cap() {
    let mut engine = engine_for(
        &["Purity", "Strike_R", "Defend_R", "Bash"],
        &[],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Purity"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("purity choice");
    assert_eq!(choice.reason, ChoiceReason::ExhaustFromHand);
    assert_eq!(choice.min_picks, 0);
    assert_eq!(choice.max_picks, 3);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.exhaust_pile.len(), 2);
    assert_eq!(engine.state.hand.len(), 1);
}

#[test]
fn secret_technique_now_uses_declarative_skill_search() {
    let def = CombatEngine::new(combat_state_with(vec![], vec![], 3), TEST_SEED)
        .card_registry
        .get("Secret Technique")
        .expect("Secret Technique should be registered");

    assert!(def.complex_hook.is_none());
    assert_eq!(
        def.effect_data,
        &[Effect::ChooseCards {
            source: Pile::Draw,
            filter: crate::effects::declarative::CardFilter::Skills,
            action: ChoiceAction::MoveToHand,
            min_picks: crate::effects::declarative::AmountSource::Fixed(1),
            max_picks: crate::effects::declarative::AmountSource::Fixed(1),
        }]
    );
}

#[test]
fn secret_technique_still_opens_a_skill_only_draw_pile_search_choice() {
    let mut engine = engine_for(
        &["Secret Technique"],
        &["Strike_R", "Shrug It Off", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Secret Technique"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("secret technique choice");
    assert_eq!(choice.reason, ChoiceReason::SearchDrawPile);
    assert_eq!(choice.options.len(), 1);
}

#[test]
#[ignore = "Headbutt still needs a typed discard-to-top-of-draw choice action; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java"]
fn headbutt_remains_hook_backed_until_discard_to_top_of_draw_is_typed() {}

#[test]
fn burning_pact_uses_choice_owned_deferred_draw_follow_up() {
    let burning_pact = global_registry().get("Burning Pact").expect("Burning Pact");
    assert_eq!(
        burning_pact.effect_data,
        &[Effect::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(burning_pact.complex_hook.is_some());
}

#[test]
#[ignore = "Violence still needs a capped filtered draw-to-hand primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java"]
fn violence_remains_hook_backed_until_capped_attack_fetch_is_typed() {}

#[test]
#[ignore = "Secret Technique can_use legality still lives outside this slice; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretTechnique.java"]
fn secret_technique_illegal_play_without_skill_in_draw_pile_remains_queued() {}
