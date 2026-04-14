#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ForeignInfluence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, GeneratedCardPool, GeneratedCostRule};
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
fn transmutation_moves_to_typed_generated_hand_surface() {
    let registry = global_registry();
    let transmutation = registry.get("Transmutation").expect("Transmutation should exist");
    let transmutation_plus = registry.get("Transmutation+").expect("Transmutation+ should exist");

    assert_eq!(
        transmutation.effect_data,
        &[E::GenerateRandomCardsToHand {
            pool: GeneratedCardPool::Colorless,
            count: A::XCost,
            cost_rule: GeneratedCostRule::ZeroThisTurn,
        }]
    );
    assert_eq!(
        transmutation_plus.effect_data,
        &[E::GenerateRandomCardsToHand {
            pool: GeneratedCardPool::Colorless,
            count: A::XCost,
            cost_rule: GeneratedCostRule::ZeroThisTurnAndUpgradeGenerated,
        }]
    );
    assert!(transmutation.complex_hook.is_none());
    assert!(transmutation_plus.complex_hook.is_none());
}

#[test]
fn transmutation_generates_zero_cost_colorless_cards_with_chemical_x_bonus() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Transmutation"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 3;
    engine.state.relics.push("Chemical X".to_string());

    assert!(play_self(&mut engine, "Transmutation"));
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.hand.len(), 5);
    for card in &engine.state.hand {
        let name = engine.card_registry.card_name(card.def_id);
        assert!(COLORLESS_CHOICES.contains(&name));
        assert_eq!(card.cost, 0);
    }
}

#[test]
fn transmutation_plus_upgrades_generated_cards() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation+"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        2,
    ));
    engine.state.hand = make_deck(&["Transmutation+"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 2;

    assert!(play_self(&mut engine, "Transmutation+"));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.is_upgraded()));
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}
