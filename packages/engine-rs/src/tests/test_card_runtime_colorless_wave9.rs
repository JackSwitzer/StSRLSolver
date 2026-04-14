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
    engine.state.hand = make_deck(&["Madness", "Strike_R", "Defend_R"]);

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
#[ignore = "Enlightenment still needs the turn-only cost-reduction primitive; Java updates costForTurn for the turn and only permanently reduces upgraded cards. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java"]
fn enlightenment_still_needs_turn_only_cost_reduction_primitive() {}
