#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Discovery.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ForeignInfluence.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java

use crate::actions::Action;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self};

const COLORLESS_CHOICES: &[&str] = &[
    "Apotheosis", "Bandage Up", "Bite", "Blind", "Chrysalis", "Dark Shackles", "Deep Breath",
    "Defend_R", "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse", "Flash of Steel",
    "Forethought", "Ghostly", "Good Instincts", "HandOfGreed", "Impatience", "J.A.X.",
    "Jack Of All Trades", "Madness", "Magnetism", "Master of Strategy", "Mayhem",
    "Metamorphosis", "Mind Blast", "Panacea", "Panache", "PanicButton", "Purity",
    "RitualDagger", "Sadistic Nature", "Secret Technique", "Secret Weapon", "Strike_R",
    "Swift Strike", "The Bomb", "Thinking Ahead", "Transmutation", "Trip", "Violence",
];

#[test]
fn discovery_moves_to_generated_choice_runtime_and_resolves_a_zero_cost_colorless_card() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Discovery", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    let hand_before = engine.state.hand.len();

    assert!(play_self(&mut engine, "Discovery"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    let choice = engine.choice.as_ref().expect("Discovery should open a generated choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    for option in &choice.options {
        let ChoiceOption::GeneratedCard(card) = option else {
            panic!("Discovery should present generated-card options");
        };
        let generated_name = engine.card_registry.card_name(card.def_id);
        assert!(
            COLORLESS_CHOICES.contains(&generated_name),
            "Discovery should generate colorless card choices, got {generated_name}"
        );
    }

    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), hand_before);
    assert!(
        engine.state.hand.iter().any(|card| card.cost == 0),
        "Discovery should resolve a zero-cost generated card into hand"
    );
    assert!(
        !engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Discovery"),
        "the played Discovery should leave the hand after resolution"
    );
}

#[test]
fn discovery_plus_keeps_the_same_choice_runtime_without_exhausting() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Discovery+", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    assert!(play_self(&mut engine, "Discovery+"));
    let choice = engine.choice.as_ref().expect("Discovery+ should open a generated choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    engine.execute_action(&Action::Choose(0));

    assert!(
        !engine
            .state
            .exhaust_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Discovery+")
    );
}
