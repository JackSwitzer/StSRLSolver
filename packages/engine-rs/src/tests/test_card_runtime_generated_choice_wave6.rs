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
    "Defend", "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse", "Flash of Steel",
    "Forethought", "Ghostly", "Good Instincts", "HandOfGreed", "Impatience", "J.A.X.",
    "Jack Of All Trades", "Madness", "Magnetism", "Master of Strategy", "Mayhem",
    "Metamorphosis", "Mind Blast", "Panacea", "Panache", "PanicButton", "Purity",
    "RitualDagger", "Sadistic Nature", "Secret Technique", "Secret Weapon", "Strike",
    "Swift Strike", "The Bomb", "Thinking Ahead", "Transmutation", "Trip", "Violence",
];

// srcColorlessCardPool order after addColorlessCards' add-to-top behavior;
// returnTrulyRandomColorlessCardInCombat then excludes HEALING Bandage Up.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
const JAVA_COLORLESS_GENERATION_POOL: &[&str] = &[
    "Madness", "Thinking Ahead", "Mind Blast", "Metamorphosis", "Jack Of All Trades",
    "Swift Strike", "Good Instincts", "Master of Strategy", "Magnetism", "Finesse",
    "Discovery", "Chrysalis", "Transmutation", "Panacea", "Purity", "Enlightenment",
    "Forethought", "Flash of Steel", "HandOfGreed", "Mayhem", "Apotheosis", "Secret Weapon",
    "Panache", "Violence", "Deep Breath", "Secret Technique", "Blind", "The Bomb",
    "Impatience", "Dramatic Entrance", "Trip", "PanicButton", "Sadistic Nature", "Dark Shackles",
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

#[test]
fn free_transmutation_plus_uses_current_x_and_chemical_x_then_spills_overflow() {
    // TransmutationAction starts from energyOnUse, adds two for Chemical X,
    // upgrades and zeroes every generated card, and skips energy.use when
    // freeToPlayOnce is true. Each random selection consumes cardRandomRng;
    // MakeTempCardInHandAction sends copies beyond ten cards to discard.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        2,
    ));
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.hand = make_deck(&[
        "Defend", "Defend", "Defend", "Defend",
        "Defend", "Defend", "Defend", "Defend",
    ]);
    engine
        .state
        .hand
        .push(engine.card_registry.make_card("Transmutation+").set_free(true));
    engine.state.relics.push("Chemical X".to_string());

    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<&str> = (0..4)
        .map(|_| {
            JAVA_COLORLESS_GENERATION_POOL
                [oracle.random_int((JAVA_COLORLESS_GENERATION_POOL.len() - 1) as i32) as usize]
        })
        .collect();
    let generic_before = engine.shuffle_rng.counter;

    assert!(play_self(&mut engine, "Transmutation+"));

    assert_eq!(engine.state.energy, 2);
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(engine.state.discard_pile.len(), 2);
    let generated: Vec<_> = engine.state.hand[8..]
        .iter()
        .chain(engine.state.discard_pile.iter())
        .collect();
    let generated_names: Vec<&str> = generated
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).trim_end_matches('+'))
        .collect();
    assert_eq!(generated_names, expected);
    assert!(generated.iter().all(|card| card.is_upgraded() && card.cost == 0));
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.shuffle_rng.counter, generic_before);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
}
